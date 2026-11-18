use std::ops::Range;

/// Which rows of a table are in view and which one is selected, with no knowledge of how the
/// table is drawn.
///
/// Like a page in a browser, the view and the selection move independently: scrolling moves
/// only the view, so the selected row can scroll off screen, and moving the selection scrolls
/// the view just far enough to bring it back. That last step waits for [`Self::settle`], because
/// only the renderer knows how many rows fit.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScrollModel {
    len: usize,
    /// Index of the first row in view.
    offset: usize,
    /// Index of the selected row.
    selected: usize,
    /// Whether the next [`Self::settle`] should scroll the selected row into view.
    reveal: bool,
}

impl ScrollModel {
    #[must_use]
    pub const fn new(len: usize) -> Self {
        Self {
            len,
            offset: 0,
            selected: 0,
            reveal: false,
        }
    }

    /// Returns the number of rows.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns the index of the first row in view.
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the index of the selected row, which may be off screen.
    #[must_use]
    pub const fn selected(&self) -> usize {
        self.selected
    }

    /// Returns the furthest the view can scroll with `visible` rows on screen.
    #[must_use]
    pub const fn max_offset(&self, visible: usize) -> usize {
        self.len.saturating_sub(visible)
    }

    /// Returns the rows in view with `visible` rows on screen.
    #[must_use]
    pub fn visible_range(&self, visible: usize) -> Range<usize> {
        self.offset..self.offset.saturating_add(visible).min(self.len)
    }

    /// Returns the selected row if it is in view with `visible` rows on screen.
    #[must_use]
    pub fn highlighted(&self, visible: usize) -> Option<usize> {
        self.visible_range(visible)
            .contains(&self.selected)
            .then_some(self.selected)
    }

    /// Settles the view for `visible` rows on screen, as the renderer does before each draw. The
    /// view never scrolls past the last row, which matters after the terminal grows taller, and
    /// if the selection moved since the last draw, the view scrolls just far enough to show it.
    pub fn settle(&mut self, visible: usize) {
        self.offset = self.offset.min(self.max_offset(visible));
        if self.reveal && visible > 0 {
            self.offset = self
                .offset
                .clamp((self.selected + 1).saturating_sub(visible), self.selected);
        }
        self.reveal = false;
    }

    /// Selects row `index`, clamped to the last row, and has the next [`Self::settle`] scroll it
    /// into view.
    pub fn select(&mut self, index: usize) {
        self.selected = index.min(self.len.saturating_sub(1));
        self.reveal = true;
    }

    /// Moves the selection down one row, stopping at the last row.
    pub fn next_row(&mut self) {
        self.select(self.selected.saturating_add(1));
    }

    /// Moves the selection up one row, stopping at the first row.
    pub fn previous_row(&mut self) {
        self.select(self.selected.saturating_sub(1));
    }

    /// Selects the first row, as Home does.
    pub fn first_row(&mut self) {
        self.select(0);
    }

    /// Selects the last row, as End does.
    pub fn last_row(&mut self) {
        self.select(usize::MAX);
    }

    /// Moves the selection down a screen, and the view with it, as Page Down does. A selected
    /// row on screen keeps its place until the view reaches the bottom, after which the selection
    /// carries on to the last row.
    pub fn page_down(&mut self, visible: usize) {
        let visible = visible.max(1);
        self.scroll_by(visible.cast_signed(), visible);
        self.select(self.selected.saturating_add(visible));
    }

    /// Moves the selection up a screen, and the view with it, as Page Up does. A selected row on
    /// screen keeps its place until the view reaches the top, after which the selection carries
    /// on to the first row.
    pub fn page_up(&mut self, visible: usize) {
        let visible = visible.max(1);
        self.scroll_by(-visible.cast_signed(), visible);
        self.select(self.selected.saturating_sub(visible));
    }

    /// Scrolls the view by `delta` rows (negative scrolls up), as a mouse wheel does, stopping at
    /// the top and bottom. This leaves the selection alone, so it can scroll off screen.
    pub fn scroll_by(&mut self, delta: isize, visible: usize) {
        let max_offset = self.max_offset(visible.max(1));
        self.offset = self
            .offset
            .min(max_offset)
            .saturating_add_signed(delta)
            .min(max_offset);
    }

    /// Scrolls the view so that row `offset` is at the top, or as near as the rows allow. This
    /// leaves the selection alone.
    pub fn scroll_to(&mut self, offset: usize, visible: usize) {
        self.offset = offset.min(self.max_offset(visible.max(1)));
    }
}

#[cfg(test)]
mod tests {
    use super::ScrollModel;

    fn view(model: &ScrollModel) -> (usize, usize) {
        (model.offset(), model.selected())
    }

    /// Selects `index` and settles with `visible` rows, as a key press followed by a draw does.
    fn select(model: &mut ScrollModel, index: usize, visible: usize) {
        model.select(index);
        model.settle(visible);
    }

