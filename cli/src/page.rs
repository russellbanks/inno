use std::fmt;

use inno::entry::{
    Component, Directory, File, FileLocation, Icon, Ini, Language, Permission, RegistryEntry, Task,
    Type,
};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::{
    Components, DeleteEntries, Directories, FileLocations, Files, Icons, IniFiles, Languages,
    Messages, Permissions, RegistryEntries, Summary, Tasks, Types,
};
use crate::scroll::TableScroll;

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
    /// Returns the page's table scroll state, or `None` for a page without a table.
    #[must_use]
    pub const fn scroll_mut(&mut self) -> Option<&mut TableScroll> {
        match self {
            Self::Header(summary) => Some(summary.scroll_mut()),
            Self::Languages(languages) => Some(languages.scroll_mut()),
            Self::Messages(messages) => Some(messages.scroll_mut()),
            Self::Permissions(permissions) => Some(permissions.scroll_mut()),
            Self::Types(types) => Some(types.scroll_mut()),
            Self::Components(components) => Some(components.scroll_mut()),
            Self::Tasks(tasks) => Some(tasks.scroll_mut()),
            Self::Directories(directories) => Some(directories.scroll_mut()),
            Self::Files(files) => Some(files.scroll_mut()),
            Self::FileLocations(file_locations) => Some(file_locations.scroll_mut()),
            Self::Icons(icons) => Some(icons.scroll_mut()),
            Self::Ini(ini_files) => Some(ini_files.scroll_mut()),
            Self::Registry(registries) => Some(registries.scroll_mut()),
            Self::DeleteInstall(delete_entries) | Self::DeleteUninstall(delete_entries) => {
                Some(delete_entries.scroll_mut())
            }
            Self::RunInstall | Self::RunUninstall => None,
        }
    }

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

impl Widget for &mut Page<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self {
            Page::Header(summary) => summary.render(area, buf),
            Page::Languages(languages) => languages.render(area, buf),
            Page::Messages(messages) => messages.render(area, buf),
            Page::Permissions(permissions) => permissions.render(area, buf),
            Page::Types(types) => types.render(area, buf),
            Page::Components(components) => components.render(area, buf),
            Page::Tasks(tasks) => tasks.render(area, buf),
            Page::Directories(directories) => directories.render(area, buf),
            Page::Files(files) => files.render(area, buf),
            Page::FileLocations(file_locations) => file_locations.render(area, buf),
            Page::Icons(icons) => icons.render(area, buf),
            Page::Ini(ini_files) => ini_files.render(area, buf),
            Page::Registry(registries) => registries.render(area, buf),
            Page::DeleteInstall(delete_entries) | Page::DeleteUninstall(delete_entries) => {
                delete_entries.render(area, buf);
            }
            Page::RunInstall | Page::RunUninstall => {}
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
