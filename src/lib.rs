//! # 🐣 Cargo Hatch

#![deny(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::option_if_let_else
)]

pub mod cargo;
pub mod dirs;
pub mod repo;
pub mod settings;
pub mod templates;
