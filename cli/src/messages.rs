use std::borrow::Cow;

use inno::entry::Message;
use ratatui::{
    buffer::Buffer,
    layout::{
        Constraint::{Fill, Length},
        Layout, Rect,
    },
    prelude::{Alignment, Modifier, Style},
    style::palette::tailwind::SKY,
    widgets::{
        Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Table, TableState, Widget,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]

pub struct Messages<'a> {
    messages: &'a [Message],
    state: TableState,
    scroll_state: ScrollbarState,
    max_name_length: u16,
    max_value_length: u16,
}

impl<'a> Messages<'a> {
    #[must_use]
    pub fn new(messages: &'a [Message]) -> Self {
        Self {
            messages,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(messages.len()),
            max_name_length: messages
                .iter()
                .filter_map(Message::name)
                .map(str::len)
                .max()
                .unwrap_or(1)
                .try_into()
                .unwrap_or(u16::MAX),
            max_value_length: messages
                .iter()
                .filter_map(Message::value)
                .map(str::len)
                .max()
                .unwrap_or(1)
                .try_into()
                .unwrap_or(u16::MAX),
        }
    }

    pub fn next_row(&mut self) {
        let index = self
            .state
            .selected()
            .filter(|&index| index < self.messages.len())
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

impl Widget for &mut Messages<'_> {
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

        let rows = self.messages.iter().enumerate().map(|(index, message)| {
            Row::new([
                Cow::Owned((index + 1).to_string()),
                Cow::Borrowed(message.name().unwrap_or_default()),
                Cow::Borrowed(message.value().unwrap_or_default()),
            ])
        });

        StatefulWidget::render(
            Table::new(
                rows,
                [
                    Length(2),
                    Length(self.max_name_length),
                    Length(self.max_value_length),
                ],
            )
            .header(
                Row::new(["#", "Name", "Value"]).style(Style::new().add_modifier(Modifier::BOLD)),
            )
            .column_spacing(2)
            .block(
                Block::bordered()
                    .title("Messages")
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
