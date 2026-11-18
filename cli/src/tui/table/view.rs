use std::ops::Range;

use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment, Constraint,
        Constraint::{Fill, Length},
        Flex, Layout, Position, Rect,
    },
    style::{
        Modifier, Style,
        palette::tailwind::{SKY, SLATE},
    },
    widgets::{
        Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Table, TableState,
    },
};

use super::model::ScrollModel;

/// Height of a table's header row.
const HEADER_HEIGHT: u16 = 1;

/// Cells at each end of the scrollbar taken by its ↑ and ↓ arrows.
const SCROLLBAR_ARROW_HEIGHT: u16 = 1;

/// Columns between adjacent table columns.
const COLUMN_SPACING: u16 = 2;

/// Style of the selected row.
const HIGHLIGHT_STYLE: Style = Style::new().add_modifier(Modifier::REVERSED).fg(SKY.c400);

/// Background of the row under the mouse cursor.
const HOVER_STYLE: Style = Style::new().bg(SLATE.c800);

/// Returns the bordered, padded block every table is framed in.
fn table_block(title: &str) -> Block<'_> {
    Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .padding(Padding::proportional(1))
}

/// Returns the area that data rows occupy when a table is drawn into `table_area`: inside
/// [`table_block`], below a [`HEADER_HEIGHT`]-row header.
fn data_rows_area(table_area: Rect) -> Rect {
    let inner = table_block("").inner(table_area);
    let [_header, rows] = Layout::vertical([Length(HEADER_HEIGHT), Fill(1)]).areas(inner);
    rows
}

/// Rounds `numerator / denominator` to the nearest integer, as ratatui's scrollbar does.
const fn rounding_divide(numerator: usize, denominator: usize) -> usize {
    (numerator + denominator / 2) / denominator
}

/// What a table draws apart from its rows.
pub struct TableSpec<'a> {
    pub title: &'a str,
    pub header: Row<'a>,
    pub widths: &'a [Constraint],
    /// How spare width is shared out, or `None` for ratatui's default.
    pub flex: Option<Flex>,
}

/// The part of a table's scrollbar under the mouse.
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

/// What a mouse press on a table started, which a drag then keeps acting on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableDrag {
    /// Selecting rows.
    Rows,
    /// Moving the scrollbar thumb, grabbed `grab` cells below its top.
    Thumb { grab: u16 },
}

/// Where the parts of a table were drawn.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Geometry {
    /// The data rows, below the header and inside the frame.
    rows: Rect,
    /// The scrollbar's column, if the rows overflow.
    scrollbar: Option<Rect>,
}

/// A table on screen: its [`ScrollModel`] plus where it was last drawn.
///
/// [`Self::render`] records the geometry it draws with, and every mouse query is answered from
/// that record, so hit-testing always matches what is on screen. Until the first render the
/// geometry is empty and no position hits anything.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TableView {
    model: ScrollModel,
    geometry: Geometry,
}

impl TableView {
    #[must_use]
    pub const fn new(len: usize) -> Self {
        Self {
            model: ScrollModel::new(len),
            geometry: Geometry {
                rows: Rect::ZERO,
                scrollbar: None,
            },
        }
    }

    /// Returns the index of the selected row, which may be off screen.
    #[must_use]
    pub const fn selected(&self) -> usize {
        self.model.selected()
    }

    /// Returns the area the data rows were last drawn in.
    #[must_use]
    pub const fn rows_area(&self) -> Rect {
        self.geometry.rows
    }

    /// Returns how many data rows fit on screen as of the last render.
    #[must_use]
    pub fn visible_rows(&self) -> usize {
        usize::from(self.geometry.rows.height)
    }

    /// Splits `area` into the table's area and, only if the rows overflow, a one-column
    /// scrollbar on the right.
    fn layout(&self, area: Rect) -> (Rect, Option<Rect>) {
        if self.model.len() > usize::from(data_rows_area(area).height) {
            let [table, scrollbar] = Layout::horizontal([Fill(1), Length(1)]).areas(area);
            (table, Some(scrollbar))
        } else {
            (area, None)
        }
    }

