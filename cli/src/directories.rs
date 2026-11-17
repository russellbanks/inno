use std::borrow::Cow;

use inno::entry::Directory;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Constraint::Max, Rect},
    prelude::Widget,
    widgets::{Row, Table},
};

use super::constraint::{index_width, width};
use crate::scroll::TableScroll;

const NAME: &str = "Name";
const PERMISSIONS: &str = "Permissions";
const ATTRIBUTES: &str = "Attributes";
const PERMISSION: &str = "Permission";
const FLAGS: &str = "Flags";

const HEADERS: [&str; 6] = ["#", NAME, PERMISSIONS, ATTRIBUTES, PERMISSION, FLAGS];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Directories<'a> {
    directories: &'a [Directory],
    scroll: TableScroll,
    constraints: [Constraint; 6],
}

impl<'a> Directories<'a> {
    pub fn new(directories: &'a [Directory]) -> Self {
        Self {
            directories,
            scroll: TableScroll::new(directories.len()),
            constraints: constraints(directories),
        }
    }

    pub const fn scroll_mut(&mut self) -> &mut TableScroll {
        &mut self.scroll
    }

    /// Returns `true` if there are no directory entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.directories.is_empty()
    }
}

impl Widget for &mut Directories<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let rows = self
            .directories
            .iter()
            .enumerate()
            .map(|(index, directory)| {
                Row::new([
                    Cow::Owned((index + 1).to_string()),
                    Cow::Borrowed(directory.name().unwrap_or_default().as_str()),
                    Cow::Borrowed(directory.permissions().unwrap_or_default()),
                    Cow::Owned(directory.attributes().to_string()),
                    Cow::Owned(directory.permission().to_string()),
                    Cow::Owned(directory.flags().to_string()),
                ])
            });

        self.scroll.render(
            Table::new(rows, self.constraints).column_spacing(2),
            HEADERS,
            "Directories",
            area,
            buf,
        );
    }
}

fn constraints(directories: &[Directory]) -> [Constraint; 6] {
    [
        Max(index_width(directories.len())),
        Max(width(
            NAME,
            directories
                .iter()
                .map(|directory| directory.name().unwrap_or_default()),
        )),
        Max(width(
            PERMISSIONS,
            directories
                .iter()
                .map(|directory| directory.permissions().unwrap_or_default()),
        )),
        Max(width(
            ATTRIBUTES,
            directories.iter().map(Directory::attributes),
        )),
        Max(width(
            PERMISSION,
            directories.iter().map(Directory::permission),
        )),
        Max(width(FLAGS, directories.iter().map(Directory::flags))),
    ]
}
