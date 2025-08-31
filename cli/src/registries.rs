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
    widgets::{
        Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};

use super::constraint::{flags_constraint, int_constraint, ints_constraint, strings_constraint};

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
    state: TableState,
    scroll_state: ScrollbarState,
    constraints: [Constraint; 9],
}

impl<'a> RegistryEntries<'a> {
    pub fn new(registries: &'a [RegistryEntry]) -> Self {
        Self {
            registries,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(registries.len()),
            constraints: constraints(registries),
        }
    }

    pub fn next_row(&mut self) {
        let index = self
            .state
            .selected()
            .filter(|&index| index < self.registries.len())
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
            &mut self.scroll_state,
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
            &mut self.state,
        );
    }
}

fn constraints(registries: &[RegistryEntry]) -> [Constraint; 9] {
    [
        Max(int_constraint(registries.len())),
        Max(strings_constraint(
            registries.iter().map(RegistryEntry::key),
            KEY,
        )),
        Max(strings_constraint(
            registries.iter().map(RegistryEntry::name),
            NAME,
        )),
        Max(strings_constraint(
            registries.iter().map(RegistryEntry::value),
            VALUE,
        )),
        Max(strings_constraint(
            registries.iter().map(RegistryEntry::permissions),
            PERMISSIONS,
        )),
        Max(strings_constraint(
            registries
                .iter()
                .map(|entry| entry.registry_root().as_str()),
            ROOT,
        )),
        Max(ints_constraint(
            registries.iter().map(RegistryEntry::permission),
            PERMISSION,
        )),
        Max(strings_constraint(
            registries.iter().map(|entry| entry.r#type().as_str()),
            TYPE,
        )),
        Max(flags_constraint(
            registries.iter().map(RegistryEntry::flags),
            FLAGS,
        )),
    ]
}
