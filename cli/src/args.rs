use std::path::PathBuf;

use clap::Parser;
use regex::Regex;

#[derive(Debug, Parser)]
#[command(version)]
pub struct Args {
    /// The path to the Inno Setup installer executable
    path: PathBuf,

    /// Output a debug representation of the entire Inno Setup structure
    #[arg(short, long, conflicts_with = "extract")]
    debug: bool,

    /// Extract files to the given directory
    #[arg(short, long, value_name = "DIRECTORY")]
    extract: Option<PathBuf>,

    /// Only extract files whose extracted path, such as `app/bin/foo.dll`, matches this regex
    #[arg(short, long, value_name = "REGEX", requires = "extract")]
    filter: Option<Regex>,
}

/// What innex was asked to do with the installer.
#[derive(Debug)]
pub enum Command {
    /// Browse it in the terminal.
    View,
    /// Print its parsed structure.
    Dump,
    /// Extract its files into `destination`, only those matching `filter` if one is given.
    Extract {
        destination: PathBuf,
        filter: Option<Regex>,
    },
}

impl Args {
    /// Returns the installer's path and what to do with it.
    #[must_use]
    pub fn into_parts(self) -> (PathBuf, Command) {
        let command = match (self.debug, self.extract) {
            (true, _) => Command::Dump,
            (false, Some(destination)) => Command::Extract {
                destination,
                filter: self.filter,
            },
            (false, None) => Command::View,
        };
        (self.path, command)
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser, error::ErrorKind};

    use super::{Args, Command};

    fn parse(args: &[&str]) -> Result<Command, ErrorKind> {
        Args::try_parse_from(["innex", "setup.exe"].iter().chain(args))
            .map(|args| args.into_parts().1)
            .map_err(|error| error.kind())
    }

    #[test]
    fn definition_is_valid() {
        Args::command().debug_assert();
    }

    #[test]
    fn modes() {
        assert!(matches!(parse(&[]), Ok(Command::View)));
        assert!(matches!(parse(&["--debug"]), Ok(Command::Dump)));
        assert!(matches!(
            parse(&["-e", "out"]),
            Ok(Command::Extract { filter: None, .. })
        ));
        assert!(matches!(
            parse(&["-e", "out", "-f", r"\.dll$"]),
            Ok(Command::Extract {
                filter: Some(_),
                ..
            })
        ));
    }

    #[test]
    fn rejects_meaningless_combinations() {
        assert_eq!(
            parse(&["--debug", "-e", "out"]).unwrap_err(),
            ErrorKind::ArgumentConflict
        );
        assert_eq!(
            parse(&["-f", "x"]).unwrap_err(),
            ErrorKind::MissingRequiredArgument
        );
        assert_eq!(
            parse(&["-e", "out", "-f", "("]).unwrap_err(),
            ErrorKind::ValueValidation
        );
    }
}
