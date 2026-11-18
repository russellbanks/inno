use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Modifier, Style, Stylize},
    text::Line,
    widgets::{Tabs, Widget},
};

use super::table::Page;

/// Columns of padding that [`Tabs`] draws on each side of a title by default.
const TAB_PADDING_WIDTH: u16 = 1;

/// Columns taken by the divider that [`Tabs`] draws between titles by default.
const TAB_DIVIDER_WIDTH: u16 = 1;

/// The pages of the viewer and which one is shown, drawn as a row of tabs.
pub struct TabBar<'a> {
    pages: Vec<Box<dyn Page + 'a>>,
    current: usize,
    /// Where the tabs were last drawn.
    area: Rect,
}

impl<'a> TabBar<'a> {
    /// Creates a tab bar showing the first of `pages`, which must not be empty.
    #[must_use]
    pub fn new(pages: Vec<Box<dyn Page + 'a>>) -> Self {
        assert!(!pages.is_empty(), "a tab bar needs at least one page");
        Self {
            pages,
            current: 0,
            area: Rect::ZERO,
        }
    }

    /// Returns the page being shown.
    pub fn current(&mut self) -> &mut (dyn Page + 'a) {
        self.pages[self.current].as_mut()
    }

    /// Shows the next page, stopping at the last.
    pub fn next(&mut self) {
        self.current = (self.current + 1).min(self.pages.len() - 1);
    }

    /// Shows the previous page, stopping at the first.
    pub const fn previous(&mut self) {
        self.current = self.current.saturating_sub(1);
    }

    /// Returns `true` if `position` is on the tab bar as last drawn.
    #[must_use]
    pub const fn contains(&self, position: Position) -> bool {
        self.area.contains(position)
    }

    /// Shows the page whose tab is drawn at `position`, as a mouse click does. Clicks on a
    /// divider or past the last tab are ignored.
    pub fn click(&mut self, position: Position) {
        if let Some(index) = tab_at(
            self.pages.iter().map(|page| page.title()),
            position,
            self.area,
        ) {
            self.current = index;
        }
    }

    /// Draws the tabs into `area` and remembers where, for [`Self::click`].
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        self.area = area;
        Tabs::new(self.pages.iter().map(|page| page.title()))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED).magenta())
            .select(self.current)
            .render(area, buf);
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
