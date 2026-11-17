use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment,
        Constraint::{Fill, Length},
        Layout, Position, Rect,
    },
    style::{Modifier, Style, palette::tailwind::SKY},
    widgets::{
        Block, BorderType, Cell, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Table, TableState,
    },
};

/// Height of a table page's header row.
const HEADER_HEIGHT: u16 = 1;

/// Returns the bordered, padded block every table page is framed in.
///
/// [`data_rows_area`] measures rows inside this same block, so the mouse lines up with what is
/// drawn.
fn table_block(title: &str) -> Block<'_> {
    Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .padding(Padding::proportional(1))
}

/// Returns the area that data rows occupy when a table is rendered into `table_area`: inside
/// [`table_block`], below a [`HEADER_HEIGHT`]-row header.
fn data_rows_area(table_area: Rect) -> Rect {
    let inner = table_block("").inner(table_area);
    let [_header, rows] = Layout::vertical([Length(HEADER_HEIGHT), Fill(1)]).areas(inner);
    rows
}

/// Cells at each end of the scrollbar taken by its ↑ and ↓ arrows.
const SCROLLBAR_ARROW_HEIGHT: u16 = 1;

/// The part of a table page's scrollbar under the mouse.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollbarHit {
    /// The ↑ arrow at the top.
    UpArrow,
    /// The track above the thumb.
    TrackAbove,
    /// The thumb, `grab` cells below its top.
    Thumb { grab: u16 },
    /// The track below the thumb.
    TrackBelow,
    /// The ↓ arrow at the bottom.
    DownArrow,
}

/// Rounds `numerator / denominator` to the nearest integer, as ratatui's scrollbar does.
const fn rounding_divide(numerator: usize, denominator: usize) -> usize {
    (numerator + denominator / 2) / denominator
}

/// Selection, viewport, and scrollbar state shared by every table page.
///
/// Like a page in a browser, the view and the selection move independently: scrolling moves
/// only the view, so the selected row can scroll off screen, and moving the selection with the
/// keyboard scrolls the view just far enough to bring it back.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableScroll {
    /// What the table is rendered with, rebuilt by [`Self::render_scrollbar`] on every render.
    ///
    /// Ratatui's `Table` always scrolls to keep its selected row on screen, so this only carries
    /// the selection while that row is on screen.
    state: TableState,
    /// Index of the first row in view.
    offset: usize,
    /// Index of the selected row.
    selected: usize,
    /// Whether the next render should scroll the selected row into view.
    reveal: bool,
    len: usize,
}

impl TableScroll {
    #[must_use]
    pub fn new(len: usize) -> Self {
        Self {
            state: TableState::new().with_selected(0),
            offset: 0,
            selected: 0,
            reveal: false,
            len,
        }
    }

    /// Returns the index of the selected row, which may be off screen.
    #[must_use]
    pub const fn selected(&self) -> usize {
        self.selected
    }

    /// Returns `true` if the table has more rows than fit when rendered into `area`.
    #[must_use]
    pub fn needs_scrollbar(&self, area: Rect) -> bool {
        self.len > usize::from(data_rows_area(area).height)
    }

    /// Splits `area` into the table's area and, only if the rows overflow, a one-column
    /// scrollbar on the right.
    #[must_use]
    pub fn layout(&self, area: Rect) -> (Rect, Option<Rect>) {
        if self.needs_scrollbar(area) {
            let [table, scrollbar] = Layout::horizontal([Fill(1), Length(1)]).areas(area);
            (table, Some(scrollbar))
        } else {
            (area, None)
        }
    }

    /// Returns the area that the table's data rows occupy when the page is rendered into `area`.
    #[must_use]
    pub fn rows_area(&self, area: Rect) -> Rect {
        data_rows_area(self.layout(area).0)
    }

    /// Returns how many rows are on screen and the furthest the view can scroll when the page is
    /// rendered into `area`.
    fn viewport(&self, area: Rect) -> (usize, usize) {
        let visible_rows = usize::from(self.rows_area(area).height);
        (visible_rows, self.len.saturating_sub(visible_rows))
    }

