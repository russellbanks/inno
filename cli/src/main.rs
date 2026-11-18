#![forbid(unsafe_code)]

mod components;
mod constraint;
mod delete;
mod directories;
mod emoji;
mod file_locations;
mod files;
mod icons;
mod ini;
mod languages;
mod messages;
mod page;
mod permissions;
mod registries;
mod scroll;
mod summary;
mod tabs;
mod tasks;
mod types;

use std::{
    fs,
    fs::File,
    io,
    io::{Read, Seek, Write},
    path::PathBuf,
    time::Duration,
};

use anstream::println;
use clap::Parser;
use components::Components;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
};
use delete::DeleteEntries;
use directories::Directories;
use file_locations::FileLocations;
use files::Files;
use icons::Icons;
use indicatif::{ProgressBar, ProgressStyle};
use ini::IniFiles;
use inno::{
    Inno,
    path::{ByName, PathError},
};
use languages::Languages;
use messages::Messages;
use page::Page;
use permissions::Permissions;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{
        Constraint::{Length, Min},
        Layout, Position, Rect,
    },
    style::{Style, palette::tailwind::SLATE},
    text::Line,
    widgets::Widget,
};
use regex::Regex;
use registries::RegistryEntries;
use scroll::{ScrollbarHit, TableScroll};
use summary::Summary;
use tabs::TabManager;
use tasks::Tasks;
use types::Types;

#[derive(Parser)]
struct Args {
    /// The path to the Inno Setup installer executable
    #[arg()]
    path: PathBuf,

    /// Output a debug representation of the entire Inno Setup structure
    #[arg(short, long)]
    debug: bool,

    /// Extract files to the given directory
    #[arg(short, long)]
    extract: Option<PathBuf>,

    /// Only extract files whose extracted path, such as `app/bin/foo.dll`, matches this regex
    #[arg(short, long)]
    filter: Option<Regex>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut file = File::open(&args.path)?;
    let mut inno = Inno::new(&mut file)?;

    if args.debug {
        println!("{:#?}", inno.inner);
        return Ok(());
    }

    if let Some(ref destination) = args.extract {
        fs::create_dir_all(destination)?;

        // The filter matches the path the file is extracted to, such as `app/bin/foo.dll`.
        let files = inno.filtered_files(|file_entry| {
            args.filter.as_ref().is_none_or(|pattern| {
                file_entry
                    .file()
                    .destination()
                    .and_then(|destination| destination.to_relative_path(ByName).ok())
                    .is_some_and(|path| pattern.is_match(&path.to_string()))
            })
        });

        let pb = ProgressBar::new(files.len() as u64)
            .with_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.cyan} [{bar:40.green/black}] {pos}/{len} {msg}")?
                    .progress_chars("───"),
            )
            .with_message(format!("Extracting files to {}", destination.display()));

        for res in files {
            let (entry, data) = res?;

            let Some(file_destination) = entry.file().destination() else {
                pb.inc(1);
                continue;
            };
            let relative_path = match file_destination.to_relative_path(ByName) {
                Ok(relative_path) => relative_path,
                Err(PathError::Empty) => {
                    pb.inc(1);
                    continue;
                }
                Err(error) => {
                    pb.println(format!("Skipping {file_destination}: {error}"));
                    pb.inc(1);
                    continue;
                }
            };

            let full_path = relative_path.join_onto(destination);
            pb.set_message(relative_path.to_string());

            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = File::create(&full_path)?;
            file.write_all(&data)?;

            // Set the file's last modified time
            file.set_modified(entry.file_location().file_time().into())?;
            pb.inc(1);
        }

        pb.finish();

        return Ok(());
    }

    let mut terminal = ratatui::init();
    let app_result = execute!(io::stdout(), EnableMouseCapture)
        .and_then(|()| App::new(&inno).run(&mut terminal));
    let _ = execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();
    app_result.map_err(anyhow::Error::from)
}

/// Rows scrolled per mouse wheel notch.
const WHEEL_SCROLL_ROWS: isize = 3;

/// Background of the row under the mouse cursor.
const HOVER_STYLE: Style = Style::new().bg(SLATE.c800);

