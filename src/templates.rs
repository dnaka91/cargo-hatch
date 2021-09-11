use std::{
    fs::{self, File},
    io::{BufWriter, Write},
};

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use ignore::WalkBuilder;
use tera::{Context as TeraContext, Tera};

pub struct RepoFile {
    path: Utf8PathBuf,
    name: Utf8PathBuf,
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
            });
        }
    }

    Ok(files)
}

pub fn render(files: Vec<RepoFile>, context: &TeraContext, target: &Utf8Path) -> Result<()> {
    let tera = {
        let mut tera = Tera::default();
        tera.add_template_files(files.iter().map(|f| (&f.path, Some(&f.name))))
            .context("failed loading templates")?;
        tera
    };

    fs::create_dir_all(target)?;

    for file in files.into_iter().map(|f| f.name) {
        if let Some(parent) = file.parent() {
            fs::create_dir_all(target.join(parent))?;
        }

        let mut out = BufWriter::new(File::create(target.join(&file))?);
        tera.render_to(file.as_str(), context, &mut out)?;
        out.flush()?;
    }

    Ok(())
}
