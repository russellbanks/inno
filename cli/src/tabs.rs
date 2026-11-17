use std::io::{Read, Seek};

use inno::Inno;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    prelude::Style,
    style::{Modifier, Stylize},
    text::Line,
    widgets::{Tabs, Widget},
};

/// Columns of padding that [`Tabs`] draws on each side of a title by default.
const TAB_PADDING_WIDTH: u16 = 1;

/// Columns taken by the divider that [`Tabs`] draws between titles by default.
const TAB_DIVIDER_WIDTH: u16 = 1;

use super::{DeleteEntries, Messages, Page, Summary};

pub struct TabManager<'a> {
    pages: Vec<Page<'a>>,
    current_tab: usize,
}

impl<'a> TabManager<'a> {
    /// Creates a new View Tab manager.
    #[must_use]
    pub fn new<R: Read + Seek>(inno: &'a Inno<R>) -> Self {
        let mut pages = vec![Page::Header(Summary::new(inno.header(), inno.version()))];

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

        let files = inno.file_entries();
        if !files.is_empty() {
            pages.push(files.into());
        }

        let file_locations = inno.file_locations();
        if !file_locations.is_empty() {
            pages.push(file_locations.into());
        }

        let icons = inno.icon_entries();
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

    /// Switches to the tab drawn at `position`, as a mouse click does, given the area the tabs
    /// were rendered into. Clicks on a divider or past the last tab are ignored.
    pub fn click(&mut self, position: Position, area: Rect) {
        if let Some(index) = tab_at(self.pages().iter().map(Page::title), position, area) {
            self.current_tab = index;
        }
    }
}

/// Returns the index of the tab whose title or padding is drawn at `position` when `titles` are
/// rendered by [`Tabs`] into `area`, mirroring its default layout: `" title "`, then a one-column
/// divider before the next title.
fn tab_at<'t>(
    titles: impl IntoIterator<Item = &'t str>,
    position: Position,
    area: Rect,
) -> Option<usize> {
    if !area.contains(position) {
        return None;
    }

    let mut x = area.x;
    for (index, title) in titles.into_iter().enumerate() {
        let title_width = u16::try_from(Line::raw(title).width()).unwrap_or(u16::MAX);
        let end = x.saturating_add(TAB_PADDING_WIDTH * 2 + title_width);
        if position.x < end {
            return (position.x >= x).then_some(index);
        }
        x = end.saturating_add(TAB_DIVIDER_WIDTH);
    }
    None
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

#[cfg(test)]
mod tests {
    use ratatui::{
        buffer::Buffer,
        layout::{Position, Rect},
        widgets::{Tabs, Widget},
    };

    use super::tab_at;

    const TITLES: [&str; 3] = ["Header", "INI", "Files"];

    fn at(x: u16) -> Option<usize> {
        tab_at(TITLES, Position::new(x, 0), Rect::new(0, 0, 80, 1))
    }

    #[test]
    fn layout_matches_rendered_tabs() {
        let area = Rect::new(0, 0, 24, 1);
        let mut buf = Buffer::empty(area);
        Tabs::new(TITLES).render(area, &mut buf);
        let rendered: String = buf.content().iter().map(|cell| cell.symbol()).collect();
        assert_eq!(rendered, " Header │ INI │ Files   ");
    }

    #[test]
    fn click_on_title_or_padding_selects_tab() {
        // " Header │ INI │ Files "
        assert_eq!(at(0), Some(0));
        assert_eq!(at(7), Some(0));
        assert_eq!(at(9), Some(1));
        assert_eq!(at(13), Some(1));
        assert_eq!(at(15), Some(2));
        assert_eq!(at(21), Some(2));
    }

    #[test]
    fn click_on_divider_or_past_end_is_ignored() {
        assert_eq!(at(8), None);
        assert_eq!(at(14), None);
        assert_eq!(at(22), None);
        assert_eq!(
            tab_at(TITLES, Position::new(1, 1), Rect::new(0, 0, 80, 1)),
            None
        );
    }
}
