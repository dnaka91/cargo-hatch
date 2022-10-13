//! Validators that ensure additional restrictions on input data.

use inquire::{
    validator::{StringValidator, Validation},
    CustomUserError,
};

// Validate that a value is not empty.
#[derive(Clone)]
pub struct Required;

impl StringValidator for Required {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        Ok(if input.is_empty() {
            Validation::Invalid("A response is required.".into())
        } else {
            Validation::Valid
        })
    }
}

// Validate that a value is a crate name, according to crates.io rules.
#[derive(Clone)]
pub struct Krate;

impl StringValidator for Krate {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        const MAX_NAME_LENGTH: usize = 64;

        Ok(
            if input.chars().take(MAX_NAME_LENGTH + 1).count() <= MAX_NAME_LENGTH
                && input.chars().next().map_or(false, char::is_alphabetic)
                && input
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                Validation::Valid
            } else {
                Validation::Invalid("value must be a valid crate name".into())
            },
        )
    }
}

// Validate that a value is a Rust identifier.
#[derive(Clone)]
pub struct Ident;

impl StringValidator for Ident {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        use check_keyword::CheckKeyword;

        fn is_ident(input: &str) -> bool {
            input
                .chars()
                .next()
                .map_or(false, unicode_ident::is_xid_start)
                && input.chars().skip(1).all(unicode_ident::is_xid_continue)
        }

        fn is_escaped_ident(input: &str) -> bool {
            input.chars().next().map_or(false, |c| c == '_')
                && input.chars().take(2).count() >= 2
                && input.chars().skip(1).all(unicode_ident::is_xid_continue)
        }

        Ok(
            if (is_ident(input) || is_escaped_ident(input)) && !input.is_keyword() {
                Validation::Valid
            } else {
                Validation::Invalid("value must be a valid Rust identifier".into())
            },
        )
    }
}

// Validate that a value is a semantic version.
#[derive(Clone)]
pub struct Semver;

impl StringValidator for Semver {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        Ok(match input.parse::<semver::Version>() {
            Ok(_) => Validation::Valid,
            Err(e) => {
                Validation::Invalid(format!("value is not a valid sematic version: {e}").into())
            }
        })
    }
}

// Validate that a value is a semantic version requirement specification.
#[derive(Clone)]
pub struct SemverReq;

impl StringValidator for SemverReq {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        Ok(match input.parse::<semver::VersionReq>() {
            Ok(_) => Validation::Valid,
            Err(e) => Validation::Invalid(
                format!("value is not a valid sematic version spec: {e}").into(),
            ),
        })
    }
}

// Validate that a value matches against the given Regular Expression.
#[derive(Clone)]
pub struct Regex(pub regex::Regex);

impl StringValidator for Regex {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        Ok(if self.0.is_match(input) {
            Validation::Valid
        } else {
            Validation::Invalid(format!("value must match regex pattern `{}`", self.0).into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::needless_pass_by_value)]
    fn valid(result: Result<Validation, CustomUserError>) -> bool {
        matches!(result, Ok(Validation::Valid))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn invalid(result: Result<Validation, CustomUserError>) -> bool {
        matches!(result, Ok(Validation::Invalid(_)))
    }

    #[test]
    fn validate_required() {
        assert!(valid(Required.validate("a")));
        assert!(invalid(Required.validate("")));
    }

    #[test]
    fn validate_krate() {
        assert!(valid(Krate.validate("anyhow")));
        assert!(valid(Krate.validate("tower-http")));
        assert!(valid(Krate.validate("tower_http")));
        assert!(invalid(Krate.validate("?")));
        assert!(invalid(Krate.validate("")));
    }

    #[test]
    fn validate_ident() {
        assert!(valid(Ident.validate("test")));
        assert!(valid(Ident.validate("TEST")));
        assert!(valid(Ident.validate("teST")));
        assert!(valid(Ident.validate("Test")));
        assert!(valid(Ident.validate("_test")));
        assert!(valid(Ident.validate("_TEST")));
        assert!(invalid(Ident.validate("_")));
        assert!(invalid(Ident.validate("?")));
        assert!(invalid(Ident.validate("")));
    }

    #[test]
    fn validate_semver() {
        assert!(valid(Semver.validate("1.0.0")));
        assert!(valid(Semver.validate("1.0.0-beta.1")));
        assert!(valid(Semver.validate("1.0.0+abc")));
        assert!(invalid(Semver.validate("1.0")));
        assert!(invalid(Semver.validate("1")));
        assert!(invalid(Semver.validate("")));
    }

    #[test]
    fn validate_semver_req() {
        assert!(valid(SemverReq.validate("1.0.0")));
        assert!(valid(SemverReq.validate("1.0.0-beta.1")));
        assert!(valid(SemverReq.validate("1.0.0+abc")));
        assert!(valid(SemverReq.validate("1.0")));
        assert!(valid(SemverReq.validate("1")));
        assert!(invalid(SemverReq.validate("")));
    }

    #[test]
    fn validate_regex() {
        assert!(valid(Regex("^[a-z]+$".parse().unwrap()).validate("test")));
        assert!(invalid(
            Regex("^[a-z]+$".parse().unwrap()).validate("test1")
        ));
        assert!(invalid(Regex("^[a-z]+$".parse().unwrap()).validate("1")));
        assert!(invalid(Regex("^[a-z]+$".parse().unwrap()).validate("")));
    }
}
