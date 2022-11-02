use std::{
    fs::OpenOptions,
    io::{self, Write},
};

use anyhow::{ensure, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Args, CommandFactory, Parser, ValueHint};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo")]
enum Cli {
    #[command(subcommand)]
    Hatch(Command),
}

#[derive(Parser)]
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
        #[arg(value_hint = ValueHint::DirPath)]
        path: Utf8PathBuf,
        #[command(flatten)]
        flags: CreationFlags,
    },
    /// Generate auto-completion scripts for various shells.
    Completions {
        /// Shell to generate an auto-completion script for.
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Generate man pages into the given directory.
    Manpages {
        /// Target directory, that must already exist and be empty. If the any file with the same
        /// name as any of the man pages already exist, it'll not be overwritten, but instead an
        /// error be returned.
        #[arg(value_hint = ValueHint::DirPath)]
        dir: Utf8PathBuf,
    },
}

#[derive(Args)]
pub struct CreationFlags {
    /// Name of the new project, using the current working directory if omitted.
    ///
    /// If the name contains slashes `/`, it is treated as a file path and the target directory is
    /// created automatically if missing. The final project name will become the last part of the
    /// path.
    pub name: Option<Utf8PathBuf>,
    /// Update all dependencies to the latest compatible version after project creation.
    #[arg(short, long)]
    pub update_deps: bool,
}

#[must_use]
pub fn parse() -> Command {
    let Cli::Hatch(cmd) = Cli::parse();
    cmd
}

/// Generate shell completions, written to the standard output.
pub fn completions(shell: Shell) {
    clap_complete::generate(
        shell,
        &mut Cli::command(),
        env!("CARGO_PKG_NAME"),
        &mut io::stdout().lock(),
    );
}

/// Generate man pages in the target directory. The directory must already exist and none of the
/// files exist, or an error is returned.
pub fn manpages(dir: &Utf8Path) -> Result<()> {
    fn print(dir: &Utf8Path, app: &clap::Command) -> Result<()> {
        let name = app.get_display_name().unwrap_or_else(|| app.get_name());
        let out = dir.join(format!("{name}.1"));
        let mut out = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&out)
            .with_context(|| format!("the file `{out}` already exists"))?;

        clap_mangen::Man::new(app.clone()).render(&mut out)?;
        out.flush()?;

        for sub in app.get_subcommands() {
            print(dir, sub)?;
        }

        Ok(())
    }

    ensure!(dir.try_exists()?, "target directory doesn't exist");

    let mut app = Command::command();
    app.build();

    print(dir, &app)
}

#[cfg(test)]
mod tests {
    use super::Cli;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
