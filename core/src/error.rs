use std::{fmt, io};

use thiserror::Error;

use crate::version::InnoVersion;

/// The maximum supported Inno Version by this library.
///
/// Inno Setup versions newer than this version are likely to have breaking changes where the
/// changes have not yet been implemented into this library.
const MAX_SUPPORTED_VERSION: InnoVersion = InnoVersion::new(6, 4, u8::MAX, u8::MAX);

#[derive(Error, Debug)]
pub enum InnoError {
    #[error("File is not an Inno installer")]
    NotInnoFile,
    #[error("Unexpected data at end of {0} Inno header stream")]
    UnexpectedExtraData(HeaderStream),
    #[error(
        "Inno Setup version {0} is newer than the maximum supported version {MAX_SUPPORTED_VERSION}"
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
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeaderStream {
    Primary,
    Secondary,
}

impl fmt::Display for HeaderStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primary => f.write_str("Primary"),
            Self::Secondary => f.write_str("Secondary"),
        }
    }
}
