use std::{convert::TryFrom, env, fs, io};

use anyhow::{anyhow, bail, ensure, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_hatch::{dirs::Utf8ProjectDirs, repo, settings, templates};
use structopt::{
    clap::{AppSettings, Shell},
    StructOpt,
};

#[derive(StructOpt)]
#[structopt(
    about,
    author,
    global_setting = AppSettings::ColoredHelp,
    global_setting = AppSettings::DeriveDisplayOrder,
    global_setting = AppSettings::VersionlessSubcommands,
)]
enum Opt {
    #[structopt(about)]
    Hatch(Command),
}

#[derive(StructOpt)]
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
        /// Name of the new project, using the current working director if omitted.
        name: Option<String>,
    },
    /// Create a new project from a template located in a remote Git repository.
    Git {
        /// An optional sub-folder within the repository that contains the template.
        #[structopt(long)]
        folder: Option<Utf8PathBuf>,
        /// HTTP or Git URL to the remote repository.
        url: String,
        /// Name of the new project, using the current working director if omitted.
        name: Option<String>,
    },
    /// Create a new project from a template located in the local file system.
    Local {
        /// Location of the template directory.
        path: Utf8PathBuf,
        /// Name of the new project, using the current working director if omitted.
        name: Option<String>,
    },
    /// Generate shell completions for cargo-hatch, writing them to the standard output.
    Completions {
        /// The shell type to generate completions for.
        #[structopt(possible_values = &Shell::variants())]
        shell: Shell,
    },
}

fn main() -> Result<()> {
    let Opt::Hatch(cmd) = Opt::from_args();
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
        Command::New { bookmark, name } => {
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

            generate_project(&path, name)?;
            println!("done!");
        }
        Command::Git { folder, url, name } => {
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

            generate_project(&path, name)?;
            println!("done!");
        }
        Command::Local { path, name } => {
            generate_project(&path, name)?;
            println!("done!");
        }
        Command::Completions { shell } => {
            Opt::clap().gen_completions_to(env!("CARGO_PKG_NAME"), shell, &mut io::stdout().lock());
        }
    }

    Ok(())
}

fn generate_project(path: &Utf8Path, name: Option<String>) -> Result<()> {
    let (name, target) = get_target_dir(name).context("failed preparing target directory")?;

    let files = templates::collect_files(path).context("failed collecting files")?;
    let repo_settings = settings::load_repo(path).context("failed loading hatch config")?;

    let mut context =
        settings::new_context(&repo_settings, &name).context("failed creating context")?;
    settings::fill_context(&mut context, repo_settings).context("failed filling context")?;

    templates::render(files, &context, &target).context("failed rendering templates")?;
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
            .is_some();
        ensure!(is_empty, "the target directory is not empty");
    }

    Ok((name, out))
}
