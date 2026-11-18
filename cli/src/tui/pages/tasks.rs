use std::borrow::Cow;

use inno::entry::Task;

use crate::tui::{
    emoji::Emoji,
    table::{Column, EntryTable},
};

pub fn table(tasks: &[Task]) -> EntryTable<'_, Task> {
    EntryTable::new(
        "Tasks",
        tasks,
        vec![
            Column::new("Name", |task| task.name().unwrap_or_default().into()),
            Column::new("Description", |task| {
                task.description().unwrap_or_default().into()
            }),
            Column::new("Group Description", |task| {
                task.group_description().unwrap_or_default().into()
            }),
            Column::new("Components", |task| {
                task.components().unwrap_or_default().into()
            }),
            Column::new("Languages", |task| {
                task.languages().unwrap_or_default().into()
            }),
            Column::new("Check", |task| task.check().unwrap_or_default().into()),
            Column::new("Level", |task| task.level().to_string().into()),
            Column::new("Used", |task| Cow::Borrowed(task.used().emoji())),
            Column::new("Flags", |task| task.flags().to_string().into()),
        ],
    )
}
