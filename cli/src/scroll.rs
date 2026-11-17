use ratatui::widgets::{ScrollbarState, TableState};

/// Rows taken up inside a table page's area by everything except data rows: the top and bottom
/// borders, the vertical padding from `Padding::proportional(1)`, and the header row.
const TABLE_CHROME_HEIGHT: u16 = 2 + 2 + 1;

/// Returns how many data rows a table page shows when it is rendered into an area `height` rows
/// tall.
#[must_use]
pub const fn visible_rows(height: u16) -> usize {
    height.saturating_sub(TABLE_CHROME_HEIGHT) as usize
}

/// Selection, viewport, and scrollbar state shared by every table page.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableScroll {
    pub state: TableState,
    pub scrollbar: ScrollbarState,
    len: usize,
}

impl TableScroll {
    #[must_use]
    pub fn new(len: usize) -> Self {
        Self {
            state: TableState::new().with_selected(0),
            scrollbar: ScrollbarState::new(len),
            len,
        }
    }

    /// Moves the selection down one row, stopping at the last row.
    pub fn next_row(&mut self) {
        let index = self.selected().saturating_add(1);
        self.select(index);
    }

    /// Moves the selection up one row, stopping at the first row.
    pub fn previous_row(&mut self) {
        let index = self.selected().saturating_sub(1);
        self.select(index);
    }

    /// Scrolls the viewport by `delta` rows (negative scrolls up), as a mouse wheel does.
    ///
    /// The content moves while the highlighted row keeps its position on screen. Once the view
    /// reaches the top or bottom it cannot move any further, so the highlight carries on to the
    /// first or last row instead.
    pub fn scroll_by(&mut self, delta: isize, visible_rows: usize) {
        if self.len == 0 {
            return;
        }

        let visible_rows = visible_rows.max(1);
        let max_offset = self.len.saturating_sub(visible_rows);
        let offset = self
            .state
            .offset()
            .min(max_offset)
            .saturating_add_signed(delta)
            .min(max_offset);

        let selected = self
            .selected()
            .saturating_add_signed(delta)
            .clamp(offset, offset + visible_rows - 1);

        *self.state.offset_mut() = offset;
        self.select(selected);
    }

    fn selected(&self) -> usize {
        self.state.selected().unwrap_or_default()
    }

    fn select(&mut self, index: usize) {
        let index = index.min(self.len.saturating_sub(1));
        self.state.select(Some(index));
        self.scrollbar = self.scrollbar.position(index);
    }
}

#[cfg(test)]
mod tests {
    use super::TableScroll;

    fn view(scroll: &TableScroll) -> (usize, Option<usize>) {
        (scroll.state.offset(), scroll.state.selected())
    }

    #[test]
    fn wheel_moves_view_and_keeps_highlight_in_place() {
        let mut scroll = TableScroll::new(100);
        scroll.select(2);
        scroll.scroll_by(3, 10);
        assert_eq!(view(&scroll), (3, Some(5)));
        scroll.scroll_by(-3, 10);
        assert_eq!(view(&scroll), (0, Some(2)));
    }

    #[test]
    fn wheel_moves_highlight_once_view_hits_an_end() {
        let mut scroll = TableScroll::new(20);
        scroll.scroll_by(100, 10);
        assert_eq!(view(&scroll), (10, Some(19)));
        scroll.scroll_by(-100, 10);
        assert_eq!(view(&scroll), (0, Some(0)));
    }

    #[test]
    fn wheel_on_short_table_moves_highlight() {
        let mut scroll = TableScroll::new(4);
        scroll.scroll_by(3, 10);
        assert_eq!(view(&scroll), (0, Some(3)));
    }

    #[test]
    fn keys_stop_at_ends() {
        let mut scroll = TableScroll::new(2);
        scroll.previous_row();
        assert_eq!(scroll.state.selected(), Some(0));
        scroll.next_row();
        scroll.next_row();
        assert_eq!(scroll.state.selected(), Some(1));
    }

    #[test]
    fn empty_table_does_not_panic() {
        let mut scroll = TableScroll::new(0);
        scroll.scroll_by(3, 10);
        scroll.next_row();
    }
}
