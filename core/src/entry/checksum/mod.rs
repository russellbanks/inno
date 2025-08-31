mod md5;
mod sha1;
mod sha256;

use core::fmt;
use std::io;

pub use md5::MD5;
pub use sha1::Sha1;
pub use sha256::Sha256;
use zerocopy::LE;

use crate::read::ReadBytesExt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Checksum {
    Adler32(u32),
    Crc32(u32),
    MD5(MD5),
    Sha1(Sha1),
    Sha256(Sha256),
    Check([u8; 4]),
}

impl Checksum {
    /// Creates a new MD5 checksum from an array of 16 bytes.
    #[must_use]
    #[inline]
    pub const fn new_md5(md5: [u8; 16]) -> Self {
        Self::MD5(MD5::new(md5))
    }

    /// Creates a new SHA-1 checksum from an array of 20 bytes.
    #[must_use]
    #[inline]
    pub const fn new_sha1(sha1: [u8; 20]) -> Self {
        Self::Sha1(Sha1::new(sha1))
    }

    /// Creates a new SHA-256 checksum from an array of 32 bytes.
    #[must_use]
    #[inline]
    pub const fn new_sha256(sha256: [u8; 32]) -> Self {
        Self::Sha256(Sha256::new(sha256))
    }

    /// Reads an Adler32 from the reader.
    pub fn read_adler32<R: io::Read>(mut reader: R) -> io::Result<Self> {
        reader.read_u32::<LE>().map(Self::Adler32)
    }

    /// Reads a CRC32 from the reader.
    pub fn read_crc32<R: io::Read>(mut reader: R) -> io::Result<Self> {
        reader.read_u32::<LE>().map(Self::Crc32)
    }

    /// Reads an MD5 from the reader.
    pub fn read_md5<R: io::Read>(mut reader: R) -> io::Result<Self> {
        reader.read_t::<MD5>().map(Self::MD5)
    }

    /// Reads a SHA-1 from the reader.
    pub fn read_sha1<R: io::Read>(mut reader: R) -> io::Result<Self> {
        reader.read_t::<Sha1>().map(Self::Sha1)
    }

    /// Reads a SHA-256 from the reader.
    pub fn read_sha256<R: io::Read>(mut reader: R) -> io::Result<Self> {
        reader.read_t::<Sha256>().map(Self::Sha256)
    }

    /// Returns `true` if the checksum is an Adler32.
    #[must_use]
    #[inline]
    pub const fn is_adler32(&self) -> bool {
        matches!(self, Self::Adler32(_))
    }

    /// Returns `true` if the checksum is a CRC32.
    #[must_use]
    #[inline]
    pub const fn is_crc32(&self) -> bool {
        matches!(self, Self::Crc32(_))
    }

    /// Returns `true` if the checksum is an MD5.
    #[must_use]
    #[inline]
    pub const fn is_md5(&self) -> bool {
        matches!(self, Self::MD5(_))
    }

    /// Returns `true` if the checksum is a SHA-1.
    #[must_use]
    #[inline]
    pub const fn is_sha1(&self) -> bool {
        matches!(self, Self::Sha1(_))
    }

    /// Returns `true` if the checksum is a SHA-256.
    #[must_use]
    #[inline]
    pub const fn is_sha256(&self) -> bool {
        matches!(self, Self::Sha256(_))
    }
}

impl fmt::Debug for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adler32(adler32) => f.debug_tuple("Adler32").field(adler32).finish(),
            Self::Crc32(crc32) => f.debug_tuple("Crc32").field(crc32).finish(),
            Self::MD5(md5) => f.debug_tuple("MD5").field(md5).finish(),
            Self::Sha1(sha1) => f.debug_tuple("SHA1").field(sha1).finish(),
            Self::Sha256(sha256) => f.debug_tuple("SHA256").field(sha256).finish(),
            Self::Check(check) => f.debug_tuple("Check").field(check).finish(),
        }
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adler32(adler32) => write!(f, "{adler32}"),
            Self::Crc32(crc32) => write!(f, "{crc32}"),
            Self::MD5(md5) => write!(f, "{md5}"),
            Self::Sha1(sha1) => write!(f, "{sha1}"),
            Self::Sha256(sha256) => write!(f, "{sha256}"),
            Self::Check(check) => write!(f, "{:?}", u32::from_le_bytes(*check)),
        }
    }
}

impl Default for Checksum {
    fn default() -> Self {
        Self::Adler32(0)
    }
}
