use std::borrow::Cow;

use inno::entry::Directory;
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Directories<'a> {
    directories: &'a [Directory],
    state: TableState,
    scroll_state: ScrollbarState,
}

impl<'a> Directories<'a> {
    pub fn new(directories: &'a [Directory]) -> Self {
        Self {
            directories,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(directories.len()),
        }
    }

    pub fn next_row(&mut self) {
        let index = self
            .state
            .selected()
            .filter(|&index| index < self.directories.len())
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

impl Widget for &mut Directories<'_> {
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

        let rows = self
            .directories
            .iter()
            .enumerate()
            .map(|(index, directory)| {
                Row::new([
                    Cow::Owned((index + 1).to_string()),
                    Cow::Borrowed(directory.name().unwrap_or_default()),
                    Cow::Borrowed(directory.permissions().unwrap_or_default()),
                    Cow::Owned(directory.attributes().to_string()),
                    Cow::Owned(directory.permission().to_string()),
                    Cow::Owned(directory.flags().to_string()),
                ])
            });

        StatefulWidget::render(
            Table::new(
                rows,
                [
                    Length(2),
                    Length(4),
                    Length(11),
                    Length(10),
                    Length(10),
                    Length(5),
                ],
            )
            .header(
                Row::new([
                    "#",
                    "Name",
                    "Permissions",
                    "Attributes",
                    "Permission",
                    "Flags",
                ])
                .style(Style::new().add_modifier(Modifier::BOLD)),
            )
            .column_spacing(2)
            .block(
                Block::bordered()
                    .title("Directories")
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
