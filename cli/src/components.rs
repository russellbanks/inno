use std::borrow::Cow;

use inno::entry::Component;
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

use super::{
    constraint::{int_constraint, ints_constraint, strings_constraint},
    emoji::Emoji,
};
use crate::constraint::flags_constraint;

const NAME: &str = "Name";
const DESCRIPTION: &str = "Description";
const TYPES: &str = "Types";
const LANGUAGES: &str = "Languages";
const CHECK: &str = "Check";
const DISK_SPACE_REQUIRED: &str = "Disk space required";
const LEVEL: &str = "Level";
const USED: &str = "Used";
const FLAGS: &str = "Flags";
const SIZE: &str = "Size";

const HEADERS: [&str; 11] = [
    "#",
    NAME,
    DESCRIPTION,
    TYPES,
    LANGUAGES,
    CHECK,
    DISK_SPACE_REQUIRED,
    LEVEL,
    USED,
    FLAGS,
    SIZE,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Components<'a> {
    components: &'a [Component],
    state: TableState,
    scroll_state: ScrollbarState,
    constraints: [Constraint; 11],
}

impl<'a> Components<'a> {
    pub fn new(components: &'a [Component]) -> Self {
        Self {
            components,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(components.len()),
            constraints: constraints(components),
        }
    }

    pub fn next_row(&mut self) {
        let index = self
            .state
            .selected()
            .filter(|&index| index < self.components.len())
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

    /// Returns `true` if there are no component entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

impl Widget for &mut Components<'_> {
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
            .components
            .iter()
            .enumerate()
            .map(|(index, component)| {
                Row::new([
                    Cow::Owned((index + 1).to_string()),
                    Cow::Borrowed(component.name().unwrap_or_default()),
                    Cow::Borrowed(component.description().unwrap_or_default()),
                    Cow::Borrowed(component.types().unwrap_or_default()),
                    Cow::Borrowed(component.languages().unwrap_or_default()),
                    Cow::Borrowed(component.check_once().unwrap_or_default()),
                    Cow::Owned(component.extra_disk_space_required().to_string()),
                    Cow::Owned(component.level().to_string()),
                    Cow::Borrowed(component.used().emoji()),
                    Cow::Owned(component.flags().to_string()),
                    Cow::Owned(component.size().to_string()),
                ])
            });

        StatefulWidget::render(
            Table::new(rows, self.constraints)
                .header(Row::new(HEADERS).style(Style::new().add_modifier(Modifier::BOLD)))
                .column_spacing(2)
                .block(
                    Block::bordered()
                        .title("Components")
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

fn constraints(components: &[Component]) -> [Constraint; 11] {
    [
        Max(int_constraint(components.len())),
        Max(strings_constraint(
            components.iter().map(Component::name),
            NAME,
        )),
        Max(strings_constraint(
            components.iter().map(Component::description),
            DESCRIPTION,
        )),
        Max(strings_constraint(
            components.iter().map(Component::types),
            TYPES,
        )),
        Max(strings_constraint(
            components.iter().map(Component::languages),
            LANGUAGES,
        )),
        Max(strings_constraint(
            components.iter().map(Component::check_once),
            CHECK,
        )),
        Max(ints_constraint(
            components.iter().map(Component::extra_disk_space_required),
            DISK_SPACE_REQUIRED,
        )),
        Max(ints_constraint(
            components.iter().map(Component::level),
            LEVEL,
        )),
        Max(USED.len() as u16),
        Max(flags_constraint(
            components.iter().map(Component::flags),
            FLAGS,
        )),
        Max(ints_constraint(
            components.iter().map(Component::size),
            SIZE,
        )),
    ]
}
