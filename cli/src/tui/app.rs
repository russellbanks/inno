use std::io;

use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{
        Constraint::{Length, Min},
        Layout, Position, Rect,
    },
    text::Line,
    widgets::Widget,
};

use super::{
    input::{self, Action},
    table::{Page, TableDrag, TableView},
    tabs::TabBar,
};

/// The viewer: a tab per page of entries, a footer of key hints, and what the mouse is doing.
pub struct App<'a> {
    tabs: TabBar<'a>,
    /// What the left mouse button was pressed on, while it is held down.
    drag: Option<Drag>,
    /// Where the mouse cursor was last seen.
    hover: Option<Position>,
    exit: bool,
}

/// What a left mouse press started on, which a drag keeps acting on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Drag {
    /// The tab bar: dragging across it switches tabs.
    Tabs,
    /// Something in the current page's table.
    Table(TableDrag),
}

impl<'a> App<'a> {
    /// Creates a viewer of `pages`, which must not be empty.
    #[must_use]
    pub fn new(pages: Vec<Box<dyn Page + 'a>>) -> Self {
        Self {
            tabs: TabBar::new(pages),
            drag: None,
            hover: None,
            exit: false,
        }
    }

    /// Draws and handles input until the user quits.
    ///
    /// # Errors
    ///
    /// Returns an error if drawing or reading input fails.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&mut *self, frame.area()))?;
            for action in input::read_batch()? {
                self.update(action);
            }
        }
        Ok(())
    }

    /// Applies `action`. Mouse positions are matched against what was last drawn.
    pub fn update(&mut self, action: Action) {
        match action {
            Action::Quit => self.exit = true,
            Action::NextTab => self.tabs.next(),
            Action::PreviousTab => self.tabs.previous(),
            Action::Up => self.table().previous_row(),
            Action::Down => self.table().next_row(),
            Action::PageUp => self.table().page_up(),
            Action::PageDown => self.table().page_down(),
            Action::Home => self.table().first_row(),
            Action::End => self.table().last_row(),
            Action::Scroll(delta) => self.table().scroll_by(delta),
            Action::Press(position) => {
                self.drag = if self.tabs.contains(position) {
                    self.tabs.click(position);
                    Some(Drag::Tabs)
                } else {
                    self.table().press(position).map(Drag::Table)
                };
            }
            Action::Drag(position) => {
                self.hover = Some(position);
                match self.drag {
                    Some(Drag::Tabs) => self.tabs.click(position),
                    Some(Drag::Table(drag)) => self.table().drag(drag, position),
                    None => {}
                }
            }
            Action::Release => self.drag = None,
            Action::Hover(position) => self.hover = Some(position),
        }
    }

    /// Returns the current page's table.
    fn table(&mut self) -> &mut TableView {
        self.tabs.current().view()
    }
}

impl Widget for &mut App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [tabs_area, page_area, footer_area] =
            Layout::vertical([Length(1), Min(0), Length(1)]).areas(area);

        self.tabs.render(tabs_area, buf);
        self.tabs.current().render(page_area, buf, self.hover);
        footer().render(footer_area, buf);
    }
}

fn footer() -> Line<'static> {
    Line::raw("◄ ► to change tab | ▲ ▼ PgUp PgDn Home End to move | Press q to quit").centered()
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use ratatui::{
        buffer::Buffer,
        layout::{Position, Rect},
        widgets::Widget,
    };

    use super::{Action, App};
    use crate::tui::table::{Column, EntryTable, Page};

    const AREA: Rect = Rect::new(0, 0, 60, 20);

    const NAMES: [&str; 50] = ["entry"; 50];

    /// An app with two 50-row pages, "First" and "Second", drawn once.
    fn app() -> App<'static> {
        let page = |title| -> Box<dyn Page> {
            Box::new(EntryTable::new(
                title,
                &NAMES,
                vec![Column::new("Name", |name| Cow::Borrowed(*name))],
            ))
        };
        let mut app = App::new(vec![page("First"), page("Second")]);
        render(&mut app);
        app
    }

    fn render(app: &mut App) -> Buffer {
        let mut buf = Buffer::empty(AREA);
        app.render(AREA, &mut buf);
        buf
    }

    fn title(app: &mut App) -> &'static str {
        app.tabs.current().title()
    }

    fn selected(app: &mut App) -> usize {
        app.tabs.current().view().selected()
    }

    /// Returns where row `index` of the current page is drawn, while it is in view.
    fn row(app: &mut App, index: u16) -> Position {
        let rows = app.tabs.current().view().rows_area();
        Position::new(rows.x + 1, rows.y + index)
    }

    #[test]
    fn keys_switch_tabs_and_move_selection() {
        let mut app = app();
        app.update(Action::Down);
        app.update(Action::Down);
        assert_eq!(selected(&mut app), 2);
        app.update(Action::End);
        assert_eq!(selected(&mut app), 49);

        app.update(Action::NextTab);
        assert_eq!(title(&mut app), "Second");
        assert_eq!(selected(&mut app), 0);
        app.update(Action::NextTab);
        assert_eq!(title(&mut app), "Second");

        // Each page keeps its own selection.
        app.update(Action::PreviousTab);
        assert_eq!((title(&mut app), selected(&mut app)), ("First", 49));
    }

    #[test]
    fn quit_ends_the_app() {
        let mut app = app();
        assert!(!app.exit);
        app.update(Action::Quit);
        assert!(app.exit);
    }

    #[test]
    fn pressing_and_dragging_rows_selects_them() {
        let mut app = app();
        let third = row(&mut app, 2);
        app.update(Action::Press(third));
        assert_eq!(selected(&mut app), 2);
        let fifth = row(&mut app, 4);
        app.update(Action::Drag(fifth));
        assert_eq!(selected(&mut app), 4);
        app.update(Action::Release);
        app.update(Action::Drag(third));
        assert_eq!(selected(&mut app), 4);
    }

    #[test]
    fn pressing_and_dragging_across_tabs_switches_them() {
        let mut app = app();
        // " First │ Second "
        app.update(Action::Press(Position::new(10, 0)));
        assert_eq!(title(&mut app), "Second");
        app.update(Action::Drag(Position::new(2, 0)));
        assert_eq!(title(&mut app), "First");
        app.update(Action::Release);
        app.update(Action::Drag(Position::new(10, 0)));
        assert_eq!(title(&mut app), "First");
    }

    #[test]
    fn wheel_scrolls_without_moving_selection() {
        let mut app = app();
        app.update(Action::Scroll(3));
        render(&mut app);
        let top = row(&mut app, 0);
        assert_eq!(app.tabs.current().view().row_at(top), Some(3));
        assert_eq!(selected(&mut app), 0);
    }

    #[test]
    fn hover_is_drawn_on_the_current_page() {
        let mut app = app();
        let fourth = row(&mut app, 3);
        app.update(Action::Hover(fourth));
        let buf = render(&mut app);
        let unhovered = buf[(fourth.x, fourth.y + 1)].bg;
        assert_ne!(buf[(fourth.x, fourth.y)].bg, unhovered);
    }
}
