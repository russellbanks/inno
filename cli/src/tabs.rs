use inno::Inno;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Style,
    style::{Modifier, Stylize},
    widgets::{Tabs, Widget},
};

use super::{
    Components, Directories, Files, Languages, Messages, Page, Permissions, Summary, Tasks, Types,
};
use crate::file_locations::FileLocations;

pub struct TabManager<'a> {
    views: [Page<'a>; 10],
    current_tab: usize,
}

impl<'a> TabManager<'a> {
    /// Creates a new View Tab manager.
    #[must_use]
    pub fn new(inno: &'a Inno) -> Self {
        Self {
            views: [
                Page::Header(Summary::new(&inno.header)),
                Page::Languages(Languages::new(inno.languages())),
                Page::Messages(Messages::new(inno.messages())),
                Page::Permissions(Permissions::new(inno.permissions())),
                Page::Types(Types::new(inno.type_entries())),
                Page::Components(Components::new(inno.components())),
                Page::Tasks(Tasks::new(inno.tasks())),
                Page::Directories(Directories::new(inno.directories())),
                Page::Files(Files::new(inno.files())),
                Page::FileLocations(FileLocations::new(inno.file_locations())),
            ],
            current_tab: 0,
        }
    }

    /// Returns a reference to the page views.
    #[must_use]
    #[inline]
    pub const fn views(&'_ self) -> &[Page<'a>; 10] {
        &self.views
    }

    /// Returns a mutable reference to the page views.
    #[must_use]
    #[inline]
    pub const fn views_mut(&'_ mut self) -> &'_ mut [Page<'a>; 10] {
        &mut self.views
    }

    /// Returns a reference to the current tab view.
    #[must_use]
    pub const fn current_tab(&'_ self) -> &'_ Page<'_> {
        &self.views()[self.current_tab]
    }

    /// Returns a mutable reference to the current tab view.
    #[must_use]
    pub const fn current_tab_mut(&mut self) -> &mut Page<'a> {
        let current_tab = self.current_tab;
        &mut self.views_mut()[current_tab]
    }

    /// Returns the current tab index.
    #[must_use]
    #[inline]
    pub const fn current_index(&self) -> usize {
        self.current_tab
    }

    /// Sets the current tab to the next tab.
    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.saturating_add(1).min(self.views.len() - 1);
    }

    /// Sets the current tab to the previous tab.
    pub const fn prev_tab(&mut self) {
        self.current_tab = self.current_tab.saturating_sub(1);
    }
}

impl Widget for &TabManager<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Tabs::new(self.views().iter().map(Page::title))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED).magenta())
            .select(self.current_tab)
            .render(area, buf);
    }
}
