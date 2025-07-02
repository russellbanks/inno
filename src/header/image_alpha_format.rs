use std::io;

use zerocopy::{Immutable, KnownLayout, TryFromBytes};

/// <https://jrsoftware.org/ishelp/index.php?topic=setup_wizardimagealphaformat>
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Immutable, KnownLayout, TryFromBytes)]
#[repr(u8)]
pub enum ImageAlphaFormat {
    #[default]
    Ignored = 0,
    Defined = 1,
    Premultiplied = 2,
}

impl ImageAlphaFormat {
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
