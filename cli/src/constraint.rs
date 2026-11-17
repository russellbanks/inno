use std::fmt::{Display, Write};

use ratatui::text::Span;

/// Returns the display width of a table column: the widest of `header` and the rendered `cells`.
///
/// Each cell is measured exactly as it is drawn, by formatting it with [`Display`] and measuring
/// the result in terminal columns (not bytes), so non-ASCII text, negative numbers and flag sets
/// are all sized correctly. Pass each cell the same way the row renders it.
///
/// A column whose cells are all empty is collapsed to a width of `0`.
pub fn width<I>(header: &str, cells: I) -> u16
where
    I: IntoIterator,
    I::Item: Display,
{
    let mut buf = String::new();
    let max_cell_width = cells
        .into_iter()
        .map(|cell| {
            buf.clear();
            let _ = write!(buf, "{cell}");
            Span::raw(buf.as_str()).width()
        })
        .max()
        .unwrap_or_default();

    if max_cell_width == 0 {
        return 0;
    }

    max_cell_width
        .max(Span::raw(header).width())
        .try_into()
        .unwrap_or(u16::MAX)
}

/// Returns the display width of the row number (`#`) column for a table with `len` rows.
pub fn index_width(len: usize) -> u16 {
    width("#", [len])
}
