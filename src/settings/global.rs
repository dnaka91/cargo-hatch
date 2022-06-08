use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::{bail, Result};
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
    pub defaults: HashMap<String, DefaultValue>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DefaultValue {
    Bool(bool),
    String(String),
    Number(i64),
    Float(f64),
    List(HashSet<String>),
}

impl DefaultValue {
    fn error<T>(&self, name: &str, ty: &str) -> Result<T> {
        bail!("Invalid default value for `{name}`. Expected `{ty}` got `{self:?}`.")
    }
    pub fn expect_bool(&self, name: &str) -> Result<bool> {
        match self {
            Self::Bool(value) => Ok(*value),
            _ => self.error(name, "bool"),
        }
    }
    pub fn expect_string(&self, name: &str) -> Result<&str> {
        match self {
            Self::String(value) => Ok(value),
            _ => self.error(name, "string"),
        }
    }
    pub fn expect_number(&self, name: &str) -> Result<i64> {
        match self {
            Self::Number(value) => Ok(*value),
            _ => self.error(name, "integer"),
        }
    }
    pub fn expect_float(&self, name: &str) -> Result<f64> {
        match self {
            Self::Float(value) => Ok(*value),
            _ => self.error(name, "float"),
        }
    }
    pub fn expect_list(&self, name: &str) -> Result<&HashSet<String>> {
        match self {
            Self::List(value) => Ok(value),
            _ => self.error(name, "[string]"),
        }
    }
}

pub fn load(dirs: &Utf8ProjectDirs) -> Result<Settings> {
    let buf = std::fs::read(dirs.config_dir().join("settings.toml"))?;
    toml::from_slice(&buf).map_err(Into::into)
}
