use std::collections::BTreeMap;

use anyhow::Result;
use camino::Utf8PathBuf;
use serde::Deserialize;

use crate::dirs::Utf8ProjectDirs;

#[derive(Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub git: Git,
    #[serde(default)]
    pub bookmarks: BTreeMap<String, Bookmark>,
}

#[derive(Default, Deserialize)]
pub struct Git {
    pub ssh_key: Option<Utf8PathBuf>,
}

#[derive(Deserialize)]
pub struct Bookmark {
    pub repository: String,
    pub description: Option<String>,
    pub folder: Option<Utf8PathBuf>,
}

pub fn load(dirs: &Utf8ProjectDirs) -> Result<Settings> {
    let buf = std::fs::read(dirs.config_dir().join("settings.toml"))?;
    toml::from_slice(&buf).map_err(Into::into)
}
