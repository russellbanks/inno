use std::{borrow::Cow, iter::once};

use inno::entry::Language;
use itoa::Buffer as ItoaBuffer;
use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment,
        Constraint::{Fill, Length, Max},
        Layout, Rect,
    },
    prelude::{Modifier, Style},
    style::palette::tailwind::SKY,
    widgets::{
        Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Table, TableState, Widget,
    },
};

use super::emoji::Emoji;

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
}

impl<'a> Languages<'a> {
    pub fn new(languages: &'a [Language]) -> Self {
        Self {
            languages,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(languages.len()),
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

    fn max_len<F>(&self, f: F, extra: usize) -> u16
    where
        F: Fn(&Language) -> usize,
    {
        self.languages
            .iter()
            .map(f)
            .chain(once(extra))
            .max()
            .unwrap_or(1)
            .try_into()
            .unwrap_or(u16::MAX)
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

        let rows = self.languages.iter().enumerate().map(|(i, lang)| {
            Row::new([
                Cow::Owned((i + 1).to_string()),
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
            Table::new(
                rows,
                [
                    Length(2),
                    Length(self.max_len(|lang| lang.name().len(), NAME.len())),
                    Length(self.max_len(|lang| lang.language_name().len(), LANGUAGE_NAME.len())),
                    Length(
                        self.max_len(|lang| ItoaBuffer::new().format(lang.id()).len(), ID.len()),
                    ),
                    Length(self.max_len(|lang| lang.codepage().name().len(), CODEPAGE.len())),
                    Max(self.max_len(
                        |lang| {
                            lang.dialog_font().len()
                                + " ()".len()
                                + ItoaBuffer::new().format(lang.dialog_font_size()).len()
                        },
                        DIALOG_FONT.len(),
                    )),
                    Max(self.max_len(
                        |lang| {
                            lang.dialog_font().len()
                                + " ()".len()
                                + ItoaBuffer::new().format(lang.dialog_font_size()).len()
                        },
                        TITLE_FONT.len(),
                    )),
                    Max(3),
                ],
            )
            .header(Row::new(HEADERS).style(Style::default().add_modifier(Modifier::BOLD)))
            .column_spacing(2)
            .block(
                Block::bordered()
                    .title("Languages")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::proportional(1)),
            )
            .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED).fg(SKY.c400)),
            layout[0],
            buf,
            &mut self.state,
        )
    }
}
