use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::Result;
use camino::Utf8PathBuf;
use serde::Deserialize;

use crate::dirs::Utf8ProjectDirs;

#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Settings {
    #[serde(default)]
    pub git: Git,
    #[serde(default)]
    pub bookmarks: BTreeMap<String, Bookmark>,
    #[serde(default)]
    pub update_deps: bool,
}

#[derive(Default, Deserialize)]
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct Git {
    pub ssh_key: Option<Utf8PathBuf>,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Bookmark {
    pub repository: String,
    pub description: Option<String>,
    pub folder: Option<Utf8PathBuf>,
    #[serde(default)]
    pub defaults: HashMap<String, DefaultSetting>,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct DefaultSetting {
    pub value: DefaultValue,
    pub skip_prompt: bool,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "snake_case")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let raw = r#"
        update_deps = true

        [git]
        ssh_key = ".ssh/id_ed25519"

        [bookmarks.server]
        repository = "test"
        description = "sample"
        folder = "a/b/c"

        [bookmarks.server.defaults]
        test_bool = { value = { bool = true }, skip_prompt = false }
        test_string = { value = { string = "value" }, skip_prompt = false }
        test_number = { value = { number = 10 }, skip_prompt = false }
        test_float = { value = { float = 2.5 }, skip_prompt = false }
        test_list = { value = { list = "one" }, skip_prompt = true }
        test_multi_list = { value = { multi_list = ["one", "two"] }, skip_prompt = true }
    "#;
        let expect = Settings {
            git: Git {
                ssh_key: Some(Utf8PathBuf::from(".ssh/id_ed25519")),
            },
            bookmarks: [(
                "server".to_owned(),
                Bookmark {
                    repository: "test".to_owned(),
                    description: Some("sample".to_owned()),
                    folder: Some(Utf8PathBuf::from("a/b/c")),
                    defaults: [
                        (
                            "test_bool".to_owned(),
                            DefaultSetting {
                                value: DefaultValue::Bool(true),
                                skip_prompt: false,
                            },
                        ),
                        (
                            "test_string".to_owned(),
                            DefaultSetting {
                                value: DefaultValue::String("value".to_owned()),
                                skip_prompt: false,
                            },
                        ),
                        (
                            "test_number".to_owned(),
                            DefaultSetting {
                                value: DefaultValue::Number(10),
                                skip_prompt: false,
                            },
                        ),
                        (
                            "test_float".to_owned(),
                            DefaultSetting {
                                value: DefaultValue::Float(2.5),
                                skip_prompt: false,
                            },
                        ),
                        (
                            "test_list".to_owned(),
                            DefaultSetting {
                                value: DefaultValue::List("one".to_owned()),
                                skip_prompt: true,
                            },
                        ),
                        (
                            "test_multi_list".to_owned(),
                            DefaultSetting {
                                value: DefaultValue::MultiList(
                                    ["one".to_owned(), "two".to_owned()].into_iter().collect(),
                                ),
                                skip_prompt: true,
                            },
                        ),
                    ]
                    .into_iter()
                    .collect(),
                },
            )]
            .into_iter()
            .collect(),
            update_deps: true,
        };

        let result = toml::from_str::<Settings>(raw);
        assert_eq!(expect, result.unwrap());
    }
}
