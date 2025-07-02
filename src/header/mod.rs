mod architecture;
mod auto_bool;
mod color;
mod compression;
pub mod flag_reader;
mod flags;
mod image_alpha_format;
mod inno_style;
mod install_verbosity;
mod language_detection;
mod log_mode;
mod privilege_level;

use std::{fmt, io};

pub use architecture::{Architecture, StoredArchitecture};
pub use auto_bool::AutoBool;
pub use color::Color;
pub use compression::Compression;
use encoding_rs::{Encoding, WINDOWS_1252};
use flag_reader::read_flags::read_flags;
pub use flags::{HeaderFlags, PrivilegesRequiredOverrides};
pub use image_alpha_format::ImageAlphaFormat;
pub use inno_style::InnoStyle;
pub use install_verbosity::InstallVerbosity;
pub use language_detection::LanguageDetection;
pub use log_mode::LogMode;
pub use privilege_level::PrivilegeLevel;
use zerocopy::LE;

use super::{
    encoding::InnoValue, read::ReadBytesExt, version::InnoVersion,
    windows_version::WindowsVersionRange,
};

// https://github.com/jrsoftware/issrc/blob/main/Projects/Src/Shared.Struct.pas
#[derive(Clone, Default)]
pub struct Header {
    pub flags: HeaderFlags,
    pub app_name: Option<String>,
    pub app_versioned_name: Option<String>,
    /// <https://jrsoftware.org/ishelp/index.php?topic=setup_appid>
    pub app_id: Option<String>,
    pub app_copyright: Option<String>,
    pub app_publisher: Option<String>,
    pub app_publisher_url: Option<String>,
    pub app_support_phone: Option<String>,
    pub app_support_url: Option<String>,
    pub app_updates_url: Option<String>,
    pub app_version: Option<String>,
    pub default_dir_name: Option<String>,
    pub default_group_name: Option<String>,
    pub uninstall_icon_name: Option<String>,
    pub base_filename: Option<String>,
    pub uninstall_files_dir: Option<String>,
    pub uninstall_name: Option<String>,
    pub uninstall_icon: Option<String>,
    pub app_mutex: Option<String>,
    pub default_user_name: Option<String>,
    pub default_user_organisation: Option<String>,
    pub default_serial: Option<String>,
    pub app_readme_file: Option<String>,
    pub app_contact: Option<String>,
    pub app_comments: Option<String>,
    pub app_modify_path: Option<String>,
    pub create_uninstall_registry_key: Option<String>,
    pub uninstallable: Option<String>,
    pub close_applications_filter: Option<String>,
    pub setup_mutex: Option<String>,
    pub changes_environment: Option<String>,
    pub changes_associations: Option<String>,
    pub architectures_allowed: Architecture,
    pub architectures_disallowed: Architecture,
    pub architectures_install_in_64_bit_mode: Architecture,
    pub close_applications_filter_excludes: Option<String>,
    pub license_text: Option<String>,
    pub info_before: Option<String>,
    pub info_after: Option<String>,
    pub uninstaller_signature: Option<String>,
    pub compiled_code: Option<Vec<u8>>,
    pub lead_bytes: [u8; 256 / u8::BITS as usize],
    pub language_count: u32,
    pub message_count: u32,
    pub permission_count: u32,
    pub type_count: u32,
    pub component_count: u32,
    pub task_count: u32,
    pub directory_count: u32,
    pub file_count: u32,
    pub data_entry_count: u32,
    pub icon_count: u32,
    pub ini_entry_count: u32,
    pub registry_entry_count: u32,
    pub delete_entry_count: u32,
    pub uninstall_delete_entry_count: u32,
    pub run_entry_count: u32,
    pub uninstall_run_entry_count: u32,
    pub windows_version_range: WindowsVersionRange,
    pub back_color: Color,
    pub back_color2: Color,
    pub image_back_color: Color,
    pub small_image_back_color: Color,
    pub wizard_style: InnoStyle,
    pub wizard_resize_percent_x: u32,
    pub wizard_resize_percent_y: u32,
    pub image_alpha_format: ImageAlphaFormat,
    pub password_salt: Option<String>,
    pub extra_disk_space_required: u64,
    pub slices_per_disk: u32,
    pub install_verbosity: InstallVerbosity,
    pub uninstall_log_mode: LogMode,
    pub uninstall_style: InnoStyle,
    pub dir_exists_warning: AutoBool,
    pub privileges_required: PrivilegeLevel,
    pub privileges_required_overrides_allowed: PrivilegesRequiredOverrides,
    pub show_language_dialog: AutoBool,
    pub language_detection: LanguageDetection,
    pub compression: Compression,
    pub signed_uninstaller_original_size: u32,
    pub signed_uninstaller_header_checksum: u32,
    pub disable_dir_page: AutoBool,
    pub disable_program_group_page: AutoBool,
    pub uninstall_display_size: u64,
}

