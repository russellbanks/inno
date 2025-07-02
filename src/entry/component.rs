use std::io::{Read, Result};

use bitflags::bitflags;
use encoding_rs::Encoding;
use zerocopy::LE;

use crate::{
    encoding::InnoValue, read::ReadBytesExt, version::InnoVersion,
    windows_version::WindowsVersionRange,
};

#[derive(Clone, Debug, Default)]
pub struct Component {
    pub name: Option<String>,
    pub description: Option<String>,
    pub types: Option<String>,
    pub languages: Option<String>,
    pub check: Option<String>,
    pub extra_disk_space_required: u64,
    pub level: u32,
    pub used: bool,
    pub flags: ComponentFlags,
    pub size: u64,
}

impl Component {
    pub fn read_from<R: Read>(
        mut src: R,
        codepage: &'static Encoding,
        version: InnoVersion,
    ) -> Result<Self> {
        let mut component = Self {
            name: InnoValue::string_from(&mut src, codepage)?,
            description: InnoValue::string_from(&mut src, codepage)?,
            types: InnoValue::string_from(&mut src, codepage)?,
            ..Self::default()
        };

        if version >= (4, 0, 1) {
            component.languages = InnoValue::string_from(&mut src, codepage)?;
        }

        if version >= (4, 0, 0) || (version.is_isx() && version >= (1, 3, 24)) {
            component.check = InnoValue::string_from(&mut src, codepage)?;
        }

        if version >= (4, 0, 0) {
            component.extra_disk_space_required = src.read_u64::<LE>()?;
        } else {
            component.extra_disk_space_required = u64::from(src.read_u32::<LE>()?);
        }

        if version >= (4, 0, 0) || (version.is_isx() && version >= (3, 0, 3)) {
            component.level = src.read_u32::<LE>()?;
        }

        if version >= (4, 0, 0) || (version.is_isx() && version >= (3, 0, 4)) {
            component.used = src.read_u8()? != 0;
        } else {
            component.used = true;
        }

        WindowsVersionRange::read_from(&mut src, version)?;

        component.flags = ComponentFlags::from_bits_retain(src.read_u8()?);

        if version >= (4, 0, 0) {
            component.size = src.read_u64::<LE>()?;
        } else if version >= (2, 0, 0) || (version.is_isx() && version >= (1, 3, 24)) {
            component.size = u64::from(src.read_u32::<LE>()?);
        }

        Ok(component)
    }
}

bitflags! {
    #[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
    pub struct ComponentFlags: u8 {
        const FIXED = 1;
        const RESTART = 1 << 1;
        const DISABLE_NO_UNINSTALL_WARNING = 1 << 2;
        const EXCLUSIVE = 1 << 3;
        const DONT_INHERIT_CHECK = 1 << 4;
    }
}
