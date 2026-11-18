use std::borrow::Cow;

use inno::entry::File;
use ratatui::{
    buffer::Buffer,
    layout::{
        Constraint,
        Constraint::{Length, Max},
        Rect,
    },
    prelude::Widget,
    widgets::{Row, Table},
};

use super::constraint::{index_width, width};
use crate::scroll::TableScroll;

const SOURCE: &str = "Source";
const DESTINATION: &str = "Destination";
const FONT_NAME: &str = "Font name";
const ASSEMBLY_NAME: &str = "Assembly name";
const SIZE: &str = "Size";
const FLAGS: &str = "Flags";
const TYPE: &str = "Type";

const HEADERS: [&str; 8] = [
    "#",
    SOURCE,
    DESTINATION,
    FONT_NAME,
    ASSEMBLY_NAME,
    SIZE,
    FLAGS,
    TYPE,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Files<'a> {
    files: &'a [File],
    scroll: TableScroll,
    constraints: [Constraint; 8],
    flags: Vec<String>,
}

impl<'a> Files<'a> {
    pub fn new(files: &'a [File]) -> Self {
        let flags = files
            .iter()
            .map(|file| file.flags().to_string())
            .collect::<Vec<_>>();

        Self {
            files,
            scroll: TableScroll::new(files.len()),
            constraints: constraints(files, &flags),
            flags,
        }
    }

    pub const fn scroll_mut(&mut self) -> &mut TableScroll {
        &mut self.scroll
    }

    /// Returns `true` if there are no file entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl Widget for &mut Files<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let rows = self.files.iter().enumerate().map(|(index, file)| {
            Row::new([
                Cow::Owned((index + 1).to_string()),
                Cow::Borrowed(file.source().unwrap_or_default()),
                Cow::Borrowed(file.destination().unwrap_or_default().as_str()),
                Cow::Borrowed(file.install_font_name().unwrap_or_default()),
                Cow::Borrowed(file.strong_assembly_name().unwrap_or_default()),
                Cow::Owned(file.external_size().to_string()),
                Cow::Borrowed(&self.flags[index]),
                Cow::Borrowed(file.r#type().as_str()),
            ])
        });

        self.scroll.render(
            Table::new(rows, self.constraints).column_spacing(2),
            HEADERS,
            "Files",
            area,
            buf,
        );
    }
}

fn constraints(files: &[File], flags: &[String]) -> [Constraint; 8] {
    [
        Length(index_width(files.len())),
        Max(width(
            SOURCE,
            files.iter().map(|file| file.source().unwrap_or_default()),
        )),
        Length(width(
            DESTINATION,
            files
                .iter()
                .map(|file| file.destination().unwrap_or_default()),
        )),
        Max(width(
            FONT_NAME,
            files
                .iter()
                .map(|file| file.install_font_name().unwrap_or_default()),
        )),
        Max(width(
            ASSEMBLY_NAME,
            files
                .iter()
                .map(|file| file.strong_assembly_name().unwrap_or_default()),
        )),
        Max(width(SIZE, files.iter().map(File::external_size))),
        Max(width(FLAGS, flags)),
        Max(width(TYPE, files.iter().map(|file| file.r#type().as_str()))),
    ]
}
