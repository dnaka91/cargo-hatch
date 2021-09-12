use std::{
    fs::{self, File},
    io::{BufWriter, Write},
};

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use ignore::WalkBuilder;
use mime_guess::mime;
use tera::{Context as TeraContext, Tera};

pub struct RepoFile {
    path: Utf8PathBuf,
    name: Utf8PathBuf,
    template: bool,
}

pub fn collect_files(dir: &Utf8Path) -> Result<Vec<RepoFile>> {
    let mut files = Vec::new();
    let walk = WalkBuilder::new(dir)
        .standard_filters(false)
        .git_ignore(true)
        .ignore(true)
        .add_custom_ignore_filename(".hatchignore")
        .filter_entry(|entry| entry.file_name().to_str() != Some(".git"))
        .build();

    for entry in walk {
        let entry = entry?;

        if entry.file_type().map(|ty| ty.is_file()).unwrap_or_default() {
            let path = entry.path();
            let path = Utf8Path::from_path(path)
                .with_context(|| format!("{:?} is not a valid UTF8 path", path))?;
            let name = path.strip_prefix(dir)?;

            if name == ".hatch.toml" || name == ".hatchignore" {
                continue;
            }

            files.push(RepoFile {
                path: path.to_owned(),
                name: name.to_owned(),
                template: !is_binary(name),
            });
        }
    }

    Ok(files)
}

fn is_binary(path: &Utf8Path) -> bool {
    let mime = mime_guess::from_path(path).first_or_text_plain();

    match mime.type_() {
        mime::AUDIO | mime::FONT | mime::IMAGE | mime::VIDEO => true,
        mime::APPLICATION => matches!(mime.subtype(), mime::OCTET_STREAM | mime::PDF),
        _ => false,
    }
}

pub fn render(files: Vec<RepoFile>, context: &TeraContext, target: &Utf8Path) -> Result<()> {
    let tera = {
        let mut tera = Tera::default();
        tera.add_template_files(
            files
                .iter()
                .filter_map(|f| f.template.then(|| (&f.path, Some(&f.name)))),
        )
        .context("failed loading templates")?;
        tera
    };

    fs::create_dir_all(target)?;

    for file in files {
        if let Some(parent) = file.name.parent() {
            fs::create_dir_all(target.join(parent))?;
        }

        if file.template {
            let mut out = BufWriter::new(File::create(target.join(&file.name))?);
            tera.render_to(file.name.as_str(), context, &mut out)?;
            out.flush()?;
        } else {
            fs::copy(file.path, target.join(file.name))?;
        }
    }

    Ok(())
}
