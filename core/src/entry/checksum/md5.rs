use core::fmt;
use std::array::TryFromSliceError;

use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(Clone, Copy, Eq, PartialEq, FromBytes, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct MD5([u8; 16]);

impl MD5 {
    /// Creates a new MD5 from an array of 16 bytes.
    #[must_use]
    #[inline]
    pub const fn new(md5: [u8; 16]) -> Self {
        Self(md5)
    }

    /// Returns the inner MD5 array.
    #[must_use]
    #[inline]
    pub const fn inner(&self) -> &[u8; 16] {
        &self.0
    }

    /// Consumes the MD5, returning the inner array.
    #[must_use]
    #[inline]
    pub const fn into_inner(self) -> [u8; 16] {
        self.0
    }
}

impl AsRef<[u8]> for MD5 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for MD5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for MD5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02X}")?;
        }
        Ok(())
    }
}

impl From<[u8; 16]> for MD5 {
    fn from(array: [u8; 16]) -> Self {
        Self::new(array)
    }
}

impl TryFrom<&[u8]> for MD5 {
    type Error = TryFromSliceError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        <[u8; 16]>::try_from(slice).map(Self::new)
    }
}

#[cfg(test)]
mod tests {
    use super::MD5;

    #[test]
    fn size() {
        assert_eq!(size_of::<MD5>(), 16);
    }
}
