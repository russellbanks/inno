use std::borrow::Cow;

use inno::entry::Component;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Constraint::Max, Rect},
    prelude::Widget,
    widgets::{Row, Table},
};

use super::{
    constraint::{index_width, width},
    emoji::Emoji,
};
use crate::scroll::TableScroll;

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
    scroll: TableScroll,
    constraints: [Constraint; 11],
}

impl<'a> Components<'a> {
    pub fn new(components: &'a [Component]) -> Self {
        Self {
            components,
            scroll: TableScroll::new(components.len()),
            constraints: constraints(components),
        }
    }

    pub const fn scroll_mut(&mut self) -> &mut TableScroll {
        &mut self.scroll
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

        self.scroll.render(
            Table::new(rows, self.constraints).column_spacing(2),
            HEADERS,
            "Components",
            area,
            buf,
        );
    }
}

fn constraints(components: &[Component]) -> [Constraint; 11] {
    [
        Max(index_width(components.len())),
        Max(width(
            NAME,
            components
                .iter()
                .map(|component| component.name().unwrap_or_default()),
        )),
        Max(width(
            DESCRIPTION,
            components
                .iter()
                .map(|component| component.description().unwrap_or_default()),
        )),
        Max(width(
            TYPES,
            components
                .iter()
                .map(|component| component.types().unwrap_or_default()),
        )),
        Max(width(
            LANGUAGES,
            components
                .iter()
                .map(|component| component.languages().unwrap_or_default()),
        )),
        Max(width(
            CHECK,
            components
                .iter()
                .map(|component| component.check_once().unwrap_or_default()),
        )),
        Max(width(
            DISK_SPACE_REQUIRED,
            components.iter().map(Component::extra_disk_space_required),
        )),
        Max(width(LEVEL, components.iter().map(Component::level))),
        Max(USED.len() as u16),
        Max(width(FLAGS, components.iter().map(Component::flags))),
        Max(width(SIZE, components.iter().map(Component::size))),
    ]
}
