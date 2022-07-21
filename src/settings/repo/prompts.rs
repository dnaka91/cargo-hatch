use std::collections::HashSet;

use anyhow::{anyhow, bail, Result};
use crossterm::style::Stylize;
use inquire::{Confirm, CustomType, MultiSelect, Select, Text};

use super::{
    validators, BoolSetting, ListSetting, MultiListSetting, Number, NumberSetting, StringSetting,
    StringValidator,
};

pub fn prompt_bool(
    description: &str,
    setting: &BoolSetting,
    default: Option<bool>,
) -> Result<bool> {
    fn default_value_formatter(value: bool) -> String {
        if value {
            format!("{}/n", "Y".green())
        } else {
            format!("y/{}", "N".red())
        }
    }

    let mut prompt =
        Confirm::new(description).with_default_value_formatter(&default_value_formatter);
    prompt.default = default.or(setting.default);

    prompt.prompt().map_err(Into::into)
}

pub fn prompt_string(
    description: &str,
    setting: StringSetting,
    default: Option<&str>,
) -> Result<String> {
    let validator: Box<dyn Fn(&str) -> Result<(), String>> = match setting.validator {
        None => Box::new(validators::required),
        Some(StringValidator::Crate) => Box::new(validators::krate),
        Some(StringValidator::Ident) => Box::new(validators::ident),
        Some(StringValidator::Semver) => Box::new(validators::semver),
        Some(StringValidator::SemverReq) => Box::new(validators::semver_req),
        Some(StringValidator::Regex(re)) => Box::new(validators::regex(re)),
    };

    if let Some(default) = default {
        validator(default).map_err(|err| anyhow!("Invalid default `{default}` {err}"))?;
    }

    let mut prompt = Text::new(description).with_validator(&*validator);
    prompt.default = default.or(setting.default.as_deref());

    prompt.prompt().map_err(Into::into)
}

pub fn prompt_number<T: Number>(
    description: &str,
    setting: &NumberSetting<T>,
    default: Option<T>,
) -> Result<T> {
    fn parser<T: Number>(value: &str, min: T, max: T) -> Result<T, ()> {
        match value.parse() {
            Ok(v) if (min..=max).contains(&v) => Ok(v),
            Ok(_) | Err(_) => Err(()),
        }
    }

    fn formatter<T: Number>(value: T) -> String {
        value.to_string()
    }

    if let Some(default) = default {
        if !(setting.min..=setting.max).contains(&default) {
            bail!(
                "Invalid default `{default}` value is not in range {} to {}",
                setting.min,
                setting.max
            );
        }
    }

    let parser = |value: &str| parser(value, setting.min, setting.max);
    let placeholder = format!("{}..={}", setting.min, setting.max);
    let help_message = format!("Number in range {}..={}", setting.min, setting.max);

    let mut prompt = CustomType::<T>::new(description)
        .with_parser(&parser)
        .with_placeholder(&placeholder)
        .with_help_message(&help_message)
        .with_error_message("Please type a valid number within range.");

    if let Some(default) = default.or(setting.default) {
        prompt.default = Some((default, &formatter));
    }

    prompt.prompt().map_err(Into::into)
}

pub fn prompt_list(
    description: &str,
    setting: ListSetting,
    default: Option<&str>,
) -> Result<String> {
    let default = setting
        .values
        .iter()
        // Maybe use `and_then?`
        .position(|v| Some(v.as_str()) == default.or(setting.default.as_deref()))
        .unwrap_or_default();

    let prompt = Select::new(description, setting.values.into_iter().collect())
        .with_starting_cursor(default);

    prompt.prompt().map_err(Into::into)
}

pub fn prompt_multi_list(
    description: &str,
    setting: MultiListSetting,
    default: Option<&HashSet<String>>,
) -> Result<Vec<String>> {
    let (index, selection) = if let Some(default) = default.or(setting.default.as_ref()) {
        let index = setting
            .values
            .iter()
            .position(|value| default.contains(value))
            .unwrap_or_default();
        let selection = setting
            .values
            .iter()
            .enumerate()
            .filter_map(|(i, value)| default.contains(value).then(|| i))
            .collect();

        (index, selection)
    } else {
        (0, Vec::new())
    };

    let prompt = MultiSelect::new(description, setting.values.into_iter().collect())
        .with_starting_cursor(index)
        .with_default(&selection);

    prompt.prompt().map_err(Into::into)
}
