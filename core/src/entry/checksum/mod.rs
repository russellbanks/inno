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

impl Checksum {
    /// Verify that the given data matches this checksum.
    ///
    /// Returns `Ok(())` if the checksum matches, or an error describing the mismatch.
    #[cfg(feature = "extract")]
    pub fn verify(&self, data: &[u8]) -> Result<(), crate::error::InnoError> {
        use crate::error::InnoError;

        let (expected, actual) = match self {
            Self::Adler32(expected) => {
                // Adler32: compute manually
                let actual = adler32(data);
                if *expected == actual {
                    return Ok(());
                }
                (format!("Adler32({expected})"), format!("Adler32({actual})"))
            }
            Self::Crc32(expected) => {
                let actual = crc32fast::hash(data);
                if *expected == actual {
                    return Ok(());
                }
                (format!("CRC32({expected})"), format!("CRC32({actual})"))
            }
            Self::MD5(expected) => {
                use ::md5::Digest as _;
                let actual_bytes: [u8; 16] = ::md5::Md5::digest(data).into();
                if *expected.inner() == actual_bytes {
                    return Ok(());
                }
                let actual = MD5::new(actual_bytes);
                (format!("MD5({expected})"), format!("{actual}"))
            }
            Self::Sha1(expected) => {
                use ::sha1::Digest as _;
                let actual_bytes: [u8; 20] = ::sha1::Sha1::digest(data).into();
                if *expected.inner() == actual_bytes {
                    return Ok(());
                }
                let actual = Sha1::new(actual_bytes);
                (format!("Sha1({expected})"), format!("{actual}"))
            }
            Self::Sha256(expected) => {
                use ::sha2::Digest as _;
                let actual_bytes: [u8; 32] = ::sha2::Sha256::digest(data).into();
                if *expected.inner() == actual_bytes {
                    return Ok(());
                }
                let actual = Sha256::new(actual_bytes);
                (format!("Sha256({expected})"), format!("{actual}"))
            }
            Self::Check(_) => {
                // Legacy check format -- skip verification
                return Ok(());
            }
        };

        Err(InnoError::ExtractChecksumMismatch { expected, actual })
    }
}

/// Simple Adler-32 checksum computation.
#[cfg(feature = "extract")]
fn adler32(data: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65521;
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }
    (b << 16) | a
}

impl Default for Checksum {
    fn default() -> Self {
        Self::Adler32(0)
    }
}
