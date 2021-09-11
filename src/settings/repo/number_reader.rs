use std::io;

use crossterm::style::Attribute;

use super::{InputReader, Number, NumberSetting, ReadError};

pub struct NumberReader<'a, T: Number> {
    setting: NumberSetting<T>,
    buf: &'a mut String,
}

impl<'a, T: Number> NumberReader<'a, T> {
    pub fn new(setting: NumberSetting<T>, buf: &'a mut String) -> Self {
        Self { setting, buf }
    }
}

impl<'a, T: Number> InputReader for NumberReader<'a, T> {
    type Output = T;

    fn read(&mut self, description: &str) -> Result<Self::Output, ReadError> {
        println!(
            "{} ({}{}..={}{}){}:",
            description,
            Attribute::Bold,
            self.setting.min,
            self.setting.max,
            Attribute::Reset,
            self.setting
                .default
                .map(|default| format!(" [default: {}]", default))
                .unwrap_or_default(),
        );

        self.buf.clear();
        io::stdin().read_line(self.buf)?;
        self.buf.make_ascii_lowercase();

        match self.buf.trim() {
            "" => self
                .setting
                .default
                .ok_or(ReadError::InvalidInput("must provide a value")),
            value => {
                let value = value
                    .parse::<T>()
                    .map_err(|_| ReadError::InvalidInput("not a number"))?;
                if value < self.setting.min {
                    return Err(ReadError::InvalidInput("value is below the minimum"));
                }
                if value > self.setting.max {
                    return Err(ReadError::InvalidInput("value is above the maximum"));
                }

                Ok(value)
            }
        }
    }
}
