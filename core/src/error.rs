use std::{fmt, io};

use thiserror::Error;

use super::{Inno, InnoVersion};

#[derive(Error, Debug)]
pub enum InnoError {
    #[error("File is not an Inno installer")]
    NotInnoFile,
    #[error("Unexpected data at end of {0} Inno header stream")]
    UnexpectedExtraData(HeaderStream),
    #[error(
        "Inno Setup version {0} is newer than the maximum supported version {max_version}",
        max_version = Inno::MAX_SUPPORTED_VERSION
    )]
    UnsupportedVersion(InnoVersion),
    #[error("Unknown Inno setup version: {0}")]
    UnknownVersion(String),
    #[error("Unknown Inno Setup loader signature: {0:?}")]
    UnknownLoaderSignature([u8; 12]),
    #[error(
        "Inno CRC32 checksum mismatch reading {location}. Expected {expected} but calculated {actual}"
    )]
    CrcChecksumMismatch {
        location: &'static str,
        actual: u32,
        expected: u32,
    },
    #[error("Encrypted installers are not supported for extraction")]
    #[cfg(feature = "extract")]
    EncryptedInstaller,
    #[error("Unsupported compression for extraction: {0}")]
    #[cfg(feature = "extract")]
    UnsupportedCompression(String),
    #[error("Invalid data chunk magic: expected [7A, 6C, 62, 1A], got {0:02X?}")]
    #[cfg(feature = "extract")]
    InvalidChunkMagic([u8; 4]),
    #[error("Checksum mismatch for extracted file: expected {expected}, got {actual}")]
    #[cfg(feature = "extract")]
    ExtractChecksumMismatch { expected: String, actual: String },
    #[error("File location index {index} is out of bounds (max: {max})")]
    #[cfg(feature = "extract")]
    FileLocationOutOfBounds { index: u32, max: usize },
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeaderStream {
    Primary,
    Secondary,
}

impl HeaderStream {
    /// Returns the header stream name as a static string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
        }
    }
}

impl fmt::Display for HeaderStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
