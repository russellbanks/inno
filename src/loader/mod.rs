mod offset;
mod version;

use std::{
    io,
    io::{Read, Seek, SeekFrom},
};

use offset::SetupLoaderOffset;
use version::{KNOWN_SETUP_LOADER_VERSIONS, SetupLoaderVersion};
use zerocopy::LE;

use super::{
    InnoError, ReadBytesExt,
    pe::{
        CoffHeader, DosHeader, OptionalHeader, SectionTable, Signature,
        resource::{ImageResourceDataEntry, ResourceDirectory, SectionReader},
    },
    read::crc32::Crc32Reader,
    version::InnoVersion,
};

pub const SIGNATURE_LEN: usize = 12;

#[derive(Debug)]
pub enum Checksum {
    Adler32(u32),
    CRC32(u32),
}

#[derive(Debug)]
pub struct SetupLoader {
    pub version: SetupLoaderVersion,
    pub revision: u32,
    /// Minimum expected size of setup.exe
    pub minimum_setup_exe_size: u32,
    /// Offset of compressed setup.e32
    pub exe_offset: u32,
    /// Size of setup.e32 after compression
    pub exe_compressed_size: u32,
    /// Size of setup.e32 before compression
    pub exe_uncompressed_size: u32,
    /// Checksum of setup.e32 before compression
    pub exe_checksum: Checksum,
    pub message_offset: u32,
    /// Offset of embedded setup-0.bin data
    pub header_offset: u32,
    /// Offset of embedded setup-1.bin data
    pub data_offset: u32,
}

impl SetupLoader {
    const OFFSET: u32 = 0x30;

    const RESOURCE: u32 = 11111;

    /// Attempts to find the setup loader via the legacy method, falling back to checking for a PE
    /// resource entry.
    pub fn read_from<R>(mut src: R) -> Result<Self, InnoError>
    where
        R: Read + Seek,
    {
        Self::read_legacy(&mut src).or_else(|_| Self::read_from_resource(&mut src))
    }

    /// Prior to Inno 5.1.5, the offset table is found by following a pointer at a constant offset.
    fn read_legacy<R>(mut reader: R) -> Result<Self, InnoError>
    where
        R: Read + Seek,
    {
        // Seek to the setup loader offset header
        reader.seek(SeekFrom::Start(Self::OFFSET.into()))?;

        // Read the setup loader offset header
        let setup_loader_offset = SetupLoaderOffset::try_read_from(&mut reader)?;

        // Seek to the setup loader offset table
        reader.seek(SeekFrom::Start(setup_loader_offset.table_offset().into()))?;

        Self::new(reader)
    }

    /// Since Inno 5.1.5, the offset table is stored as a PE resource entry
    fn read_from_resource<R>(mut reader: R) -> Result<Self, InnoError>
    where
        R: Read + Seek,
    {
        // Seek to the start of the file
        reader.seek(SeekFrom::Start(0))?;

        // Read DOS Header
        let dos_header = DosHeader::try_read_from_io(&mut reader)?;

        // Seek to PE header
        reader.seek(SeekFrom::Start(dos_header.pe_pointer().into()))?;

        // Read PE Signature
        let _signature = Signature::try_read_from_io(&mut reader)?; // PE/0/0

        // Read COFF header
        let coff_header = reader.read_t::<CoffHeader>()?;

        // Read optional header
        let optional_header = OptionalHeader::read_from(&mut reader)?;

        // Read the section table
        let section_table = SectionTable::read_from(&mut reader, coff_header)?;

        // Get the resource table data directory header
        let resource_table = optional_header
            .data_directories
            .resource_table()
            .ok_or_else(|| {
                InnoError::Io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "No resource table found in executable",
                ))
            })?;

        // Get the actual file offset of the resource directory section
        let resource_directory_offset = resource_table.file_offset(&section_table)?;

        let section_reader = SectionReader::new(
            &mut reader,
            resource_directory_offset.into(),
            resource_table.size().into(),
        )?;

        let mut resource_directory = ResourceDirectory::new(section_reader)?;

        let _rc_data = resource_directory.find_rc_data()?;

        let loader_directory_table =
            resource_directory.find_directory_table_by_id(Self::RESOURCE)?;

        let loader_directory = loader_directory_table.entries().next().ok_or_else(|| {
            InnoError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                "No loader data entry found in loader directory",
            ))
        })?;

        let loader_data_entry_offset = loader_directory.file_offset(resource_directory_offset);
        reader.seek(SeekFrom::Start(loader_data_entry_offset.into()))?;

        let loader_data_entry = reader.read_t::<ImageResourceDataEntry>()?;
        let setup_loader_offset =
            section_table.to_file_offset(loader_data_entry.offset_to_data())?;

        reader.seek(SeekFrom::Start(setup_loader_offset.into()))?;

        Self::new(reader.take(loader_data_entry.size().into()))
    }

    fn new<R>(reader: R) -> Result<Self, InnoError>
    where
        R: Read,
    {
        let mut checksum = Crc32Reader::new(reader);
        let mut signature = [0; SIGNATURE_LEN];
        checksum.read_exact(&mut signature)?;

        let loader_version = KNOWN_SETUP_LOADER_VERSIONS
            .into_iter()
            .find(|setup_loader_version| setup_loader_version.signature == signature)
            .ok_or(InnoError::UnknownLoaderSignature(signature))?;

        let revision = if loader_version >= (5, 1, 5) {
            checksum.read_u32::<LE>()?
        } else {
            0
        };

        let minimum_setup_exe_size = checksum.read_u32::<LE>()?;

        let exe_offset = checksum.read_u32::<LE>()?;

        let exe_compressed_size = if loader_version >= (4, 1, 6) {
            0
        } else {
            checksum.read_u32::<LE>()?
        };

        let exe_uncompressed_size = checksum.read_u32::<LE>()?;

        let exe_checksum = if loader_version >= (4, 0, 3) {
            Checksum::CRC32(checksum.read_u32::<LE>()?)
        } else {
            Checksum::Adler32(checksum.read_u32::<LE>()?)
        };

        let message_offset = if loader_version >= (4, 0, 0) {
            0
        } else {
            checksum.get_mut().read_u32::<LE>()?
        };

        let header_offset = checksum.read_u32::<LE>()?;
        let data_offset = checksum.read_u32::<LE>()?;

        if loader_version >= (4, 0, 10) {
            let expected_checksum = checksum.get_mut().read_u32::<LE>()?;
            let actual_checksum = checksum.finalize();
            if actual_checksum != expected_checksum {
                return Err(InnoError::CrcChecksumMismatch {
                    actual: actual_checksum,
                    expected: expected_checksum,
                });
            }
        }

        Ok(Self {
            version: loader_version,
            revision,
            minimum_setup_exe_size,
            exe_offset,
            exe_compressed_size,
            exe_uncompressed_size,
            exe_checksum,
            message_offset,
            header_offset,
            data_offset,
        })
    }
}
