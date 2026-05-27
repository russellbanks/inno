use std::cmp::Ordering;

use crate::entry::{File, FileLocation};

/// A file to extract, pairing the logical file entry with its location metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtractEntry {
    file: File,
    location: FileLocation,
}

impl ExtractEntry {
    /// Creates a new [`crate::extract::ExtractEntry`] from a reference to a [`File`] and a reference to a
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

impl PartialOrd for ExtractEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExtractEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.file_location()
            .file()
            .offset()
            .cmp(&other.file_location().file().offset())
            .then_with(|| self.location_index().cmp(&other.location_index()))
    }
}
