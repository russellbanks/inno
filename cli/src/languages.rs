use std::borrow::Cow;

use inno::entry::Language;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Constraint::Max, Flex, Rect},
    widgets::{Row, Table, Widget},
};

use super::emoji::Emoji;
use crate::{
    constraint::{index_width, width},
    scroll::TableScroll,
};

const NAME: &str = "Name";
const LANGUAGE_NAME: &str = "Language name";
const ID: &str = "ID";
const CODEPAGE: &str = "Codepage";
const DIALOG_FONT: &str = "Dialog Font (pt)";
const TITLE_FONT: &str = "Title Font (pt)";
const RTL: &str = "RTL";

const HEADERS: [&str; 8] = [
    "#",
    NAME,
    LANGUAGE_NAME,
    ID,
    CODEPAGE,
    DIALOG_FONT,
    TITLE_FONT,
    RTL,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Languages<'a> {
    languages: &'a [Language],
    scroll: TableScroll,
    constraints: [Constraint; 8],
}

impl<'a> Languages<'a> {
    pub fn new(languages: &'a [Language]) -> Self {
        Self {
            languages,
            scroll: TableScroll::new(languages.len()),
            constraints: constraints(languages),
        }
    }

    pub const fn scroll_mut(&mut self) -> &mut TableScroll {
        &mut self.scroll
    }

    /// Returns `true` if there are no language entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.languages.is_empty()
    }
}

impl Widget for &mut Languages<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let rows = self.languages.iter().enumerate().map(|(index, lang)| {
            Row::new([
                Cow::Owned((index + 1).to_string()),
                Cow::Borrowed(lang.name()),
                Cow::Borrowed(lang.language_name()),
                Cow::Owned(lang.id().to_string()),
                Cow::Borrowed(lang.codepage().name()),
                Cow::Owned(format!(
                    "{} ({})",
                    lang.dialog_font().name(),
                    lang.dialog_font().size()
                )),
                Cow::Owned(format!(
                    "{} ({})",
                    lang.title_font(),
                    lang.title_font_size()
                )),
                Cow::Borrowed(lang.right_to_left().emoji()),
            ])
        });

        self.scroll.render(
            Table::new(rows, self.constraints)
                .column_spacing(2)
                .flex(Flex::Legacy),
            HEADERS,
            "Languages",
            area,
            buf,
        );
    }
}

fn constraints(languages: &[Language]) -> [Constraint; 8] {
    [
        Max(index_width(languages.len())),
        Max(width(NAME, languages.iter().map(Language::name))),
        Max(width(
            LANGUAGE_NAME,
            languages.iter().map(Language::language_name),
        )),
        Max(width(ID, languages.iter().map(Language::id))),
        Max(width(
            CODEPAGE,
            languages.iter().map(|language| language.codepage().name()),
        )),
        Max(width(
            DIALOG_FONT,
            languages.iter().map(|language| {
                format!(
                    "{} ({})",
                    language.dialog_font().name(),
                    language.dialog_font().size()
                )
            }),
        )),
        Max(width(
            TITLE_FONT,
            languages.iter().map(|language| {
                format!("{} ({})", language.title_font(), language.title_font_size())
            }),
        )),
        Max(RTL.len() as u16),
    ]
}
