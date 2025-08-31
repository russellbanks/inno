use std::fmt;

use inno::entry::{
    Component, Directory, File, FileLocation, Icon, Ini, Language, Permission, RegistryEntry, Task,
    Type,
};

use super::{
    Components, DeleteEntries, Directories, FileLocations, Files, Icons, IniFiles, Languages,
    Messages, Permissions, RegistryEntries, Summary, Tasks, Types,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Page<'a> {
    Header(Summary<'a>),
    Languages(Languages<'a>),
    Messages(Messages<'a, 'a>),
    Permissions(Permissions<'a>),
    Types(Types<'a>),
    Components(Components<'a>),
    Tasks(Tasks<'a>),
    Directories(Directories<'a>),
    Files(Files<'a>),
    Icons(Icons<'a>),
    Ini(IniFiles<'a>),
    Registry(RegistryEntries<'a>),
    DeleteInstall(DeleteEntries<'a>),
    DeleteUninstall(DeleteEntries<'a>),
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
            Self::Icons(_) => "Icons",
            Self::Ini(_) => "INI",
            Self::Registry(_) => "Registry",
            Self::DeleteInstall(_) => "Delete (Install)",
            Self::DeleteUninstall(_) => "Delete (Uninstall)",
            Self::RunInstall => "Run (Install)",
            Self::RunUninstall => "Run (Uninstall)",
            Self::FileLocations(_) => "File Locations",
        }
    }

    /// Returns `true` if the page's entries contain no elements.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        match self {
            Self::Languages(languages) => languages.is_empty(),
            Self::Messages(messages) => messages.is_empty(),
            Self::Types(types) => types.is_empty(),
            Self::Components(components) => components.is_empty(),
            Self::Directories(directories) => directories.is_empty(),
            Self::Icons(icons) => icons.is_empty(),
            Self::Registry(registries) => registries.is_empty(),
            Self::DeleteInstall(delete_installs) => delete_installs.is_empty(),
            Self::DeleteUninstall(delete_uninstalls) => delete_uninstalls.is_empty(),
            _ => false,
        }
    }
}

impl fmt::Display for Page<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.title().fmt(f)
    }
}

impl<'a> From<&'a [Language]> for Page<'a> {
    fn from(languages: &'a [Language]) -> Self {
        Self::Languages(Languages::new(languages))
    }
}

impl<'a> From<&'a [Permission]> for Page<'a> {
    fn from(permissions: &'a [Permission]) -> Self {
        Self::Permissions(Permissions::new(permissions))
    }
}

impl<'a> From<&'a [Type]> for Page<'a> {
    fn from(types: &'a [Type]) -> Self {
        Self::Types(Types::new(types))
    }
}

impl<'a> From<&'a [Component]> for Page<'a> {
    fn from(components: &'a [Component]) -> Self {
        Self::Components(Components::new(components))
    }
}

impl<'a> From<&'a [Task]> for Page<'a> {
    fn from(tasks: &'a [Task]) -> Self {
        Self::Tasks(Tasks::new(tasks))
    }
}

impl<'a> From<&'a [Directory]> for Page<'a> {
    fn from(directories: &'a [Directory]) -> Self {
        Self::Directories(Directories::new(directories))
    }
}

impl<'a> From<&'a [File]> for Page<'a> {
    fn from(files: &'a [File]) -> Self {
        Self::Files(Files::new(files))
    }
}

impl<'a> From<&'a [FileLocation]> for Page<'a> {
    fn from(file_locations: &'a [FileLocation]) -> Self {
        Self::FileLocations(FileLocations::new(file_locations))
    }
}

impl<'a> From<&'a [Icon]> for Page<'a> {
    fn from(icons: &'a [Icon]) -> Self {
        Self::Icons(Icons::new(icons))
    }
}

impl<'a> From<&'a [Ini]> for Page<'a> {
    fn from(ini_files: &'a [Ini]) -> Self {
        Self::Ini(IniFiles::new(ini_files))
    }
}

impl<'a> From<&'a [RegistryEntry]> for Page<'a> {
    fn from(registry_entries: &'a [RegistryEntry]) -> Self {
        Self::Registry(RegistryEntries::new(registry_entries))
    }
}

impl<'a> From<DeleteEntries<'a>> for Page<'a> {
    fn from(delete_entries: DeleteEntries<'a>) -> Self {
        Self::DeleteInstall(delete_entries)
    }
}
