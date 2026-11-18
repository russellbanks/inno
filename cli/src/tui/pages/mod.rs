//! One table per kind of installer entry. Each module only declares its columns.

mod components;
mod delete;
mod directories;
mod file_locations;
mod files;
mod icons;
mod ini;
mod languages;
mod messages;
mod permissions;
mod registry;
mod summary;
mod tasks;
mod types;

use std::io::{Read, Seek};

use inno::Inno;

use super::table::{EntryTable, Page};

/// Returns a page for each kind of entry the installer has, in tab order. The summary is always
/// first, and entry kinds with no entries are left out.
pub fn all<'a, R: Read + Seek>(inno: &'a Inno<R>) -> Vec<Box<dyn Page + 'a>> {
    let mut pages = vec![page(summary::table(inno.header(), inno.version()))];
    pages.extend(
        [
            non_empty(languages::table(inno.languages())),
            non_empty(messages::table(inno.messages().collect())),
            non_empty(permissions::table(inno.permissions())),
            non_empty(types::table(inno.type_entries())),
            non_empty(components::table(inno.components())),
            non_empty(tasks::table(inno.tasks())),
            non_empty(directories::table(inno.directories())),
            non_empty(files::table(inno.file_entries())),
            non_empty(file_locations::table(inno.file_locations())),
            non_empty(icons::table(inno.icon_entries())),
            non_empty(ini::table(inno.ini_entries())),
            non_empty(registry::table(inno.registry_entries())),
            non_empty(delete::install_table(inno.delete_entries())),
            non_empty(delete::uninstall_table(inno.uninstall_delete_entries())),
        ]
        .into_iter()
        .flatten(),
    );
    pages
}

fn page<'a, T: 'a>(table: EntryTable<'a, T>) -> Box<dyn Page + 'a> {
    Box::new(table)
}

fn non_empty<'a, T: 'a>(table: EntryTable<'a, T>) -> Option<Box<dyn Page + 'a>> {
    (!table.is_empty()).then(|| page(table))
}
