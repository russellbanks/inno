#![forbid(unsafe_code)]

mod args;
mod extract;
mod tui;

use std::fs::File;

use anstream::println;
use args::{Args, Command};
use clap::Parser;
use inno::Inno;

fn main() -> anyhow::Result<()> {
    let (path, command) = Args::parse().into_parts();

    let mut file = File::open(&path)?;
    let mut inno = Inno::new(&mut file)?;

    match command {
        Command::View => tui::run(&inno)?,
        Command::Dump => println!("{:#?}", inno.inner),
        Command::Extract {
            destination,
            filter,
        } => extract::run(&mut inno, &destination, filter.as_ref())?,
    }

    Ok(())
}
