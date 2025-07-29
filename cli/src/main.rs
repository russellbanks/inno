mod components;
mod directories;
mod emoji;
mod file_locations;
mod files;
mod languages;
mod messages;
mod permissions;
mod summary;
mod tabs;
mod tasks;
mod types;
mod view;

use std::{fs::File, io};

use anstream::println;
use camino::Utf8PathBuf;
use clap::Parser;
use components::Components;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use directories::Directories;
use files::Files;
use inno::Inno;
use languages::Languages;
use messages::Messages;
use permissions::Permissions;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{
        Constraint::{Length, Min},
        Layout, Rect,
    },
    text::Line,
    widgets::Widget,
};
use summary::Summary;
use tabs::TabManager;
use tasks::Tasks;
use types::Types;
use view::Page;

#[derive(Parser)]
struct Args {
    /// The path to the Inno Setup installer executable
    #[arg()]
    path: Utf8PathBuf,

    /// Output a debug representation of the entire Inno Setup structure
    #[arg(short, long)]
    debug: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let file = File::open(&args.path)?;
    let inno = Inno::new(file)?;

    if args.debug {
        println!("{inno:#?}");
        return Ok(());
    }

    let mut terminal = ratatui::init();
    let app_result = App::new(&inno, args.path.file_stem()).run(&mut terminal);
    ratatui::restore();
    app_result.map_err(anyhow::Error::from)
}

struct App<'a> {
    file_stem: Option<&'a str>,
    tabs: TabManager<'a>,
    exit: bool,
}

impl<'a> App<'a> {
    fn new<T: Into<Option<&'a str>>>(inno: &'a Inno, file_name: T) -> Self {
        Self {
            file_stem: file_name.into(),
            tabs: TabManager::new(inno),
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

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q' | 'Q') | KeyCode::Esc => self.exit(),
            KeyCode::Left | KeyCode::Char('a' | 'A') => self.prev_view(),
            KeyCode::Right | KeyCode::Char('d' | 'D') => self.next_view(),
            KeyCode::Up => self.scroll_up(),
            KeyCode::Down => self.scroll_down(),
            _ => {}
        }
    }

    const fn exit(&mut self) {
        self.exit = true;
    }

    #[must_use]
    const fn current_view(&self) -> &Page<'_> {
        self.tabs.current_tab()
    }

    #[must_use]
    const fn current_view_mut(&mut self) -> &mut Page<'a> {
        self.tabs.current_tab_mut()
    }

    fn next_view(&mut self) {
        self.tabs.next_tab();
    }

    const fn prev_view(&mut self) {
        self.tabs.prev_tab();
    }

    fn scroll_up(&mut self) {
        match self.current_view_mut() {
            Page::Header(summary) => summary.previous_row(),
            Page::Languages(languages) => languages.previous_row(),
            Page::Messages(messages) => messages.previous_row(),
            Page::Permissions(permissions) => permissions.previous_row(),
            Page::Types(types) => types.previous_row(),
            Page::Components(components) => components.previous_row(),
            Page::Tasks(tasks) => tasks.previous_row(),
            Page::Directories(directories) => directories.previous_row(),
            Page::Files(files) => files.previous_row(),
            Page::FileLocations(file_locations) => file_locations.previous_row(),
            _ => todo!(),
        }
    }

    fn scroll_down(&mut self) {
        match self.current_view_mut() {
            Page::Header(summary) => summary.next_row(),
            Page::Languages(languages) => languages.next_row(),
            Page::Messages(messages) => messages.next_row(),
            Page::Permissions(permissions) => permissions.next_row(),
            Page::Types(types) => types.next_row(),
            Page::Components(components) => components.next_row(),
            Page::Tasks(tasks) => tasks.next_row(),
            Page::Directories(directories) => directories.next_row(),
            Page::Files(files) => files.next_row(),
            Page::FileLocations(file_locations) => file_locations.next_row(),
            _ => todo!(),
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

        self.tabs.render(tabs_area, buf);
        match self.tabs.current_tab_mut() {
            Page::Header(summary) => summary.render(inner_area, buf),
            Page::Languages(languages) => languages.render(inner_area, buf),
            Page::Messages(messages) => messages.render(inner_area, buf),
            Page::Permissions(permissions) => permissions.render(inner_area, buf),
            Page::Types(types) => types.render(inner_area, buf),
            Page::Components(components) => components.render(inner_area, buf),
            Page::Tasks(tasks) => tasks.render(inner_area, buf),
            Page::Directories(directories) => directories.render(inner_area, buf),
            Page::Files(files) => files.render(inner_area, buf),
            Page::FileLocations(file_locations) => file_locations.render(inner_area, buf),
            _ => todo!(),
        }
        footer().render(footer_area, buf);
    }
}

fn footer() -> Line<'static> {
    Line::raw("◄ ► to change tab | Press q to quit").centered()
}
