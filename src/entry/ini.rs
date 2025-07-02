use std::io;

use bitflags::bitflags;
use encoding_rs::Encoding;
use zerocopy::LE;

use crate::{
    encoding::InnoValue, entry::Condition, read::ReadBytesExt, version::InnoVersion,
    windows_version::WindowsVersionRange,
};

#[derive(Clone, Debug, Default)]
pub struct Ini {
    pub file: String,
    pub section: Option<String>,
    pub key: Option<String>,
    pub value: Option<String>,
    pub flags: IniFlags,
}

impl Ini {
    const DEFAULT_FILE: &'static str = "{windows}/WIN.INI";

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

        let mut ini = Self {
            file: InnoValue::string_from(&mut src, codepage)?
                .unwrap_or_else(|| Self::DEFAULT_FILE.to_string()),
            section: InnoValue::string_from(&mut src, codepage)?,
            key: InnoValue::string_from(&mut src, codepage)?,
            value: InnoValue::string_from(&mut src, codepage)?,
            ..Self::default()
        };

        Condition::read_from(&mut src, codepage, version)?;

        WindowsVersionRange::read_from(&mut src, version)?;

        ini.flags = IniFlags::from_bits_retain(src.read_u8()?);

        Ok(ini)
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default)]
    pub struct IniFlags: u8 {
        const CREATE_KEY_IF_DOESNT_EXIST = 1;
        const UNINSTALL_DELETE_ENTRY = 1 << 1;
        const UNINSTALL_DELETE_ENTIRE_SECTION = 1 << 2;
        const UNINSTALL_DELETE_SECTION_IF_EMPTY = 1 << 3;
        const HAS_VALUE = 1 << 4;
    }
}
