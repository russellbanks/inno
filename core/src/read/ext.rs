use std::io::{self, Read, Result};

use encoding_rs::Encoding;
use zerocopy::{ByteOrder, FromBytes, I16, I32, I64, LE, LittleEndian, U16, U32, U64};

use crate::string::PascalString;

/// How many bytes [`ReadBytesExt::read_exact_vec`] reads at a time, the same as innoextract.
const CHUNK_SIZE: usize = 10 * 1024;

/// Extends [`Read`] with methods for reading numbers. (For `std::io`.)
///
/// Most of the methods defined here have an unconstrained type parameter that
/// must be explicitly instantiated. Typically, it is instantiated with either
/// the [`BigEndian`] or [`LittleEndian`] types defined in this crate.
///
/// [`BigEndian`]: enum.BigEndian.html
/// [`LittleEndian`]: enum.LittleEndian.html
/// [`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
pub trait ReadBytesExt: Read {
    /// Read a type that implements [`FromBytes`] from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_t<T: FromBytes>(&mut self) -> Result<T> {
        T::read_from_io(self)
    }

    /// Reads an unsigned 8-bit integer from the underlying reader.
    ///
    /// Note that since this reads a single byte, no byte order conversions are used.
    /// It is included for completeness.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        u8::read_from_io(self)
    }

    /// Reads an unsigned 16-bit integer from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_u16<T: ByteOrder>(&mut self) -> Result<u16> {
        U16::<T>::read_from_io(self).map(U16::get)
    }

    /// Reads a signed 16-bit integer from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_i16<T: ByteOrder>(&mut self) -> Result<i16> {
        I16::<T>::read_from_io(self).map(I16::get)
    }

    /// Reads an unsigned 32-bit integer from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_u32<T: ByteOrder>(&mut self) -> Result<u32> {
        U32::<T>::read_from_io(self).map(U32::get)
    }

    /// Reads a signed 32-bit integer from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_i32<T: ByteOrder>(&mut self) -> Result<i32> {
        I32::<T>::read_from_io(self).map(I32::get)
    }

    /// Reads an unsigned 64-bit integer from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_u64<T: ByteOrder>(&mut self) -> Result<u64> {
        U64::<T>::read_from_io(self).map(U64::get)
    }

    /// Reads a signed 64-bit integer from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_i64<T: ByteOrder>(&mut self) -> Result<i64> {
        I64::<T>::read_from_io(self).map(I64::get)
    }

    /// Reads exactly `len` bytes into a new [`Vec`].
    ///
    /// `len` usually comes from the installer, so it is not trusted for the allocation: a damaged
    /// or hostile installer can claim gigabytes while holding only a few bytes. Like innoextract's
    /// [`binary_string::load`], this reads [`CHUNK_SIZE`] bytes at a time. Unlike innoextract, it
    /// stops with an error as soon as the input ends, so the buffer only grows as bytes actually
    /// arrive, instead of being padded out to `len`.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`], including
    /// [`io::ErrorKind::UnexpectedEof`] if the reader ends before `len` bytes.
    ///
    /// [`binary_string::load`]: https://github.com/dscharrer/innoextract/blob/1.9/src/util/load.cpp#L29-L45
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    fn read_exact_vec(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        while buffer.len() < len {
            let filled = buffer.len();
            buffer.resize(filled + (len - filled).min(CHUNK_SIZE), 0);
            self.read_exact(&mut buffer[filled..])?;
        }

        Ok(buffer)
    }

    /// Reads a UCSD Pascal-style string from the underlying reader.
    ///
    /// Assumes the string is prefixed with a 32-bit length and encoded in the specified codepage.
    #[inline]
    fn read_pascal_string(&mut self) -> Result<Option<PascalString>> {
        PascalString::read(self)
    }

    /// Reads a UCSD Pascal-style string from the underlying reader and decodes it using the
    /// specified codepage.
    fn read_decoded_pascal_string(
        &mut self,
        codepage: &'static Encoding,
    ) -> Result<Option<String>> {
        Ok(self
            .read_pascal_string()?
            .map(|pascal_string| pascal_string.decoded(codepage).into_string()))
    }

    fn read_sized_decoded_pascal_string(
        &mut self,
        size: u32,
        codepage: &'static Encoding,
    ) -> Result<Option<String>> {
        Ok(PascalString::read_sized_decoded(self, size, codepage)?.map(PascalString::into_string))
    }

    fn read_raw_pascal_string(&mut self) -> Result<Vec<u8>> {
        let length = self.read_u32::<LittleEndian>()?;

        self.read_exact_vec(length as usize)
    }

    /// Discards a UCSD Pascal-style string from the underlying reader.
    fn discard_pascal_string(&mut self) -> Result<()> {
        let length = self.read_u32::<LE>()?;

        // Discard the bytes by copying them to a sink
        io::copy(&mut self.take(length.into()), &mut io::sink())?;

        Ok(())
    }
}

/// All types that implement `Read` get methods defined in `ReadBytesExt` for free.
impl<R: Read + ?Sized> ReadBytesExt for R {}

#[cfg(test)]
mod tests {
    use std::io;

    use super::{CHUNK_SIZE, ReadBytesExt};

    #[test]
    fn read_exact_vec_reads_only_len_bytes() {
        let mut reader = b"Inno Setup".as_slice();

        assert_eq!(reader.read_exact_vec(4).unwrap(), b"Inno");
        assert_eq!(reader, b" Setup");
    }

    #[test]
    fn read_exact_vec_of_zero_bytes_reads_nothing() {
        let mut reader = b"Inno".as_slice();

        assert_eq!(reader.read_exact_vec(0).unwrap(), b"");
        assert_eq!(reader, b"Inno");
    }

    #[test]
    fn read_exact_vec_reads_across_chunks() {
        // Three whole chunks, then part of a fourth
        let data = b"Inno Setup".repeat(3 * CHUNK_SIZE / 10 + 1);

        assert_eq!(data.as_slice().read_exact_vec(data.len()).unwrap(), data);
    }

    #[test]
    fn read_exact_vec_past_the_end_is_unexpected_eof() {
        let error = b"Inno".as_slice().read_exact_vec(5).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn read_exact_vec_does_not_allocate_len_up_front() {
        // Allocating `usize::MAX` bytes before reading would panic with "capacity overflow"
        let error = b"Inno".as_slice().read_exact_vec(usize::MAX).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
    }
}
