mod copy_mode;
mod flags;

use std::io;

pub use copy_mode::FileCopyMode;
use encoding_rs::Encoding;
pub use flags::FileFlags;
use zerocopy::{Immutable, KnownLayout, LE, TryFromBytes};

use crate::{
    encoding::InnoValue, entry::Condition, header::flag_reader::read_flags::read_flags,
    read::ReadBytesExt, version::InnoVersion, windows_version::WindowsVersionRange,
};

/// <https://github.com/jrsoftware/issrc/blob/is-6_4_3/Projects/Src/Shared.Struct.pas#L225>
#[derive(Clone, Debug, Default)]
pub struct File {
    source: Option<String>,
    destination: Option<String>,
    install_font_name: Option<String>,
    strong_assembly_name: Option<String>,
    /// Index into the data entry list
    location: u32,
    attributes: u32,
    external_size: u64,
    /// Index into the permission entry list
    permission: i16,
    flags: FileFlags,
    r#type: FileType,
}

impl File {
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

        let mut file = Self {
            source: InnoValue::string_from(&mut src, codepage)?,
            destination: InnoValue::string_from(&mut src, codepage)?,
            install_font_name: InnoValue::string_from(&mut src, codepage)?,
            permission: -1,
            ..Self::default()
        };

        if version >= (5, 2, 5) {
            file.strong_assembly_name = InnoValue::string_from(&mut src, codepage)?;
        }

        Condition::read_from(&mut src, codepage, version)?;

        WindowsVersionRange::read_from(&mut src, version)?;

        file.location = src.read_u32::<LE>()?;
        file.attributes = src.read_u32::<LE>()?;
        file.external_size = if version >= (4, 0, 0) {
            src.read_u64::<LE>()?
        } else {
            u64::from(src.read_u32::<LE>()?)
        };

        if version < (3, 0, 5) {
            match FileCopyMode::try_read_from_io(&mut src)? {
                FileCopyMode::Normal => file.flags |= FileFlags::PROMPT_IF_OLDER,
                FileCopyMode::IfDoesntExist => {
                    file.flags |= FileFlags::ONLY_IF_DOESNT_EXIST | FileFlags::PROMPT_IF_OLDER;
                }
                FileCopyMode::AlwaysOverwrite => {
                    file.flags |= FileFlags::IGNORE_VERSION | FileFlags::PROMPT_IF_OLDER;
                }
                FileCopyMode::AlwaysSkipIfSameOrOlder => {}
            }
        }

        if version >= (4, 1, 0) {
            file.permission = src.read_i16::<LE>()?;
        }

        file.flags |= read_flags!(&mut src,
            [
                FileFlags::CONFIRM_OVERWRITE,
                FileFlags::NEVER_UNINSTALL,
                FileFlags::RESTART_REPLACE,
                FileFlags::DELETE_AFTER_INSTALL,
                FileFlags::REGISTER_SERVER,
                FileFlags::REGISTER_TYPE_LIB,
                FileFlags::SHARED_FILE,
            ],
            if version < (2, 0, 0) && !version.is_isx() => FileFlags::IS_README_FILE,
            [FileFlags::COMPARE_TIME_STAMP, FileFlags::FONT_IS_NOT_TRUE_TYPE],
            if version >= (1, 2, 5) => FileFlags::SKIP_IF_SOURCE_DOESNT_EXIST,
            if version >= (1, 2, 6) => FileFlags::OVERWRITE_READ_ONLY,
            if version >= (1, 3, 21) => [
                FileFlags::OVERWRITE_SAME_VERSION,
                FileFlags::CUSTOM_DEST_NAME
            ],
            if version >= (1, 3, 25) => FileFlags::ONLY_IF_DEST_FILE_EXISTS,
            if version >= (2, 0, 5) => FileFlags::NO_REG_ERROR,
            if version >= (3, 0, 1) => FileFlags::UNINS_RESTART_DELETE,
            if version >= (3, 0, 5) => [
                FileFlags::ONLY_IF_DOESNT_EXIST,
                FileFlags::IGNORE_VERSION,
                FileFlags::PROMPT_IF_OLDER,
            ],
            if version >= (4, 0, 0)
                || (version.is_isx() && version >= (3, 0, 6)) => FileFlags::DONT_COPY,
            if version >= (4, 0, 5) => FileFlags::UNINS_REMOVE_READ_ONLY,
            if version >= (4, 1, 8) => FileFlags::RECURSE_SUB_DIRS_EXTERNAL,
            if version >= (4, 2, 1) => FileFlags::REPLACE_SAME_VERSION_IF_CONTENTS_DIFFER,
            if version >= (4, 2, 5) => FileFlags::DONT_VERIFY_CHECKSUM,
            if version >= (5, 0, 3) => FileFlags::UNINS_NO_SHARED_FILE_PROMPT,
            if version >= (5, 1, 0) => FileFlags::CREATE_ALL_SUB_DIRS,
            if version >= (5, 1, 2) => FileFlags::BITS_32, FileFlags::BITS_64,
            if version >= (5, 2, 0) => [
                FileFlags::EXTERNAL_SIZE_PRESET,
                FileFlags::SET_NTFS_COMPRESSION,
                FileFlags::UNSET_NTFS_COMPRESSION,
            ],
            if version >= (5, 2, 5) => FileFlags::GAC_INSTALL
        )?;

        file.r#type = FileType::try_read_from_io(&mut src)?;

        Ok(file)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Immutable, KnownLayout, TryFromBytes)]
#[repr(u8)]
enum FileType {
    #[default]
    UserFile = 0,
    UninstallExe = 1,
    RegSvrExe = 2,
}

impl FileType {
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
