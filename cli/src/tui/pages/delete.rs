use std::borrow::Cow;

use inno::entry::DeleteEntry;

use crate::tui::table::{Column, EntryTable};

/// Files and directories deleted when installing.
pub fn install_table(entries: &[DeleteEntry]) -> EntryTable<'_, DeleteEntry> {
    table("Delete (Install)", entries)
}

/// Files and directories deleted when uninstalling.
pub fn uninstall_table(entries: &[DeleteEntry]) -> EntryTable<'_, DeleteEntry> {
    table("Delete (Uninstall)", entries)
}

fn table<'a>(title: &'static str, entries: &'a [DeleteEntry]) -> EntryTable<'a, DeleteEntry> {
    EntryTable::new(
        title,
        entries,
        vec![
            Column::new("Name", |entry| Cow::Borrowed(entry.name().as_str())),
            Column::new("Target type", |entry| {
                Cow::Borrowed(entry.target_type().as_str())
            }),
        ],
    )
}