    /// Returns the scrollbar's track, the column between its arrows, when the page is rendered
    /// into `area`, or `None` if there is no scrollbar.
    fn track(&self, area: Rect) -> Option<Rect> {
        let mut track = self.layout(area).1?;
        track.y += SCROLLBAR_ARROW_HEIGHT;
        track.height = track.height.saturating_sub(SCROLLBAR_ARROW_HEIGHT * 2);
        Some(track)
    }

    /// Returns where the thumb starts within `track_length` cells of track, and how long it is,
    /// for the view's offset. This is the same arithmetic ratatui 0.29's `Scrollbar` draws with:
    /// both ends of the visible rows are scaled onto the track and rounded separately.
    fn thumb(&self, track_length: u16, area: Rect) -> (u16, u16) {
        let track_length = usize::from(track_length);
        let (visible_rows, max_offset) = self.viewport(area);
        let total = max_offset + visible_rows;
        if track_length == 0 || total == 0 {
            return (0, 0);
        }
        let offset = self.offset.min(max_offset);
        let start = rounding_divide(offset * track_length, total).min(track_length - 1);
        let end = rounding_divide((offset + visible_rows) * track_length, total).min(track_length);
        let length = end.saturating_sub(start).max(1);
        // Both are at most `track_length`, which came from a `u16`.
        (start as u16, length as u16)
    }

    /// Returns the part of the scrollbar at `position` when the page is rendered into `area`, or
    /// `None` if there is no scrollbar or `position` is not on it.
    #[must_use]
    pub fn scrollbar_hit(&self, position: Position, area: Rect) -> Option<ScrollbarHit> {
        if !self.layout(area).1?.contains(position) {
            return None;
        }
        let track = self.track(area)?;
        if position.y < track.top() {
            return Some(ScrollbarHit::UpArrow);
        }
        if position.y >= track.bottom() {
            return Some(ScrollbarHit::DownArrow);
        }
        let (start, length) = self.thumb(track.height, area);
        let y = position.y - track.y;
        Some(if y < start {
            ScrollbarHit::TrackAbove
        } else if y < start + length {
            ScrollbarHit::Thumb { grab: y - start }
        } else {
            ScrollbarHit::TrackBelow
        })
    }

    /// Scrolls the view so that the thumb's top sits `grab` cells above row `y`, as dragging the
    /// thumb does, given the area the page is rendered into. Like the wheel, this leaves the
    /// selection alone.
    pub fn drag_thumb(&mut self, y: u16, grab: u16, area: Rect) {
        let Some(track) = self.track(area).filter(|track| track.height > 0) else {
            return;
        };
        let (visible_rows, max_offset) = self.viewport(area);
        let thumb_start = y.saturating_sub(track.y).saturating_sub(grab);
        self.offset = rounding_divide(
            usize::from(thumb_start) * (max_offset + visible_rows),
            usize::from(track.height),
        )
        .min(max_offset);
    }

    /// Moves the selection down a screen, and the view with it, as Page Down does. A selected
    /// row on screen keeps its place until the view reaches the bottom, after which the selection
    /// carries on to the last row.
    pub fn page_down(&mut self, visible_rows: usize) {
        let visible_rows = visible_rows.max(1);
        self.scroll_by(visible_rows.cast_signed(), visible_rows);
        self.select(self.selected.saturating_add(visible_rows));
    }

    /// Moves the selection up a screen, and the view with it, as Page Up does. A selected row on
    /// screen keeps its place until the view reaches the top, after which the selection carries
    /// on to the first row.
    pub fn page_up(&mut self, visible_rows: usize) {
        let visible_rows = visible_rows.max(1);
        self.scroll_by(-visible_rows.cast_signed(), visible_rows);
        self.select(self.selected.saturating_sub(visible_rows));
    }

    /// Selects the first row, as Home does.
    pub fn first_row(&mut self) {
        self.select(0);
    }

    /// Selects the last row, as End does.
    pub fn last_row(&mut self) {
        self.select(usize::MAX);
    }

