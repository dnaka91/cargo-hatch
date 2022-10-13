use std::io;

use camino::Utf8PathBuf;
use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo")]
enum Opt {
    #[command(subcommand)]
    Hatch(Command),
}

#[derive(Subcommand)]
#[command(about, author, version)]
pub enum Command {
    /// Initialize a new template with a sample configuration.
    Init {
        /// Name of the new template, using the current working directory if omitted.
        name: Option<String>,
    },
    /// List all configured bookmarks with name and description.
    List,
    /// Create a new project from configured bookmarks.
    New {
        /// Bookmark as defined in the global configuration.
        bookmark: String,
        #[command(flatten)]
        flags: CreationFlags,
    },
    /// Create a new project from a template located in a remote Git repository.
    Git {
        /// An optional sub-folder within the repository that contains the template.
        #[arg(long)]
        folder: Option<Utf8PathBuf>,
        /// HTTP or Git URL to the remote repository.
        url: String,
        #[command(flatten)]
        flags: CreationFlags,
    },
    /// Create a new project from a template located in the local file system.
    Local {
        /// Location of the template directory.
        path: Utf8PathBuf,
        #[command(flatten)]
        flags: CreationFlags,
    },
    /// Generate shell completions for cargo-hatch, writing them to the standard output.
    Completions {
        /// The shell type to generate completions for.
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Args)]
pub struct CreationFlags {
    /// Name of the new project, using the current working directory if omitted.
    pub name: Option<String>,
    /// Update all dependencies to the latest compatible version after project creation.
    #[arg(short, long)]
    pub update_deps: bool,
}

#[must_use]
pub fn parse() -> Command {
    let Opt::Hatch(cmd) = Opt::parse();
    cmd
}

pub fn completions(shell: Shell) {
    clap_complete::generate(
        shell,
        &mut Opt::command(),
        env!("CARGO_PKG_NAME"),
        &mut io::stdout().lock(),
    );
}