impl Header {
    pub fn read_from<R>(
        mut src: R,
        codepage: &'static Encoding,
        version: InnoVersion,
    ) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut header = Self::default();

        if version < (1, 3, 0) {
            // Uncompressed size of the setup header
            src.read_u32::<LE>()?;
        }

        header.app_name = InnoValue::string_from(&mut src, codepage)?;
        header.app_versioned_name = InnoValue::string_from(&mut src, codepage)?;
        if version >= (1, 3, 0) {
            header.app_id = InnoValue::string_from(&mut src, codepage)?;
        }
        header.app_copyright = InnoValue::string_from(&mut src, codepage)?;
        if version >= (1, 3, 0) {
            header.app_publisher = InnoValue::string_from(&mut src, codepage)?;
            header.app_publisher_url = InnoValue::string_from(&mut src, codepage)?;
        }
        if version >= (5, 1, 13) {
            header.app_support_phone = InnoValue::string_from(&mut src, codepage)?;
        }
        if version >= (1, 3, 0) {
            header.app_support_url = InnoValue::string_from(&mut src, codepage)?;
            header.app_updates_url = InnoValue::string_from(&mut src, codepage)?;
            header.app_version = InnoValue::string_from(&mut src, codepage)?;
        }
        header.default_dir_name = InnoValue::string_from(&mut src, codepage)?;
        header.default_group_name = InnoValue::string_from(&mut src, codepage)?;
        if version < (3, 0, 0) {
            header.uninstall_icon_name = InnoValue::string_from(&mut src, WINDOWS_1252)?;
        }
        header.base_filename = InnoValue::string_from(&mut src, codepage)?;
        if ((1, 3, 0)..(5, 2, 5)).contains(&version) {
            header.license_text = InnoValue::string_from(&mut src, WINDOWS_1252)?;
            header.info_before = InnoValue::string_from(&mut src, WINDOWS_1252)?;
            header.info_after = InnoValue::string_from(&mut src, WINDOWS_1252)?;
        }
        if version >= (1, 3, 3) {
            header.uninstall_files_dir = InnoValue::string_from(&mut src, codepage)?;
        }
        if version >= (1, 3, 6) {
            header.uninstall_name = InnoValue::string_from(&mut src, codepage)?;
            header.uninstall_icon = InnoValue::string_from(&mut src, codepage)?;
        }
        if version >= (1, 3, 14) {
            header.app_mutex = InnoValue::string_from(&mut src, codepage)?;
        }
        if version >= (3, 0, 0) {
            header.default_user_name = InnoValue::string_from(&mut src, codepage)?;
            header.default_user_organisation = InnoValue::string_from(&mut src, codepage)?;
        }
        if version >= (4, 0, 0) {
            header.default_serial = InnoValue::string_from(&mut src, codepage)?;
        }
        if ((4, 0, 0)..(5, 2, 5)).contains(&version) || (version.is_isx() && version >= (1, 3, 24))
        {
            header.compiled_code = InnoValue::raw_from(&mut src)?;
        }
        if version >= (4, 2, 4) {
            header.app_readme_file = InnoValue::string_from(&mut src, codepage)?;
            header.app_contact = InnoValue::string_from(&mut src, codepage)?;
            header.app_comments = InnoValue::string_from(&mut src, codepage)?;
            header.app_modify_path = InnoValue::string_from(&mut src, codepage)?;
        }
        if version >= (5, 3, 8) {
            header.create_uninstall_registry_key = InnoValue::string_from(&mut src, codepage)?;
        }
        if version >= (5, 3, 10) {
            header.uninstallable = InnoValue::string_from(&mut src, codepage)?;
        }
        if version >= (5, 5, 0) {
            header.close_applications_filter = InnoValue::string_from(&mut src, codepage)?;
        }
        if version >= (5, 5, 6) {
            header.setup_mutex = InnoValue::string_from(&mut src, codepage)?;
        }
        if version >= (5, 6, 1) {
            header.changes_environment = InnoValue::string_from(&mut src, codepage)?;
            header.changes_associations = InnoValue::string_from(&mut src, codepage)?;
        }
        if version >= (6, 3, 0) {
            let (allowed, disallowed) = InnoValue::string_from(&mut src, codepage)?.map_or_else(
                || (Architecture::X86_COMPATIBLE, Architecture::empty()),
                |architecture| Architecture::from_expression(&architecture),
            );
            header.architectures_allowed = allowed;
            header.architectures_disallowed = disallowed;
            header.architectures_install_in_64_bit_mode =
                InnoValue::string_from(&mut src, codepage)?
                    .map_or(Architecture::X86_COMPATIBLE, |architecture| {
                        Architecture::from_expression(&architecture).0
                    });
        }
        if version >= (6, 4, 2) {
            header.close_applications_filter_excludes = InnoValue::string_from(&mut src, codepage)?;
        }
        if version >= (5, 2, 5) {
            header.license_text = InnoValue::string_from(&mut src, WINDOWS_1252)?;
            header.info_before = InnoValue::string_from(&mut src, WINDOWS_1252)?;
            header.info_after = InnoValue::string_from(&mut src, WINDOWS_1252)?;
        }
        if ((5, 2, 1)..(5, 3, 10)).contains(&version) {
            header.uninstaller_signature = InnoValue::string_from(&mut src, codepage)?;
        }
        if version >= (5, 2, 5) {
            header.compiled_code = InnoValue::raw_from(&mut src)?;
        }
        if version >= (2, 0, 6) && !version.is_unicode() {
            let mut buf = [0; 256 / u8::BITS as usize];
            src.read_exact(&mut buf)?;
            header.lead_bytes = buf;
        }
        if version >= (4, 0, 0) {
            header.language_count = src.read_u32::<LE>()?;
        } else if version >= (2, 0, 1) {
            header.language_count = 1;
        }
        if version >= (4, 2, 1) {
            header.message_count = src.read_u32::<LE>()?;
        }
        if version >= (4, 1, 0) {
            header.permission_count = src.read_u32::<LE>()?;
        }
        if version >= (2, 0, 0) || version.is_isx() {
            header.type_count = src.read_u32::<LE>()?;
            header.component_count = src.read_u32::<LE>()?;
        }
        if version >= (2, 0, 0) || (version.is_isx() && version >= (1, 3, 17)) {
            header.task_count = src.read_u32::<LE>()?;
        }
        header.directory_count = src.read_u32::<LE>()?;
        header.file_count = src.read_u32::<LE>()?;
        header.data_entry_count = src.read_u32::<LE>()?;
        header.icon_count = src.read_u32::<LE>()?;
        header.ini_entry_count = src.read_u32::<LE>()?;
        header.registry_entry_count = src.read_u32::<LE>()?;
        header.delete_entry_count = src.read_u32::<LE>()?;
        header.uninstall_delete_entry_count = src.read_u32::<LE>()?;
        header.run_entry_count = src.read_u32::<LE>()?;
        header.uninstall_run_entry_count = src.read_u32::<LE>()?;
        let license_size = if version < (1, 3, 0) {
            src.read_u32::<LE>()?
        } else {
            0
        };
        let info_before_size = if version < (1, 3, 0) {
            src.read_u32::<LE>()?
        } else {
            0
        };
        let info_after_size = if version < (1, 3, 0) {
            src.read_u32::<LE>()?
        } else {
            0
        };
        header.windows_version_range = WindowsVersionRange::read_from(&mut src, version)?;
        if version < (6, 4, 0, 1) {
            header.back_color = src.read_t::<Color>()?;
        }
        if version >= (1, 3, 3) && version < (6, 4, 0, 1) {
            header.back_color2 = src.read_t::<Color>()?;
        }
        if version < (5, 5, 7) {
            header.image_back_color = src.read_t::<Color>()?;
        }
        if ((2, 0, 0)..(5, 0, 4)).contains(&version) || version.is_isx() {
            header.small_image_back_color = src.read_t::<Color>()?;
        }
        if version >= (6, 0, 0) {
            header.wizard_style = InnoStyle::try_read_from_io(&mut src)?;
            header.wizard_resize_percent_x = src.read_u32::<LE>()?;
            header.wizard_resize_percent_y = src.read_u32::<LE>()?;
        }
        if version >= (5, 5, 7) {
            header.image_alpha_format = ImageAlphaFormat::try_read_from_io(&mut src)?;
        }
        if version >= (6, 4, 0) {
            let _sha256 = src.read_u32::<LE>()?;
        } else if version >= (5, 3, 9) {
            let mut sha1_buf = [0; 160 / u8::BITS as usize]; // SHA1 bit length in bytes
            src.read_exact(&mut sha1_buf)?;
        } else if version >= (4, 2, 0) {
            let mut md5_buf = [0; 128 / u8::BITS as usize]; // MD5 bit length in bytes
            src.read_exact(&mut md5_buf)?;
        } else {
            let _crc32 = src.read_u32::<LE>()?;
        }
        if version >= (6, 4, 0) {
            header.password_salt = Some(password_salt::<44>(&mut src)?);
        } else if version >= (4, 2, 2) {
            header.password_salt = Some(password_salt::<8>(&mut src)?);
        }
        if version >= (4, 0, 0) {
            header.extra_disk_space_required = src.read_u64::<LE>()?;
            header.slices_per_disk = src.read_u32::<LE>()?;
        } else {
            header.extra_disk_space_required = u64::from(src.read_u32::<LE>()?);
            header.slices_per_disk = 1;
        }
        if ((2, 0, 0)..(5, 0, 0)).contains(&version) || (version.is_isx() && version >= (1, 3, 4)) {
            header.install_verbosity = InstallVerbosity::try_read_from_io(&mut src)?;
        }
        if version >= (1, 3, 0) {
            header.uninstall_log_mode = LogMode::try_read_from_io(&mut src)?;
        }
        if version >= (5, 0, 0) {
            header.uninstall_style = InnoStyle::Modern;
        } else if version >= (2, 0, 0) || (version.is_isx() && version >= (1, 3, 13)) {
            header.uninstall_style = InnoStyle::try_read_from_io(&mut src)?;
        }
        if version >= (1, 3, 6) {
            header.dir_exists_warning = AutoBool::try_read_from_io(&mut src)?;
        }
        if version.is_isx() && ((2, 0, 10)..(3, 0, 0)).contains(&version) {
            let _code_line_offset = src.read_u32::<LE>()?;
        }
        if ((3, 0, 0)..(3, 0, 3)).contains(&version) {
            match AutoBool::try_read_from_io(&mut src) {
                Ok(AutoBool::Yes) => header.flags |= HeaderFlags::ALWAYS_RESTART,
                Ok(AutoBool::Auto) => {
                    header.flags |= HeaderFlags::RESTART_IF_NEEDED_BY_RUN;
                }
                _ => {}
            }
        }
        if version >= (3, 0, 4) || (version.is_isx() && version >= (3, 0, 3)) {
            header.privileges_required = PrivilegeLevel::try_read_from_io(&mut src)?;
        }
        if version >= (5, 7, 0) {
            header.privileges_required_overrides_allowed =
                PrivilegesRequiredOverrides::from_bits_retain(src.read_u8()?);
        }
        if version >= (4, 0, 10) {
            header.show_language_dialog = AutoBool::try_read_from_io(&mut src)?;
            header.language_detection = LanguageDetection::try_read_from_io(&mut src)?;
        }
        if version >= (5, 3, 9) {
            header.compression = Compression::try_read_from_io(&mut src)?;
        }
        if ((5, 1, 0)..(6, 3, 0)).contains(&version) {
            header.architectures_allowed =
                StoredArchitecture::from_bits_retain(src.read_u8()?).into();
            header.architectures_install_in_64_bit_mode =
                StoredArchitecture::from_bits_retain(src.read_u8()?).into();
        } else if version < (5, 1, 0) {
            header.architectures_allowed = StoredArchitecture::all().into();
            header.architectures_install_in_64_bit_mode = StoredArchitecture::all().into();
        }
        if ((5, 2, 1)..(5, 3, 10)).contains(&version) {
            header.signed_uninstaller_original_size = src.read_u32::<LE>()?;
            header.signed_uninstaller_header_checksum = src.read_u32::<LE>()?;
        }
        if version >= (5, 3, 3) {
            header.disable_dir_page = AutoBool::try_read_from_io(&mut src)?;
            header.disable_program_group_page = AutoBool::try_read_from_io(&mut src)?;
        }
        if version >= (5, 5, 0) {
            header.uninstall_display_size = src.read_u64::<LE>()?;
        } else if version >= (5, 3, 6) {
            header.uninstall_display_size = u64::from(src.read_u32::<LE>()?);
        }

