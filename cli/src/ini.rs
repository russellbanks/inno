use std::borrow::Cow;

use inno::entry::Ini;
use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment, Constraint,
        Constraint::{Fill, Length, Max},
        Layout, Rect,
    },
    prelude::{Modifier, StatefulWidget, Style, Widget},
    style::palette::tailwind::SKY,
    widgets::{
        Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};

use super::constraint::{flags_constraint, int_constraint, strings_constraint};

const FILE: &str = "File";
const SECTION: &str = "Section";
const KEY: &str = "Key";
const VALUE: &str = "Value";
const FLAGS: &str = "Flags";

const HEADERS: [&str; 6] = ["#", FILE, SECTION, KEY, VALUE, FLAGS];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IniFiles<'a> {
    ini_files: &'a [Ini],
    state: TableState,
    scroll_state: ScrollbarState,
    constraints: [Constraint; 6],
}

impl<'a> IniFiles<'a> {
    pub fn new(ini_files: &'a [Ini]) -> Self {
        Self {
            ini_files,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(ini_files.len()),
            constraints: constraints(ini_files),
        }
    }

    pub fn next_row(&mut self) {
        let index = self
            .state
            .selected()
            .filter(|&index| index < self.ini_files.len())
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

    /// Returns `true` if there are no ini entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.ini_files.is_empty()
    }
}

impl Widget for &mut IniFiles<'_> {
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

        let rows = self.ini_files.iter().enumerate().map(|(i, ini)| {
            Row::new([
                Cow::Owned((i + 1).to_string()),
                Cow::Borrowed(ini.file_path()),
                Cow::Borrowed(ini.section_name().unwrap_or_default()),
                Cow::Borrowed(ini.key_name().unwrap_or_default()),
                Cow::Borrowed(ini.value().unwrap_or_default()),
                Cow::Owned(ini.flags().to_string()),
            ])
        });

        StatefulWidget::render(
            Table::new(rows, self.constraints)
                .header(Row::new(HEADERS).style(Style::default().add_modifier(Modifier::BOLD)))
                .column_spacing(2)
                .block(
                    Block::bordered()
                        .title("Ini files")
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

fn constraints(ini_files: &[Ini]) -> [Constraint; 6] {
    [
        Max(int_constraint(ini_files.len())),
        Max(strings_constraint(
            ini_files.iter().map(Ini::file_path),
            FILE,
        )),
        Max(strings_constraint(
            ini_files.iter().map(Ini::section_name),
            SECTION,
        )),
        Max(strings_constraint(ini_files.iter().map(Ini::key_name), KEY)),
        Max(strings_constraint(ini_files.iter().map(Ini::value), VALUE)),
        Max(flags_constraint(ini_files.iter().map(Ini::flags), FLAGS)),
    ]
}
