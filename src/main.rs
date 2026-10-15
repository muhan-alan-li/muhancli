use clap::Parser;
use muhancli::render;
use muhancli::{Cli, Commands, OutputOpts, Portfolio};

fn main() {
    let args = Cli::parse();
    let opts = OutputOpts {
        json: args.json,
        plain: args.plain,
    };
    let portfolio = Portfolio::load();
    let show_intro = args.command.is_none() && !args.json;
    let command = args.command.unwrap_or(Commands::About);

    let output = match &command {
        Commands::About => render::render_about(&portfolio, opts),
        Commands::Experience { id } => match id {
            None => render::render_experience_list(&portfolio, opts),
            Some(id) => match portfolio.find_experience(id) {
                Some(e) => render::render_experience_show(e, opts),
                None => {
                    eprintln!(
                        "error: unknown experience id '{id}'. Valid ids: {}",
                        portfolio
                            .experience
                            .iter()
                            .map(|e| e.id.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    std::process::exit(2);
                }
            },
        },
        Commands::Projects { slug, tag } => match slug {
            Some(slug) => match portfolio.find_project(slug) {
                Some(p) => render::render_project_show(p, opts),
                None => {
                    eprintln!(
                        "error: unknown project slug '{slug}'. Valid slugs: {}",
                        portfolio
                            .projects
                            .iter()
                            .map(|p| p.slug.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    std::process::exit(2);
                }
            },
            None => {
                let list: Vec<&muhancli::data::Project> = match tag {
                    Some(t) => portfolio.projects_by_tag(t),
                    None => portfolio.projects.iter().collect(),
                };
                render::render_projects_list(&list, tag.as_deref(), opts)
            }
        },
        Commands::Skills { group } => match group {
            Some(g) => match portfolio.skill_group(g) {
                Some(sg) => render::render_skills(&[sg], opts),
                None => {
                    eprintln!(
                        "error: unknown skill group '{g}'. Valid groups: {}",
                        portfolio
                            .skills
                            .iter()
                            .map(|s| s.group.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    std::process::exit(2);
                }
            },
            None => {
                let all: Vec<&muhancli::data::SkillGroup> = portfolio.skills.iter().collect();
                render::render_skills(&all, opts)
            }
        },
        Commands::Contact => render::render_contact(&portfolio, opts),
        Commands::Version => render::render_version(),
    };

    println!("{output}");
    if show_intro {
        println!("\nExplore: muhan experience | muhan projects | muhan skills | muhan contact");
        println!("All commands: muhan --help");
    }
}