    /// Renders a table page into `area`: `table` framed in [`table_block`] with `title`, a bold
    /// one-row `header`, the selected row highlighted, and a scrollbar if the rows overflow.
    ///
    /// Pages supply only their content in `table`: rows, column widths, spacing, and flex. Every
    /// row must be one line tall, as [`Row::new`] makes them, since mouse hit-testing counts one
    /// row per line.
    pub fn render<'a, H>(
        &mut self,
        table: Table<'a>,
        header: H,
        title: &'a str,
        area: Rect,
        buf: &mut Buffer,
    ) where
        H: IntoIterator,
        H::Item: Into<Cell<'a>>,
    {
        let table_area = self.render_scrollbar(area, buf);
        StatefulWidget::render(
            table
                .header(
                    Row::new(header)
                        .height(HEADER_HEIGHT)
                        .style(Style::new().add_modifier(Modifier::BOLD)),
                )
                .block(table_block(title))
                .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED).fg(SKY.c400)),
            table_area,
            buf,
            &mut self.state,
        );
    }

    /// Draws the scrollbar into `area` if the rows overflow it, and returns the area left for
    /// the table, which [`Self::render`] then draws with [`Self::state`].
    ///
    /// This first settles the view: it never scrolls past the last row (which matters after the
    /// terminal grows taller), and after the selection was moved with the keyboard it scrolls
    /// just far enough to bring the selected row on screen. It then rebuilds [`Self::state`] for
    /// the table, which only carries the selection if the selected row is on screen, since
    /// ratatui would otherwise scroll back to it.
    ///
    /// The scrollbar follows the view, like a browser's: it moves only when the view scrolls, and
    /// its thumb shows which part of the rows is on screen.
    fn render_scrollbar(&mut self, area: Rect, buf: &mut Buffer) -> Rect {
        let (table_area, scrollbar_area) = self.layout(area);
        let visible_rows = usize::from(data_rows_area(table_area).height);
        let max_offset = self.len.saturating_sub(visible_rows);
        self.offset = self.offset.min(max_offset);
        if self.reveal && visible_rows > 0 {
            self.offset = self.offset.clamp(
                (self.selected + 1).saturating_sub(visible_rows),
                self.selected,
            );
        }
        self.reveal = false;

        let on_screen = (self.offset..self.offset + visible_rows).contains(&self.selected);
        self.state = TableState::new()
            .with_offset(self.offset)
            .with_selected((on_screen && self.len > 0).then_some(self.selected));

        if let Some(scrollbar_area) = scrollbar_area {
            // Ratatui sizes the thumb as `viewport / (content_length - 1 + viewport)`, so a content
            // length of one more than the furthest offset makes the track stand for every row.
            let mut scrollbar = ScrollbarState::new(max_offset + 1)
                .viewport_content_length(visible_rows)
                .position(self.offset);
            Scrollbar::new(ScrollbarOrientation::VerticalRight).render(
                scrollbar_area,
                buf,
                &mut scrollbar,
            );
        }
        table_area
    }

    /// Moves the selection down one row, stopping at the last row.
    pub fn next_row(&mut self) {
        self.select(self.selected.saturating_add(1));
    }

    /// Moves the selection up one row, stopping at the first row.
    pub fn previous_row(&mut self) {
        self.select(self.selected.saturating_sub(1));
    }

    /// Scrolls the view by `delta` rows (negative scrolls up), as a mouse wheel does, stopping at
    /// the top and bottom. Like scrolling a page in a browser, this leaves the selection alone,
    /// so the selected row can scroll off screen.
    pub fn scroll_by(&mut self, delta: isize, visible_rows: usize) {
        let max_offset = self.len.saturating_sub(visible_rows.max(1));
        self.offset = self
            .offset
            .min(max_offset)
            .saturating_add_signed(delta)
            .min(max_offset);
    }

    /// Returns the index of the row drawn at `position`, given the area returned by
    /// [`Self::rows_area`], or `None` if no row is drawn there.
    #[must_use]
    pub fn row_at(&self, position: Position, rows_area: Rect) -> Option<usize> {
        if !rows_area.contains(position) {
            return None;
        }
        let index = self.offset + usize::from(position.y - rows_area.y);
        (index < self.len).then_some(index)
    }

    /// Selects the row drawn at `position`, as a mouse click does, given the area returned by
    /// [`Self::rows_area`]. Returns `true` if `position` was on a row.
    pub fn click(&mut self, position: Position, rows_area: Rect) -> bool {
        let index = self.row_at(position, rows_area);
        if let Some(index) = index {
            self.select(index);
        }
        index.is_some()
    }

    /// Selects the row level with `position`, as dragging with the mouse button held does, given
    /// the area returned by [`Self::rows_area`]. The cursor's column is ignored, and a cursor above
    /// or below the rows selects the first or last row in view.
    pub fn drag_to(&mut self, position: Position, rows_area: Rect) {
        if rows_area.is_empty() {
            return;
        }
        let y = position.y.clamp(rows_area.top(), rows_area.bottom() - 1);
        self.select(self.offset + usize::from(y - rows_area.y));
    }

    /// Selects row `index`, clamped to the last row, and has the next render scroll it into
    /// view.
    fn select(&mut self, index: usize) {
        self.selected = index.min(self.len.saturating_sub(1));
        self.reveal = true;
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{
        buffer::Buffer,
        layout::{Constraint::Fill, Position, Rect},
        widgets::{Row, Table},
    };

    use super::{ScrollbarHit, TableScroll};

    /// A 40×20 page: 15 rows on screen, and a scrollbar track of 18 cells from y = 1 to 18.
    const PAGE: Rect = Rect::new(0, 0, 40, 20);

    fn view(scroll: &TableScroll) -> (usize, usize) {
        (scroll.offset, scroll.selected)
    }

    /// Renders `scroll` as a page does, scrollbar then table, into `area`, and returns what the
    /// table was rendered with: its offset and the row it highlighted.
    fn render(scroll: &mut TableScroll, area: Rect) -> (usize, Option<usize>) {
        let mut buf = Buffer::empty(area);
        let rows = (0..scroll.len).map(|index| Row::new([index.to_string()]));
        scroll.render(Table::new(rows, [Fill(1)]), ["#"], "Test", area, &mut buf);
        (scroll.state.offset(), scroll.state.selected())
    }

    #[test]
    fn rows_are_drawn_where_rows_area_says() {
        let mut scroll = TableScroll::new(100);
        scroll.scroll_by(40, 15);
        let mut buf = Buffer::empty(PAGE);
        let rows = (0..scroll.len).map(|index| Row::new([format!("row {index}")]));
        scroll.render(Table::new(rows, [Fill(1)]), ["#"], "Test", PAGE, &mut buf);

        let rows_area = scroll.rows_area(PAGE);
        let line = |y: u16| -> String {
            (rows_area.left()..rows_area.right())
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
                .trim_end()
                .to_owned()
        };
        assert_eq!(line(rows_area.top() - 1), "#");
        assert_eq!(line(rows_area.top()), "row 40");
        assert_eq!(line(rows_area.bottom() - 1), "row 54");
        let row_at = |y| scroll.row_at(Position::new(rows_area.x, y), rows_area);
        assert_eq!(row_at(rows_area.top()), Some(40));
        assert_eq!(row_at(rows_area.bottom() - 1), Some(54));
    }

    #[test]
    fn wheel_moves_view_only() {
        let mut scroll = TableScroll::new(100);
        scroll.select(2);
        scroll.scroll_by(3, 10);
        assert_eq!(view(&scroll), (3, 2));
        scroll.scroll_by(-3, 10);
        assert_eq!(view(&scroll), (0, 2));
    }

    #[test]
    fn wheel_stops_at_ends() {
        let mut scroll = TableScroll::new(20);
        scroll.scroll_by(100, 10);
        assert_eq!(view(&scroll), (10, 0));
        scroll.scroll_by(-100, 10);
        assert_eq!(view(&scroll), (0, 0));
    }

    #[test]
    fn wheel_on_short_table_does_nothing() {
        let mut scroll = TableScroll::new(4);
        scroll.scroll_by(3, 10);
        assert_eq!(view(&scroll), (0, 0));
    }

    #[test]
    fn selection_can_scroll_off_screen() {
        let mut scroll = TableScroll::new(100);
        scroll.select(2);
        assert_eq!(render(&mut scroll, PAGE), (0, Some(2)));
        // Scrolled away, the table is drawn without a highlight and does not snap back to it.
        scroll.scroll_by(30, 15);
        assert_eq!(render(&mut scroll, PAGE), (30, None));
        assert_eq!(view(&scroll), (30, 2));
        // Scrolling back brings the highlight back.
        scroll.scroll_by(-30, 15);
        assert_eq!(render(&mut scroll, PAGE), (0, Some(2)));
    }

    #[test]
    fn keys_bring_off_screen_selection_into_view() {
        let mut scroll = TableScroll::new(100);
        scroll.select(2);
        render(&mut scroll, PAGE);
        // Selection above the view: the next row lands at the top.
        scroll.scroll_by(30, 15);
        render(&mut scroll, PAGE);
        scroll.next_row();
        assert_eq!(render(&mut scroll, PAGE), (3, Some(3)));
        // Selection below the view: the previous row lands at the bottom.
        scroll.select(80);
        render(&mut scroll, PAGE);
        scroll.scroll_by(-80, 15);
        render(&mut scroll, PAGE);
        scroll.previous_row();
        assert_eq!(render(&mut scroll, PAGE), (65, Some(79)));
    }

    #[test]
    fn keys_scroll_only_as_far_as_needed() {
        let mut scroll = TableScroll::new(30);
        for _ in 0..14 {
            scroll.next_row();
        }
        assert_eq!(render(&mut scroll, PAGE), (0, Some(14)));
        scroll.next_row();
        assert_eq!(render(&mut scroll, PAGE), (1, Some(15)));
    }

    #[test]
    fn keys_stop_at_ends() {
        let mut scroll = TableScroll::new(2);
        scroll.previous_row();
        assert_eq!(scroll.selected(), 0);
        scroll.next_row();
        scroll.next_row();
        assert_eq!(scroll.selected(), 1);
    }

    #[test]
    fn empty_table_does_not_panic() {
        let mut scroll = TableScroll::new(0);
        scroll.scroll_by(3, 10);
        scroll.next_row();
        scroll.page_down(10);
        assert_eq!(render(&mut scroll, PAGE), (0, None));
    }

    #[test]
    fn rows_area_excludes_chrome() {
        // 1 border + 2 padding on the left; scrollbar + 1 border + 2 padding on the right.
        // 1 border + 1 padding + 1 header on top; 1 padding + 1 border on the bottom.
        let area = Rect::new(0, 1, 40, 20);
        assert_eq!(
            TableScroll::new(100).rows_area(area),
            Rect::new(3, 4, 33, 15)
        );
        // Without a scrollbar, the table reclaims its column.
        assert_eq!(TableScroll::new(3).rows_area(area), Rect::new(3, 4, 34, 15));
    }

    #[test]
    fn scrollbar_only_when_rows_overflow() {
        // A 20-row-tall area shows 15 data rows.
        assert!(!TableScroll::new(15).needs_scrollbar(PAGE));
        assert!(TableScroll::new(16).needs_scrollbar(PAGE));
        assert_eq!(TableScroll::new(15).layout(PAGE), (PAGE, None));
        assert_eq!(
            TableScroll::new(16).layout(PAGE),
            (Rect::new(0, 0, 39, 20), Some(Rect::new(39, 0, 1, 20)))
        );
    }

    #[test]
    fn growing_taller_pulls_view_back() {
        let mut scroll = TableScroll::new(20);
        scroll.scroll_by(100, 10);
        assert_eq!(view(&scroll), (10, 0));
        assert_eq!(render(&mut scroll, Rect::new(0, 0, 40, 30)), (0, Some(0)));
    }

    /// Renders `scroll` into [`PAGE`] and returns its scrollbar column, top to bottom.
    fn scrollbar_column(scroll: &mut TableScroll) -> String {
        let mut buf = Buffer::empty(PAGE);
        scroll.render_scrollbar(PAGE, &mut buf);
        (0..PAGE.height)
            .map(|y| buf[(PAGE.right() - 1, y)].symbol())
            .collect()
    }

    #[test]
    fn scrollbar_follows_view_not_selection() {
        // 30 rows, 15 of them on screen.
        let mut scroll = TableScroll::new(30);
        let top = scrollbar_column(&mut scroll);

        // Moving the selection within the view leaves the view, and so the scrollbar, alone.
        for _ in 0..14 {
            scroll.next_row();
        }
        assert_eq!(scrollbar_column(&mut scroll), top);

        // Moving past the bottom scrolls by one row, which moves the scrollbar.
        scroll.next_row();
        assert_ne!(scrollbar_column(&mut scroll), top);
    }

    #[test]
    fn scrollbar_thumb_spans_track_at_ends() {
        let mut scroll = TableScroll::new(30);
        let thumb = |column: &str| {
            let column: Vec<char> = column.chars().collect();
            (column[1] == '█', column[column.len() - 2] == '█')
        };
        // At the top, the thumb starts right below the up arrow, and half the track is thumb.
        let top = scrollbar_column(&mut scroll);
        assert_eq!(thumb(&top), (true, false));
        assert_eq!(top.chars().filter(|&c| c == '█').count(), 9);
        // At the bottom, it ends right above the down arrow.
        scroll.scroll_by(100, 15);
        assert_eq!(thumb(&scrollbar_column(&mut scroll)), (false, true));
    }

    #[test]
    fn click_selects_row_under_cursor() {
        let area = Rect::new(3, 4, 33, 10);
        let mut scroll = TableScroll::new(100);
        scroll.scroll_by(20, 10);
        assert!(scroll.click(Position::new(10, 6), area));
        assert_eq!(view(&scroll), (20, 22));
    }

    #[test]
    fn click_outside_rows_is_ignored() {
        let area = Rect::new(3, 4, 33, 10);
        let mut scroll = TableScroll::new(3);
        // Header row, left padding, and a blank line below the last row.
        assert!(!scroll.click(Position::new(10, 3), area));
        assert!(!scroll.click(Position::new(2, 5), area));
        assert!(!scroll.click(Position::new(10, 7), area));
        assert_eq!(scroll.selected(), 0);
    }

    #[test]
    fn drag_selects_row_level_with_cursor() {
        let area = Rect::new(3, 4, 33, 10);
        let mut scroll = TableScroll::new(100);
        scroll.scroll_by(20, 10);
        // Off to the side of the table.
        scroll.drag_to(Position::new(60, 8), area);
        assert_eq!(view(&scroll), (20, 24));
        // Above and below the rows.
        scroll.drag_to(Position::new(10, 0), area);
        assert_eq!(view(&scroll), (20, 20));
        scroll.drag_to(Position::new(10, 40), area);
        assert_eq!(view(&scroll), (20, 29));
    }

    #[test]
    fn drag_below_short_table_selects_last_row() {
        let area = Rect::new(3, 4, 33, 10);
        let mut scroll = TableScroll::new(3);
        scroll.drag_to(Position::new(10, 12), area);
        assert_eq!(scroll.selected(), 2);
    }

    /// Returns where the thumb starts on the track and how long it is, as drawn by ratatui.
    fn drawn_thumb(scroll: &mut TableScroll) -> (u16, u16) {
        let mut buf = Buffer::empty(PAGE);
        scroll.render_scrollbar(PAGE, &mut buf);
        let thumb: Vec<u16> = (1..PAGE.height - 1)
            .filter(|&y| buf[(PAGE.right() - 1, y)].symbol() == "█")
            .map(|y| y - 1)
            .collect();
        (thumb[0], u16::try_from(thumb.len()).unwrap())
    }

    #[test]
    fn thumb_matches_ratatui() {
        for (len, offset) in [
            (16, 0),
            (16, 1),
            (30, 7),
            (100, 0),
            (100, 42),
            (100, 85),
            (1000, 500),
        ] {
            let mut scroll = TableScroll::new(len);
            scroll.scroll_by(offset, 15);
            let expected = drawn_thumb(&mut scroll);
            assert_eq!(
                scroll.thumb(18, PAGE),
                expected,
                "len {len}, offset {offset}"
            );
        }
    }

    #[test]
    fn scrollbar_hit_finds_each_part() {
        let mut scroll = TableScroll::new(30);
        scroll.scroll_by(5, 15);
        // 18-cell track, 15 of 30 rows: the thumb is 9 cells long, 3 cells into the track.
        assert_eq!(drawn_thumb(&mut scroll), (3, 9));
        let hit = |y| scroll.scrollbar_hit(Position::new(39, y), PAGE);
        assert_eq!(hit(0), Some(ScrollbarHit::UpArrow));
        assert_eq!(hit(1), Some(ScrollbarHit::TrackAbove));
        assert_eq!(hit(3), Some(ScrollbarHit::TrackAbove));
        assert_eq!(hit(4), Some(ScrollbarHit::Thumb { grab: 0 }));
        assert_eq!(hit(12), Some(ScrollbarHit::Thumb { grab: 8 }));
        assert_eq!(hit(13), Some(ScrollbarHit::TrackBelow));
        assert_eq!(hit(18), Some(ScrollbarHit::TrackBelow));
        assert_eq!(hit(19), Some(ScrollbarHit::DownArrow));
        assert_eq!(scroll.scrollbar_hit(Position::new(38, 5), PAGE), None);
        assert_eq!(
            TableScroll::new(3).scrollbar_hit(Position::new(39, 5), PAGE),
            None
        );
    }

    #[test]
    fn dragging_thumb_scrolls_view_only() {
        let mut scroll = TableScroll::new(100);
        scroll.select(2);
        render(&mut scroll, PAGE);
        // Grab the thumb by its top cell and drag it to the bottom of the track.
        scroll.drag_thumb(18, 0, PAGE);
        assert_eq!(view(&scroll), (85, 2));
        assert_eq!(render(&mut scroll, PAGE), (85, None));
        // Dragging well past either end clamps.
        scroll.drag_thumb(200, 0, PAGE);
        assert_eq!(view(&scroll), (85, 2));
        scroll.drag_thumb(0, 0, PAGE);
        assert_eq!(view(&scroll), (0, 2));
        // Halfway down the track is halfway through the rows.
        scroll.drag_thumb(10, 0, PAGE);
        assert_eq!(view(&scroll), (50, 2));
        // The drawn thumb lands where it was dragged to.
        assert_eq!(drawn_thumb(&mut scroll).0, 9);
    }

    #[test]
    fn page_keys_move_a_screen_and_stop_at_ends() {
        let mut scroll = TableScroll::new(40);
        scroll.select(2);
        scroll.page_down(15);
        assert_eq!(render(&mut scroll, PAGE), (15, Some(17)));
        // The view can only move 10 more rows, but the selection moves a whole screen.
        scroll.page_down(15);
        assert_eq!(render(&mut scroll, PAGE), (25, Some(32)));
        scroll.page_down(15);
        assert_eq!(render(&mut scroll, PAGE), (25, Some(39)));
        scroll.page_up(15);
        assert_eq!(render(&mut scroll, PAGE), (10, Some(24)));
        scroll.page_up(15);
        assert_eq!(render(&mut scroll, PAGE), (0, Some(9)));
        scroll.page_up(15);
        assert_eq!(render(&mut scroll, PAGE), (0, Some(0)));
    }

    #[test]
    fn page_down_from_off_screen_selection_brings_it_into_view() {
        let mut scroll = TableScroll::new(100);
        scroll.select(2);
        render(&mut scroll, PAGE);
        scroll.scroll_by(50, 15);
        render(&mut scroll, PAGE);
        scroll.page_down(15);
        assert_eq!(render(&mut scroll, PAGE), (17, Some(17)));
    }

    #[test]
    fn home_and_end_select_ends() {
        let mut scroll = TableScroll::new(40);
        scroll.last_row();
        assert_eq!(render(&mut scroll, PAGE), (25, Some(39)));
        scroll.first_row();
        assert_eq!(render(&mut scroll, PAGE), (0, Some(0)));
    }

    #[test]
    fn row_at_only_finds_drawn_rows() {
        let area = Rect::new(3, 4, 33, 10);
        let scroll = TableScroll::new(3);
        assert_eq!(scroll.row_at(Position::new(10, 4), area), Some(0));
        assert_eq!(scroll.row_at(Position::new(10, 6), area), Some(2));
        assert_eq!(scroll.row_at(Position::new(10, 3), area), None);
        assert_eq!(scroll.row_at(Position::new(2, 5), area), None);
        assert_eq!(scroll.row_at(Position::new(10, 7), area), None);
    }
}
