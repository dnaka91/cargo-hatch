use std::{convert::TryFrom, env, fs, io};

use anyhow::{anyhow, bail, ensure, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_hatch::{cargo, dirs::Utf8ProjectDirs, repo, settings, templates};
use clap::{AppSettings, Args, CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[clap(
    about,
    author,
    version,
    global_setting = AppSettings::DeriveDisplayOrder,
)]
enum Opt {
    #[clap(subcommand)]
    Hatch(Command),
}

#[derive(Subcommand)]
enum Command {
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
        #[clap(flatten)]
        flags: CreationFlags,
    },
    /// Create a new project from a template located in a remote Git repository.
    Git {
        /// An optional sub-folder within the repository that contains the template.
        #[clap(long)]
        folder: Option<Utf8PathBuf>,
        /// HTTP or Git URL to the remote repository.
        url: String,
        #[clap(flatten)]
        flags: CreationFlags,
    },
    /// Create a new project from a template located in the local file system.
    Local {
        /// Location of the template directory.
        path: Utf8PathBuf,
        #[clap(flatten)]
        flags: CreationFlags,
    },
    /// Generate shell completions for cargo-hatch, writing them to the standard output.
    Completions {
        /// The shell type to generate completions for.
        #[clap(arg_enum)]
        shell: Shell,
    },
}

#[derive(Args)]
struct CreationFlags {
    /// Name of the new project, using the current working directory if omitted.
    name: Option<String>,
    /// Update all dependencies to the latest compatible version after project creation.
    #[clap(short, long)]
    update_deps: bool,
}

fn main() -> Result<()> {
    let Opt::Hatch(cmd) = Opt::parse();
    let dirs = Utf8ProjectDirs::new()?;

    match cmd {
        Command::Init { name } => {
            let mut cwd = Utf8PathBuf::try_from(env::current_dir()?)?;
            if let Some(name) = name {
                cwd.push(name);
            }

            println!("TODO! init at {}", cwd);
        }
        Command::List => {
            let settings = settings::load_global(&dirs)?;
            let width = settings
                .bookmarks
                .keys()
                .map(|name| name.len())
                .max()
                .unwrap_or_default();

            for (name, info) in settings.bookmarks {
                println!(
                    "{:width$} - {}",
                    name,
                    info.description.as_deref().unwrap_or_default(),
                    width = width,
                );
            }
        }
        Command::New { bookmark, flags } => {
            let settings = settings::load_global(&dirs)?;
            let bookmark = settings
                .bookmarks
                .get(&bookmark)
                .ok_or_else(|| anyhow!("bookmark with name `{}` unknown", bookmark))?;

            let mut path = if bookmark.repository.starts_with("git@")
                || bookmark.repository.starts_with("http:")
                || bookmark.repository.starts_with("https:")
            {
                let path = {
                    let base = dirs.cache_dir();
                    let repo_name = repo::find_repo_name(&bookmark.repository)
                        .context("can't determine repo name from git URL")?;
                    base.join(repo_name)
                };

                fs::create_dir_all(&path)?;

                repo::clone_or_update(&bookmark.repository, &path).context("failed cloning")?;

                path
            } else if fs::metadata(&bookmark.repository)
                .map(|meta| meta.is_dir())
                .unwrap_or_default()
            {
                Utf8PathBuf::from(&bookmark.repository)
            } else {
                bail!("configured bookmark repository doesn't seem to be remote git repo URL nor a local machine folder");
            };

            if let Some(folder) = &bookmark.folder {
                path.push(folder);
            }

            generate_project(&path, flags)?;
            println!("done!");
        }
        Command::Git { folder, url, flags } => {
            let mut path = {
                let base = dirs.cache_dir();
                let repo_name =
                    repo::find_repo_name(&url).context("can't determine repo name from git URL")?;
                base.join(repo_name)
            };

            fs::create_dir_all(&path)?;

            repo::clone_or_update(&url, &path).context("failed cloning")?;

            if let Some(folder) = folder {
                path.push(folder);
            }

            generate_project(&path, flags)?;
            println!("done!");
        }
        Command::Local { path, flags } => {
            generate_project(&path, flags)?;
            println!("done!");
        }
        Command::Completions { shell } => {
            clap_complete::generate(
                shell,
                &mut Opt::command(),
                env!("CARGO_PKG_NAME"),
                &mut io::stdout().lock(),
            );
        }
    }

    Ok(())
}

fn generate_project(path: &Utf8Path, flags: CreationFlags) -> Result<()> {
    let (name, target) = get_target_dir(flags.name).context("failed preparing target directory")?;

    let files = templates::collect_files(path).context("failed collecting files")?;
    let repo_settings = settings::load_repo(path).context("failed loading hatch config")?;

    let mut context =
        settings::new_context(&repo_settings, &name).context("failed creating context")?;
    settings::fill_context(&mut context, repo_settings.args).context("failed filling context")?;

    let files = templates::filter_ignored(files, &context, repo_settings.ignore)?;
    templates::render(&files, &context, &target).context("failed rendering templates")?;

    if flags.update_deps {
        cargo::update_all_cargo_tomls(&target, &files)?;
    }

    repo::init(&target).context("failed initializing git repository")?;

    Ok(())
}

fn get_target_dir(name: Option<String>) -> Result<(String, Utf8PathBuf)> {
    let out = env::current_dir().context("failed getting current directory")?;
    let out = Utf8PathBuf::try_from(out).context("current directory is not valid UTF-8")?;

    let (name, out) = match name {
        Some(name) => {
            let out = out.join(&name);
            (name, out)
        }
        None => (
            out.file_name()
                .context("current directory can't be used as project name")?
                .to_owned(),
            out,
        ),
    };

    let exists = out.exists();
    ensure!(!exists || out.is_dir(), "target path is not a directory");

    if exists {
        let is_empty = out
            .read_dir()
            .context("failed listing directory entries")?
            .next()
            .is_none();
        ensure!(is_empty, "the target directory is not empty");
    }

    Ok((name, out))
}
