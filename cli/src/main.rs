use std::{error, fs::File, path::PathBuf};

use anstream::println;
use clap::Parser;
use inno::Inno;

#[derive(Parser)]
struct Args {
    #[arg()]
    path: PathBuf,

    /// Output a debug representation of the entire Inno Setup structure
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Args::parse();

    let file = File::open(args.path)?;
    let inno = Inno::new(file)?;

    if args.debug {
        println!("{inno:#?}");
        return Ok(());
    }

    println!("{:#?}", inno.setup_loader);

    println!("Inno Setup {}", inno.version);
    if let Some(app_name) = inno.header.app_name() {
        println!("{app_name}");
    }

    Ok(())
}
