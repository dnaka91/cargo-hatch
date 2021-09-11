use std::io;

use super::{InputReader, ReadError, StringSetting};

pub struct StringReader<'a> {
    setting: StringSetting,
    buf: &'a mut String,
}

impl<'a> StringReader<'a> {
    pub fn new(setting: StringSetting, buf: &'a mut String) -> Self {
        Self { setting, buf }
    }
}

impl<'a> InputReader for StringReader<'a> {
    type Output = String;

    fn read(&mut self, description: &str) -> Result<Self::Output, ReadError> {
        if let Some(default) = &self.setting.default {
            println!("{} [default: {}]:", description, default);
        } else {
            println!("{}:", description);
        }

        self.buf.clear();
        io::stdin().read_line(self.buf)?;

        Ok(match self.buf.trim() {
            "" => self
                .setting
                .default
                .clone()
                .ok_or(ReadError::InvalidInput("must provide a value"))?,
            value => value.to_owned(),
        })
    }
}
