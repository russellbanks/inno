use std::borrow::Cow;

use inno::entry::FileLocation;
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

const UNCOMPRESSED_SIZE: &str = "Uncompressed size";
const FILE_TIME: &str = "File time";
const FILE_VERSION: &str = "File version";
const OPTIONS: &str = "Options";
const SIGN_MODE: &str = "Sign mode";

const HEADERS: [&str; 6] = [
    "#",
    UNCOMPRESSED_SIZE,
    FILE_TIME,
    FILE_VERSION,
    OPTIONS,
    SIGN_MODE,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileLocations<'a> {
    file_locations: &'a [FileLocation],
    state: TableState,
    scroll_state: ScrollbarState,
}

impl<'a> FileLocations<'a> {
    pub fn new(file_locations: &'a [FileLocation]) -> Self {
        Self {
            file_locations,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(file_locations.len()),
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
                Cow::Owned(file.file_time().to_string()),
                Cow::Owned(file.file_version().to_string()),
                Cow::Owned(file.file_option_flags().to_string()),
                Cow::Borrowed(file.sign_mode().as_str()),
            ])
        });

        StatefulWidget::render(
            Table::new(
                rows,
                [
                    Length(2),
                    Length(17),
                    Length(9),
                    Length(12),
                    Length(12),
                    Length(
                        self.file_locations
                            .iter()
                            .map(|location| location.sign_mode().as_str().len())
                            .max()
                            .unwrap_or(SIGN_MODE.len())
                            .try_into()
                            .unwrap_or(u16::MAX),
                    ),
                ],
            )
            .header(Row::new(HEADERS).style(Style::new().add_modifier(Modifier::BOLD)))
            .column_spacing(2)
            .block(
                Block::bordered()
                    .title("Files locations")
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
