//! Main templating handling, which involves finding files, filtering them and finally rendering
//! them to a target directory.
//!
//! Rendering means, that the file is processed through the [`Tera`] templating engine, in case it
//! is considered a template file.

use std::{
    fs::{self, File},
    io::{BufWriter, Write},
};

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use mime_guess::mime;
use tera::{Context as TeraContext, Tera};

use crate::settings::{IgnoreFrom, IgnorePattern};

/// A single file from a template repository, that shall be rendered into a target directory. If it
/// is considered a template, it's processed through the [`Tera`] engine.
pub struct RepoFile {
    /// Full path to the file for reading.
    path: Utf8PathBuf,
    /// Relative path in regards to the directory it came from.
    name: Utf8PathBuf,
    /// Whether the file matches any ignore pattern
    ignore_from: Option<IgnoreFrom>,
}

impl RepoFile {
    /// Path of this file, relative to the directory it was loaded from.
    #[must_use]
    pub fn name(&self) -> &Utf8Path {
        &self.name
    }
}

pub fn collect_files(dir: &Utf8Path) -> Result<Vec<RepoFile>> {
    // Builtin filters for files or dirs that are always ignored
    static FILTERS: &[&str] = &[".git", ".hatch.toml", ".hatchignore"];

    let mut files = Vec::new();
    let walk = WalkBuilder::new(dir)
        .standard_filters(false)
        .git_ignore(true)
        .ignore(true)
        .add_custom_ignore_filename(".hatchignore")
        .filter_entry(|entry| {
            entry
                .file_name()
                .to_str()
                .map_or(false, |name| !FILTERS.contains(&name))
        })
        .build();

    for entry in walk {
        let entry = entry?;

        if entry.file_type().map_or(false, |ty| ty.is_file()) {
            let path = entry.path();
            let path = Utf8Path::from_path(path)
                .with_context(|| format!("{path:?} is not a valid UTF8 path"))?;
            let name = path
                .strip_prefix(dir)
                .with_context(|| format!("failed to get relative path for {path}"))?;

            files.push(RepoFile {
                path: path.to_owned(),
                name: name.to_owned(),
                ignore_from: is_binary(name).then_some(IgnoreFrom::Template),
            });
        }
    }

    Ok(files)
}

/// Determine, whether the given path is considered a binary file, that should not be treated as
/// template in further processing.
fn is_binary(path: &Utf8Path) -> bool {
    let mime = mime_guess::from_path(path).first_or_text_plain();

    match mime.type_() {
        mime::AUDIO | mime::FONT | mime::IMAGE | mime::VIDEO => true,
        mime::APPLICATION => matches!(mime.subtype(), mime::OCTET_STREAM | mime::PDF),
        _ => false,
    }
}

/// Filter out the collected files from [`collect_files`] with the given ignore rules.
pub fn filter_ignored(
    files: Vec<RepoFile>,
    context: &TeraContext,
    ignore: &[IgnorePattern],
) -> Result<Vec<RepoFile>> {
    let template_ignore = file_ignore_to_glob_set(context, ignore, IgnoreFrom::Template)?;
    let render_ignore = file_ignore_to_glob_set(context, ignore, IgnoreFrom::Render)?;
    let all_ignore = file_ignore_to_glob_set(context, ignore, IgnoreFrom::All)?;

    Ok(files
        .into_iter()
        .filter_map(|mut file| {
            if all_ignore.is_match(file.name()) {
                None
            } else {
                if template_ignore.is_match(file.name()) {
                    file.ignore_from = Some(IgnoreFrom::Template);
                } else if render_ignore.is_match(file.name()) {
                    file.ignore_from = Some(IgnoreFrom::Render);
                }
                Some(file)
            }
        })
        .collect())
}

fn file_ignore_to_glob_set(
    context: &TeraContext,
    ignore: &[IgnorePattern],
    ignore_filter: IgnoreFrom,
) -> Result<GlobSet> {
    let mut set = GlobSetBuilder::new();
    for rule in ignore
        .iter()
        .filter(|rule| rule.ignore_from == ignore_filter)
    {
        if let Some(condition) = &rule.condition {
            let result = Tera::one_off(condition, context, false)?;
            let active = result.trim().parse::<bool>()?;
            if !active {
                continue;
            }
        }

        for path in &rule.paths {
            set.add(
                GlobBuilder::new(path.as_str())
                    .literal_separator(true)
                    .build()
                    .with_context(|| format!("invalid glob pattern `{path}`"))?,
            );
        }
    }

    set.build().context("failed to build the glob set")
}

/// Render all the given files to the target path.
///
/// - If the a file is a template, it is processed through the [`Tera`] engine.
/// - Otherwise, it's copied as-is, without any further processing.
pub fn render(files: &[RepoFile], context: &TeraContext, target: &Utf8Path) -> Result<()> {
    let tera = {
        let mut tera = Tera::default();
        tera.add_template_files(files.iter().filter_map(|f| {
            matches!(f.ignore_from, None | Some(IgnoreFrom::Render))
                .then_some((&f.path, Some(&f.name)))
        }))
        .context("failed loading templates")?;
        tera
    };

    fs::create_dir_all(target)?;

    for file in files {
        if let Some(parent) = file.name.parent() {
            fs::create_dir_all(target.join(parent))
                .with_context(|| format!("failed to directories for `{parent}`"))?;
        }

        match file.ignore_from {
            None => {
                let mut out = BufWriter::new(File::create(target.join(&file.name))?);
                tera.render_to(file.name.as_str(), context, &mut out)
                    .with_context(|| format!("failed to render template for `{}`", file.name))?;
                out.flush().context("failed to flush output file")?;
            }
            Some(IgnoreFrom::Template) => {
                fs::copy(&file.path, target.join(&file.name))
                    .with_context(|| format!("faile to copy file `{}`", file.name))?;
            }
            _ => {}
        }
    }

    Ok(())
}
