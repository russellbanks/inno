use std::io::{self, Result};

use zerocopy::{ByteOrder, FromBytes, I16, I32, U16, U32, U64};

/// Extends [`Read`] with methods for reading numbers. (For `std::io`.)
///
/// Most of the methods defined here have an unconstrained type parameter that
/// must be explicitly instantiated. Typically, it is instantiated with either
/// the [`BigEndian`] or [`LittleEndian`] types defined in this crate.
///
/// [`BigEndian`]: enum.BigEndian.html
/// [`LittleEndian`]: enum.LittleEndian.html
/// [`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
pub trait ReadBytesExt: io::Read {
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
}

/// All types that implement `Read` get methods defined in `ReadBytesExt` for free.
impl<R: io::Read + ?Sized> ReadBytesExt for R {}
