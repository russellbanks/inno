use std::{fmt, io};

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

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ignored => "Ignored",
            Self::Defined => "Defined",
            Self::Premultiplied => "Premultiplied",
        }
    }
}

impl fmt::Display for ImageAlphaFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
