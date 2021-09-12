use std::{collections::HashSet, convert::TryInto, io};

use crossterm::{
    cursor::{self, MoveToNextLine, MoveToPreviousLine, RestorePosition, SavePosition},
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, ScrollUp},
};

use super::{InputReader, MultiListSetting, ReadError};

pub struct MultiListReader {
    setting: MultiListSetting,
    selection: HashSet<usize>,
}

impl MultiListReader {
    #[must_use]
    pub fn new(setting: MultiListSetting) -> Self {
        Self {
            setting,
            selection: HashSet::new(),
        }
    }

    fn print_list(&self, index: usize) -> io::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        queue!(handle, RestorePosition)?;

        for (i, value) in self.setting.values.iter().enumerate() {
            queue!(
                handle,
                SetForegroundColor(if i == index {
                    Color::Green
                } else {
                    Color::Reset
                }),
                Print(if i == index { "* " } else { "  " }),
                Print(if self.selection.contains(&i) {
                    "[x] "
                } else {
                    "[ ] "
                }),
                Print(value),
                MoveToNextLine(1),
            )?;
        }

        execute!(handle, SetForegroundColor(Color::Reset))
    }
}

impl InputReader for MultiListReader {
    type Output = Vec<String>;

    fn read(&mut self, description: &str) -> Result<Self::Output, ReadError> {
        let (mut index, selection) = if let Some(default) = self.setting.default.as_ref() {
            let index = self
                .setting
                .values
                .iter()
                .position(|value| default.contains(value))
                .unwrap_or_default();
            let selection = self
                .setting
                .values
                .iter()
                .enumerate()
                .filter_map(|(i, value)| default.contains(value).then(|| i))
                .collect();

            (index, selection)
        } else {
            (0, HashSet::new())
        };

        let length = self.setting.values.len().try_into().unwrap_or(u16::MAX);
        self.selection = selection;

        terminal::enable_raw_mode()?;

        prepare_area(description, length)?;
        self.print_list(index)?;

        while let Ok(event) = crossterm::event::read() {
            if let Event::Key(KeyEvent { code, modifiers }) = event {
                match code {
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        terminal::disable_raw_mode()?;
                        return Err(ReadError::Cancelled);
                    }
                    KeyCode::Enter => break,
                    KeyCode::Up => {
                        if index == 0 {
                            index = self.setting.values.len() - 1;
                        } else {
                            index -= 1;
                        }
                        self.print_list(index)?;
                    }
                    KeyCode::Down => {
                        if index == self.setting.values.len() - 1 {
                            index = 0;
                        } else {
                            index += 1;
                        }
                        self.print_list(index)?;
                    }
                    KeyCode::Char(' ') | KeyCode::Tab => {
                        if !self.selection.insert(index) {
                            self.selection.remove(&index);
                        }
                        self.print_list(index)?;
                    }
                    _ => {}
                }
            }
        }

        terminal::disable_raw_mode()?;

        Ok(self
            .setting
            .values
            .iter()
            .enumerate()
            .filter_map(|(i, value)| self.selection.contains(&i).then(|| value.clone()))
            .collect())
    }
}

fn prepare_area(description: &str, length: u16) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    let space = terminal::size()?.1 - cursor::position()?.1;

    queue!(handle, Print(format!("{}:", description)))?;

    if space > length {
        queue!(handle, MoveToNextLine(1), SavePosition)
    } else {
        queue!(
            handle,
            ScrollUp(2 + length - space),
            MoveToPreviousLine(1 + length - space),
            SavePosition,
        )
    }
}