        if version.is_blackbox() {
            src.read_u8()?;
        }

        header.flags |= Self::read_flags(&mut src, version)?;
        if version < (3, 0, 4) {
            header.privileges_required = PrivilegeLevel::from(header.flags);
        }
        if version < (4, 0, 10) {
            header.show_language_dialog =
                AutoBool::from_header_flags(&header.flags, HeaderFlags::SHOW_LANGUAGE_DIALOG);
            header.language_detection = LanguageDetection::from(header.flags);
        }
        if version < (4, 1, 5) {
            header.compression = Compression::from(header.flags);
        }
        if version < (5, 3, 3) {
            header.disable_dir_page =
                AutoBool::from_header_flags(&header.flags, HeaderFlags::DISABLE_DIR_PAGE);
            header.disable_program_group_page =
                AutoBool::from_header_flags(&header.flags, HeaderFlags::DISABLE_PROGRAM_GROUP_PAGE);
        }
        if version < (1, 3, 0) {
            header.license_text =
                InnoValue::sized_string_from(&mut src, license_size, WINDOWS_1252)?;
            header.info_before =
                InnoValue::sized_string_from(&mut src, info_before_size, WINDOWS_1252)?;
            header.info_after =
                InnoValue::sized_string_from(&mut src, info_after_size, WINDOWS_1252)?;
        }

