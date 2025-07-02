use std::io;

use bitflags::bitflags;
use encoding_rs::Encoding;
use zerocopy::{Immutable, KnownLayout, LE, TryFromBytes, try_transmute};

use super::{Condition, RegRoot};
use crate::{
    InnoVersion, ReadBytesExt, WindowsVersionRange, encoding::InnoValue,
    header::flag_reader::read_flags::read_flags,
};

#[derive(Clone, Debug, Default)]
pub struct Registry {
    key: Option<String>,
    name: Option<String>,
    value: Option<String>,
    permissions: Option<String>,
    reg_root: RegRoot,
    permission: i16,
    r#type: RegistryType,
    flags: RegistryFlags,
}

impl Registry {
    pub fn read_from<R: io::Read>(
        mut src: R,
        codepage: &'static Encoding,
        version: InnoVersion,
    ) -> io::Result<Self> {
        if version < (1, 3, 0) {
            let _uncompressed_size = src.read_u32::<LE>()?;
        }

        let mut registry = Self {
            key: InnoValue::string_from(&mut src, codepage)?,
            name: InnoValue::string_from(&mut src, codepage)?,
            value: InnoValue::string_from(&mut src, codepage)?,
            permission: -1,
            ..Self::default()
        };

        Condition::read_from(&mut src, codepage, version)?;

        if ((4, 0, 11)..(4, 1, 0)).contains(&version) {
            registry.permissions = InnoValue::string_from(&mut src, codepage)?;
        }

        WindowsVersionRange::read_from(&mut src, version)?;

        registry.reg_root = try_transmute!(src.read_u32::<LE>()? | 0x8000_0000).unwrap_or_default();

        if version >= (4, 1, 0) {
            registry.permission = src.read_i16::<LE>()?;
        }

        registry.r#type = RegistryType::try_read_from_io(&mut src)?;

        registry.flags = read_flags!(&mut src,
            [
                RegistryFlags::CREATE_VALUE_IF_DOESNT_EXIST,
                RegistryFlags::UNINSTALL_DELETE_VALUE,
                RegistryFlags::UNINSTALL_CLEAR_VALUE,
                RegistryFlags::UNINSTALL_DELETE_ENTIRE_KEY,
                RegistryFlags::UNINSTALL_DELETE_ENTIRE_KEY_IF_EMPTY,
            ],
            if version >= (1, 2, 6) => RegistryFlags::PRESERVE_STRING_TYPE,
            if version >= (1, 3, 9) => [RegistryFlags::DELETE_KEY, RegistryFlags::DELETE_VALUE],
            if version >= (1, 3, 12) => RegistryFlags::NO_ERROR,
            if version >= (1, 3, 16) => RegistryFlags::DONT_CREATE_KEY,
            if version >= (5, 1, 0) => [RegistryFlags::BITS_32, RegistryFlags::BITS_64]
        )?;

        Ok(registry)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Immutable, KnownLayout, TryFromBytes)]
#[repr(u8)]
enum RegistryType {
    #[default]
    None = 0,
    String = 1,
    ExpandString = 2,
    DWord = 3,
    Binary = 4,
    MultiString = 5,
    QWord = 6,
}

impl RegistryType {
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
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct RegistryFlags: u16 {
        const CREATE_VALUE_IF_DOESNT_EXIST = 1;
        const UNINSTALL_DELETE_VALUE = 1 << 1;
        const UNINSTALL_CLEAR_VALUE = 1 << 2;
        const UNINSTALL_DELETE_ENTIRE_KEY = 1 << 3;
        const UNINSTALL_DELETE_ENTIRE_KEY_IF_EMPTY = 1 << 4;
        const PRESERVE_STRING_TYPE = 1 << 5;
        const DELETE_KEY = 1 << 6;
        const DELETE_VALUE = 1 << 7;
        const NO_ERROR = 1 << 8;
        const DONT_CREATE_KEY = 1 << 9;
        const BITS_32 = 1 << 10;
        const BITS_64 = 1 << 11;
    }
}
