use std::borrow::Cow;

use inno::entry::{Language, Message};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Constraint::Max, Rect},
    widgets::{Row, Table, Widget},
};

use super::constraint::{index_width, width};
use crate::scroll::TableScroll;

const NAME: &str = "Name";
const VALUE: &str = "Value";
const LANGUAGE: &str = "Language";
const HEADERS: [&str; 4] = ["#", NAME, VALUE, LANGUAGE];

#[derive(Clone, Debug, Eq, PartialEq)]

pub struct Messages<'a, 'language> {
    messages: Vec<Message<'a, 'language>>,
    scroll: TableScroll,
    constraints: [Constraint; 4],
}

impl<'a, 'language> Messages<'a, 'language> {
    #[must_use]
    pub fn new(messages: Vec<Message<'a, 'language>>) -> Self {
        Self {
            scroll: TableScroll::new(messages.len()),
            constraints: constraints(&messages),
            messages,
        }
    }

    pub const fn scroll_mut(&mut self) -> &mut TableScroll {
        &mut self.scroll
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
        let rows = self.messages.iter().enumerate().map(|(index, message)| {
            Row::new([
                Cow::Owned((index + 1).to_string()),
                Cow::Borrowed(message.name().unwrap_or_default()),
                Cow::Borrowed(message.value().unwrap_or_default()),
                Cow::Borrowed(message.language().map(Language::name).unwrap_or_default()),
            ])
        });

        self.scroll.render(
            Table::new(rows, self.constraints).column_spacing(2),
            HEADERS,
            "Messages",
            area,
            buf,
        );
    }
}

fn constraints(messages: &[Message]) -> [Constraint; 4] {
    [
        Max(index_width(messages.len())),
        Max(width(
            NAME,
            messages
                .iter()
                .map(|message| message.name().unwrap_or_default()),
        )),
        Max(width(
            VALUE,
            messages
                .iter()
                .map(|message| message.value().unwrap_or_default()),
        )),
        Max(width(
            LANGUAGE,
            messages
                .iter()
                .map(|message| message.language().map(Language::name).unwrap_or_default()),
        )),
    ]
}
