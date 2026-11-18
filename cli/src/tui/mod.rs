//! The interactive viewer.
//!
//! [`pages`] turns each kind of installer entry into an [`EntryTable`](table::EntryTable), the
//! [`TabBar`](tabs::TabBar) holds them, and [`App`] maps each [`Action`](input::Action) from the
//! keyboard or mouse onto the tab bar or the current page's table.

mod app;
mod emoji;
mod input;
mod pages;
mod table;
mod tabs;
mod terminal;

use std::io::{self, Read, Seek};

use app::App;
use inno::Inno;

/// Shows `inno` in the terminal until the user quits.
///
/// # Errors
///
/// Returns an error if the terminal cannot be set up, drawn to, or read from.
pub fn run<R: Read + Seek>(inno: &Inno<R>) -> io::Result<()> {
    let mut terminal = terminal::init()?;
    App::new(pages::all(inno)).run(&mut terminal)
}
