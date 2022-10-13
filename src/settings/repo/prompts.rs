#![allow(clippy::needless_pass_by_value)]

use std::collections::HashSet;

use anyhow::Result;
use crossterm::style::Stylize;
use inquire::{Confirm, CustomType, MultiSelect, Select, Text};

use super::{
    validators, BoolSetting, ListSetting, MultiListSetting, Number, NumberSetting, StringSetting,
    StringValidator,
};

pub fn prompt_bool(description: &str, setting: BoolSetting) -> Result<bool> {
    fn default_value_formatter(value: bool) -> String {
        if value {
            format!("{}/n", "Y".green())
        } else {
            format!("y/{}", "N".red())
        }
    }

    let mut prompt =
        Confirm::new(description).with_default_value_formatter(&default_value_formatter);
    prompt.default = setting.default;

    prompt.prompt().map_err(Into::into)
}

#[allow(clippy::type_complexity)]
pub fn prompt_string(description: &str, setting: StringSetting) -> Result<String> {
    let mut prompt = Text::new(description);
    prompt.default = setting.default.as_deref();

    let prompt = match setting.validator {
        None => prompt.with_validator(validators::Required),
        Some(StringValidator::Crate) => prompt.with_validator(validators::Krate),
        Some(StringValidator::Ident) => prompt.with_validator(validators::Ident),
        Some(StringValidator::Semver) => prompt.with_validator(validators::Semver),
        Some(StringValidator::SemverReq) => prompt.with_validator(validators::SemverReq),
        Some(StringValidator::Regex(re)) => prompt.with_validator(validators::Regex(re)),
    };

    prompt.prompt().map_err(Into::into)
}

pub fn prompt_number<T: Number>(description: &str, setting: NumberSetting<T>) -> Result<T> {
    fn parser<T: Number>(value: &str, min: T, max: T) -> Result<T, ()> {
        match value.parse() {
            Ok(v) if (min..=max).contains(&v) => Ok(v),
            Ok(_) | Err(_) => Err(()),
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

    prompt.default = setting.default;

    prompt.prompt().map_err(Into::into)
}

pub fn prompt_list(description: &str, setting: ListSetting) -> Result<String> {
    let default = setting
        .values
        .iter()
        .position(|v| Some(v) == setting.default.as_ref())
        .unwrap_or_default();

    let prompt = Select::new(description, setting.values.into_iter().collect())
        .with_starting_cursor(default);

    prompt.prompt().map_err(Into::into)
}

pub fn prompt_multi_list(description: &str, setting: MultiListSetting) -> Result<HashSet<String>> {
    let (index, selection) = if let Some(default) = setting.default.as_ref() {
        let index = setting
            .values
            .iter()
            .position(|value| default.contains(value))
            .unwrap_or_default();
        let selection = setting
            .values
            .iter()
            .enumerate()
            .filter_map(|(i, value)| default.contains(value).then_some(i))
            .collect();

        (index, selection)
    } else {
        (0, Vec::new())
    };

    let prompt = MultiSelect::new(description, setting.values.into_iter().collect())
        .with_starting_cursor(index)
        .with_default(&selection);

    prompt
        .prompt()
        .map(|v| v.into_iter().collect())
        .map_err(Into::into)
}
