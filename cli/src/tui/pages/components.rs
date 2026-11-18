use std::borrow::Cow;

use inno::entry::Component;

use crate::tui::{
    emoji::Emoji,
    table::{Column, EntryTable},
};

pub fn table(components: &[Component]) -> EntryTable<'_, Component> {
    EntryTable::new(
        "Components",
        components,
        vec![
            Column::new("Name", |component| {
                component.name().unwrap_or_default().into()
            }),
            Column::new("Description", |component| {
                component.description().unwrap_or_default().into()
            }),
            Column::new("Types", |component| {
                component.types().unwrap_or_default().into()
            }),
            Column::new("Languages", |component| {
                component.languages().unwrap_or_default().into()
            }),
            Column::new("Check", |component| {
                component.check_once().unwrap_or_default().into()
            }),
            Column::new("Disk space required", |component| {
                component.extra_disk_space_required().to_string().into()
            }),
            Column::new("Level", |component| component.level().to_string().into()),
            Column::new("Used", |component| Cow::Borrowed(component.used().emoji())),
            Column::new("Flags", |component| component.flags().to_string().into()),
            Column::new("Size", |component| component.size().to_string().into()),
        ],
    )
}
