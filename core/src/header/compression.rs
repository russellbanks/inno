use std::io;

use zerocopy::{Immutable, KnownLayout, TryFromBytes};

use super::HeaderFlags;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Immutable, KnownLayout, TryFromBytes)]
#[repr(u8)]
pub enum Compression {
    Stored = 0,
    Zlib = 1,
    BZip2 = 2,
    LZMA1 = 3,
    LZMA2 = 4,
    #[default]
    Unknown = u8::MAX, // Set to u8::MAX to avoid conflicts with future variants
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
