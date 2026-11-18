use std::borrow::Cow;

use inno::entry::Language;
use ratatui::layout::Flex;

use crate::tui::{
    emoji::Emoji,
    table::{Column, EntryTable},
};

pub fn table(languages: &[Language]) -> EntryTable<'_, Language> {
    EntryTable::new(
        "Languages",
        languages,
        vec![
            Column::new("Name", |language| Cow::Borrowed(language.name())),
            Column::new("Language name", |language| {
                Cow::Borrowed(language.language_name())
            }),
            Column::new("ID", |language| language.id().to_string().into()),
            Column::new("Codepage", |language| {
                Cow::Borrowed(language.codepage().name())
            }),
            Column::new("Dialog Font (pt)", |language| {
                let font = language.dialog_font();
                format!("{} ({})", font.name(), font.size()).into()
            }),
            Column::new("Title Font (pt)", |language| {
                format!("{} ({})", language.title_font(), language.title_font_size()).into()
            }),
            Column::new("RTL", |language| {
                Cow::Borrowed(language.right_to_left().emoji())
            }),
        ],
    )
    .flex(Flex::Legacy)
}
