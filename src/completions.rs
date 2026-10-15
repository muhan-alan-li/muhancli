use std::collections::BTreeSet;

use clap::{builder::PossibleValuesParser, Command, CommandFactory};

use crate::{Cli, Portfolio};

pub fn complete() {
    let mut args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    // Windows PowerShell 5.1 drops empty strings in native argument lists.
    // Our PowerShell adapter transports words as JSON to preserve empty and
    // quoted arguments without evaluating the user's command line as code.
    if std::env::var("MUHAN_COMPLETE").as_deref() == Ok("powershell") {
        if let Ok(json) = std::env::var("MUHAN_COMPLETE_WORDS") {
            let words: Vec<String> = serde_json::from_str(&json).unwrap_or_else(|error| {
                eprintln!("error: invalid completion words: {error}");
                std::process::exit(2);
            });
            args.truncate(1);
            args.push("--".into());
            args.extend(words.into_iter().map(Into::into));
        }
    }
    let completed = clap_complete::env::CompleteEnv::with_factory(|| command(&Portfolio::load()))
        .var("MUHAN_COMPLETE")
        .try_complete(args, std::env::current_dir().ok().as_deref())
        .unwrap_or_else(|error| error.exit());
    if completed {
        std::process::exit(0);
    }
}

/// Generate a shell adapter that asks the installed binary for candidates.
/// Source it on shell startup so upgrades refresh the adapter as well as data.
pub fn write_registration(
    shell: clap_complete::Shell,
    output: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    if shell == clap_complete::Shell::PowerShell {
        return output.write_all(include_bytes!("completions/powershell.ps1"));
    }
    clap_complete::env::Shells::builtins()
        .completer(&shell.to_string())
        .expect("all supported shells have dynamic completers")
        .write_registration("MUHAN_COMPLETE", "muhan", "muhan", "muhan", output)
}

/// Add portfolio values only to the completion command tree. Normal parsing
/// remains case insensitive for IDs and accepts arbitrary tag search terms.
pub fn command(portfolio: &Portfolio) -> Command {
    Cli::command()
        .mut_subcommand("experience", |cmd| {
            cmd.mut_arg("id", |arg| {
                arg.value_parser(PossibleValuesParser::new(
                    portfolio.experience.iter().map(|e| e.id.clone()),
                ))
            })
        })
        .mut_subcommand("projects", |cmd| {
            cmd.mut_arg("slug", |arg| {
                arg.value_parser(PossibleValuesParser::new(
                    portfolio.projects.iter().map(|p| p.slug.clone()),
                ))
            })
            .mut_arg("tag", |arg| {
                let tags: BTreeSet<String> = portfolio
                    .projects
                    .iter()
                    .flat_map(|p| p.stack.iter().cloned())
                    .collect();
                arg.value_parser(PossibleValuesParser::new(tags))
            })
        })
        .mut_subcommand("skills", |cmd| {
            cmd.mut_arg("group", |arg| {
                arg.value_parser(PossibleValuesParser::new(
                    portfolio.skills.iter().map(|s| s.group.clone()),
                ))
            })
        })
}
