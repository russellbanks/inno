use std::borrow::Cow;

use inno::entry::Ini;

use crate::tui::table::{Column, EntryTable};

pub fn table(entries: &[Ini]) -> EntryTable<'_, Ini> {
    EntryTable::new(
        "INI",
        entries,
        vec![
            Column::new("File", |ini| Cow::Borrowed(ini.file_path().as_str())),
            Column::new("Section", |ini| {
                ini.section_name().unwrap_or_default().into()
            }),
            Column::new("Key", |ini| ini.key_name().unwrap_or_default().into()),
            Column::new("Value", |ini| ini.value().unwrap_or_default().into()),
            Column::new("Flags", |ini| ini.flags().to_string().into()),
        ],
    )
}
