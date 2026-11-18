use inno::entry::Directory;

use crate::tui::table::{Column, EntryTable};

pub fn table(directories: &[Directory]) -> EntryTable<'_, Directory> {
    EntryTable::new(
        "Directories",
        directories,
        vec![
            Column::new("Name", |directory| {
                directory.name().unwrap_or_default().as_str().into()
            }),
            Column::new("Permissions", |directory| {
                directory.permissions().unwrap_or_default().into()
            }),
            Column::new("Attributes", |directory| {
                directory.attributes().to_string().into()
            }),
            Column::new("Permission", |directory| {
                directory.permission().to_string().into()
            }),
            Column::new("Flags", |directory| directory.flags().to_string().into()),
        ],
    )
}
