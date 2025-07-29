use std::borrow::Cow;

use inno::entry::Type;
use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment,
        Constraint::{Fill, Length},
        Layout, Rect,
    },
    prelude::{Modifier, StatefulWidget, Style, Widget},
    style::palette::tailwind::SKY,
    widgets::{
        Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};

use super::emoji::Emoji;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Types<'a> {
    types: &'a [Type],
    state: TableState,
    scroll_state: ScrollbarState,
}

impl<'a> Types<'a> {
    pub fn new(types: &'a [Type]) -> Self {
        Self {
            types,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(types.len()),
        }
    }

    pub fn next_row(&mut self) {
        let index = self
            .state
            .selected()
            .filter(|&index| index < self.types.len())
            .map(|index| index + 1)
            .unwrap_or_default();

        self.state.select(Some(index));
        self.scroll_state = self.scroll_state.position(index);
    }

    pub fn previous_row(&mut self) {
        let index = self
            .state
            .selected()
            .map(|index| index.saturating_sub(1))
            .unwrap_or_default();

        self.state.select(Some(index));
        self.scroll_state = self.scroll_state.position(index);
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
            &mut self.scroll_state,
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
            ])
        });

        StatefulWidget::render(
            Table::new(
                rows,
                [
                    Length(2),
                    Length(4),
                    Length(11),
                    Length(9),
                    Length(5),
                    Length(6),
                    Length(5),
                    Length(4),
                ],
            )
            .header(
                Row::new([
                    "#",
                    "Name",
                    "Description",
                    "Languages",
                    "Check",
                    "Custom",
                    "Setup",
                    "Size",
                ])
                .style(Style::new().add_modifier(Modifier::BOLD)),
            )
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
            &mut self.state,
        );
    }
}