    #[test]
    fn wheel_moves_view_only() {
        let mut model = ScrollModel::new(100);
        select(&mut model, 2, 10);
        model.scroll_by(3, 10);
        assert_eq!(view(&model), (3, 2));
        model.scroll_by(-3, 10);
        assert_eq!(view(&model), (0, 2));
    }

    #[test]
    fn wheel_stops_at_ends() {
        let mut model = ScrollModel::new(20);
        model.scroll_by(100, 10);
        assert_eq!(view(&model), (10, 0));
        model.scroll_by(-100, 10);
        assert_eq!(view(&model), (0, 0));
    }

    #[test]
    fn wheel_on_short_table_does_nothing() {
        let mut model = ScrollModel::new(4);
        model.scroll_by(3, 10);
        assert_eq!(view(&model), (0, 0));
    }

    #[test]
    fn selection_can_scroll_off_screen() {
        let mut model = ScrollModel::new(100);
        select(&mut model, 2, 15);
        assert_eq!(model.highlighted(15), Some(2));
        model.scroll_by(30, 15);
        model.settle(15);
        assert_eq!(view(&model), (30, 2));
        assert_eq!(model.highlighted(15), None);
        model.scroll_by(-30, 15);
        model.settle(15);
        assert_eq!(model.highlighted(15), Some(2));
    }

    #[test]
    fn keys_bring_off_screen_selection_into_view() {
        let mut model = ScrollModel::new(100);
        select(&mut model, 2, 15);
        // Selection above the view: the next row lands at the top.
        model.scroll_by(30, 15);
        model.settle(15);
        model.next_row();
        model.settle(15);
        assert_eq!(view(&model), (3, 3));
        // Selection below the view: the previous row lands at the bottom.
        select(&mut model, 80, 15);
        model.scroll_by(-80, 15);
        model.settle(15);
        model.previous_row();
        model.settle(15);
        assert_eq!(view(&model), (65, 79));
    }

    #[test]
    fn keys_scroll_only_as_far_as_needed() {
        let mut model = ScrollModel::new(30);
        for _ in 0..14 {
            model.next_row();
        }
        model.settle(15);
        assert_eq!(view(&model), (0, 14));
        model.next_row();
        model.settle(15);
        assert_eq!(view(&model), (1, 15));
    }

    #[test]
    fn keys_stop_at_ends() {
        let mut model = ScrollModel::new(2);
        model.previous_row();
        assert_eq!(model.selected(), 0);
        model.next_row();
        model.next_row();
        assert_eq!(model.selected(), 1);
    }

    #[test]
    fn empty_table_does_not_panic() {
        let mut model = ScrollModel::new(0);
        model.scroll_by(3, 10);
        model.next_row();
        model.page_down(10);
        model.settle(10);
        assert_eq!(view(&model), (0, 0));
        assert_eq!(model.highlighted(10), None);
        assert_eq!(model.visible_range(10), 0..0);
    }

    #[test]
    fn growing_taller_pulls_view_back() {
        let mut model = ScrollModel::new(20);
        model.scroll_by(100, 10);
        assert_eq!(view(&model), (10, 0));
        model.settle(25);
        assert_eq!(view(&model), (0, 0));
    }

    #[test]
    fn page_keys_move_a_screen_and_stop_at_ends() {
        let mut model = ScrollModel::new(40);
        select(&mut model, 2, 15);
        let page = |model: &mut ScrollModel, down: bool| {
            if down {
                model.page_down(15);
            } else {
                model.page_up(15);
            }
            model.settle(15);
            view(model)
        };
        assert_eq!(page(&mut model, true), (15, 17));
        // The view can only move 10 more rows, but the selection moves a whole screen.
        assert_eq!(page(&mut model, true), (25, 32));
        assert_eq!(page(&mut model, true), (25, 39));
        assert_eq!(page(&mut model, false), (10, 24));
        assert_eq!(page(&mut model, false), (0, 9));
        assert_eq!(page(&mut model, false), (0, 0));
    }

    #[test]
    fn page_down_from_off_screen_selection_brings_it_into_view() {
        let mut model = ScrollModel::new(100);
        select(&mut model, 2, 15);
        model.scroll_by(50, 15);
        model.settle(15);
        model.page_down(15);
        model.settle(15);
        assert_eq!(view(&model), (17, 17));
    }

    #[test]
    fn home_and_end_select_ends() {
        let mut model = ScrollModel::new(40);
        model.last_row();
        model.settle(15);
        assert_eq!(view(&model), (25, 39));
        model.first_row();
        model.settle(15);
        assert_eq!(view(&model), (0, 0));
    }

    #[test]
    fn scroll_to_clamps() {
        let mut model = ScrollModel::new(40);
        model.scroll_to(100, 15);
        assert_eq!(model.offset(), 25);
        model.scroll_to(7, 15);
        assert_eq!(model.offset(), 7);
    }

    #[test]
    fn visible_range_stops_at_last_row() {
        let mut model = ScrollModel::new(20);
        model.scroll_by(15, 10);
        assert_eq!(model.visible_range(10), 10..20);
        assert_eq!(ScrollModel::new(3).visible_range(10), 0..3);
    }
}
