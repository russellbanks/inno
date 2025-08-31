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
        block::{INNO_BLOCK_SIZE, InnoBlockReader},
        crc32::Crc32Reader,
        decoder::Decoder,
    },
    version::InnoVersion,
};

pub struct InnoStreamReader<R: Read> {
    inner: Decoder<InnoBlockReader<Take<R>>>,
    compression: Compression,
    inno_version: InnoVersion,
}

impl<R: Read> InnoStreamReader<R> {
    pub fn new(mut inner: R, version: InnoVersion) -> Result<Self> {
        let compression = Self::read_header(&mut inner, version)?;

        let mut chunk_reader = InnoBlockReader::new(inner.take(compression.size().into()));

        Ok(Self {
            inner: match compression {
                Compression::LZMA1(_) => {
                    let stream = LzmaStreamHeader::read(&mut chunk_reader)?;
                    Decoder::LZMA1(XzDecoder::new_stream(chunk_reader, stream))
                }
                Compression::Zlib(_) => Decoder::Zlib(ZlibDecoder::new(chunk_reader)),
                Compression::Stored(_) => Decoder::Stored(chunk_reader),
            },
            compression,
            inno_version: version,
        })
    }

    fn read_header(reader: &mut R, version: InnoVersion) -> Result<Compression> {
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

            let mut compression = if compressed_size.cast_signed() == -1 {
                Compression::Stored(uncompressed_size)
            } else {
                Compression::Zlib(compressed_size)
            };

            // Add the size of a CRC32 checksum for each 4KiB sub-block
            *compression.size_mut() += compression.size().div_ceil(u32::from(INNO_BLOCK_SIZE)) * 4;

            compression
        };

        let actual_crc32 = crc32_reader.finalize();
        if actual_crc32 != expected_crc32 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                InnoError::CrcChecksumMismatch {
                    location: "Inno stream header",
                    actual: actual_crc32,
                    expected: expected_crc32,
                },
            ));
        }

        Ok(compression)
    }

    /// Consumes the stream reader and returns a new one.
    pub fn reset(self) -> Result<Self> {
        let version = self.inno_version;
        let reader = self
            .into_inner() // Decoder<InnoBlockReader<Take<R>>>
            .into_inner() // InnoBlockReader<Take<R>>
            .into_inner() // Take<R>
            .into_inner(); // R
        Self::new(reader, version)
    }

    /// Gets a reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    #[must_use]
    #[inline]
    pub const fn get_ref(&self) -> &Decoder<InnoBlockReader<Take<R>>> {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    #[must_use]
    #[inline]
    pub const fn get_mut(&mut self) -> &mut Decoder<InnoBlockReader<Take<R>>> {
        &mut self.inner
    }

    /// Consumes this stream reader, returning the underlying reader.
    #[must_use]
    #[inline]
    pub fn into_inner(self) -> Decoder<InnoBlockReader<Take<R>>> {
        self.inner
    }

    /// Returns true if the reader is at the end of the stream.
    ///
    /// This means that the number of compressed bytes specified in the stream header has been read.
    #[must_use]
    pub fn is_end_of_stream(&self) -> bool {
        self.get_ref().get_ref().total_in() == self.compression.size() as usize
    }
}

impl<R: Read> Read for InnoStreamReader<R> {
    fn read(&mut self, dest: &mut [u8]) -> Result<usize> {
        self.inner.read(dest)
    }
}
