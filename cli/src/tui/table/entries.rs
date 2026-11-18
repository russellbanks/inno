use std::{borrow::Cow, ops::Deref};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Position, Rect},
    widgets::Row,
};

use super::{
    view::{TableSpec, TableView},
    width::{index_width, width},
};

/// Returns the text of one cell for an entry.
pub type CellFn<T> = for<'r> fn(&'r T) -> Cow<'r, str>;

/// How a column's width is decided.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Sizing {
    /// At most its widest cell, shrinking if the table is too narrow.
    Max,
    /// Exactly its widest cell.
    Length,
    /// Whatever width is left over.
    Fill,
}

/// One column of an [`EntryTable`]: its header and how to draw each entry's cell.
///
/// The same function is used to size the column and to draw it, so the two cannot disagree.
pub struct Column<T> {
    header: &'static str,
    sizing: Sizing,
    cell: CellFn<T>,
}

impl<T> Column<T> {
    /// A column as wide as its widest cell, which shrinks if the table is too narrow.
    #[must_use]
    pub const fn new(header: &'static str, cell: CellFn<T>) -> Self {
        Self {
            header,
            sizing: Sizing::Max,
            cell,
        }
    }

    /// Keeps this column at its full width, so that other columns shrink first.
    #[must_use]
    pub const fn fixed(mut self) -> Self {
        self.sizing = Sizing::Length;
        self
    }

    /// Gives this column whatever width is left over.
    #[must_use]
    pub const fn fill(mut self) -> Self {
        self.sizing = Sizing::Fill;
        self
    }

    /// Returns this column's width constraint, measured over `entries`.
    fn constraint(&self, entries: &[T]) -> Constraint {
        let measure = || width(self.header, entries.iter().map(self.cell));
        match self.sizing {
            Sizing::Max => Constraint::Max(measure()),
            Sizing::Length => Constraint::Length(measure()),
            Sizing::Fill => Constraint::Fill(1),
        }
    }
}

/// The entries a table shows: borrowed from the installer, or built for the table.
enum Entries<'a, T> {
    Borrowed(&'a [T]),
    Owned(Vec<T>),
}

impl<T> Deref for Entries<'_, T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        match self {
            Self::Borrowed(entries) => entries,
            Self::Owned(entries) => entries,
        }
    }
}

/// A page that lists entries in a table, one row per entry and one [`Column`] per field.
///
/// Rows are numbered in a leading `#` column unless [`Self::unnumbered`] is used. Column widths
/// are measured once over every entry, so they do not change as the table scrolls, and each frame
/// builds only the rows in view.
pub struct EntryTable<'a, T> {
    title: &'static str,
    entries: Entries<'a, T>,
    columns: Vec<Column<T>>,
    /// The width of each of `columns`, measured once.
    widths: Vec<Constraint>,
    numbered: bool,
    flex: Option<Flex>,
    view: TableView,
}

impl<'a, T> EntryTable<'a, T> {
    /// A table of `entries`, which it borrows.
    #[must_use]
    pub fn new(title: &'static str, entries: &'a [T], columns: Vec<Column<T>>) -> Self {
        Self::with_entries(title, Entries::Borrowed(entries), columns)
    }

    /// A table of `entries`, which it owns.
    #[must_use]
    pub fn owned(title: &'static str, entries: Vec<T>, columns: Vec<Column<T>>) -> Self {
        Self::with_entries(title, Entries::Owned(entries), columns)
    }

    fn with_entries(title: &'static str, entries: Entries<'a, T>, columns: Vec<Column<T>>) -> Self {
        let widths = columns
            .iter()
            .map(|column| column.constraint(&entries))
            .collect();
        Self {
            title,
            view: TableView::new(entries.len()),
            entries,
            columns,
            widths,
            numbered: true,
            flex: None,
        }
    }

    /// Leaves out the `#` column.
    #[must_use]
    pub const fn unnumbered(mut self) -> Self {
        self.numbered = false;
        self
    }

    /// Sets how spare width is shared between columns.
    #[must_use]
    pub const fn flex(mut self, flex: Flex) -> Self {
        self.flex = Some(flex);
        self
    }

    /// Returns `true` if there are no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// A tab of the viewer.
pub trait Page {
    /// Returns the page's title, shown on its tab and above its table.
    fn title(&self) -> &'static str;

    /// Returns the page's table, for moving its selection and view.
    fn view(&mut self) -> &mut TableView;

