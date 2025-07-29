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
mod summary;
mod tabs;
mod tasks;
mod types;

use std::{fs::File, io};

use anstream::println;
use camino::Utf8PathBuf;
use clap::Parser;
use components::Components;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use delete::DeleteEntries;
use directories::Directories;
use file_locations::FileLocations;
use files::Files;
use icons::Icons;
use ini::IniFiles;
use inno::Inno;
use languages::Languages;
use messages::Messages;
use page::Page;
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
use registries::RegistryEntries;
use summary::Summary;
use tabs::TabManager;
use tasks::Tasks;
use types::Types;

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
    let app_result = App::new(&inno).run(&mut terminal);
    ratatui::restore();
    app_result.map_err(anyhow::Error::from)
}

struct App<'a> {
    tabs: TabManager<'a>,
    exit: bool,
}

impl<'a> App<'a> {
    fn new(inno: &'a Inno) -> Self {
        Self {
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

    fn scroll_up(&mut self) {
        match self.current_page_mut() {
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
            Page::Icons(icons) => icons.previous_row(),
            Page::Ini(ini_files) => ini_files.previous_row(),
            Page::Registry(registries) => registries.previous_row(),
            Page::DeleteInstall(delete_installs) => delete_installs.previous_row(),
            Page::DeleteUninstall(delete_uninstalls) => delete_uninstalls.previous_row(),
            _ => {}
        }
    }

    fn scroll_down(&mut self) {
        match self.current_page_mut() {
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
            Page::Icons(icons) => icons.next_row(),
            Page::Ini(ini_files) => ini_files.next_row(),
            Page::Registry(registries) => registries.next_row(),
            Page::DeleteInstall(delete_installs) => delete_installs.next_row(),
            Page::DeleteUninstall(delete_uninstalls) => delete_uninstalls.next_row(),
            _ => {}
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
            Page::Icons(icons) => icons.render(inner_area, buf),
            Page::Ini(ini_files) => ini_files.render(inner_area, buf),
            Page::Registry(registries) => registries.render(inner_area, buf),
            Page::DeleteInstall(delete_installs) => delete_installs.render(inner_area, buf),
            Page::DeleteUninstall(delete_uninstalls) => delete_uninstalls.render(inner_area, buf),
            _ => {}
        }
        footer().render(footer_area, buf);
    }
}

fn footer() -> Line<'static> {
    Line::raw("◄ ► to change tab | Press q to quit").centered()
}
