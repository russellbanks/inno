use std::borrow::Cow;

use inno::entry::Task;
use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment, Constraint,
        Constraint::{Fill, Length, Max},
        Layout, Rect,
    },
    prelude::{Modifier, StatefulWidget, Style, Widget},
    style::palette::tailwind::SKY,
    widgets::{Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, Table},
};

use super::emoji::Emoji;
use crate::{
    constraint::{index_width, width},
    scroll::TableScroll,
};

const NAME: &str = "Name";
const DESCRIPTION: &str = "Description";
const GROUP_DESCRIPTION: &str = "Group Description";
const COMPONENTS: &str = "Components";
const LANGUAGES: &str = "Languages";
const CHECK: &str = "Check";
const LEVEL: &str = "Level";
const USED: &str = "Used";
const FLAGS: &str = "Flags";

const HEADERS: [&str; 10] = [
    "#",
    NAME,
    DESCRIPTION,
    GROUP_DESCRIPTION,
    COMPONENTS,
    LANGUAGES,
    CHECK,
    LEVEL,
    USED,
    FLAGS,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tasks<'a> {
    tasks: &'a [Task],
    scroll: TableScroll,
    constraints: [Constraint; 10],
}

impl<'a> Tasks<'a> {
    pub fn new(tasks: &'a [Task]) -> Self {
        Self {
            tasks,
            scroll: TableScroll::new(tasks.len()),
            constraints: constraints(tasks),
        }
    }

    pub const fn scroll_mut(&mut self) -> &mut TableScroll {
        &mut self.scroll
    }

    /// Returns `true` if there are no task entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.tasks.is_empty()
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
            &mut self.scroll.scrollbar,
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
            Table::new(rows, self.constraints)
                .header(Row::new(HEADERS).style(Style::new().add_modifier(Modifier::BOLD)))
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
            &mut self.scroll.state,
        );
    }
}

fn constraints(tasks: &[Task]) -> [Constraint; 10] {
    [
        Max(index_width(tasks.len())),
        Max(width(
            NAME,
            tasks.iter().map(|task| task.name().unwrap_or_default()),
        )),
        Max(width(
            DESCRIPTION,
            tasks
                .iter()
                .map(|task| task.description().unwrap_or_default()),
        )),
        Max(width(
            GROUP_DESCRIPTION,
            tasks
                .iter()
                .map(|task| task.group_description().unwrap_or_default()),
        )),
        Max(width(
            COMPONENTS,
            tasks
                .iter()
                .map(|task| task.components().unwrap_or_default()),
        )),
        Max(width(
            LANGUAGES,
            tasks
                .iter()
                .map(|task| task.languages().unwrap_or_default()),
        )),
        Max(width(
            CHECK,
            tasks.iter().map(|task| task.check().unwrap_or_default()),
        )),
        Max(width(LEVEL, tasks.iter().map(Task::level))),
        Max(USED.len() as u16),
        Max(width(FLAGS, tasks.iter().map(Task::flags))),
    ]
}
