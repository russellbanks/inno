use std::io;

use bitflags::bitflags;
use encoding_rs::Encoding;
use zerocopy::LE;

use crate::{
    InnoVersion, ReadBytesExt, WindowsVersionRange, encoding::InnoValue,
    header::flag_reader::read_flags::read_flags,
};

#[derive(Clone, Debug, Default)]
pub struct Task {
    pub name: Option<String>,
    pub description: Option<String>,
    pub group_description: Option<String>,
    pub components: Option<String>,
    pub languages: Option<String>,
    pub check: Option<String>,
    pub level: u32,
    pub used: bool,
    pub flags: TaskFlags,
}

impl Task {
    pub fn read_from<R>(
        mut src: R,
        codepage: &'static Encoding,
        version: InnoVersion,
    ) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut task = Self {
            name: InnoValue::string_from(&mut src, codepage)?,
            description: InnoValue::string_from(&mut src, codepage)?,
            group_description: InnoValue::string_from(&mut src, codepage)?,
            components: InnoValue::string_from(&mut src, codepage)?,
            ..Self::default()
        };

        if version >= (4, 0, 1) {
            task.languages = InnoValue::string_from(&mut src, codepage)?;
        }

        if version >= (4, 0, 0) || (version.is_isx() && version >= (1, 3, 24)) {
            task.check = InnoValue::string_from(&mut src, codepage)?;
        }

        if version >= (4, 0, 0) || (version.is_isx() && version >= (3, 0, 3)) {
            task.level = src.read_u32::<LE>()?;
        }

        if version >= (4, 0, 0) || (version.is_isx() && version >= (3, 0, 4)) {
            task.used = src.read_u8()? != 0;
        } else {
            task.used = true;
        }

        WindowsVersionRange::read_from(&mut src, version)?;

        task.flags = read_flags!(&mut src,
            [TaskFlags::EXCLUSIVE, TaskFlags::UNCHECKED],
            if version >= (2, 0, 5) => TaskFlags::RESTART,
            if version >= (2, 0, 6) => TaskFlags::CHECKED_ONCE,
            if version >= (4, 2, 3) => TaskFlags::DONT_INHERIT_CHECK
        )?;

        Ok(task)
    }
}

bitflags! {
    #[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
    pub struct TaskFlags: u8 {
        const EXCLUSIVE = 1;
        const UNCHECKED = 1 << 1;
        const RESTART = 1 << 2;
        const CHECKED_ONCE = 1 << 3;
        const DONT_INHERIT_CHECK = 1 << 4;
    }
}
