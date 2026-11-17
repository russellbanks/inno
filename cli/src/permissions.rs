use std::borrow::Cow;

use inno::entry::Permission;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Constraint::Max, Rect},
    widgets::{Row, Table, Widget},
};

use crate::{
    constraint::{index_width, width},
    scroll::TableScroll,
};

const PERMISSIONS: &str = "Permissions";

const HEADERS: [&str; 2] = ["#", PERMISSIONS];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Permissions<'a> {
    permissions: &'a [Permission],
    scroll: TableScroll,
    constraints: [Constraint; 2],
}

impl<'a> Permissions<'a> {
    pub fn new(permissions: &'a [Permission]) -> Self {
        Self {
            permissions,
            scroll: TableScroll::new(permissions.len()),
            constraints: constraints(permissions),
        }
    }

    pub const fn scroll_mut(&mut self) -> &mut TableScroll {
        &mut self.scroll
    }
}

impl Widget for &mut Permissions<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let rows = self
            .permissions
            .iter()
            .enumerate()
            .map(|(index, permission)| {
                Row::new([
                    Cow::Owned((index + 1).to_string()),
                    Cow::Borrowed(permission.as_str()),
                ])
            });

        self.scroll.render(
            Table::new(rows, self.constraints).column_spacing(2),
            HEADERS,
            "Permissions",
            area,
            buf,
        );
    }
}

fn constraints(permissions: &[Permission]) -> [Constraint; 2] {
    [
        Max(index_width(permissions.len())),
        Max(width(
            PERMISSIONS,
            permissions.iter().map(Permission::as_str),
        )),
    ]
}
