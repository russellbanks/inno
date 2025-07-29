use std::borrow::Cow;

use inno::entry::DeleteEntry;
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

use super::constraint::{int_constraint, strings_constraint};

const NAME: &str = "Name";
const TARGET_TYPE: &str = "Target type";

const HEADERS: [&str; 3] = ["#", NAME, TARGET_TYPE];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeleteEntries<'a> {
    delete_entries: &'a [DeleteEntry],
    state: TableState,
    scroll_state: ScrollbarState,
    constraints: [Constraint; 3],
    title: &'static str,
}

impl<'a> DeleteEntries<'a> {
    pub fn new_install(delete_entries: &'a [DeleteEntry]) -> Self {
        Self::new(delete_entries, "Delete (Install)")
    }

    pub fn new_uninstall(delete_entries: &'a [DeleteEntry]) -> Self {
        Self::new(delete_entries, "Delete (Uninstall)")
    }

    fn new(delete_entries: &'a [DeleteEntry], title: &'static str) -> Self {
        Self {
            delete_entries,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(delete_entries.len()),
            constraints: constraints(delete_entries),
            title,
        }
    }

    pub fn next_row(&mut self) {
        let index = self
            .state
            .selected()
            .filter(|&index| index < self.delete_entries.len())
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

    /// Returns `true` if there are no delete entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.delete_entries.is_empty()
    }
}

impl Widget for &mut DeleteEntries<'_> {
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
            .delete_entries
            .iter()
            .enumerate()
            .map(|(index, delete_entry)| {
                Row::new([
                    Cow::Owned((index + 1).to_string()),
                    Cow::Borrowed(delete_entry.name()),
                    Cow::Borrowed(delete_entry.target_type().as_str()),
                ])
            });

        StatefulWidget::render(
            Table::new(rows, self.constraints)
                .header(Row::new(HEADERS).style(Style::new().add_modifier(Modifier::BOLD)))
                .column_spacing(2)
                .block(
                    Block::bordered()
                        .title(self.title)
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

fn constraints(delete_entries: &[DeleteEntry]) -> [Constraint; 3] {
    [
        Length(int_constraint(delete_entries.len())),
        Max(strings_constraint(
            delete_entries.iter().map(DeleteEntry::name),
            NAME,
        )),
        Max(strings_constraint(
            delete_entries
                .iter()
                .map(|delete| delete.target_type().as_str()),
            TARGET_TYPE,
        )),
    ]
}
