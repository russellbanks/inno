use std::borrow::Cow;

use inno::entry::FileLocation;
use ratatui::layout::Flex;

use crate::tui::table::{Column, EntryTable};

pub fn table(file_locations: &[FileLocation]) -> EntryTable<'_, FileLocation> {
    EntryTable::new(
        "File Locations",
        file_locations,
        vec![
            Column::new("Uncompressed size", |location| {
                location.uncompressed_size().to_string().into()
            }),
            Column::new("Created at", |location| {
                location.date_time().to_string().into()
            }),
            Column::new("File version", |location| {
                location.file_version().to_string().into()
            }),
            Column::new("Options", |location| {
                location.file_option_flags().to_string().into()
            }),
            Column::new("Sign mode", |location| {
                Cow::Borrowed(location.sign_mode().as_str())
            }),
        ],
    )
    .flex(Flex::Legacy)
}
