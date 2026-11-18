use inno::entry::{Language, Message};

use crate::tui::table::{Column, EntryTable};

pub fn table<'a>(messages: Vec<Message<'a, 'a>>) -> EntryTable<'a, Message<'a, 'a>> {
    EntryTable::owned(
        "Messages",
        messages,
        vec![
            Column::new("Name", |message| message.name().unwrap_or_default().into()),
            Column::new("Value", |message| {
                message.value().unwrap_or_default().into()
            }),
            Column::new("Language", |message| {
                message
                    .language()
                    .map(Language::name)
                    .unwrap_or_default()
                    .into()
            }),
        ],
    )
}