    /// Draws the table into `area`: framed with `spec.title`, a bold header, the selected row
    /// highlighted, the row under `hover` shaded, and a scrollbar if the rows overflow.
    ///
    /// `rows` is called with the range of rows in view and must return exactly those rows, each
    /// one line tall, since hit-testing counts one row per line. Only rows in view are built.
    pub fn render<'r, F>(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        spec: TableSpec<'r>,
        rows: F,
        hover: Option<Position>,
    ) where
        F: FnOnce(Range<usize>) -> Vec<Row<'r>>,
    {
        let (table_area, scrollbar) = self.layout(area);
        self.geometry = Geometry {
            rows: data_rows_area(table_area),
            scrollbar,
        };
        let visible = self.visible_rows();
        self.model.settle(visible);

        if let Some(scrollbar) = scrollbar {
            // Ratatui sizes the thumb as `viewport / (content_length - 1 + viewport)`, so a content
            // length of one more than the furthest offset makes the track stand for every row.
            let mut state = ScrollbarState::new(self.model.max_offset(visible) + 1)
                .viewport_content_length(visible)
                .position(self.model.offset());
            Scrollbar::new(ScrollbarOrientation::VerticalRight).render(scrollbar, buf, &mut state);
        }

        let range = self.model.visible_range(visible);
        let first = range.start;
        let mut state = TableState::new()
            .with_selected(self.model.highlighted(visible).map(|index| index - first));
        let mut table = Table::new(rows(range), spec.widths.iter().copied())
            .header(
                spec.header
                    .height(HEADER_HEIGHT)
                    .style(Style::new().add_modifier(Modifier::BOLD)),
            )
            .block(table_block(spec.title))
            .row_highlight_style(HIGHLIGHT_STYLE)
            .column_spacing(COLUMN_SPACING);
        if let Some(flex) = spec.flex {
            table = table.flex(flex);
        }
        StatefulWidget::render(table, table_area, buf, &mut state);

        self.render_hover(hover, buf);
    }

    /// Shades the row under `hover`, unless it is the selected row, which has its own highlight.
    fn render_hover(&self, hover: Option<Position>, buf: &mut Buffer) {
        let Some(hover) = hover else {
            return;
        };
        if let Some(index) = self.row_at(hover)
            && Some(index) != self.model.highlighted(self.visible_rows())
        {
            let rows = self.geometry.rows;
            buf.set_style(Rect::new(rows.x, hover.y, rows.width, 1), HOVER_STYLE);
        }
    }

    /// Returns the index of the row drawn at `position`, or `None` if no row is drawn there.
    #[must_use]
    pub fn row_at(&self, position: Position) -> Option<usize> {
        let rows = self.geometry.rows;
        if !rows.contains(position) {
            return None;
        }
        let index = self.model.offset() + usize::from(position.y - rows.y);
        (index < self.model.len()).then_some(index)
    }

    /// Returns the scrollbar's track, the column between its arrows, or `None` if there is no
    /// scrollbar.
    fn track(&self) -> Option<Rect> {
        let mut track = self.geometry.scrollbar?;
        track.y += SCROLLBAR_ARROW_HEIGHT;
        track.height = track.height.saturating_sub(SCROLLBAR_ARROW_HEIGHT * 2);
        Some(track)
    }

    /// Returns where the thumb starts within `track_length` cells of track, and how long it is.
    /// This is the arithmetic ratatui 0.29's `Scrollbar` draws with: both ends of the visible
    /// rows are scaled onto the track and rounded separately.
    fn thumb(&self, track_length: u16) -> (u16, u16) {
        let track_length = usize::from(track_length);
        let visible = self.visible_rows();
        let max_offset = self.model.max_offset(visible);
        let total = max_offset + visible;
        if track_length == 0 || total == 0 {
            return (0, 0);
        }
        let offset = self.model.offset().min(max_offset);
        let start = rounding_divide(offset * track_length, total).min(track_length - 1);
        let end = rounding_divide((offset + visible) * track_length, total).min(track_length);
        let length = end.saturating_sub(start).max(1);
        // Both are at most `track_length`, which came from a `u16`.
        (start as u16, length as u16)
    }

    /// Returns the part of the scrollbar at `position`, or `None` if there is no scrollbar or
    /// `position` is not on it.
    #[must_use]
    pub fn scrollbar_hit(&self, position: Position) -> Option<ScrollbarHit> {
        if !self.geometry.scrollbar?.contains(position) {
            return None;
        }
        let track = self.track()?;
        if position.y < track.top() {
            return Some(ScrollbarHit::UpArrow);
        }
        if position.y >= track.bottom() {
            return Some(ScrollbarHit::DownArrow);
        }
        let (start, length) = self.thumb(track.height);
        let y = position.y - track.y;
        Some(if y < start {
            ScrollbarHit::TrackAbove
        } else if y < start + length {
            ScrollbarHit::Thumb { grab: y - start }
        } else {
            ScrollbarHit::TrackBelow
        })
    }

    /// Acts on a left mouse press at `position`: the scrollbar's arrows scroll a row and its
    /// track a screen, and a row is selected. Returns what a drag from here should act on: the
    /// thumb or a row. Returns `None` for anywhere else.
    pub fn press(&mut self, position: Position) -> Option<TableDrag> {
        if let Some(hit) = self.scrollbar_hit(position) {
            let screen = self.visible_rows().cast_signed();
            match hit {
                ScrollbarHit::UpArrow => self.scroll_by(-1),
                ScrollbarHit::DownArrow => self.scroll_by(1),
                ScrollbarHit::TrackAbove => self.scroll_by(-screen),
                ScrollbarHit::TrackBelow => self.scroll_by(screen),
                ScrollbarHit::Thumb { grab } => return Some(TableDrag::Thumb { grab }),
            }
            return None;
        }
        let index = self.row_at(position)?;
        self.model.select(index);
        Some(TableDrag::Rows)
    }

    /// Follows the cursor to `position` during a drag that started with `drag`.
    pub fn drag(&mut self, drag: TableDrag, position: Position) {
        match drag {
            TableDrag::Rows => self.drag_rows(position),
            TableDrag::Thumb { grab } => self.drag_thumb(position.y, grab),
        }
    }

    /// Selects the row level with `position`. The cursor's column is ignored, and a cursor above
    /// or below the rows selects the first or last row in view.
    fn drag_rows(&mut self, position: Position) {
        let rows = self.geometry.rows;
        if rows.is_empty() {
            return;
        }
        let y = position.y.clamp(rows.top(), rows.bottom() - 1);
        self.model
            .select(self.model.offset() + usize::from(y - rows.y));
    }

    /// Scrolls the view so that the thumb's top sits `grab` cells above row `y`. Like the wheel,
    /// this leaves the selection alone.
    fn drag_thumb(&mut self, y: u16, grab: u16) {
        let Some(track) = self.track().filter(|track| track.height > 0) else {
            return;
        };
        let visible = self.visible_rows();
        let thumb_start = y.saturating_sub(track.y).saturating_sub(grab);
        let offset = rounding_divide(
            usize::from(thumb_start) * (self.model.max_offset(visible) + visible),
            usize::from(track.height),
        );
        self.model.scroll_to(offset, visible);
    }

    /// Scrolls the view by `delta` rows, as the mouse wheel does, leaving the selection alone.
    pub fn scroll_by(&mut self, delta: isize) {
        let visible = self.visible_rows();
        self.model.scroll_by(delta, visible);
    }

    /// Moves the selection down one row.
    pub fn next_row(&mut self) {
        self.model.next_row();
    }

    /// Moves the selection up one row.
    pub fn previous_row(&mut self) {
        self.model.previous_row();
    }

    /// Moves the selection, and the view, down a screen.
    pub fn page_down(&mut self) {
        let visible = self.visible_rows();
        self.model.page_down(visible);
    }

    /// Moves the selection, and the view, up a screen.
    pub fn page_up(&mut self) {
        let visible = self.visible_rows();
        self.model.page_up(visible);
    }

    /// Selects the first row.
    pub fn first_row(&mut self) {
        self.model.first_row();
    }

    /// Selects the last row.
    pub fn last_row(&mut self) {
        self.model.last_row();
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, ops::Range};

    use ratatui::{
        buffer::Buffer,
        layout::{Constraint::Fill, Position, Rect},
        style::Modifier,
        widgets::Row,
    };

    use super::{HOVER_STYLE, ScrollbarHit, TableDrag, TableSpec, TableView};

    /// A 40×20 page: 15 rows on screen, and a scrollbar track of 18 cells from y = 1 to 18.
    const PAGE: Rect = Rect::new(0, 0, 40, 20);

    /// The data rows of [`PAGE`] when it has a scrollbar.
    const ROWS: Rect = Rect::new(3, 3, 33, 15);

    fn render_into(view: &mut TableView, area: Rect, hover: Option<Position>) -> Buffer {
        let mut buf = Buffer::empty(area);
        let widths = [Fill(1)];
        view.render(
            area,
            &mut buf,
            TableSpec {
                title: "Test",
                header: Row::new(["#"]),
                widths: &widths,
                flex: None,
            },
            |range| {
                range
                    .map(|index| Row::new([format!("row {index}")]))
                    .collect()
            },
            hover,
        );
        buf
    }

    fn render(view: &mut TableView) -> Buffer {
        render_into(view, PAGE, None)
    }

    /// Returns the text drawn on line `y` of the data rows.
    fn line(buf: &Buffer, y: u16) -> String {
        (ROWS.left()..ROWS.right())
            .map(|x| buf[(x, y)].symbol())
            .collect::<String>()
            .trim_end()
            .to_owned()
    }

    /// Returns the lines of the data rows drawn with the selection highlight.
    fn highlighted_lines(buf: &Buffer) -> Vec<u16> {
        (ROWS.top()..ROWS.bottom())
            .filter(|&y| buf[(ROWS.x, y)].modifier.contains(Modifier::REVERSED))
            .collect()
    }

    fn view(view: &TableView) -> (usize, usize) {
        (view.model.offset(), view.model.selected())
    }

    #[test]
    fn rows_are_drawn_where_rows_area_says() {
        let mut table = TableView::new(100);
        table.model.scroll_by(40, 15);
        let buf = render(&mut table);

        assert_eq!(table.rows_area(), ROWS);
        assert_eq!(line(&buf, ROWS.top() - 1), "#");
        assert_eq!(line(&buf, ROWS.top()), "row 40");
        assert_eq!(line(&buf, ROWS.bottom() - 1), "row 54");
        let row_at = |y| table.row_at(Position::new(ROWS.x, y));
        assert_eq!(row_at(ROWS.top()), Some(40));
        assert_eq!(row_at(ROWS.bottom() - 1), Some(54));
    }

    #[test]
    fn builds_only_rows_in_view() {
        let mut table = TableView::new(1000);
        table.model.scroll_by(500, 15);
        let requested = Cell::new(0..0);
        let mut buf = Buffer::empty(PAGE);
        let widths = [Fill(1)];
        table.render(
            PAGE,
            &mut buf,
            TableSpec {
                title: "Test",
                header: Row::new(["#"]),
                widths: &widths,
                flex: None,
            },
            |range: Range<usize>| {
                requested.set(range.clone());
                range.map(|index| Row::new([index.to_string()])).collect()
            },
            None,
        );
        assert_eq!(requested.take(), 500..515);
    }

    #[test]
    fn rows_area_excludes_chrome() {
        // 1 border + 2 padding on the left; scrollbar + 1 border + 2 padding on the right.
        // 1 border + 1 padding + 1 header on top; 1 padding + 1 border on the bottom.
        let area = Rect::new(0, 1, 40, 20);
        let mut table = TableView::new(100);
        render_into(&mut table, area, None);
        assert_eq!(table.rows_area(), Rect::new(3, 4, 33, 15));
        // Without a scrollbar, the table reclaims its column.
        let mut table = TableView::new(3);
        render_into(&mut table, area, None);
        assert_eq!(table.rows_area(), Rect::new(3, 4, 34, 15));
    }

    #[test]
    fn scrollbar_only_when_rows_overflow() {
        let mut table = TableView::new(15);
        render(&mut table);
        assert_eq!(table.geometry.scrollbar, None);
        let mut table = TableView::new(16);
        render(&mut table);
        assert_eq!(table.geometry.scrollbar, Some(Rect::new(39, 0, 1, 20)));
    }

    #[test]
    fn highlight_is_drawn_only_while_selection_is_in_view() {
        let mut table = TableView::new(100);
        table.model.select(2);
        assert_eq!(highlighted_lines(&render(&mut table)), [ROWS.y + 2]);
        table.scroll_by(30);
        assert_eq!(highlighted_lines(&render(&mut table)), []);
        assert_eq!(view(&table), (30, 2));
        table.scroll_by(-30);
        assert_eq!(highlighted_lines(&render(&mut table)), [ROWS.y + 2]);
    }

    #[test]
    fn growing_taller_pulls_view_back() {
        let mut table = TableView::new(20);
        table.model.scroll_by(100, 10);
        render_into(&mut table, Rect::new(0, 0, 40, 30), None);
        assert_eq!(view(&table), (0, 0));
    }

    /// Renders `table` into [`PAGE`] and returns its scrollbar column, top to bottom.
    fn scrollbar_column(table: &mut TableView) -> String {
        let buf = render(table);
        (0..PAGE.height)
            .map(|y| buf[(PAGE.right() - 1, y)].symbol())
            .collect()
    }

    #[test]
    fn scrollbar_follows_view_not_selection() {
        // 30 rows, 15 of them on screen.
        let mut table = TableView::new(30);
        let top = scrollbar_column(&mut table);

        // Moving the selection within the view leaves the view, and so the scrollbar, alone.
        for _ in 0..14 {
            table.next_row();
        }
        assert_eq!(scrollbar_column(&mut table), top);

        // Moving past the bottom scrolls by one row, which moves the scrollbar.
        table.next_row();
        assert_ne!(scrollbar_column(&mut table), top);
    }

    #[test]
    fn scrollbar_thumb_spans_track_at_ends() {
        let mut table = TableView::new(30);
        let thumb = |column: &str| {
            let column: Vec<char> = column.chars().collect();
            (column[1] == '█', column[column.len() - 2] == '█')
        };
        // At the top, the thumb starts right below the up arrow, and half the track is thumb.
        let top = scrollbar_column(&mut table);
        assert_eq!(thumb(&top), (true, false));
        assert_eq!(top.chars().filter(|&c| c == '█').count(), 9);
        // At the bottom, it ends right above the down arrow.
        table.scroll_by(100);
        assert_eq!(thumb(&scrollbar_column(&mut table)), (false, true));
    }

    /// Returns where the thumb starts on the track and how long it is, as drawn by ratatui.
    fn drawn_thumb(table: &mut TableView) -> (u16, u16) {
        let buf = render(table);
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
            let mut table = TableView::new(len);
            table.model.scroll_by(offset, 15);
            let expected = drawn_thumb(&mut table);
            assert_eq!(table.thumb(18), expected, "len {len}, offset {offset}");
        }
    }

    #[test]
    fn scrollbar_hit_finds_each_part() {
        let mut table = TableView::new(30);
        table.model.scroll_by(5, 15);
        // 18-cell track, 15 of 30 rows: the thumb is 9 cells long, 3 cells into the track.
        assert_eq!(drawn_thumb(&mut table), (3, 9));
        let hit = |y| table.scrollbar_hit(Position::new(39, y));
        assert_eq!(hit(0), Some(ScrollbarHit::UpArrow));
        assert_eq!(hit(1), Some(ScrollbarHit::TrackAbove));
        assert_eq!(hit(3), Some(ScrollbarHit::TrackAbove));
        assert_eq!(hit(4), Some(ScrollbarHit::Thumb { grab: 0 }));
        assert_eq!(hit(12), Some(ScrollbarHit::Thumb { grab: 8 }));
        assert_eq!(hit(13), Some(ScrollbarHit::TrackBelow));
        assert_eq!(hit(18), Some(ScrollbarHit::TrackBelow));
        assert_eq!(hit(19), Some(ScrollbarHit::DownArrow));
        assert_eq!(table.scrollbar_hit(Position::new(38, 5)), None);

        let mut short = TableView::new(3);
        render(&mut short);
        assert_eq!(short.scrollbar_hit(Position::new(39, 5)), None);
    }

    #[test]
    fn nothing_is_hit_before_first_render() {
        let mut table = TableView::new(100);
        assert_eq!(table.row_at(Position::new(10, 5)), None);
        assert_eq!(table.press(Position::new(39, 5)), None);
        assert_eq!(view(&table), (0, 0));
    }

    #[test]
    fn pressing_a_row_selects_it() {
        let mut table = TableView::new(100);
        table.model.scroll_by(20, 15);
        render(&mut table);
        assert_eq!(
            table.press(Position::new(10, ROWS.y + 2)),
            Some(TableDrag::Rows)
        );
        assert_eq!(view(&table), (20, 22));
    }

    #[test]
    fn pressing_outside_rows_is_ignored() {
        let mut table = TableView::new(3);
        render(&mut table);
        let rows = table.rows_area();
        // Header row, left padding, and a blank line below the last row.
        assert_eq!(table.press(Position::new(10, rows.y - 1)), None);
        assert_eq!(table.press(Position::new(rows.x - 1, rows.y + 1)), None);
        assert_eq!(table.press(Position::new(10, rows.y + 3)), None);
        assert_eq!(table.selected(), 0);
    }

    #[test]
    fn pressing_scrollbar_arrows_and_track_scrolls() {
        let mut table = TableView::new(100);
        render(&mut table);
        assert_eq!(table.press(Position::new(39, 19)), None);
        assert_eq!(view(&table), (1, 0));
        assert_eq!(table.press(Position::new(39, 0)), None);
        assert_eq!(view(&table), (0, 0));
        // The thumb covers the top of the track, so the bottom is track below it.
        assert_eq!(table.press(Position::new(39, 18)), None);
        assert_eq!(view(&table), (15, 0));
        render(&mut table);
        assert_eq!(table.press(Position::new(39, 1)), None);
        assert_eq!(view(&table), (0, 0));
    }

    #[test]
    fn dragging_rows_selects_row_level_with_cursor() {
        let mut table = TableView::new(100);
        table.model.scroll_by(20, 15);
        render(&mut table);
        // Off to the side of the table.
        table.drag(TableDrag::Rows, Position::new(60, ROWS.y + 4));
        assert_eq!(view(&table), (20, 24));
        // Above and below the rows.
        table.drag(TableDrag::Rows, Position::new(10, 0));
        assert_eq!(view(&table), (20, 20));
        table.drag(TableDrag::Rows, Position::new(10, 40));
        assert_eq!(view(&table), (20, 34));
    }

    #[test]
    fn dragging_below_short_table_selects_last_row() {
        let mut table = TableView::new(3);
        render(&mut table);
        table.drag(TableDrag::Rows, Position::new(10, 12));
        assert_eq!(table.selected(), 2);
    }

    #[test]
    fn dragging_thumb_scrolls_view_only() {
        let mut table = TableView::new(100);
        table.model.select(2);
        render(&mut table);
        let thumb = TableDrag::Thumb { grab: 0 };
        // Grab the thumb by its top cell and drag it to the bottom of the track.
        table.drag(thumb, Position::new(39, 18));
        assert_eq!(view(&table), (85, 2));
        assert_eq!(highlighted_lines(&render(&mut table)), []);
        // Dragging well past either end clamps.
        table.drag(thumb, Position::new(39, 200));
        assert_eq!(view(&table), (85, 2));
        table.drag(thumb, Position::new(39, 0));
        assert_eq!(view(&table), (0, 2));
        // Halfway down the track is halfway through the rows.
        table.drag(thumb, Position::new(39, 10));
        assert_eq!(view(&table), (50, 2));
        // The drawn thumb lands where it was dragged to.
        assert_eq!(drawn_thumb(&mut table).0, 9);
    }

    #[test]
    fn hover_shades_row_under_cursor_but_not_selected_row() {
        let mut table = TableView::new(100);
        table.model.select(2);
        let hovered = |buf: &Buffer, y| buf[(ROWS.x, y)].bg == HOVER_STYLE.bg.unwrap();

        let buf = render_into(&mut table, PAGE, Some(Position::new(10, ROWS.y + 5)));
        assert!(hovered(&buf, ROWS.y + 5));
        assert!(!hovered(&buf, ROWS.y + 4));

        let buf = render_into(&mut table, PAGE, Some(Position::new(10, ROWS.y + 2)));
        assert!(!hovered(&buf, ROWS.y + 2));

        // Off the rows, nothing is shaded.
        let buf = render_into(&mut table, PAGE, Some(Position::new(1, 1)));
        assert!((ROWS.top()..ROWS.bottom()).all(|y| !hovered(&buf, y)));
    }
}
