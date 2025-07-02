use std::io::{Error, ErrorKind, Read, Result, Take};

use flate2::read::ZlibDecoder;
use liblzma::read::XzDecoder;
use zerocopy::LE;

use crate::{
    ReadBytesExt,
    compression::Compression,
    error::InnoError,
    lzma_stream_header::LzmaStreamHeader,
    read::{
        chunk::{INNO_CHUNK_SIZE, InnoChunkReader},
        crc32::Crc32Reader,
        decoder::Decoder,
    },
    version::InnoVersion,
};

pub struct InnoBlockReader<R: Read> {
    inner: Decoder<InnoChunkReader<Take<R>>>,
}

impl<R: Read> InnoBlockReader<R> {
    pub fn get(mut inner: R, version: InnoVersion) -> Result<Self> {
        let compression = Self::read_header(&mut inner, version)?;

        let mut chunk_reader = InnoChunkReader::new(inner.take(compression.size().into()));

        Ok(Self {
            inner: match compression {
                Compression::LZMA1(_) => {
                    let stream = LzmaStreamHeader::read_from(&mut chunk_reader)?;
                    Decoder::LZMA1(XzDecoder::new_stream(chunk_reader, stream))
                }
                Compression::Zlib(_) => Decoder::Zlib(ZlibDecoder::new(chunk_reader)),
                Compression::Stored(_) => Decoder::Stored(chunk_reader),
            },
        })
    }

    pub fn read_header(reader: &mut R, version: InnoVersion) -> Result<Compression> {
        let expected_crc32 = reader.read_u32::<LE>()?;

        let mut crc32_reader = Crc32Reader::new(reader);

        let compression = if version >= (4, 0, 9) {
            let size = crc32_reader.read_u32::<LE>()?;
            let compressed = crc32_reader.read_u8()? != 0;

            if compressed {
                if version >= (4, 1, 6) {
                    Compression::LZMA1(size)
                } else {
                    Compression::Zlib(size)
                }
            } else {
                Compression::Stored(size)
            }
        } else {
            let compressed_size = crc32_reader.read_u32::<LE>()?;
            let uncompressed_size = crc32_reader.read_u32::<LE>()?;

            let mut compression = if compressed_size == u32::MAX {
                Compression::Stored(uncompressed_size)
            } else {
                Compression::Zlib(compressed_size)
            };

            // Add the size of a CRC32 checksum for each 4KiB sub-block
            *compression.size_mut() += compression.size().div_ceil(u32::from(INNO_CHUNK_SIZE)) * 4;

            compression
        };

        let actual_crc32 = crc32_reader.finalize();
        if actual_crc32 != expected_crc32 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                InnoError::CrcChecksumMismatch {
                    actual: actual_crc32,
                    expected: expected_crc32,
                },
            ));
        }

        Ok(compression)
    }
}

impl<R: Read> Read for InnoBlockReader<R> {
    fn read(&mut self, dest: &mut [u8]) -> Result<usize> {
        self.inner.read(dest)
    }
}
