use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Compression {
    Stored(u32),
    Zlib(u32),
    LZMA1(u32),
}

impl Compression {
    pub const fn size(self) -> u32 {
        match self {
            Self::Stored(size) | Self::Zlib(size) | Self::LZMA1(size) => size,
        }
    }

    pub const fn size_mut(&mut self) -> &mut u32 {
        match self {
            Self::Stored(size) | Self::Zlib(size) | Self::LZMA1(size) => size,
        }
    }

    /// Returns true if the compression is stored (no compression).
    #[inline]
    pub const fn is_stored(self) -> bool {
        matches!(self, Self::Stored(_))
    }

    /// Returns true if the compression is Zlib.
    #[inline]
    pub const fn is_zlib(self) -> bool {
        matches!(self, Self::Zlib(_))
    }

    /// Returns true if the compression is LZMA1
    #[inline]
    pub const fn is_lzma1(self) -> bool {
        matches!(self, Self::LZMA1(_))
    }
}

impl fmt::Display for Compression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stored(_) => f.write_str("Stored"),
            Self::Zlib(_) => f.write_str("Zlib"),
            Self::LZMA1(_) => f.write_str("LZMA1"),
        }
    }
}
