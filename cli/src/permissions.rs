use std::borrow::Cow;

use inno::entry::Permission;
use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment, Constraint,
        Constraint::{Fill, Length, Max},
        Layout, Rect,
    },
    prelude::{Modifier, Style},
    style::palette::tailwind::SKY,
    widgets::{
        Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, StatefulWidget, Table,
        Widget,
    },
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
        let layout = Layout::horizontal([Fill(1), Length(1)]).split(area);

        StatefulWidget::render(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            layout[1],
            buf,
            &mut self.scroll.scrollbar,
        );

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

        StatefulWidget::render(
            Table::new(rows, self.constraints)
                .header(Row::new(HEADERS).style(Style::new().add_modifier(Modifier::BOLD)))
                .column_spacing(2)
                .block(
                    Block::bordered()
                        .title("Permissions")
                        .title_alignment(Alignment::Center)
                        .border_type(BorderType::Rounded)
                        .padding(Padding::proportional(1)),
                )
                .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED).fg(SKY.c400)),
            layout[0],
            buf,
            &mut self.scroll.state,
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
