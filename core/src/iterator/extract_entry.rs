use crate::entry::{File, FileLocation};

/// A file to extract, pairing the logical file entry with its location metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtractEntry {
    file: File,
    location: FileLocation,
}

impl ExtractEntry {
    /// Creates a new [`crate::extract::ExtractEntry`] from a [`File`] and a
    /// [`FileLocation`].
    #[must_use]
    #[inline]
    pub const fn new(file: File, location: FileLocation) -> Self {
        Self { file, location }
    }

    /// Returns the file's location index into the data entry list.
    #[must_use]
    #[inline]
    pub const fn location_index(&self) -> u32 {
        self.file.location()
    }

    /// Returns the [`File`] reference.
    #[must_use]
    #[inline]
    pub const fn file(&self) -> &File {
        &self.file
    }

    /// Returns the [`FileLocation`] reference.
    #[must_use]
    #[inline]
    pub const fn file_location(&self) -> &FileLocation {
        &self.location
    }
}
