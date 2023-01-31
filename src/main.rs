use std::{collections::HashMap, convert::TryFrom, env, fs};

use anyhow::{anyhow, bail, ensure, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_hatch::{
    cargo,
    cli::{self, Command, CreationFlags},
    dirs::Utf8ProjectDirs,
    repo,
    settings::{self, DefaultSetting},
    templates,
};
use inquire::Confirm;

fn main() -> Result<()> {
    let cmd = cli::parse();
    let dirs = Utf8ProjectDirs::new()?;

    match cmd {
        Command::Init { name } => {
            let mut cwd = Utf8PathBuf::try_from(env::current_dir()?)?;
            if let Some(name) = name {
                cwd.push(name);
            }

            println!("TODO! init at {cwd}");
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
                    "{name:width$} - {}",
                    info.description.as_deref().unwrap_or_default(),
                );
            }
        }
        Command::New { bookmark, flags } => {
            let mut settings = settings::load_global(&dirs)?;
            let bookmark = settings
                .bookmarks
                .remove(&bookmark)
                .ok_or_else(|| anyhow!("bookmark with name `{bookmark}` unknown"))?;

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

            generate_project(&path, flags, bookmark.defaults)?;
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

            generate_project(&path, flags, HashMap::new())?;
            println!("done!");
        }
        Command::Local { path, flags } => {
            generate_project(&path, flags, HashMap::new())?;
            println!("done!");
        }
        Command::Completions { shell } => cli::completions(shell),
        Command::Manpages { dir } => cli::manpages(&dir)?,
    }

    Ok(())
}

fn generate_project(
    path: &Utf8Path,
    flags: CreationFlags,
    defaults: HashMap<String, DefaultSetting>,
) -> Result<()> {
    let (name, target) = get_target_dir(flags.name).context("failed preparing target directory")?;

    let files = templates::collect_files(path).context("failed collecting files")?;
    let repo_settings = settings::load_repo(path).context("failed loading hatch config")?;

    let mut context =
        settings::new_context(&repo_settings, &name).context("failed creating context")?;
    settings::fill_context(&mut context, repo_settings.args, defaults)
        .context("failed filling context")?;

    let files = templates::filter_ignored(files, &context, repo_settings.ignore)?;
    templates::render(&files, &context, &target).context("failed rendering templates")?;

    if flags.update_deps {
        cargo::update_all_cargo_tomls(&target, &files)?;
    }

    repo::init(&target).context("failed initializing git repository")?;

    Ok(())
}

/// Locate the target directory, and ensure it is suitable for project generation.
fn get_target_dir(name: Option<Utf8PathBuf>) -> Result<(String, Utf8PathBuf)> {
    let out = env::current_dir().context("failed getting current directory")?;
    let mut out = Utf8PathBuf::try_from(out).context("current directory is not valid UTF-8")?;

    if let Some(name) = name {
        out.push(name);
    }

    let name = out
        .file_name()
        .context("directory can't be used as project name")?
        .to_owned();

    ensure!(
        !out.exists() || out.is_dir(),
        "target diretory appears to be an existing file"
    );

    if out.exists() && !is_dir_empty(&out)? {
        let mut prompt = Confirm::new("target directory already exists. Do you want to continue?");
        prompt.default = Some(false);
        prompt.help_message = Some("if you continue, the directory will be cleared beforehand");

        if prompt.prompt().context("failed to prompt for user input")? {
            fs::remove_dir_all(&out).context("failed clearing output directory")?;
        } else {
            bail!("generation cancelled by user");
        }
    }

    Ok((name, out))
}

fn is_dir_empty(path: &Utf8Path) -> Result<bool> {
    Ok(path
        .read_dir()
        .context("failed listing directory entries")?
        .next()
        .is_none())
}
