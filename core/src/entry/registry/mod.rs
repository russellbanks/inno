mod flags;
mod root;
mod r#type;

use std::io;

use encoding_rs::Encoding;
pub use flags::RegistryFlags;
pub use root::RegRoot;
pub use r#type::RegistryValueType;
use zerocopy::{LE, try_transmute};

use super::Condition;
use crate::{
    InnoVersion, ReadBytesExt, WindowsVersionRange, header::flag_reader::read_flags::read_flags,
    string::PascalString,
};

#[derive(Clone, Debug)]
pub struct RegistryEntry {
    key: Option<String>,
    name: Option<String>,
    value: Option<PascalString>,
    permissions: Option<PascalString>,
    reg_root: RegRoot,
    permission: i16,
    r#type: RegistryValueType,
    flags: RegistryFlags,
}

impl RegistryEntry {
    pub fn read<R>(
        mut reader: R,
        codepage: &'static Encoding,
        version: InnoVersion,
    ) -> io::Result<Self>
    where
        R: io::Read,
    {
        if version < 1.3 {
            let _uncompressed_size = reader.read_u32::<LE>()?;
        }

        let mut registry = Self {
            key: reader.read_decoded_pascal_string(codepage)?,
            name: reader.read_decoded_pascal_string(codepage)?,
            value: reader.read_pascal_string()?,
            ..Self::default()
        };

        Condition::read(&mut reader, codepage, version)?;

        if ((4, 0, 11)..(4, 1, 0)).contains(&version) {
            registry.permissions = reader.read_pascal_string()?;
        }

        WindowsVersionRange::read_from(&mut reader, version)?;

        registry.reg_root =
            try_transmute!(reader.read_u32::<LE>()? & !0x8000_0000).unwrap_or_default();

        if version >= 4.1 {
            registry.permission = reader.read_i16::<LE>()?;
        }

        registry.r#type = RegistryValueType::try_read_from_io(&mut reader)?;

        registry.flags = read_flags!(&mut reader,
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
            if version >= 5.1 => [RegistryFlags::BITS_32, RegistryFlags::BITS_64]
        )?;

        Ok(registry)
    }
}

impl Default for RegistryEntry {
    fn default() -> Self {
        Self {
            key: None,
            name: None,
            value: None,
            permissions: None,
            reg_root: RegRoot::default(),
            permission: -1,
            r#type: RegistryValueType::default(),
            flags: RegistryFlags::default(),
        }
    }
}
