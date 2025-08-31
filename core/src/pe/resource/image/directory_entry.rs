use std::{
    io,
    io::{Read, Seek},
};

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, LittleEndian, U32};

use crate::{
    pe::resource::{
        ImageResourceDataEntry, ResourceDirectory, ResourceDirectoryTable,
        directory_entry_data::ResourceDirectoryEntryData,
    },
    read::ReadBytesExt,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct ImageResourceDirectoryEntry {
    name_or_id: U32<LittleEndian>,
    offset_to_data_or_directory: U32<LittleEndian>,
}

impl ImageResourceDirectoryEntry {
    const IS_DIRECTORY_MASK: u32 = 1 << 31;

    #[inline]
    pub const fn name_or_id(self) -> u32 {
        self.name_or_id.get()
    }

    #[inline]
    const fn offset_to_data_or_directory(self) -> u32 {
        self.offset_to_data_or_directory.get()
    }

    /// Returns true if the entry is a subtable.
    pub const fn is_table(self) -> bool {
        self.offset_to_data_or_directory() & Self::IS_DIRECTORY_MASK != 0
    }

    /// Returns the section offset of the associated table or data.
    pub const fn data_offset(self) -> u32 {
        self.offset_to_data_or_directory() & !Self::IS_DIRECTORY_MASK
    }

    /// Returns the data associated to this directory entry.
    pub fn data<R>(
        self,
        section: &mut ResourceDirectory<R>,
    ) -> io::Result<ResourceDirectoryEntryData>
    where
        R: Read + Seek,
    {
        let section_reader = section.reader_mut();
        section_reader.seek(io::SeekFrom::Start(self.data_offset().into()))?;
        if self.is_table() {
            ResourceDirectoryTable::read_from(section_reader).map(ResourceDirectoryEntryData::Table)
        } else {
            section_reader
                .read_t::<ImageResourceDataEntry>()
                .map(ResourceDirectoryEntryData::Data)
        }
    }

    pub const fn file_offset(self, resource_offset: u32) -> u32 {
        self.data_offset() + resource_offset
    }
}