    /// Draws the page into `area`, shading the row under `hover`.
    fn render(&mut self, area: Rect, buf: &mut Buffer, hover: Option<Position>);
}

impl<T> Page for EntryTable<'_, T> {
    fn title(&self) -> &'static str {
        self.title
    }

    fn view(&mut self) -> &mut TableView {
        &mut self.view
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, hover: Option<Position>) {
        let Self {
            title,
            entries,
            columns,
            widths,
            numbered,
            flex,
            view,
        } = self;
        let numbered = *numbered;

        // `Max` rather than `Length`: with `Flex::Legacy`, ratatui stretches `Length` columns.
        let index_column = numbered.then(|| Constraint::Max(index_width(entries.len())));
        let widths: Vec<Constraint> = index_column
            .into_iter()
            .chain(widths.iter().copied())
            .collect();
        let header = numbered
            .then_some("#")
            .into_iter()
            .chain(columns.iter().map(|column| column.header));

        view.render(
            area,
            buf,
            TableSpec {
                title,
                header: Row::new(header),
                widths: &widths,
                flex: *flex,
            },
            |range| {
                let first = range.start;
                entries[range]
                    .iter()
                    .enumerate()
                    .map(|(offset, entry)| {
                        let number = numbered.then(|| Cow::Owned((first + offset + 1).to_string()));
                        Row::new(
                            number
                                .into_iter()
                                .chain(columns.iter().map(|column| (column.cell)(entry))),
                        )
                    })
                    .collect()
            },
            hover,
        );
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use ratatui::{
        buffer::Buffer,
        layout::{Constraint, Rect},
    };

    use super::{Column, EntryTable, Page};

    struct Entry {
        name: &'static str,
        size: u64,
    }

    const ENTRIES: [Entry; 3] = [
        Entry {
            name: "a.txt",
            size: 5,
        },
        Entry {
            name: "longer-name.dll",
            size: 123_456,
        },
        Entry { name: "", size: 0 },
    ];

    fn columns() -> Vec<Column<Entry>> {
        vec![
            Column::new("Name", |entry| Cow::Borrowed(entry.name)),
            Column::new("Size", |entry: &Entry| Cow::Owned(entry.size.to_string())).fixed(),
            Column::new("Empty", |_| Cow::Borrowed("")),
        ]
    }

    #[test]
    fn widths_are_measured_with_the_cells_drawn() {
        let table = EntryTable::new("Test", &ENTRIES, columns());
        assert_eq!(
            table.widths,
            [
                Constraint::Max(15),
                Constraint::Length(6),
                Constraint::Max(0)
            ]
        );
        let filled = EntryTable::new(
            "Test",
            &ENTRIES,
            vec![Column::new("Name", |entry: &Entry| Cow::Borrowed(entry.name)).fill()],
        );
        assert_eq!(filled.widths, [Constraint::Fill(1)]);
    }

    /// Renders `page` into a 60×10 area and returns its first data row's text.
    fn first_row(page: &mut dyn Page) -> String {
        let area = Rect::new(0, 0, 60, 10);
        let mut buf = Buffer::empty(area);
        page.render(area, &mut buf, None);
        let rows = page.view().rows_area();
        (rows.left()..rows.right())
            .map(|x| buf[(x, rows.y)].symbol())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn rows_are_numbered_unless_unnumbered() {
        let mut table = EntryTable::new("Test", &ENTRIES, columns());
        assert_eq!(first_row(&mut table), "1 a.txt 5");
        let mut table = EntryTable::new("Test", &ENTRIES, columns()).unnumbered();
        assert_eq!(first_row(&mut table), "a.txt 5");
    }

    #[test]
    fn numbers_follow_the_view() {
        let entries: Vec<Entry> = (0..100).map(|size| Entry { name: "x", size }).collect();
        let mut table = EntryTable::owned("Test", entries, columns());
        first_row(&mut table);
        table.view().scroll_by(40);
        assert_eq!(first_row(&mut table), "41 x 40");
    }

    #[test]
    fn title_is_shown_on_the_frame() {
        let mut table = EntryTable::new("My Title", &ENTRIES, columns());
        let area = Rect::new(0, 0, 60, 10);
        let mut buf = Buffer::empty(area);
        table.render(area, &mut buf, None);
        let top: String = (0..area.width).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(top.contains("My Title"), "{top}");
        assert_eq!(table.title(), "My Title");
    }
}
