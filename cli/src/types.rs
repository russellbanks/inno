use std::borrow::Cow;

use inno::entry::Type;
use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment, Constraint,
        Constraint::{Fill, Length, Max},
        Layout, Rect,
    },
    prelude::{Modifier, StatefulWidget, Style, Widget},
    style::palette::tailwind::SKY,
    widgets::{Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, Table},
};

use super::emoji::Emoji;
use crate::{
    constraint::{index_width, width},
    scroll::TableScroll,
};

const NAME: &str = "Name";
const DESCRIPTION: &str = "Description";
const LANGUAGES: &str = "Languages";
const CHECK: &str = "Check";
const IS_CUSTOM: &str = "Custom";
const SETUP: &str = "Setup";
const SIZE: &str = "Size";

const HEADERS: [&str; 8] = [
    "#",
    NAME,
    DESCRIPTION,
    LANGUAGES,
    CHECK,
    IS_CUSTOM,
    SETUP,
    SIZE,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Types<'a> {
    types: &'a [Type],
    scroll: TableScroll,
    constraints: [Constraint; 8],
}

impl<'a> Types<'a> {
    pub fn new(types: &'a [Type]) -> Self {
        Self {
            types,
            scroll: TableScroll::new(types.len()),
            constraints: constraints(types),
        }
    }

    pub const fn scroll_mut(&mut self) -> &mut TableScroll {
        &mut self.scroll
    }

    /// Returns `true` if there are no type entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}

impl Widget for &mut Types<'_> {
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

        let rows = self.types.iter().enumerate().map(|(index, r#type)| {
            Row::new([
                Cow::Owned((index + 1).to_string()),
                Cow::Borrowed(r#type.name().unwrap_or_default()),
                Cow::Borrowed(r#type.description().unwrap_or_default()),
                Cow::Borrowed(r#type.languages().unwrap_or_default()),
                Cow::Borrowed(r#type.check().unwrap_or_default()),
                Cow::Borrowed(r#type.is_custom().emoji()),
                Cow::Borrowed(r#type.setup().as_str()),
                Cow::Owned(r#type.size().to_string()),
            ])
        });

        StatefulWidget::render(
            Table::new(rows, self.constraints)
                .header(Row::new(HEADERS).style(Style::new().add_modifier(Modifier::BOLD)))
                .column_spacing(2)
                .block(
                    Block::bordered()
                        .title("Types")
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

fn constraints(types: &[Type]) -> [Constraint; 8] {
    [
        Max(index_width(types.len())),
        Max(width(
            NAME,
            types.iter().map(|r#type| r#type.name().unwrap_or_default()),
        )),
        Max(width(
            DESCRIPTION,
            types
                .iter()
                .map(|r#type| r#type.description().unwrap_or_default()),
        )),
        Max(width(
            LANGUAGES,
            types
                .iter()
                .map(|r#type| r#type.languages().unwrap_or_default()),
        )),
        Max(width(
            CHECK,
            types
                .iter()
                .map(|r#type| r#type.check().unwrap_or_default()),
        )),
        Max(IS_CUSTOM.len() as u16),
        Max(width(
            SETUP,
            types.iter().map(|r#type| r#type.setup().as_str()),
        )),
        Max(width(SIZE, types.iter().map(Type::size))),
    ]
}
