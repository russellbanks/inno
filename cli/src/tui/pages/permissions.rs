use std::borrow::Cow;

use inno::entry::Permission;

use crate::tui::table::{Column, EntryTable};

pub fn table(permissions: &[Permission]) -> EntryTable<'_, Permission> {
    EntryTable::new(
        "Permissions",
        permissions,
        vec![Column::new("Permissions", |permission| {
            Cow::Borrowed(permission.as_str())
        })],
    )
}