struct App<'a> {
    tabs: TabManager<'a>,
    /// Area of the tab bar as of the last render.
    tabs_area: Rect,
    /// Area the current page was rendered into as of the last render.
    page_area: Rect,
    /// What the left mouse button was pressed on, while it is held down.
    drag: Option<DragTarget>,
    /// Where the mouse cursor was last seen.
    hover: Option<Position>,
    exit: bool,
}

/// The part of the screen a left mouse press started on, which a drag keeps acting on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DragTarget {
    Tabs,
    Rows,
    /// The scrollbar thumb, grabbed `grab` cells below its top.
    Thumb {
        grab: u16,
    },
}

impl<'a> App<'a> {
    fn new<R: Read + Seek>(inno: &'a Inno<R>) -> Self {
        Self {
            tabs: TabManager::new(inno),
            tabs_area: Rect::ZERO,
            page_area: Rect::ZERO,
            drag: None,
            hover: None,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// Blocks for the next event, then also handles every event already queued before the next
    /// redraw. A fast spin of the mouse wheel produces a burst of events, and redrawing once per
    /// burst instead of once per event keeps scrolling smooth.
    fn handle_events(&mut self) -> io::Result<()> {
        let mut wheel_delta = 0;
        let mut event = event::read()?;
        loop {
            match event {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.scroll_by(wheel_delta);
                    wheel_delta = 0;
                    self.handle_key_event(key_event);
                }
                Event::Mouse(MouseEvent {
                    kind, column, row, ..
                }) => match kind {
                    MouseEventKind::ScrollDown => wheel_delta += WHEEL_SCROLL_ROWS,
                    MouseEventKind::ScrollUp => wheel_delta -= WHEEL_SCROLL_ROWS,
                    MouseEventKind::Down(MouseButton::Left) => {
                        self.scroll_by(wheel_delta);
                        wheel_delta = 0;
                        self.click(Position::new(column, row));
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        self.scroll_by(wheel_delta);
                        wheel_delta = 0;
                        self.hover = Some(Position::new(column, row));
                        self.drag_to(Position::new(column, row));
                    }
                    MouseEventKind::Up(MouseButton::Left) => self.drag = None,
                    MouseEventKind::Moved => self.hover = Some(Position::new(column, row)),
                    _ => {}
                },
                _ => {}
            }

            if self.exit || !event::poll(Duration::ZERO)? {
                break;
            }
            event = event::read()?;
        }
        self.scroll_by(wheel_delta);
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('c' | 'C'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => self.exit(),
            KeyEvent { code, .. } => match code {
                KeyCode::Char('q' | 'Q') | KeyCode::Esc => self.exit(),
                KeyCode::Left | KeyCode::Char('a' | 'A') => self.previous_page(),
                KeyCode::Right | KeyCode::Char('d' | 'D') => self.next_view(),
                KeyCode::Up => self.scroll_up(),
                KeyCode::Down => self.scroll_down(),
                KeyCode::PageUp => {
                    let visible_rows = self.visible_rows();
                    self.with_scroll(|scroll| scroll.page_up(visible_rows));
                }
                KeyCode::PageDown => {
                    let visible_rows = self.visible_rows();
                    self.with_scroll(|scroll| scroll.page_down(visible_rows));
                }
                KeyCode::Home => self.with_scroll(TableScroll::first_row),
                KeyCode::End => self.with_scroll(TableScroll::last_row),
                _ => {}
            },
        }
    }

    const fn exit(&mut self) {
        self.exit = true;
    }

    #[must_use]
    const fn current_page_mut(&mut self) -> &mut Page<'a> {
        self.tabs.current_tab_mut()
    }

    #[inline]
    fn next_view(&mut self) {
        self.tabs.next_tab();
    }

    #[inline]
    const fn previous_page(&mut self) {
        self.tabs.previous_tab();
    }

    fn current_scroll_mut(&mut self) -> Option<&mut TableScroll> {
        self.current_page_mut().scroll_mut()
    }

    /// Runs `f` on the current page's scroll state, if the page has a table.
    fn with_scroll(&mut self, f: impl FnOnce(&mut TableScroll)) {
        if let Some(scroll) = self.current_scroll_mut() {
            f(scroll);
        }
    }

