use std::io;

use bitflags::bitflags;
use encoding_rs::Encoding;
use zerocopy::{Immutable, KnownLayout, LE, TryFromBytes};

use super::Condition;
use crate::{
    InnoVersion, ReadBytesExt, WindowsVersionRange, encoding::InnoValue,
    header::flag_reader::read_flags::read_flags,
};

#[derive(Clone, Debug, Default)]
pub struct Icon {
    pub name: Option<String>,
    pub filename: Option<String>,
    pub parameters: Option<String>,
    pub working_directory: Option<String>,
    pub file: Option<String>,
    pub comment: Option<String>,
    pub app_user_model_id: Option<String>,
    pub app_user_model_toast_activator_clsid: String,
    pub index: i32,
    pub show_command: i32,
    pub close_on_exit: CloseSetting,
    pub hotkey: u16,
    pub flags: IconFlags,
}

impl Icon {
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

        let mut icon = Self {
            name: InnoValue::string_from(&mut src, codepage)?,
            filename: InnoValue::string_from(&mut src, codepage)?,
            parameters: InnoValue::string_from(&mut src, codepage)?,
            working_directory: InnoValue::string_from(&mut src, codepage)?,
            file: InnoValue::string_from(&mut src, codepage)?,
            comment: InnoValue::string_from(&mut src, codepage)?,
            ..Self::default()
        };

        Condition::read_from(&mut src, codepage, version)?;

        if version >= (5, 3, 5) {
            icon.app_user_model_id = InnoValue::string_from(&mut src, codepage)?;
        }

        if version >= (6, 1, 0) {
            let mut buf = [0; 16];
            src.read_exact(&mut buf)?;
            icon.app_user_model_toast_activator_clsid = codepage.decode(&buf).0.into_owned();
        }

        WindowsVersionRange::read_from(&mut src, version)?;

        icon.index = src.read_i32::<LE>()?;

        icon.show_command = if version >= (1, 3, 24) {
            src.read_i32::<LE>()?
        } else {
            1
        };

        if version >= (1, 3, 15) {
            icon.close_on_exit = CloseSetting::try_read_from_io(&mut src)?;
        }

        if version >= (2, 0, 7) {
            icon.hotkey = src.read_u16::<LE>()?;
        }

        icon.flags = read_flags!(&mut src,
            IconFlags::NEVER_UNINSTALL,
            if version < (1, 3, 26) => IconFlags::RUN_MINIMIZED,
            [IconFlags::CREATE_ONLY_IF_FILE_EXISTS, IconFlags::USE_APP_PATHS],
            if ((5, 0, 3)..(6, 3, 0)).contains(&version) => IconFlags::FOLDER_SHORTCUT,
            if version >= (5, 4, 2) => IconFlags::EXCLUDE_FROM_SHOW_IN_NEW_INSTALL,
            if version >= (5, 5, 0) => IconFlags::PREVENT_PINNING,
            if version >= (6, 1, 0) => IconFlags::HAS_APP_USER_MODEL_TOAST_ACTIVATOR_CLSID
        )?;

        Ok(icon)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Immutable, KnownLayout, TryFromBytes)]
#[repr(u8)]
pub enum CloseSetting {
    #[default]
    NoSetting = 0,
    CloseOnExit = 1,
    DontCloseOnExit = 2,
}

impl CloseSetting {
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
    pub struct IconFlags: u8 {
        const NEVER_UNINSTALL = 1;
        const CREATE_ONLY_IF_FILE_EXISTS = 1 << 1;
        const USE_APP_PATHS = 1 << 2;
        const FOLDER_SHORTCUT = 1 << 3;
        const EXCLUDE_FROM_SHOW_IN_NEW_INSTALL = 1 << 4;
        const PREVENT_PINNING = 1 << 5;
        const HAS_APP_USER_MODEL_TOAST_ACTIVATOR_CLSID = 1 << 6;
        const RUN_MINIMIZED = 1 << 7;
    }
}