        Ok(header)
    }

    fn read_flags<R>(reader: &mut R, version: InnoVersion) -> io::Result<HeaderFlags>
    where
        R: io::Read,
    {
        read_flags!(reader,
            HeaderFlags::DISABLE_STARTUP_PROMPT,
            if version < (5, 3, 10) => HeaderFlags::UNINSTALLABLE,
            HeaderFlags::CREATE_APP_DIR,
            if version < (5, 3, 3) => HeaderFlags::DISABLE_DIR_PAGE,
            if version < (1, 3, 6) => HeaderFlags::DISABLE_DIR_EXISTS_WARNING,
            if version < (5, 3, 3) => HeaderFlags::DISABLE_PROGRAM_GROUP_PAGE,
            HeaderFlags::ALLOW_NO_ICONS,
            if !((3, 0, 0)..(3, 0, 3)).contains(&version) => HeaderFlags::ALWAYS_RESTART,
            if version < (1, 3, 3) => HeaderFlags::BACK_SOLID,
            HeaderFlags::ALWAYS_USE_PERSONAL_GROUP,
            if version < (6, 4, 0, 1) => [
                HeaderFlags::WINDOW_VISIBLE,
                HeaderFlags::WINDOW_SHOW_CAPTION,
                HeaderFlags::WINDOW_RESIZABLE,
                HeaderFlags::WINDOW_START_MAXIMISED,
            ],
            HeaderFlags::ENABLED_DIR_DOESNT_EXIST_WARNING,
            if version < (4, 1, 2) => HeaderFlags::DISABLE_APPEND_DIR,
            HeaderFlags::PASSWORD,
            if version >= (1, 2, 6) => HeaderFlags::ALLOW_ROOT_DIRECTORY,
            if version >= (1, 2, 14) => HeaderFlags::DISABLE_FINISHED_PAGE,
            if version < (3, 0, 4) => HeaderFlags::ADMIN_PRIVILEGES_REQUIRED,
            if version < (3, 0, 0) => HeaderFlags::ALWAYS_CREATE_UNINSTALL_ICON,
            if version < (1, 3, 6) => HeaderFlags::OVERWRITE_UNINSTALL_REG_ENTRIES,
            if version < (5, 6, 1) => HeaderFlags::CHANGES_ASSOCIATIONS,
            if ((1, 3, 0)..(5, 3, 8)).contains(&version) => HeaderFlags::CREATE_UNINSTALL_REG_KEY,
            if version >= (1, 3, 1) => HeaderFlags::USE_PREVIOUS_APP_DIR,
            if version >= (1, 3, 3) && version < (6, 4, 0, 1) => HeaderFlags::BACK_COLOR_HORIZONTAL,
            if version >= (1, 3, 10) => HeaderFlags::USE_PREVIOUS_GROUP,
            if version >= (1, 3, 20) => HeaderFlags::UPDATE_UNINSTALL_LOG_APP_NAME,
            if version >= (2, 0, 0) || (version.is_isx() && version >= (1, 3, 10)) => HeaderFlags::USE_PREVIOUS_SETUP_TYPE,
            if version >= (2, 0, 0) => [
                HeaderFlags::DISABLE_READY_MEMO,
                HeaderFlags::ALWAYS_SHOW_COMPONENTS_LIST,
                HeaderFlags::FLAT_COMPONENTS_LIST,
                HeaderFlags::SHOW_COMPONENT_SIZES,
                HeaderFlags::USE_PREVIOUS_TASKS,
                HeaderFlags::DISABLE_READY_PAGE,
            ],
            if version >= (2, 0, 7) => [
                HeaderFlags::ALWAYS_SHOW_DIR_ON_READY_PAGE,
                HeaderFlags::ALWAYS_SHOW_GROUP_ON_READY_PAGE,
            ],
            if ((2, 0, 17)..(4, 1, 5)).contains(&version) => HeaderFlags::BZIP_USED,
            if version >= (2, 0, 18) => HeaderFlags::ALLOW_UNC_PATH,
            if version >= (3, 0, 0) => [
                HeaderFlags::USER_INFO_PAGE,
                HeaderFlags::USE_PREVIOUS_USER_INFO,
            ],
            if version >= (3, 0, 1) => HeaderFlags::UNINSTALL_RESTART_COMPUTER,
            if version >= (3, 0, 3) => HeaderFlags::RESTART_IF_NEEDED_BY_RUN,
            if version >= (4, 0, 0) || (version.is_isx() && version >= (3, 0, 3)) => HeaderFlags::SHOW_TASKS_TREE_LINES,
            if ((4, 0, 1)..(4, 0, 10)).contains(&version) => HeaderFlags::DETECT_LANGUAGE_USING_LOCALE,
            if version >= (4, 0, 9) => HeaderFlags::ALLOW_CANCEL_DURING_INSTALL,
            if version >= (4, 1, 3) => HeaderFlags::WIZARD_IMAGE_STRETCH,
            if version >= (4, 1, 8) => [
                HeaderFlags::APPEND_DEFAULT_DIR_NAME,
                HeaderFlags::APPEND_DEFAULT_GROUP_NAME,
            ],
            if version >= (4, 2, 2) => HeaderFlags::ENCRYPTION_USED,
            if ((5, 0, 4)..(5, 6, 1)).contains(&version) => HeaderFlags::CHANGES_ENVIRONMENT,
            if version >= (5, 1, 7) && !version.is_unicode() => HeaderFlags::SHOW_UNDISPLAYABLE_LANGUAGES,
            if version >= (5, 1, 13) => HeaderFlags::SETUP_LOGGING,
            if version >= (5, 2, 1) => HeaderFlags::SIGNED_UNINSTALLER,
            if version >= (5, 3, 8) => HeaderFlags::USE_PREVIOUS_LANGUAGE,
            if version >= (5, 3, 9) => HeaderFlags::DISABLE_WELCOME_PAGE,
            if version >= (5, 5, 0) => [
                HeaderFlags::CLOSE_APPLICATIONS,
                HeaderFlags::RESTART_APPLICATIONS,
                HeaderFlags::ALLOW_NETWORK_DRIVE,
            ],
            if version >= (5, 5, 7) => HeaderFlags::FORCE_CLOSE_APPLICATIONS,
            if version >= (6, 0, 0) => [
                HeaderFlags::APP_NAME_HAS_CONSTS,
                HeaderFlags::USE_PREVIOUS_PRIVILEGES,
                HeaderFlags::WIZARD_RESIZABLE,
            ],
            if version >= (6, 3, 0) => HeaderFlags::UNINSTALL_LOGGING
        ).map(|mut read_flags| {
            if version < (4, 0, 9) {
                read_flags |= HeaderFlags::ALLOW_CANCEL_DURING_INSTALL;
            }
            if version < (5, 5, 0) {
                read_flags |= HeaderFlags::ALLOW_NETWORK_DRIVE;
            }
            read_flags
        })
    }

    /// Returns the Product Code as it would appear in Windows Registry.
    #[must_use]
    pub fn product_code(&self) -> Option<String> {
        /// Inno tags '_is1' onto the end of the App ID to create the Uninstall registry key
        const IS1_SUFFIX: &str = "_is1";

        self.app_id.as_deref().map(|app_id| {
            // Remove escaped bracket
            let product_code = if app_id.starts_with("{{") {
                &app_id[1..]
            } else {
                app_id
            };

            format!("{product_code}{IS1_SUFFIX}")
        })
    }
}

