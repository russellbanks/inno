use std::borrow::Cow;

use inno::entry::RegistryEntry;

use crate::tui::table::{Column, EntryTable};

pub fn table(entries: &[RegistryEntry]) -> EntryTable<'_, RegistryEntry> {
    EntryTable::new(
        "Registry",
        entries,
        vec![
            Column::new("Key", |entry| entry.key().unwrap_or_default().into()),
            Column::new("Name", |entry| entry.name().unwrap_or_default().into()),
            Column::new("Value", |entry| entry.value().unwrap_or_default().into()),
            Column::new("Permissions", |entry| {
                entry.permissions().unwrap_or_default().into()
            }),
            Column::new("Root", |entry| {
                Cow::Borrowed(entry.registry_root().as_str())
            }),
            Column::new("Permission", |entry| entry.permission().to_string().into()),
            Column::new("Type", |entry| Cow::Borrowed(entry.r#type().as_str())),
            Column::new("Flags", |entry| entry.flags().to_string().into()),
        ],
    )
}
