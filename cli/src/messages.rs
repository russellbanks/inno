use std::borrow::Cow;

use inno::entry::{Language, Message};
use ratatui::{
    buffer::Buffer,
    layout::{
        Constraint,
        Constraint::{Fill, Length, Max},
        Layout, Rect,
    },
    prelude::{Alignment, Modifier, Style},
    style::palette::tailwind::SKY,
    widgets::{
        Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Table, TableState, Widget,
    },
};

use super::constraint::{int_constraint, strings_constraint};

const NAME: &str = "Name";
const VALUE: &str = "Value";
const LANGUAGE: &str = "Language";
const HEADERS: [&str; 4] = ["#", NAME, VALUE, LANGUAGE];

#[derive(Clone, Debug, Eq, PartialEq)]

pub struct Messages<'a, 'language> {
    messages: Vec<Message<'a, 'language>>,
    state: TableState,
    scroll_state: ScrollbarState,
    constraints: [Constraint; 4],
}

impl<'a, 'language> Messages<'a, 'language> {
    #[must_use]
    pub fn new(messages: Vec<Message<'a, 'language>>) -> Self {
        Self {
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(messages.len()),
            constraints: constraints(&messages),
            messages,
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

    /// Returns `true` if there are no message entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl Widget for &mut Messages<'_, '_> {
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
                Cow::Borrowed(message.language().map(Language::name).unwrap_or_default()),
            ])
        });

        StatefulWidget::render(
            Table::new(rows, self.constraints)
                .header(Row::new(HEADERS).style(Style::new().add_modifier(Modifier::BOLD)))
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

fn constraints(messages: &[Message]) -> [Constraint; 4] {
    [
        Max(int_constraint(messages.len())),
        Max(strings_constraint(messages.iter().map(Message::name), NAME)),
        Max(strings_constraint(
            messages.iter().map(Message::value),
            VALUE,
        )),
        Max(strings_constraint(
            messages
                .iter()
                .filter_map(|message| message.language().map(Language::name)),
            LANGUAGE,
        )),
    ]
}
