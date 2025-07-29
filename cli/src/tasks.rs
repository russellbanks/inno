use std::borrow::Cow;

use inno::entry::Task;
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

use super::emoji::Emoji;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tasks<'a> {
    tasks: &'a [Task],
    state: TableState,
    scroll_state: ScrollbarState,
}

impl<'a> Tasks<'a> {
    pub fn new(tasks: &'a [Task]) -> Self {
        Self {
            tasks,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(tasks.len()),
        }
    }

    pub fn next_row(&mut self) {
        let index = self
            .state
            .selected()
            .filter(|&index| index < self.tasks.len())
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

impl Widget for &mut Tasks<'_> {
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

        let rows = self.tasks.iter().enumerate().map(|(index, task)| {
            Row::new([
                Cow::Owned((index + 1).to_string()),
                Cow::Borrowed(task.name().unwrap_or_default()),
                Cow::Borrowed(task.description().unwrap_or_default()),
                Cow::Borrowed(task.group_description().unwrap_or_default()),
                Cow::Borrowed(task.components().unwrap_or_default()),
                Cow::Borrowed(task.languages().unwrap_or_default()),
                Cow::Borrowed(task.check().unwrap_or_default()),
                Cow::Owned(task.level().to_string()),
                Cow::Borrowed(task.used().emoji()),
                Cow::Owned(task.flags().to_string()),
            ])
        });

        StatefulWidget::render(
            Table::new(
                rows,
                [
                    Length(2),
                    Length(4),
                    Length(11),
                    Length(17),
                    Length(10),
                    Length(9),
                    Length(5),
                    Length(5),
                    Length(4),
                    Length(5),
                ],
            )
            .header(
                Row::new([
                    "#",
                    "Name",
                    "Description",
                    "Group Description",
                    "Components",
                    "Languages",
                    "Check",
                    "Level",
                    "Used",
                    "Flags",
                ])
                .style(Style::new().add_modifier(Modifier::BOLD)),
            )
            .column_spacing(2)
            .block(
                Block::bordered()
                    .title("Tasks")
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
