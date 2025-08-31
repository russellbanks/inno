use std::io;

use flate2::read::ZlibDecoder;
use liblzma::read::XzDecoder;

pub enum Decoder<R: io::Read> {
    Stored(R),
    Zlib(ZlibDecoder<R>),
    LZMA1(XzDecoder<R>),
}

impl<R: io::Read> Decoder<R> {
    /// Returns `true` if the underlying stream is compressed.
    #[must_use]
    #[inline]
    pub const fn is_compressed(&self) -> bool {
        !self.is_stored()
    }

    /// Returns `true` if the stream is "stored". I.e, it is not compressed.
    #[must_use]
    #[inline]
    pub const fn is_stored(&self) -> bool {
        matches!(self, Self::Stored(_))
    }

    /// Returns `true` if the stream is compressed with zlib.
    #[must_use]
    #[inline]
    pub const fn is_zlib(&self) -> bool {
        matches!(self, Self::Zlib(_))
    }

    /// Returns `true` if the stream is compressed with LZMA 1.
    #[must_use]
    #[inline]
    pub const fn is_lzma1(&self) -> bool {
        matches!(self, Self::LZMA1(_))
    }

    /// Gets a reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    #[must_use]
    pub fn get_ref(&self) -> &R {
        match self {
            Self::Stored(reader) => reader,
            Self::Zlib(reader) => reader.get_ref(),
            Self::LZMA1(reader) => reader.get_ref(),
        }
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    #[must_use]
    pub fn get_mut(&mut self) -> &mut R {
        match self {
            Self::Stored(reader) => reader,
            Self::Zlib(reader) => reader.get_mut(),
            Self::LZMA1(reader) => reader.get_mut(),
        }
    }

    /// Consumes this decoder, returning the underlying reader.
    #[must_use]
    pub fn into_inner(self) -> R {
        match self {
            Self::Stored(reader) => reader,
            Self::Zlib(reader) => reader.into_inner(),
            Self::LZMA1(reader) => reader.into_inner(),
        }
    }
}

impl<R: io::Read> io::Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stored(reader) => reader.read(buf),
            Self::Zlib(reader) => reader.read(buf),
            Self::LZMA1(reader) => reader.read(buf),
        }
    }
}
