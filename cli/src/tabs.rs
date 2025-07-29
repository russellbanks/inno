use inno::Inno;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Style,
    style::{Modifier, Stylize},
    widgets::{Tabs, Widget},
};

use super::{DeleteEntries, Messages, Page, Summary};

pub struct TabManager<'a> {
    pages: Vec<Page<'a>>,
    current_tab: usize,
}

impl<'a> TabManager<'a> {
    /// Creates a new View Tab manager.
    #[must_use]
    pub fn new(inno: &'a Inno) -> Self {
        let mut pages = vec![Page::Header(Summary::new(&inno.header))];

        let languages = inno.languages();
        if !languages.is_empty() {
            pages.push(languages.into());
        }

        if !inno.message_entries().is_empty() {
            pages.push(Page::Messages(Messages::new(
                inno.messages().collect::<Vec<_>>(),
            )));
        }

        let permissions = inno.permissions();
        if !permissions.is_empty() {
            pages.push(permissions.into());
        }

        let types = inno.type_entries();
        if !types.is_empty() {
            pages.push(types.into());
        }

        let components = inno.components();
        if !components.is_empty() {
            pages.push(components.into());
        }

        let tasks = inno.tasks();
        if !tasks.is_empty() {
            pages.push(tasks.into());
        }

        let directories = inno.directories();
        if !directories.is_empty() {
            pages.push(directories.into());
        }

        let files = inno.files();
        if !files.is_empty() {
            pages.push(files.into());
        }

        let file_locations = inno.file_locations();
        if !file_locations.is_empty() {
            pages.push(file_locations.into());
        }

        let icons = inno.icons();
        if !icons.is_empty() {
            pages.push(icons.into());
        }

        let ini_files = inno.ini_entries();
        if !ini_files.is_empty() {
            pages.push(ini_files.into());
        }

        let registry_entries = inno.registry_entries();
        if !registry_entries.is_empty() {
            pages.push(registry_entries.into());
        }

        let delete_entries = inno.delete_entries();
        if !delete_entries.is_empty() {
            pages.push(DeleteEntries::new_install(delete_entries).into());
        }

        let uninstall_delete_entries = inno.uninstall_delete_entries();
        if !uninstall_delete_entries.is_empty() {
            pages.push(DeleteEntries::new_uninstall(uninstall_delete_entries).into());
        }

        Self {
            pages,
            current_tab: 0,
        }
    }

    /// Returns a reference to the pages.
    #[must_use]
    #[inline]
    pub const fn pages(&self) -> &[Page<'a>] {
        self.pages.as_slice()
    }

    /// Returns a mutable reference to the pages.
    #[must_use]
    #[inline]
    pub const fn pages_mut(&mut self) -> &mut [Page<'a>] {
        self.pages.as_mut_slice()
    }

    /// Returns a reference to the current tab view.
    #[must_use]
    pub const fn current_tab(&self) -> &Page<'a> {
        &self.pages()[self.current_tab]
    }

    /// Returns a mutable reference to the current tab view.
    #[must_use]
    pub const fn current_tab_mut(&mut self) -> &mut Page<'a> {
        let current_tab = self.current_tab;
        &mut self.pages_mut()[current_tab]
    }

    /// Returns the current tab index.
    #[must_use]
    #[inline]
    pub const fn current_index(&self) -> usize {
        self.current_tab
    }

    /// Sets the current tab to the next tab.
    pub fn next_tab(&mut self) {
        self.current_tab = self
            .current_tab
            .saturating_add(1)
            .min(self.pages().len() - 1);
    }

    /// Sets the current tab to the previous tab.
    pub const fn previous_tab(&mut self) {
        self.current_tab = self.current_tab.saturating_sub(1);
    }
}

impl Widget for &TabManager<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Tabs::new(self.pages().iter().map(Page::title))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED).magenta())
            .select(self.current_tab)
            .render(area, buf);
    }
}
