use std::collections::{BTreeMap, HashMap, HashSet};

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
    #[serde(default)]
    pub update_deps: bool,
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
    #[serde(default)]
    pub defaults: HashMap<String, DefaultSetting>,
}

#[derive(Deserialize)]
pub struct DefaultSetting {
    pub value: DefaultValue,
    pub skip_prompt: bool,
}

#[derive(Debug, Deserialize)]
pub enum DefaultValue {
    Bool(bool),
    String(String),
    Number(i64),
    Float(f64),
    List(String),
    MultiList(HashSet<String>),
}

pub fn load(dirs: &Utf8ProjectDirs) -> Result<Settings> {
    let buf = std::fs::read(dirs.config_dir().join("settings.toml"))?;
    toml::from_slice(&buf).map_err(Into::into)
}
