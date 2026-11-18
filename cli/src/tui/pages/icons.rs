use std::borrow::Cow;

use inno::entry::Icon;
use ratatui::layout::Flex;

use crate::tui::table::{Column, EntryTable};

pub fn table(icons: &[Icon]) -> EntryTable<'_, Icon> {
    EntryTable::new(
        "Icons",
        icons,
        vec![
            Column::new("Name", |icon| {
                icon.name().unwrap_or_default().as_str().into()
            }),
            Column::new("Filename", |icon| {
                icon.filename().unwrap_or_default().as_str().into()
            }),
            Column::new("Parameters", |icon| {
                icon.parameters().unwrap_or_default().into()
            }),
            Column::new("Working Directory", |icon| {
                icon.working_directory().unwrap_or_default().into()
            }),
            Column::new("File", |icon| icon.file().unwrap_or_default().into()),
            Column::new("Comment", |icon| icon.comment().unwrap_or_default().into()),
            Column::new("App user model ID", |icon| {
                icon.app_user_model_id().unwrap_or_default().into()
            }),
            Column::new("Toast activator CLSID", |icon| {
                Cow::Borrowed(icon.app_user_model_toast_activator_clsid())
            }),
            Column::new("Index", |icon| icon.index().to_string().into()),
            Column::new("Show command", |icon| {
                icon.show_command().to_string().into()
            }),
            Column::new("Close on exit", |icon| {
                Cow::Borrowed(icon.close_on_exit().as_str())
            }),
            Column::new("Hotkey", |icon| icon.hotkey().to_string().into()),
            Column::new("Flags", |icon| icon.flags().to_string().into()),
        ],
    )
    .flex(Flex::Legacy)
}
