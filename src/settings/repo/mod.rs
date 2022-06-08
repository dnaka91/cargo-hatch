use std::{collections::HashSet, fmt::Display, fs, iter::FromIterator, str::FromStr};

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use git2::Config as GitConfig;
use indexmap::{IndexMap, IndexSet};
use num_traits::Num;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tera::Context as TeraContext;

mod prompts;
mod validators;

#[derive(Deserialize)]
pub struct RepoSettings {
    crate_type: Option<CrateType>,
    #[serde(default)]
    pub ignore: Vec<FileIgnore>,
    #[serde(flatten)]
    pub args: IndexMap<String, RepoSetting>,
}

#[derive(Deserialize)]
pub struct FileIgnore {
    #[serde(with = "serde_with::As::<serde_with::OneOrMany<serde_with::Same>>")]
    pub paths: Vec<Utf8PathBuf>,
    pub condition: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CrateType {
    Bin,
    Lib,
}

#[derive(Deserialize)]
pub struct RepoSetting {
    description: String,
    #[serde(flatten)]
    ty: SettingType,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SettingType {
    Bool(BoolSetting),
    String(StringSetting),
    Number(NumberSetting<i64>),
    Float(NumberSetting<f64>),
    List(ListSetting),
    MultiList(MultiListSetting),
}

#[derive(Deserialize)]
pub struct BoolSetting {
    default: Option<bool>,
}

#[derive(Deserialize)]
pub struct StringSetting {
    default: Option<String>,
    validator: Option<StringValidator>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StringValidator {
    Crate,
    Ident,
    Semver,
    SemverReq,
    #[serde(with = "serde_with::rust::display_fromstr")]
    Regex(Regex),
}

pub trait Number: Num + Copy + Display + FromStr + PartialOrd + Serialize {}

impl<T: Num + Copy + Display + FromStr + PartialOrd + Serialize> Number for T {}

#[derive(Deserialize)]
pub struct NumberSetting<T: Number> {
    min: T,
    max: T,
    default: Option<T>,
}

#[derive(Deserialize)]
pub struct ListSetting {
    values: IndexSet<String>,
    default: Option<String>,
}

#[derive(Deserialize)]
pub struct MultiListSetting {
    values: IndexSet<String>,
    default: Option<HashSet<String>>,
}

impl RepoSetting {
    /// Check the setting for invalid values and return a error message describing the problem if
    /// an invalid configuration was found.
    #[must_use]
    pub fn validate(&self) -> Option<&'static str> {
        match &self.ty {
            SettingType::Bool(_) | SettingType::String(_) => None,
            SettingType::Number(setting) => Self::validate_number(setting),
            SettingType::Float(setting) => Self::validate_number(setting),
            SettingType::List(ListSetting { values, default }) => {
                default.as_ref().and_then(|default| {
                    (!values.contains(default))
                        .then(|| "default value isn't part of the possible values")
                })
            }
            SettingType::MultiList(MultiListSetting { values, default }) => {
                default.as_ref().and_then(|default| {
                    default
                        .iter()
                        .any(|def| !values.contains(def))
                        .then(|| "one of the default values isn't part of the possible values")
                })
            }
        }
    }

    fn validate_number<T: Number>(
        NumberSetting { min, max, default }: &NumberSetting<T>,
    ) -> Option<&'static str> {
        (min >= max)
            .then(|| "minimum is greater or equal the maximum value")
            .or_else(|| {
                default
                    .as_ref()
                    .map(|d| !(*min..*max).contains(d))
                    .and_then(|invalid| {
                        invalid.then(|| "default value is not within the min/max range")
                    })
            })
    }
}

pub fn load(path: &Utf8Path) -> Result<RepoSettings> {
    let buf = fs::read(path.join(".hatch.toml")).context("failed reading hatch config file")?;
    let settings = toml::from_slice::<RepoSettings>(&buf).context("invalid hatch settings")?;

    if let Some((name, error)) = settings
        .args
        .iter()
        .find_map(|(name, setting)| setting.validate().map(|error| (name, error)))
    {
        bail!("invalid setting `{}`: {}", name, error);
    }

    Ok(settings)
}

pub fn new_context(settings: &RepoSettings, project_name: &str) -> Result<TeraContext> {
    let mut ctx = TeraContext::new();

    ctx.try_insert("project_name", &project_name)
        .context("failed adding value to context")?;

    let config = GitConfig::open_default()
        .context("failed opening default git config")?
        .snapshot()
        .context("failed creating git config snapshot")?;

    let name = config
        .get_str("user.name")
        .context("failed getting name from git config")?;
    let email = config
        .get_str("user.email")
        .context("failed getting email from git config")?;

    ctx.try_insert("git_author", &format!("{} <{}>", name, email))
        .context("failed adding value to context")?;
    ctx.try_insert("git_name", &name)
        .context("failed adding value to context")?;
    ctx.try_insert("git_email", &email)
        .context("failed adding value to context")?;

    let crate_type = if let Some(ty) = settings.crate_type {
        ty
    } else {
        let setting = ListSetting {
            values: IndexSet::from_iter(["bin".to_owned(), "lib".to_owned()]),
            default: None,
        };
        match prompts::prompt_list("what crate type would you like to create?", setting)?.as_ref() {
            "bin" => CrateType::Bin,
            "lib" => CrateType::Lib,
            _ => unreachable!(),
        }
    };

    ctx.try_insert("crate_type", &crate_type)
        .context("failed adding value to context")?;
    ctx.try_insert("crate_bin", &(crate_type == CrateType::Bin))
        .context("failed adding value to context")?;
    ctx.try_insert("crate_lib", &(crate_type == CrateType::Lib))
        .context("failed adding value to context")?;

    Ok(ctx)
}

pub fn fill_context(ctx: &mut TeraContext, args: IndexMap<String, RepoSetting>) -> Result<()> {
    for (name, setting) in args {
        match setting.ty {
            SettingType::Bool(value) => {
                let value = prompts::prompt_bool(&setting.description, &value)?;

                ctx.try_insert(name, &value)
                    .context("failed adding value to context")?;
            }
            SettingType::String(value) => {
                let value = prompts::prompt_string(&setting.description, value)?;

                ctx.try_insert(name, &value)
                    .context("failed adding value to context")?;
            }
            SettingType::Number(value) => {
                let value = prompts::prompt_number(&setting.description, &value)?;

                ctx.try_insert(name, &value)
                    .context("failed adding value to context")?;
            }
            SettingType::Float(value) => {
                let value = prompts::prompt_number(&setting.description, &value)?;

                ctx.try_insert(name, &value)
                    .context("failed adding value to context")?;
            }
            SettingType::List(value) => {
                let value = prompts::prompt_list(&setting.description, value)?;

                ctx.try_insert(name, &value)
                    .context("failed adding value to context")?;
            }
            SettingType::MultiList(value) => {
                let value = prompts::prompt_multi_list(&setting.description, value)?;

                ctx.try_insert(name, &value)
                    .context("failed adding value to context")?;
            }
        }
    }

    Ok(())
}
