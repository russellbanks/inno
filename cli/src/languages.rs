use std::borrow::Cow;

use inno::entry::Language;
use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment, Constraint,
        Constraint::{Fill, Length, Max},
        Flex, Layout, Rect,
    },
    prelude::{Modifier, Style},
    style::palette::tailwind::SKY,
    widgets::{
        Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Table, TableState, Widget,
    },
};

use super::emoji::Emoji;
use crate::constraint::{int_constraint, ints_constraint, strings_constraint};

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
    state: TableState,
    scroll_state: ScrollbarState,
    constraints: [Constraint; 8],
}

impl<'a> Languages<'a> {
    pub fn new(languages: &'a [Language]) -> Self {
        Self {
            languages,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(languages.len()),
            constraints: constraints(languages),
        }
    }

    pub fn next_row(&mut self) {
        let index = self
            .state
            .selected()
            .filter(|&index| index < self.languages.len())
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
        let layout = Layout::horizontal([Fill(1), Length(1)]).split(area);

        StatefulWidget::render(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            layout[1],
            buf,
            &mut self.scroll_state,
        );

        let rows = self.languages.iter().enumerate().map(|(index, lang)| {
            Row::new([
                Cow::Owned((index + 1).to_string()),
                Cow::Borrowed(lang.name()),
                Cow::Borrowed(lang.language_name()),
                Cow::Owned(lang.id().to_string()),
                Cow::Borrowed(lang.codepage().name()),
                Cow::Owned(format!(
                    "{} ({})",
                    lang.dialog_font(),
                    lang.dialog_font_size()
                )),
                Cow::Owned(format!(
                    "{} ({})",
                    lang.title_font(),
                    lang.title_font_size()
                )),
                Cow::Borrowed(lang.right_to_left().emoji()),
            ])
        });

        StatefulWidget::render(
            Table::new(rows, self.constraints)
                .header(Row::new(HEADERS).style(Style::default().add_modifier(Modifier::BOLD)))
                .column_spacing(2)
                .block(
                    Block::bordered()
                        .title("Languages")
                        .title_alignment(Alignment::Center)
                        .border_type(BorderType::Rounded)
                        .padding(Padding::proportional(1)),
                )
                .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED).fg(SKY.c400))
                .flex(Flex::Legacy),
            layout[0],
            buf,
            &mut self.state,
        );
    }
}

fn constraints(languages: &[Language]) -> [Constraint; 8] {
    [
        Max(int_constraint(languages.len())),
        Max(strings_constraint(
            languages.iter().map(Language::name),
            NAME,
        )),
        Max(strings_constraint(
            languages.iter().map(Language::language_name),
            LANGUAGE_NAME,
        )),
        Max(ints_constraint(languages.iter().map(Language::id), ID)),
        Max(strings_constraint(
            languages.iter().map(|language| language.codepage().name()),
            CODEPAGE,
        )),
        Max(languages
            .iter()
            .map(|language| {
                language.dialog_font().len()
                    + language
                        .dialog_font_size()
                        .checked_ilog10()
                        .map(|log10| log10 + 1)
                        .unwrap_or_default() as usize
                    + " ()".len()
            })
            .max()
            .map(|max_len| max_len.max(DIALOG_FONT.len()))
            .unwrap_or_default() as u16),
        Max(languages
            .iter()
            .map(|language| {
                language.title_font().len()
                    + language
                        .title_font_size()
                        .checked_ilog10()
                        .map(|log10| log10 + 1)
                        .unwrap_or_default() as usize
                    + " ()".len()
            })
            .max()
            .map(|max_len| max_len.max(TITLE_FONT.len()))
            .unwrap_or_default() as u16),
        Max(RTL.len() as u16),
    ]
}
