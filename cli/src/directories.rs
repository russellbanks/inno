use std::borrow::Cow;

use inno::entry::Directory;
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

use super::constraint::{int_constraint, ints_constraint, strings_constraint};

const NAME: &str = "Name";
const PERMISSIONS: &str = "Permissions";
const ATTRIBUTES: &str = "Attributes";
const PERMISSION: &str = "Permission";
const FLAGS: &str = "Flags";

const HEADERS: [&str; 6] = ["#", NAME, PERMISSIONS, ATTRIBUTES, PERMISSION, FLAGS];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Directories<'a> {
    directories: &'a [Directory],
    state: TableState,
    scroll_state: ScrollbarState,
    constraints: [Constraint; 6],
}

impl<'a> Directories<'a> {
    pub fn new(directories: &'a [Directory]) -> Self {
        Self {
            directories,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(directories.len()),
            constraints: constraints(directories),
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

    /// Returns `true` if there are no directory entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.directories.is_empty()
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
            Table::new(rows, self.constraints)
                .header(Row::new(HEADERS).style(Style::new().add_modifier(Modifier::BOLD)))
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

fn constraints(directories: &[Directory]) -> [Constraint; 6] {
    [
        Max(int_constraint(directories.len())),
        Max(strings_constraint(
            directories.iter().map(Directory::name),
            NAME,
        )),
        Max(strings_constraint(
            directories.iter().map(Directory::permissions),
            PERMISSION,
        )),
        Max(ints_constraint(
            directories.iter().map(Directory::attributes),
            ATTRIBUTES,
        )),
        Max(ints_constraint(
            directories.iter().map(Directory::permission),
            PERMISSION,
        )),
        Length(FLAGS.len() as u16),
    ]
}
