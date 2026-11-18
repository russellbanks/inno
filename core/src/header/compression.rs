use std::{fmt, io};

use zerocopy::{Immutable, KnownLayout, TryFromBytes};

use super::HeaderFlags;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Immutable, KnownLayout, TryFromBytes)]
#[repr(u8)]
pub enum Compression {
    #[default]
    Stored = 0,
    Zlib = 1,
    BZip2 = 2,
    LZMA1 = 3,
    LZMA2 = 4,
}

impl Compression {
    pub fn try_read_from_io<R>(mut src: R) -> io::Result<Self>
    where
        Self: Sized,
        R: io::Read,
    {
        let mut buf = [0; size_of::<Self>()];
        src.read_exact(&mut buf)?;
        Self::try_read_from_bytes(&buf)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
    }

    /// Returns the Compression as a static string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Stored => "Stored",
            Self::Zlib => "Zlib",
            Self::BZip2 => "BZip2",
            Self::LZMA1 => "LZMA1",
            Self::LZMA2 => "LZMA2",
        }
    }

    /// Returns `true` if there is no compression.
    #[must_use]
    #[inline]
    pub const fn is_stored(&self) -> bool {
        matches!(self, Self::Stored)
    }
}

impl fmt::Display for Compression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl From<HeaderFlags> for Compression {
    fn from(flags: HeaderFlags) -> Self {
        if flags.contains(HeaderFlags::BZIP_USED) {
            Self::BZip2
        } else {
            Self::Zlib
        }
    }
}
