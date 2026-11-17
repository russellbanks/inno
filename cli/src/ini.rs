use std::borrow::Cow;

use inno::entry::Ini;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Constraint::Max, Rect},
    prelude::Widget,
    widgets::{Row, Table},
};

use super::constraint::{index_width, width};
use crate::scroll::TableScroll;

const FILE: &str = "File";
const SECTION: &str = "Section";
const KEY: &str = "Key";
const VALUE: &str = "Value";
const FLAGS: &str = "Flags";

const HEADERS: [&str; 6] = ["#", FILE, SECTION, KEY, VALUE, FLAGS];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IniFiles<'a> {
    ini_files: &'a [Ini],
    scroll: TableScroll,
    constraints: [Constraint; 6],
}

impl<'a> IniFiles<'a> {
    pub fn new(ini_files: &'a [Ini]) -> Self {
        Self {
            ini_files,
            scroll: TableScroll::new(ini_files.len()),
            constraints: constraints(ini_files),
        }
    }

    pub const fn scroll_mut(&mut self) -> &mut TableScroll {
        &mut self.scroll
    }

    /// Returns `true` if there are no ini entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.ini_files.is_empty()
    }
}

impl Widget for &mut IniFiles<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let rows = self.ini_files.iter().enumerate().map(|(i, ini)| {
            Row::new([
                Cow::Owned((i + 1).to_string()),
                Cow::Borrowed(ini.file_path()),
                Cow::Borrowed(ini.section_name().unwrap_or_default()),
                Cow::Borrowed(ini.key_name().unwrap_or_default()),
                Cow::Borrowed(ini.value().unwrap_or_default()),
                Cow::Owned(ini.flags().to_string()),
            ])
        });

        self.scroll.render(
            Table::new(rows, self.constraints).column_spacing(2),
            HEADERS,
            "Ini files",
            area,
            buf,
        );
    }
}

fn constraints(ini_files: &[Ini]) -> [Constraint; 6] {
    [
        Max(index_width(ini_files.len())),
        Max(width(FILE, ini_files.iter().map(Ini::file_path))),
        Max(width(
            SECTION,
            ini_files
                .iter()
                .map(|ini| ini.section_name().unwrap_or_default()),
        )),
        Max(width(
            KEY,
            ini_files
                .iter()
                .map(|ini| ini.key_name().unwrap_or_default()),
        )),
        Max(width(
            VALUE,
            ini_files.iter().map(|ini| ini.value().unwrap_or_default()),
        )),
        Max(width(FLAGS, ini_files.iter().map(Ini::flags))),
    ]
}
