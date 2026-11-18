use std::borrow::Cow;

use inno::entry::Type;

use crate::tui::{
    emoji::Emoji,
    table::{Column, EntryTable},
};

pub fn table(types: &[Type]) -> EntryTable<'_, Type> {
    EntryTable::new(
        "Types",
        types,
        vec![
            Column::new("Name", |r#type| r#type.name().unwrap_or_default().into()),
            Column::new("Description", |r#type| {
                r#type.description().unwrap_or_default().into()
            }),
            Column::new("Languages", |r#type| {
                r#type.languages().unwrap_or_default().into()
            }),
            Column::new("Check", |r#type| r#type.check().unwrap_or_default().into()),
            Column::new("Custom", |r#type| Cow::Borrowed(r#type.is_custom().emoji())),
            Column::new("Setup", |r#type| Cow::Borrowed(r#type.setup().as_str())),
            Column::new("Size", |r#type| r#type.size().to_string().into()),
        ],
    )
}
