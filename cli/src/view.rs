use std::fmt;

use super::{Components, Languages, Messages, Permissions, Summary, tasks::Tasks, types::Types};
use crate::{directories::Directories, file_locations::FileLocations, files::Files};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Page<'a> {
    Header(Summary<'a>),
    Languages(Languages<'a>),
    Messages(Messages<'a>),
    Permissions(Permissions<'a>),
    Types(Types<'a>),
    Components(Components<'a>),
    Tasks(Tasks<'a>),
    Directories(Directories<'a>),
    Files(Files<'a>),
    Icons,
    Ini,
    Registry,
    DeleteInstall,
    DeleteUninstall,
    RunInstall,
    RunUninstall,
    FileLocations(FileLocations<'a>),
}

impl Page<'_> {
    #[must_use]
    pub const fn title(&self) -> &'static str {
        match self {
            Self::Header(_) => "Header",
            Self::Languages(_) => "Languages",
            Self::Messages(_) => "Messages",
            Self::Permissions(_) => "Permissions",
            Self::Types(_) => "Types",
            Self::Components(_) => "Components",
            Self::Tasks(_) => "Tasks",
            Self::Directories(_) => "Directories",
            Self::Files(_) => "Files",
            Self::Icons => "Icons",
            Self::Ini => "INI",
            Self::Registry => "Registry",
            Self::DeleteInstall => "Delete (Install)",
            Self::DeleteUninstall => "Delete (Uninstall)",
            Self::RunInstall => "Run (Install)",
            Self::RunUninstall => "Run (Uninstall)",
            Self::FileLocations(_) => "File Locations",
        }
    }
}

impl fmt::Display for Page<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.title().fmt(f)
    }
}