    /// Returns the current page's table data rows area as of the last render.
    fn rows_area(&mut self) -> Rect {
        let page_area = self.page_area;
        self.current_scroll_mut()
            .map_or(Rect::ZERO, |scroll| scroll.rows_area(page_area))
    }

    /// Returns how many data rows the current page shows as of the last render.
    fn visible_rows(&mut self) -> usize {
        usize::from(self.rows_area().height)
    }

    fn scroll_up(&mut self) {
        self.with_scroll(TableScroll::previous_row);
    }

    fn scroll_down(&mut self) {
        self.with_scroll(TableScroll::next_row);
    }

    /// Scrolls the current page's view by `delta` rows, as the mouse wheel does.
    fn scroll_by(&mut self, delta: isize) {
        if delta == 0 {
            return;
        }
        let visible_rows = self.visible_rows();
        self.with_scroll(|scroll| scroll.scroll_by(delta, visible_rows));
    }

    /// Acts on a left mouse press at `position`: switches to the tab there, scrolls by a row or a
    /// screen for the scrollbar's arrows or track, or selects the row there. Pressing a tab, the
    /// scrollbar thumb, or a row also starts a drag over it.
    fn click(&mut self, position: Position) {
        self.drag = None;
        if self.tabs_area.contains(position) {
            self.drag = Some(DragTarget::Tabs);
            self.tabs.click(position, self.tabs_area);
            return;
        }

        let page_area = self.page_area;
        let hit = self
            .current_scroll_mut()
            .and_then(|scroll| scroll.scrollbar_hit(position, page_area));
        if let Some(hit) = hit {
            let screen = self.visible_rows().cast_signed();
            match hit {
                ScrollbarHit::UpArrow => self.scroll_by(-1),
                ScrollbarHit::DownArrow => self.scroll_by(1),
                ScrollbarHit::TrackAbove => self.scroll_by(-screen),
                ScrollbarHit::TrackBelow => self.scroll_by(screen),
                ScrollbarHit::Thumb { grab } => self.drag = Some(DragTarget::Thumb { grab }),
            }
            return;
        }

        let rows_area = self.rows_area();
        let on_row = self
            .current_scroll_mut()
            .is_some_and(|scroll| scroll.click(position, rows_area));
        if on_row {
            self.drag = Some(DragTarget::Rows);
        }
    }

    /// Follows the cursor while the left mouse button is held: a drag that started on the tab
    /// bar switches to the tab under the cursor, one that started on a row selects the row level
    /// with the cursor, and one that started on the scrollbar thumb moves the thumb with it.
    fn drag_to(&mut self, position: Position) {
        let page_area = self.page_area;
        match self.drag {
            Some(DragTarget::Tabs) => self.tabs.click(position, self.tabs_area),
            Some(DragTarget::Rows) => {
                let rows_area = self.rows_area();
                self.with_scroll(|scroll| scroll.drag_to(position, rows_area));
            }
            Some(DragTarget::Thumb { grab }) => {
                self.with_scroll(|scroll| scroll.drag_thumb(position.y, grab, page_area));
            }
            None => {}
        }
    }

    /// Shades the row under the mouse cursor, unless it is the selected row, which already has
    /// its own highlight.
    fn render_hover(&mut self, buf: &mut Buffer) {
        let Some(hover) = self.hover else {
            return;
        };
        let rows_area = self.rows_area();
        let Some(scroll) = self.current_scroll_mut() else {
            return;
        };
        if let Some(index) = scroll.row_at(hover, rows_area)
            && index != scroll.selected()
        {
            buf.set_style(
                Rect::new(rows_area.x, hover.y, rows_area.width, 1),
                HOVER_STYLE,
            );
        }
    }
}

impl Widget for &mut App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);

        let [tabs_area, inner_area, footer_area] = vertical.areas(area);

        self.tabs_area = tabs_area;
        self.page_area = inner_area;

        self.tabs.render(tabs_area, buf);
        self.current_page_mut().render(inner_area, buf);
        self.render_hover(buf);
        footer().render(footer_area, buf);
    }
}

fn footer() -> Line<'static> {
    Line::raw("◄ ► to change tab | ▲ ▼ PgUp PgDn Home End to move | Press q to quit").centered()
}
