use std::io;

use crossterm::style::{style, Attribute, Stylize};

use super::{BoolSetting, InputReader, ReadError};

pub struct BoolReader<'a> {
    setting: BoolSetting,
    buf: &'a mut String,
}

impl<'a> BoolReader<'a> {
    pub fn new(setting: BoolSetting, buf: &'a mut String) -> Self {
        Self { setting, buf }
    }
}

impl<'a> InputReader for BoolReader<'a> {
    type Output = bool;

    fn read(&mut self, description: &str) -> Result<Self::Output, ReadError> {
        println!(
            "{} [{}{}{}{}]:",
            description,
            Attribute::Bold,
            if self.setting.default.unwrap_or(false) {
                "Y".green()
            } else {
                style("y")
            },
            if self.setting.default.unwrap_or(true) {
                style("n")
            } else {
                "N".red()
            },
            Attribute::Reset,
        );

        self.buf.clear();
        io::stdin().read_line(self.buf)?;
        self.buf.make_ascii_lowercase();

        Ok(match self.buf.trim() {
            "" => self
                .setting
                .default
                .ok_or(ReadError::InvalidInput("must provide a value"))?,
            "y" | "yes" | "true" => true,
            "n" | "no" | "false" => false,
            _ => return Err(ReadError::InvalidInput("unknown input")),
        })
    }
}
