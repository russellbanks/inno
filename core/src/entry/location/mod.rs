mod checksum;
mod compression_filter;
mod file;
mod flags;
mod sign;

use std::io;

pub use checksum::Checksum;
pub use file::File;
pub use flags::DataEntryFlags;
use nt_time::FileTime;
pub use sign::SignMode;
use zerocopy::LE;

use crate::{
    entry::location::compression_filter::CompressionFilter,
    header::{Compression, Header, flag_reader::read_flags::read_flags},
    read::{
        ReadBytesExt,
        chunk::{Chunk, Encryption},
    },
    version::InnoVersion,
};

#[derive(Clone, Debug, Default)]
pub struct FileLocation {
    chunk: Chunk,
    file: File,
    uncompressed_size: u64,
    file_time: FileTime,
    file_version: u64,
    options: DataEntryFlags,
    sign_mode: SignMode,
}

impl FileLocation {
    pub fn read<R>(mut reader: R, header: &Header, version: InnoVersion) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut chunk = Chunk {
            first_slice: reader.read_u32::<LE>()?,
            last_slice: reader.read_u32::<LE>()?,
            ..Chunk::default()
        };

        if version < 4 && chunk.first_slice >= 1 && chunk.last_slice >= 1 {
            chunk.first_slice -= 1;
            chunk.last_slice -= 1;
        }

        chunk.offset = reader.read_u32::<LE>()?;
        chunk.sort_offset = chunk.offset;

        let mut file = File::default();

        if version >= (4, 0, 1) {
            file.offset = reader.read_u64::<LE>()?;
        }

        if version >= 4 {
            file.size = reader.read_u64::<LE>()?;
            chunk.size = reader.read_u64::<LE>()?;
        } else {
            file.size = reader.read_u32::<LE>()?.into();
            chunk.size = reader.read_u32::<LE>()?.into();
        }

        let uncompressed_size = file.size;

        file.checksum = if version >= 6.4 {
            let mut sha256_buf = [0; 32];
            reader.read_exact(&mut sha256_buf)?;
            Checksum::Sha256(sha256_buf)
        } else if version >= (5, 3, 9) {
            let mut sha1_buf = [0; 20];
            reader.read_exact(&mut sha1_buf)?;
            Checksum::Sha1(sha1_buf)
        } else if version >= 4.2 {
            let mut md5_buf = [0; 16];
            reader.read_exact(&mut md5_buf)?;
            Checksum::Md5(md5_buf)
        } else if version >= (4, 0, 1) {
            Checksum::Crc32(reader.read_u32::<LE>()?)
        } else {
            Checksum::Adler32(reader.read_u32::<LE>()?)
        };

        let time = if version.is_16_bit() {
            // 16-bit installers use the FAT filetime format
            let time = reader.read_u16::<LE>()?;
            let date = reader.read_u16::<LE>()?;
            FileTime::from_dos_date_time(date, time).unwrap_or(FileTime::UNIX_EPOCH)
        } else {
            // 32-bit installers use the Win32 FILETIME format
            FileTime::new(reader.read_u64::<LE>()?)
        };

        let file_version_ms = reader.read_u32::<LE>()?;
        let file_version_ls = reader.read_u32::<LE>()?;
        let file_version =
            u64::from(file_version_ms) << u64::from(u32::BITS) | u64::from(file_version_ls);

        let mut options = read_flags!(&mut reader,
            [DataEntryFlags::VERSION_INFO_VALID, DataEntryFlags::VERSION_INFO_NOT_VALID],
            if ((2, 0, 17)..(4, 0, 1)).contains(&version) => DataEntryFlags::BZIPPED,
            if version >= (4, 0, 10) => DataEntryFlags::TIMESTAMP_IN_UTC,
            if version >= 4.1 => DataEntryFlags::IS_UNINSTALLER_EXE,
            if version >= (4, 1, 8) => DataEntryFlags::CALL_INSTRUCTION_OPTIMIZED,
            if version >= 4.2 => DataEntryFlags::TOUCH,
            if version >= (4, 2, 2) => DataEntryFlags::CHUNK_ENCRYPTED,
            if version >= (4, 2, 5) => DataEntryFlags::CHUNK_COMPRESSED,
            if version >= (5, 1, 13) => DataEntryFlags::SOLID_BREAK,
            if version >= (5, 5, 7) && version < 6.3 => [
                DataEntryFlags::SIGN,
                DataEntryFlags::SIGN_ONCE
            ]
        )?;

        if version < (4, 2, 5) {
            options |= DataEntryFlags::CHUNK_COMPRESSED;
        }

        let sign_mode = if version >= 6.3 {
            SignMode::try_read_from_io(&mut reader)?
        } else {
            SignMode::from(options)
        };

        if options.contains(DataEntryFlags::CHUNK_COMPRESSED) {
            chunk.compression = header.compression();
        } else {
            chunk.compression = Compression::Stored;
        }

        if options.contains(DataEntryFlags::BZIPPED) {
            options |= DataEntryFlags::CHUNK_COMPRESSED;
            chunk.compression = Compression::BZip2;
        }

        chunk.encryption = if options.contains(DataEntryFlags::CHUNK_ENCRYPTED) {
            if version >= 6.4 {
                Encryption::XChaCha20
            } else if version >= (5, 3, 9) {
                Encryption::Arc4Sha1
            } else {
                Encryption::Arc4Md5
            }
        } else {
            Encryption::Plaintext
        };

        file.compression_filter = if options.contains(DataEntryFlags::CALL_INSTRUCTION_OPTIMIZED) {
            if version < 5.2 {
                CompressionFilter::InstructionFilter4108
            } else if version < (5, 3, 9) {
                CompressionFilter::InstructionFilter5200
            } else {
                CompressionFilter::InstructionFilter5309
            }
        } else {
            CompressionFilter::NoFilter
        };

        Ok(Self {
            chunk,
            file,
            uncompressed_size,
            file_time: time,
            file_version,
            options,
            sign_mode,
        })
    }
}
