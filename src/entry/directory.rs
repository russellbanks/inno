use std::io;

use bitflags::bitflags;
use encoding_rs::Encoding;
use zerocopy::LE;

use crate::{
    ReadBytesExt, encoding::InnoValue, entry::Condition, version::InnoVersion,
    windows_version::WindowsVersionRange,
};

#[derive(Debug, Default)]
pub struct Directory {
    pub name: Option<String>,
    pub permissions: Option<String>,
    pub attributes: u32,
    /// Index into the permission entry list
    pub permission: i16,
    pub flags: DirectoryFlags,
}

impl Directory {
    pub fn read_from<R>(
        mut src: R,
        codepage: &'static Encoding,
        version: InnoVersion,
    ) -> io::Result<Self>
    where
        R: io::Read,
    {
        if version < (1, 3, 0) {
            let _uncompressed_size = src.read_u32::<LE>()?;
        }

        let mut directory = Self {
            name: InnoValue::string_from(&mut src, codepage)?,
            permission: -1,
            ..Self::default()
        };

        Condition::read_from(&mut src, codepage, version)?;

        if ((4, 0, 11)..(4, 1, 0)).contains(&version) {
            directory.permissions = InnoValue::string_from(&mut src, codepage)?;
        }

        if version >= (2, 0, 11) {
            directory.attributes = src.read_u32::<LE>()?;
        }

        WindowsVersionRange::read_from(&mut src, version)?;

        if version >= (4, 1, 0) {
            directory.permission = src.read_i16::<LE>()?;
        }

        directory.flags = DirectoryFlags::from_bits_retain(src.read_u8()?);

        Ok(directory)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct DirectoryFlags: u8 {
        const NEVER_UNINSTALL = 1;
        const DELETE_AFTER_INSTALL = 1 << 1;
        const ALWAYS_UNINSTALL = 1 << 2;
        const SET_NTFS_COMPRESSION = 1 << 3;
        const UNSET_NTFS_COMPRESSION = 1 << 4;
    }
}
