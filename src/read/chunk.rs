use std::{cmp::min, io};

use zerocopy::LE;

use crate::{ReadBytesExt, error::InnoError};

pub const INNO_CHUNK_SIZE: u16 = 1 << 12;

pub struct InnoChunkReader<R: io::Read> {
    inner: R,
    buffer: [u8; INNO_CHUNK_SIZE as usize],
    pos: usize,
    length: usize,
}

impl<R: io::Read> InnoChunkReader<R> {
    pub const fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: [0; INNO_CHUNK_SIZE as usize],
            pos: 0,
            length: 0,
        }
    }

    fn read_chunk(&mut self) -> io::Result<bool> {
        let Ok(block_crc32) = self.inner.read_u32::<LE>() else {
            return Ok(false);
        };

        self.length = self.inner.read(&mut self.buffer)?;

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
                    actual: actual_crc32,
                    expected: block_crc32,
                },
            ));
        }

        self.pos = 0;

        Ok(true)
    }
}

impl<R: io::Read> io::Read for InnoChunkReader<R> {
    fn read(&mut self, dest: &mut [u8]) -> io::Result<usize> {
        let mut total_read = 0;

        while total_read < dest.len() {
            if self.pos == self.length && !self.read_chunk()? {
                return Ok(total_read);
            }

            let to_copy = min(dest.len() - total_read, self.length - self.pos);

            dest[total_read..total_read + to_copy]
                .copy_from_slice(&self.buffer[self.pos..self.pos + to_copy]);

            self.pos += to_copy;
            total_read += to_copy;
        }

        Ok(total_read)
    }
}
