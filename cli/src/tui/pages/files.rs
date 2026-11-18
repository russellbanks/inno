use std::borrow::Cow;

use inno::entry::File;

use crate::tui::table::{Column, EntryTable};

pub fn table(files: &[File]) -> EntryTable<'_, File> {
    EntryTable::new(
        "Files",
        files,
        vec![
            Column::new("Source", |file| file.source().unwrap_or_default().into()),
            Column::new("Destination", |file: &File| {
                file.destination().unwrap_or_default().as_str().into()
            })
            .fixed(),
            Column::new("Font name", |file| {
                file.install_font_name().unwrap_or_default().into()
            }),
            Column::new("Assembly name", |file| {
                file.strong_assembly_name().unwrap_or_default().into()
            }),
            Column::new("Size", |file| file.external_size().to_string().into()),
            Column::new("Flags", |file| file.flags().to_string().into()),
            Column::new("Type", |file| Cow::Borrowed(file.r#type().as_str())),
        ],
    )
}
