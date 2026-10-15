use std::collections::BTreeSet;

use clap::{builder::PossibleValuesParser, Command, CommandFactory};

use crate::{Cli, Portfolio};

/// Generate a shell adapter that asks the installed binary for candidates.
/// Source it on shell startup so upgrades refresh the adapter as well as data.
pub fn write_registration(
    shell: clap_complete::Shell,
    output: &mut dyn std::io::Write,
) -> std::io::Result<()> {
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
