use std::{
    io,
    io::{Read, Seek},
};

use super::{ResourceDirectoryTable, ResourceType, SectionReader};

pub struct ResourceDirectory<R: Read + Seek> {
    reader: SectionReader<R>,
    current_directory_table: ResourceDirectoryTable,
}

impl<R: Read + Seek> ResourceDirectory<R> {
    pub fn new(mut reader: SectionReader<R>) -> io::Result<Self> {
        ResourceDirectoryTable::read_from(&mut reader).map(|root_directory_table| Self {
            reader,
            current_directory_table: root_directory_table,
        })
    }

    #[inline]
    pub const fn reader_mut(&mut self) -> &mut SectionReader<R> {
        &mut self.reader
    }

    pub fn find_rc_data(&mut self) -> io::Result<&ResourceDirectoryTable> {
        self.find_directory_table_by_id(ResourceType::RCData.id())
    }

    pub fn find_directory_table_by_id(&mut self, id: u32) -> io::Result<&ResourceDirectoryTable> {
        let directory_entry = *self
            .current_directory_table
            .find_id_entry(id)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{id} not found in current directory table"),
                )
            })?;

        self.current_directory_table = directory_entry.data(self)?.table().unwrap();
        Ok(&self.current_directory_table)
    }
}
