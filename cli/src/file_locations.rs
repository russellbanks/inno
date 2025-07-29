use std::borrow::Cow;

use inno::entry::FileLocation;
use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment, Constraint,
        Constraint::{Fill, Length, Max},
        Flex, Layout, Rect,
    },
    prelude::{Modifier, StatefulWidget, Style, Widget},
    style::palette::tailwind::SKY,
    widgets::{
        Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};

use super::constraint::{flags_constraint, int_constraint, ints_constraint, strings_constraint};

const UNCOMPRESSED_SIZE: &str = "Uncompressed size";
const CREATED_AT: &str = "Created at";
const FILE_VERSION: &str = "File version";
const OPTIONS: &str = "Options";
const SIGN_MODE: &str = "Sign mode";

const HEADERS: [&str; 6] = [
    "#",
    UNCOMPRESSED_SIZE,
    CREATED_AT,
    FILE_VERSION,
    OPTIONS,
    SIGN_MODE,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileLocations<'a> {
    file_locations: &'a [FileLocation],
    state: TableState,
    scroll_state: ScrollbarState,
    constraints: [Constraint; 6],
}

impl<'a> FileLocations<'a> {
    pub fn new(file_locations: &'a [FileLocation]) -> Self {
        Self {
            file_locations,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(file_locations.len()),
            constraints: constraints(file_locations),
        }
    }

    pub fn next_row(&mut self) {
        let index = self
            .state
            .selected()
            .filter(|&index| index < self.file_locations.len())
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

    /// Returns `true` if there are no file location entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.file_locations.is_empty()
    }
}

impl Widget for &mut FileLocations<'_> {
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

        let rows = self.file_locations.iter().enumerate().map(|(index, file)| {
            Row::new([
                Cow::Owned((index + 1).to_string()),
                Cow::Owned(file.uncompressed_size().to_string()),
                Cow::Owned(file.date_time().to_string()),
                Cow::Owned(file.file_version().to_string()),
                Cow::Owned(file.file_option_flags().to_string()),
                Cow::Borrowed(file.sign_mode().as_str()),
            ])
        });

        StatefulWidget::render(
            Table::new(rows, self.constraints)
                .header(Row::new(HEADERS).style(Style::new().add_modifier(Modifier::BOLD)))
                .column_spacing(2)
                .block(
                    Block::bordered()
                        .title("Files locations")
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

fn constraints(file_locations: &[FileLocation]) -> [Constraint; 6] {
    [
        Max(int_constraint(file_locations.len())),
        Max(ints_constraint(
            file_locations.iter().map(FileLocation::uncompressed_size),
            UNCOMPRESSED_SIZE,
        )),
        Max(19), // Constant DateTime length
        Max(ints_constraint(
            file_locations.iter().map(FileLocation::file_version),
            FILE_VERSION,
        )),
        Max(flags_constraint(
            file_locations.iter().map(FileLocation::file_option_flags),
            OPTIONS,
        )),
        Max(strings_constraint(
            file_locations
                .iter()
                .map(|file_location| file_location.sign_mode().as_str()),
            SIGN_MODE,
        )),
    ]
}
