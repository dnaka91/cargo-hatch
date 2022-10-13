#![allow(clippy::needless_pass_by_value)]

use std::collections::HashSet;

use anyhow::{bail, Result};

use crate::settings::global::{DefaultSetting, DefaultValue};

pub fn get_bool(default: DefaultSetting) -> Result<bool> {
    if let DefaultValue::Bool(value) = default.value {
        Ok(value)
    } else {
        bail!(
            "invalid default value for boolean setting ({:?})",
            default.value
        )
    }
}

pub fn get_string(default: DefaultSetting) -> Result<String> {
    if let DefaultValue::String(value) = default.value {
        Ok(value)
    } else {
        bail!(
            "invalid default value for string setting ({:?}",
            default.value
        )
    }
}

pub fn get_number(default: DefaultSetting) -> Result<i64> {
    if let DefaultValue::Number(value) = default.value {
        Ok(value)
    } else {
        bail!(
            "invalid default value for number setting ({:?}",
            default.value
        )
    }
}

pub fn get_float(default: DefaultSetting) -> Result<f64> {
    if let DefaultValue::Float(value) = default.value {
        Ok(value)
    } else {
        bail!(
            "invalid default value for float setting ({:?}",
            default.value
        )
    }
}

pub fn get_list(default: DefaultSetting) -> Result<String> {
    if let DefaultValue::List(value) = default.value {
        Ok(value)
    } else {
        bail!(
            "invalid default value for list setting ({:?}",
            default.value
        )
    }
}

pub fn get_multi_list(default: DefaultSetting) -> Result<HashSet<String>> {
    if let DefaultValue::MultiList(value) = default.value {
        Ok(value)
    } else {
        bail!(
            "invalid default value for multi-list setting ({:?}",
            default.value
        )
    }
}
