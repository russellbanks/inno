use std::borrow::Cow;

use inno::entry::RegistryEntry;
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

use super::constraint::{index_width, width};
use crate::scroll::TableScroll;

const KEY: &str = "Key";
const NAME: &str = "Name";
const VALUE: &str = "Value";
const PERMISSIONS: &str = "Permissions";
const ROOT: &str = "Root";
const PERMISSION: &str = "Permission";
const TYPE: &str = "Type";
const FLAGS: &str = "Flags";

const HEADERS: [&str; 9] = [
    "#",
    KEY,
    NAME,
    VALUE,
    PERMISSIONS,
    ROOT,
    PERMISSION,
    TYPE,
    FLAGS,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryEntries<'a> {
    registries: &'a [RegistryEntry],
    scroll: TableScroll,
    constraints: [Constraint; 9],
}

impl<'a> RegistryEntries<'a> {
    pub fn new(registries: &'a [RegistryEntry]) -> Self {
        Self {
            registries,
            scroll: TableScroll::new(registries.len()),
            constraints: constraints(registries),
        }
    }

    pub const fn scroll_mut(&mut self) -> &mut TableScroll {
        &mut self.scroll
    }

    /// Returns `true` if there are no registry entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.registries.is_empty()
    }
}

impl Widget for &mut RegistryEntries<'_> {
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

        let rows = self.registries.iter().enumerate().map(|(index, registry)| {
            Row::new([
                Cow::Owned((index + 1).to_string()),
                Cow::Borrowed(registry.key().unwrap_or_default()),
                Cow::Borrowed(registry.name().unwrap_or_default()),
                Cow::Borrowed(registry.value().unwrap_or_default()),
                Cow::Borrowed(registry.permissions().unwrap_or_default()),
                Cow::Borrowed(registry.registry_root().as_str()),
                Cow::Owned(registry.permission().to_string()),
                Cow::Borrowed(registry.r#type().as_str()),
                Cow::Owned(registry.flags().to_string()),
            ])
        });

        StatefulWidget::render(
            Table::new(rows, self.constraints)
                .header(Row::new(HEADERS).style(Style::new().add_modifier(Modifier::BOLD)))
                .column_spacing(2)
                .block(
                    Block::bordered()
                        .title("Permissions")
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

fn constraints(registries: &[RegistryEntry]) -> [Constraint; 9] {
    [
        Max(index_width(registries.len())),
        Max(width(
            KEY,
            registries
                .iter()
                .map(|registry| registry.key().unwrap_or_default()),
        )),
        Max(width(
            NAME,
            registries
                .iter()
                .map(|registry| registry.name().unwrap_or_default()),
        )),
        Max(width(
            VALUE,
            registries
                .iter()
                .map(|registry| registry.value().unwrap_or_default()),
        )),
        Max(width(
            PERMISSIONS,
            registries
                .iter()
                .map(|registry| registry.permissions().unwrap_or_default()),
        )),
        Max(width(
            ROOT,
            registries
                .iter()
                .map(|registry| registry.registry_root().as_str()),
        )),
        Max(width(
            PERMISSION,
            registries.iter().map(RegistryEntry::permission),
        )),
        Max(width(
            TYPE,
            registries.iter().map(|registry| registry.r#type().as_str()),
        )),
        Max(width(FLAGS, registries.iter().map(RegistryEntry::flags))),
    ]
}
