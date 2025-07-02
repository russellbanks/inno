use std::io;

use bitflags::bitflags;
use encoding_rs::Encoding;
use zerocopy::{Immutable, KnownLayout, LE, TryFromBytes};

use crate::{
    InnoVersion, encoding::InnoValue, read::ReadBytesExt, windows_version::WindowsVersionRange,
};

#[derive(Clone, Debug, Default)]
pub struct Type {
    pub name: String,
    pub description: Option<String>,
    pub languages: Option<String>,
    pub check: Option<String>,
    pub is_custom: bool,
    pub setup: SetupType,
    pub size: u64,
}

impl Type {
    pub fn read_from<R>(
        mut src: R,
        codepage: &'static Encoding,
        version: InnoVersion,
    ) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut r#type = Self {
            name: InnoValue::string_from(&mut src, codepage)?.unwrap_or_default(),
            description: InnoValue::string_from(&mut src, codepage)?,
            ..Self::default()
        };

        if version >= (4, 0, 1) {
            r#type.languages = InnoValue::string_from(&mut src, codepage)?;
        }

        if version >= (4, 0, 0) || (version.is_isx() && version >= (1, 3, 24)) {
            r#type.check = InnoValue::string_from(&mut src, codepage)?;
        }

        WindowsVersionRange::read_from(&mut src, version)?;

        let flags = TypeFlags::from_bits_retain(src.read_u8()?);
        r#type.is_custom = flags.contains(TypeFlags::CUSTOM_SETUP_TYPE);

        if version >= (4, 0, 3) {
            r#type.setup = SetupType::try_read_from_io(&mut src)?;
        }

        r#type.size = if version >= (4, 0, 0) {
            src.read_u64::<LE>()?
        } else {
            u64::from(src.read_u32::<LE>()?)
        };

        Ok(r#type)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, TryFromBytes, KnownLayout, Immutable)]
#[repr(u8)]
pub enum SetupType {
    #[default]
    User = 0,
    DefaultFull = 1,
    DefaultCompact = 2,
    DefaultCustom = 3,
}

impl SetupType {
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

bitflags! {
    #[derive(Clone, Copy, Debug, Default)]
    pub struct TypeFlags: u8 {
        const CUSTOM_SETUP_TYPE = 1;
    }
}