fn password_salt<const LEN: usize>(src: &mut impl io::Read) -> io::Result<String> {
    let mut password_salt_buf = [0; LEN];
    src.read_exact(&mut password_salt_buf)?;
    Ok(String::from_utf8_lossy(&password_salt_buf).into_owned())
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Header")
            .field("flags", &self.flags)
            .field("app_name", &self.app_name)
            .field("app_versioned_name", &self.app_versioned_name)
            .field("app_id", &self.app_id)
            .field("app_copyright", &self.app_copyright)
            .field("app_publisher", &self.app_publisher)
            .field("app_publisher_url", &self.app_publisher_url)
            .field("app_support_phone", &self.app_support_phone)
            .field("app_support_url", &self.app_support_url)
            .field("app_updates_url", &self.app_updates_url)
            .field("app_version", &self.app_version)
            .field("default_dir_name", &self.default_dir_name)
            .field("default_group_name", &self.default_group_name)
            .field("uninstall_icon_name", &self.uninstall_icon_name)
            .field("base_filename", &self.base_filename)
            .field("uninstall_files_dir", &self.uninstall_files_dir)
            .field("uninstall_name", &self.uninstall_name)
            .field("uninstall_icon", &self.uninstall_icon)
            .field("app_mutex", &self.app_mutex)
            .field("default_user_name", &self.default_user_name)
            .field("default_user_organisation", &self.default_user_organisation)
            .field("default_serial", &self.default_serial)
            .field("app_readme_file", &self.app_readme_file)
            .field("app_contact", &self.app_contact)
            .field("app_comments", &self.app_comments)
            .field("app_modify_path", &self.app_modify_path)
            .field(
                "create_uninstall_registry_key",
                &self.create_uninstall_registry_key,
            )
            .field("uninstallable", &self.uninstallable)
            .field("close_applications_filter", &self.close_applications_filter)
            .field("setup_mutex", &self.setup_mutex)
            .field("changes_environment", &self.changes_environment)
            .field("changes_associations", &self.changes_associations)
            .field("architectures_allowed", &self.architectures_allowed)
            .field("architectures_disallowed", &self.architectures_disallowed)
            .field(
                "architectures_install_in_64_bit_mode",
                &self.architectures_install_in_64_bit_mode,
            )
            .field(
                "close_applications_filter_excludes",
                &self.close_applications_filter_excludes,
            )
            // Skip license text field
            .field("info_before", &self.info_before)
            .field("info_after", &self.info_after)
            .field("uninstaller_signature", &self.uninstaller_signature)
            // Skip compiled code
            // Skip lead bytes
            .field("language_count", &self.language_count)
            .field("message_count", &self.message_count)
            .field("permission_count", &self.permission_count)
            .field("type_count", &self.type_count)
            .field("component_count", &self.component_count)
            .field("task_count", &self.task_count)
            .field("directory_count", &self.directory_count)
            .field("file_count", &self.file_count)
            .field("data_entry_count", &self.data_entry_count)
            .field("icon_count", &self.icon_count)
            .field("ini_entry_count", &self.ini_entry_count)
            .field("registry_entry_count", &self.registry_entry_count)
            .field("delete_entry_count", &self.delete_entry_count)
            .field(
                "uninstall_delete_entry_count",
                &self.uninstall_delete_entry_count,
            )
            .field("run_entry_count", &self.run_entry_count)
            .field("uninstall_run_entry_count", &self.uninstall_run_entry_count)
            // Skip Windows version range
            .field("back_color", &self.back_color)
            .field("back_color2", &self.back_color2)
            .field("image_back_color", &self.image_back_color)
            .field("small_image_back_color", &self.small_image_back_color)
            .field("wizard_style", &self.wizard_style)
            .field("wizard_resize_percent_x", &self.wizard_resize_percent_x)
            .field("wizard_resize_percent_y", &self.wizard_resize_percent_y)
            .field("image_alpha_format", &self.image_alpha_format)
            // Skip password salt
            .field("extra_disk_space_required", &self.extra_disk_space_required)
            .field("slices_per_disk", &self.slices_per_disk)
            .field("install_verbosity", &self.install_verbosity)
            .field("uninstall_log_mode", &self.uninstall_log_mode)
            .field("uninstall_style", &self.uninstall_style)
            .field("dir_exists_warning", &self.dir_exists_warning)
            .field("privileges_required", &self.privileges_required)
            .field(
                "privileges_required_overrides_allowed",
                &self.privileges_required_overrides_allowed,
            )
            .field("show_language_dialog", &self.show_language_dialog)
            .field("language_detection", &self.language_detection)
            .field("compression", &self.compression)
            .field(
                "signed_uninstaller_original_size",
                &self.signed_uninstaller_original_size,
            )
            .field(
                "signed_uninstaller_header_checksum",
                &self.signed_uninstaller_header_checksum,
            )
            .field("disable_dir_page", &self.disable_dir_page)
            .field(
                "disable_program_group_page",
                &self.disable_program_group_page,
            )
            .field("uninstall_display_size", &self.uninstall_display_size)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::Header;

    #[test]
    fn product_code() {
        let header = Header {
            app_id: Some(String::from("{{31AA9DE2-36A2-4FB7-921F-865D4B0657D5}")),
            ..Default::default()
        };

        assert_eq!(
            header.product_code(),
            Some(String::from("{31AA9DE2-36A2-4FB7-921F-865D4B0657D5}_is1"))
        )
    }
}
