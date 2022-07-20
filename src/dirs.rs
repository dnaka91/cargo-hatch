use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use directories::ProjectDirs;

/// Wrapper around [`directories::ProjectDirs`] that provides UTF8-safe paths.
pub struct Utf8ProjectDirs {
    cache_dir: Utf8PathBuf,
    config_dir: Utf8PathBuf,
}

impl Utf8ProjectDirs {
    pub fn new() -> Result<Self> {
        let dirs = ProjectDirs::from("rocks", "dnaka91", "cargo-hatch")
            .context("failed finding project dirs")?;

        let cache_dir = Utf8Path::from_path(dirs.cache_dir())
            .context("project cache dir is not valid UTF8")?
            .to_owned();
        let config_dir = Utf8Path::from_path(dirs.config_dir())
            .context("project config dir is not valid UTF8")?
            .to_owned();

        Ok(Self {
            cache_dir,
            config_dir,
        })
    }

    /// Returns the path to the project’s cache directory.
    #[must_use]
    pub fn cache_dir(&self) -> &Utf8Path {
        &self.cache_dir
    }

    /// Returns the path to the project’s config directory.
    #[must_use]
    pub fn config_dir(&self) -> &Utf8Path {
        &self.config_dir
    }
}
