use clap::{Parser, Subcommand};

/// Muhan Li's portfolio in the terminal.
#[derive(Debug, Parser)]
#[command(
    name = "muhan",
    version,
    about = "Portfolio CLI — experience, projects, skills"
)]
pub struct Cli {
    /// Output raw JSON instead of human-readable text.
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable colors and unicode decorations (also respects NO_COLOR).
    #[arg(long, global = true)]
    pub plain: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Show bio, education, and links.
    About,
    /// List work experience or show one role in detail.
    Experience {
        /// Show detail for this id (e.g. `apera-ai`). Omit to list all.
        #[arg(value_name = "ID")]
        id: Option<String>,
    },
    /// List projects or show one project in detail.
    Projects {
        /// Show detail for this slug (e.g. `dune-imperium`). Omit to list all.
        #[arg(value_name = "SLUG")]
        slug: Option<String>,
        /// Filter list by stack/keyword (e.g. `--tag react`).
        #[arg(long)]
        tag: Option<String>,
    },
    /// Show skills, optionally filtered to one group.
    Skills {
        /// Filter to one group (e.g. `--group Languages`).
        #[arg(long)]
        group: Option<String>,
    },
    /// Show contact info.
    Contact,
    /// Generate a shell completion script (see README for setup).
    Completions {
        /// Shell that will load the generated script.
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Show version, git sha, and build date.
    Version,
}
