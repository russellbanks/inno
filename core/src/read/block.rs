use std::{cmp::min, io};

use zerocopy::LE;

use crate::{ReadBytesExt, error::InnoError};

pub const INNO_BLOCK_SIZE: u16 = 1 << 12;

pub struct InnoBlockReader<R: io::Read> {
    /// The underlying reader.
    inner: R,

    /// The buffer for the block.
    buffer: [u8; INNO_BLOCK_SIZE as usize],

    /// The position of the reader within the current block.
    pos: usize,

    /// The length of the current block.
    ///
    /// This is always 4096, expect for the last block.
    length: usize,

    /// The total number of bytes read.
    total_in: usize,

    /// The total number of bytes produced.
    total_out: usize,
}

impl<R: io::Read> InnoBlockReader<R> {
    /// Creates a new `InnoBlockReader` from the given reader.
    #[must_use]
    pub const fn new(reader: R) -> Self {
        Self {
            inner: reader,
            buffer: [0; INNO_BLOCK_SIZE as usize],
            pos: 0,
            length: 0,
            total_in: 0,
            total_out: 0,
        }
    }

    /// Gets a reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    #[must_use]
    #[inline]
    pub const fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    #[must_use]
    #[inline]
    pub const fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Consumes the chunk reader, returning the underlying reader.
    #[must_use]
    #[inline]
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Returns the number of bytes the block reader has read.
    #[must_use]
    #[inline]
    pub const fn total_in(&self) -> usize {
        self.total_in
    }

    /// Returns the number of bytes that the block reader has produced.
    #[must_use]
    #[inline]
    pub const fn total_out(&self) -> usize {
        self.total_out
    }

    fn read_block(&mut self) -> io::Result<bool> {
        let block_crc32 = match self.inner.read_u32::<LE>() {
            Ok(block_crc32) => block_crc32,
            Err(error) => {
                return if error.kind() == io::ErrorKind::UnexpectedEof {
                    Ok(false)
                } else {
                    Err(error)
                };
            }
        };

        self.total_in += size_of::<u32>();

        self.length = self.inner.read(&mut self.buffer)?;

        self.total_in += self.length;

        if self.length == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Unexpected Inno block end",
            ));
        }

        let actual_crc32 = crc32fast::hash(&self.buffer[..self.length]);

        if actual_crc32 != block_crc32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                InnoError::CrcChecksumMismatch {
                    location: "Inno block",
                    actual: actual_crc32,
                    expected: block_crc32,
                },
            ));
        }

        self.pos = 0;

        Ok(true)
    }
}

impl<R: io::Read> io::Read for InnoBlockReader<R> {
    fn read(&mut self, dest: &mut [u8]) -> io::Result<usize> {
        let mut total_read = 0;

        while total_read < dest.len() {
            if self.pos == self.length && !self.read_block()? {
                return Ok(total_read);
            }

            let to_copy = min(dest.len() - total_read, self.length - self.pos);

            dest[total_read..total_read + to_copy]
                .copy_from_slice(&self.buffer[self.pos..self.pos + to_copy]);

            self.pos += to_copy;
            total_read += to_copy;
        }

        self.total_out += total_read;

        Ok(total_read)
    }
}
