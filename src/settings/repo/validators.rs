//! Validators that ensure additional restrictions on input data.

use regex::Regex;

// Validate that a value is not empty.
pub fn required(value: &str) -> Result<(), String> {
    if value.is_empty() {
        Err("A response is required.".to_owned())
    } else {
        Ok(())
    }
}

// Validate that a value is a crate name, according to crates.io rules.
pub fn krate(value: &str) -> Result<(), String> {
    const MAX_NAME_LENGTH: usize = 64;

    if value.chars().take(MAX_NAME_LENGTH + 1).count() <= MAX_NAME_LENGTH
        && value.chars().next().map_or(false, char::is_alphabetic)
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        Ok(())
    } else {
        Err("value must be a valid crate name".to_owned())
    }
}

// Validate that a value is a Rust identifier.
pub fn ident(value: &str) -> Result<(), String> {
    use check_keyword::CheckKeyword;

    fn is_ident(value: &str) -> bool {
        value
            .chars()
            .next()
            .map_or(false, unicode_ident::is_xid_start)
            && value.chars().skip(1).all(unicode_ident::is_xid_continue)
    }

    fn is_escaped_ident(value: &str) -> bool {
        value.chars().next().map_or(false, |c| c == '_')
            && value.chars().take(2).count() >= 2
            && value.chars().skip(1).all(unicode_ident::is_xid_continue)
    }

    if (is_ident(value) || is_escaped_ident(value)) && !value.is_keyword() {
        Ok(())
    } else {
        Err("value must be a valid Rust identifier".to_owned())
    }
}

// Validate that a value is a semantic version.
pub fn semver(value: &str) -> Result<(), String> {
    match value.parse::<semver::Version>() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("value is not a valid sematic version: {e}")),
    }
}

// Validate that a value is a semantic version requirement specification.
pub fn semver_req(value: &str) -> Result<(), String> {
    match value.parse::<semver::VersionReq>() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("value is not a valid sematic version spec: {e}")),
    }
}

// Validate that a value matches against the given Regular Expression.
pub fn regex(regex: Regex) -> impl Fn(&str) -> Result<(), String> {
    move |value| {
        if regex.is_match(value) {
            Ok(())
        } else {
            Err(format!("value must match regex pattern `{regex}`"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_required() {
        assert!(required("a").is_ok());
        assert!(required("").is_err());
    }

    #[test]
    fn validate_krate() {
        assert!(krate("anyhow").is_ok());
        assert!(krate("tower-http").is_ok());
        assert!(krate("tower_http").is_ok());
        assert!(krate("?").is_err());
        assert!(krate("").is_err());
    }

    #[test]
    fn validate_ident() {
        assert!(ident("test").is_ok());
        assert!(ident("TEST").is_ok());
        assert!(ident("teST").is_ok());
        assert!(ident("Test").is_ok());
        assert!(ident("_test").is_ok());
        assert!(ident("_TEST").is_ok());
        assert!(ident("_").is_err());
        assert!(ident("?").is_err());
        assert!(ident("").is_err());
    }

    #[test]
    fn validate_semver() {
        assert!(semver("1.0.0").is_ok());
        assert!(semver("1.0.0-beta.1").is_ok());
        assert!(semver("1.0.0+abc").is_ok());
        assert!(semver("1.0").is_err());
        assert!(semver("1").is_err());
        assert!(semver("").is_err());
    }

    #[test]
    fn validate_semver_req() {
        assert!(semver_req("1.0.0").is_ok());
        assert!(semver_req("1.0.0-beta.1").is_ok());
        assert!(semver_req("1.0.0+abc").is_ok());
        assert!(semver_req("1.0").is_ok());
        assert!(semver_req("1").is_ok());
        assert!(semver_req("").is_err());
    }

    #[test]
    fn validate_regex() {
        assert!(regex("^[a-z]+$".parse().unwrap())("test").is_ok());
        assert!(regex("^[a-z]+$".parse().unwrap())("test1").is_err());
        assert!(regex("^[a-z]+$".parse().unwrap())("1").is_err());
        assert!(regex("^[a-z]+$".parse().unwrap())("").is_err());
    }
}
