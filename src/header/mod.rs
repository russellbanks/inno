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

use super::{InnoVersion, WindowsVersionRange, read::ReadBytesExt};
use crate::string::PascalString;

// https://github.com/jrsoftware/issrc/blob/main/Projects/Src/Shared.Struct.pas
#[derive(Clone, Default)]
pub struct Header {
    pub flags: HeaderFlags,
    #[doc(alias = "AppName")]
    app_name: Option<PascalString>,
    #[doc(alias = "AppVerName")]
    pub app_versioned_name: Option<PascalString>,
    /// <https://jrsoftware.org/ishelp/index.php?topic=setup_appid>
    pub app_id: Option<PascalString>,
    pub app_copyright: Option<PascalString>,
    pub app_publisher: Option<PascalString>,
    pub app_publisher_url: Option<PascalString>,
    pub app_support_phone: Option<PascalString>,
    pub app_support_url: Option<PascalString>,
    pub app_updates_url: Option<PascalString>,
    pub app_version: Option<PascalString>,
    pub default_dir_name: Option<PascalString>,
    pub default_group_name: Option<PascalString>,
    pub uninstall_icon_name: Option<PascalString>,
    pub base_filename: Option<PascalString>,
    pub uninstall_files_dir: Option<PascalString>,
    pub uninstall_name: Option<PascalString>,
    pub uninstall_icon: Option<PascalString>,
    pub app_mutex: Option<PascalString>,
    pub default_user_name: Option<PascalString>,
    pub default_user_organisation: Option<PascalString>,
    pub default_serial: Option<PascalString>,
    pub app_readme_file: Option<PascalString>,
    pub app_contact: Option<PascalString>,
    pub app_comments: Option<PascalString>,
    pub app_modify_path: Option<PascalString>,
    pub create_uninstall_registry_key: Option<PascalString>,
    pub uninstallable: Option<PascalString>,
    pub close_applications_filter: Option<PascalString>,
    pub setup_mutex: Option<PascalString>,
    pub changes_environment: Option<PascalString>,
    pub changes_associations: Option<PascalString>,
    pub architectures_allowed_expr: Option<PascalString>,
    pub architectures_allowed: Architecture,
    pub architectures_disallowed: Architecture,
    pub architectures_install_in_64_bit_mode: Architecture,
    pub architectures_install_in_64_bit_mode_expr: Option<PascalString>,
    pub close_applications_filter_excludes: Option<PascalString>,
    pub license_text: Option<PascalString>,
    pub info_before: Option<PascalString>,
    pub info_after: Option<PascalString>,
    pub uninstaller_signature: Option<PascalString>,
    pub compiled_code: Vec<u8>,
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
    pub fn read<R>(mut reader: R, version: InnoVersion) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut header = Self::default();

        if version < (1, 3, 0) {
            // Uncompressed size of the setup header
            reader.read_u32::<LE>()?;
        }

        header.app_name = PascalString::read(&mut reader)?;
        header.app_versioned_name = PascalString::read(&mut reader)?;
        if version >= 1.3 {
            header.app_id = PascalString::read(&mut reader)?;
        }
        header.app_copyright = PascalString::read(&mut reader)?;
        if version >= 1.3 {
            header.app_publisher = PascalString::read(&mut reader)?;
            header.app_publisher_url = PascalString::read(&mut reader)?;
        }
        if version >= (5, 1, 13) {
            header.app_support_phone = PascalString::read(&mut reader)?;
        }
        if version >= 1.3 {
            header.app_support_url = PascalString::read(&mut reader)?;
            header.app_updates_url = PascalString::read(&mut reader)?;
            header.app_version = PascalString::read(&mut reader)?;
        }
        header.default_dir_name = PascalString::read(&mut reader)?;
        header.default_group_name = PascalString::read(&mut reader)?;
        if version < 3 {
            header.uninstall_icon_name = PascalString::read_decoded(&mut reader, WINDOWS_1252)?;
        }
        header.base_filename = PascalString::read(&mut reader)?;
        if ((1, 3, 0)..(5, 2, 5)).contains(&version) {
            header.license_text = PascalString::read_decoded(&mut reader, WINDOWS_1252)?;
            header.info_before = PascalString::read_decoded(&mut reader, WINDOWS_1252)?;
            header.info_after = PascalString::read_decoded(&mut reader, WINDOWS_1252)?;
        }
        if version >= (1, 3, 3) {
            header.uninstall_files_dir = PascalString::read(&mut reader)?;
        }
        if version >= (1, 3, 6) {
            header.uninstall_name = PascalString::read(&mut reader)?;
            header.uninstall_icon = PascalString::read(&mut reader)?;
        }
        if version >= (1, 3, 14) {
            header.app_mutex = PascalString::read(&mut reader)?;
        }
        if version >= 3 {
            header.default_user_name = PascalString::read(&mut reader)?;
            header.default_user_organisation = PascalString::read(&mut reader)?;
        }
        if version >= 4 {
            header.default_serial = PascalString::read(&mut reader)?;
        }
        if ((4, 0, 0)..(5, 2, 5)).contains(&version) || (version.is_isx() && version >= (1, 3, 24))
        {
            header.compiled_code = reader.read_raw_pascal_string()?;
        }
        if version >= (4, 2, 4) {
            header.app_readme_file = PascalString::read(&mut reader)?;
            header.app_contact = PascalString::read(&mut reader)?;
            header.app_comments = PascalString::read(&mut reader)?;
            header.app_modify_path = PascalString::read(&mut reader)?;
        }
        if version >= (5, 3, 8) {
            header.create_uninstall_registry_key = PascalString::read(&mut reader)?;
        }
        if version >= (5, 3, 10) {
            header.uninstallable = PascalString::read(&mut reader)?;
        }
        if version >= (5, 5, 0) {
            header.close_applications_filter = PascalString::read(&mut reader)?;
        }
        if version >= (5, 5, 6) {
            header.setup_mutex = PascalString::read(&mut reader)?;
        }
        if version >= (5, 6, 1) {
            header.changes_environment = PascalString::read(&mut reader)?;
            header.changes_associations = PascalString::read(&mut reader)?;
        }
        if version >= 6.3 {
            /*let (allowed, disallowed) = PascalString::read(&mut src)?.map_or_else(
                || (Architecture::X86_COMPATIBLE, Architecture::empty()),
                |architecture| Architecture::from_expression(&architecture),
            );
            header.architectures_allowed = allowed;
            header.architectures_disallowed = disallowed;*/
            header.architectures_allowed_expr = PascalString::read(&mut reader)?;
            /*header.architectures_install_in_64_bit_mode =
            InnoValue::ansi_string_from(&mut src, codepage)?
                .map_or(Architecture::X86_COMPATIBLE, |architecture| {
                    Architecture::from_expression(&architecture).0
                });*/
            header.architectures_install_in_64_bit_mode_expr = PascalString::read(&mut reader)?;
        }
        if version >= (6, 4, 2) {
            header.close_applications_filter_excludes = PascalString::read(&mut reader)?;
        }
        if version >= (5, 2, 5) {
            header.license_text = PascalString::read_decoded(&mut reader, WINDOWS_1252)?;
            header.info_before = PascalString::read_decoded(&mut reader, WINDOWS_1252)?;
            header.info_after = PascalString::read_decoded(&mut reader, WINDOWS_1252)?;
        }
        if ((5, 2, 1)..(5, 3, 10)).contains(&version) {
            header.uninstaller_signature = PascalString::read(&mut reader)?;
        }
        if version >= (5, 2, 5) {
            header.compiled_code = reader.read_raw_pascal_string()?;
        }
        if version >= (2, 0, 6) && !version.is_unicode() {
            let mut buf = [0; 256 / u8::BITS as usize];
            reader.read_exact(&mut buf)?;
            header.lead_bytes = buf;
        }
        if version >= (4, 0, 0) {
            header.language_count = reader.read_u32::<LE>()?;
        } else if version >= (2, 0, 1) {
            header.language_count = 1;
        }
        if version >= (4, 2, 1) {
            header.message_count = reader.read_u32::<LE>()?;
        }
        if version >= (4, 1, 0) {
            header.permission_count = reader.read_u32::<LE>()?;
        }
        if version >= (2, 0, 0) || version.is_isx() {
            header.type_count = reader.read_u32::<LE>()?;
            header.component_count = reader.read_u32::<LE>()?;
        }
        if version >= (2, 0, 0) || (version.is_isx() && version >= (1, 3, 17)) {
            header.task_count = reader.read_u32::<LE>()?;
        }
        header.directory_count = reader.read_u32::<LE>()?;
        header.file_count = reader.read_u32::<LE>()?;
        header.data_entry_count = reader.read_u32::<LE>()?;
        header.icon_count = reader.read_u32::<LE>()?;
        header.ini_entry_count = reader.read_u32::<LE>()?;
        header.registry_entry_count = reader.read_u32::<LE>()?;
        header.delete_entry_count = reader.read_u32::<LE>()?;
        header.uninstall_delete_entry_count = reader.read_u32::<LE>()?;
        header.run_entry_count = reader.read_u32::<LE>()?;
        header.uninstall_run_entry_count = reader.read_u32::<LE>()?;
        let license_size = if version < (1, 3, 0) {
            reader.read_u32::<LE>()?
        } else {
            0
        };
        let info_before_size = if version < (1, 3, 0) {
            reader.read_u32::<LE>()?
        } else {
            0
        };
        let info_after_size = if version < (1, 3, 0) {
            reader.read_u32::<LE>()?
        } else {
            0
        };
        WindowsVersionRange::read_from(&mut reader, version)?;
        if version < (6, 4, 0, 1) {
            header.back_color = reader.read_t::<Color>()?;
        }
        if version >= (1, 3, 3) && version < (6, 4, 0, 1) {
            header.back_color2 = reader.read_t::<Color>()?;
        }
        if version < (5, 5, 7) {
            header.image_back_color = reader.read_t::<Color>()?;
        }
        if ((2, 0, 0)..(5, 0, 4)).contains(&version) || version.is_isx() {
            header.small_image_back_color = reader.read_t::<Color>()?;
        }
        if version >= (6, 0, 0) {
            header.wizard_style = InnoStyle::try_read_from_io(&mut reader)?;
            header.wizard_resize_percent_x = reader.read_u32::<LE>()?;
            header.wizard_resize_percent_y = reader.read_u32::<LE>()?;
        }
        if version >= (5, 5, 7) {
            header.image_alpha_format = ImageAlphaFormat::try_read_from_io(&mut reader)?;
        }
        if version >= (6, 4, 0) {
            let _sha256 = reader.read_u32::<LE>()?;
        } else if version >= (5, 3, 9) {
            let mut sha1_buf = [0; 160 / u8::BITS as usize]; // SHA1 bit length in bytes
            reader.read_exact(&mut sha1_buf)?;
        } else if version >= (4, 2, 0) {
            let mut md5_buf = [0; 128 / u8::BITS as usize]; // MD5 bit length in bytes
            reader.read_exact(&mut md5_buf)?;
        } else {
            let _crc32 = reader.read_u32::<LE>()?;
        }
        if version >= 6.4 {
            header.password_salt = Some(password_salt::<44>(&mut reader)?);
        } else if version >= (4, 2, 2) {
            header.password_salt = Some(password_salt::<8>(&mut reader)?);
        }
        if version >= 4 {
            header.extra_disk_space_required = reader.read_u64::<LE>()?;
            header.slices_per_disk = reader.read_u32::<LE>()?;
        } else {
            header.extra_disk_space_required = u64::from(reader.read_u32::<LE>()?);
            header.slices_per_disk = 1;
        }
        if (2..5).contains(&version) || (version.is_isx() && version >= (1, 3, 4)) {
            header.install_verbosity = InstallVerbosity::try_read_from_io(&mut reader)?;
        }
        if version >= 1.3 {
            header.uninstall_log_mode = LogMode::try_read_from_io(&mut reader)?;
        }
        if version >= 5 {
            header.uninstall_style = InnoStyle::Modern;
        } else if version >= 2 || (version.is_isx() && version >= (1, 3, 13)) {
            header.uninstall_style = InnoStyle::try_read_from_io(&mut reader)?;
        }
        if version >= (1, 3, 6) {
            header.dir_exists_warning = AutoBool::try_read_from_io(&mut reader)?;
        }
        if version.is_isx() && ((2, 0, 10)..(3, 0, 0)).contains(&version) {
            let _code_line_offset = reader.read_u32::<LE>()?;
        }
        if ((3, 0, 0)..(3, 0, 3)).contains(&version) {
            match AutoBool::try_read_from_io(&mut reader) {
                Ok(AutoBool::Yes) => header.flags |= HeaderFlags::ALWAYS_RESTART,
                Ok(AutoBool::Auto) => {
                    header.flags |= HeaderFlags::RESTART_IF_NEEDED_BY_RUN;
                }
                _ => {}
            }
        }
        if version >= (3, 0, 4) || (version.is_isx() && version >= (3, 0, 3)) {
            header.privileges_required = PrivilegeLevel::try_read_from_io(&mut reader)?;
        }
        if version >= 5.7 {
            header.privileges_required_overrides_allowed =
                PrivilegesRequiredOverrides::from_bits_retain(reader.read_u8()?);
        }
        if version >= (4, 0, 10) {
            header.show_language_dialog = AutoBool::try_read_from_io(&mut reader)?;
            header.language_detection = LanguageDetection::try_read_from_io(&mut reader)?;
        }
        if version >= (5, 3, 9) {
            header.compression = Compression::try_read_from_io(&mut reader)?;
        }
        if ((5, 1, 0)..(6, 3, 0)).contains(&version) {
            header.architectures_allowed =
                StoredArchitecture::from_bits_retain(reader.read_u8()?).into();
            header.architectures_install_in_64_bit_mode =
                StoredArchitecture::from_bits_retain(reader.read_u8()?).into();
        } else if version < 5.1 {
            header.architectures_allowed = StoredArchitecture::all().into();
            header.architectures_install_in_64_bit_mode = StoredArchitecture::all().into();
        }
        if ((5, 2, 1)..(5, 3, 10)).contains(&version) {
            header.signed_uninstaller_original_size = reader.read_u32::<LE>()?;
            header.signed_uninstaller_header_checksum = reader.read_u32::<LE>()?;
        }
        if version >= (5, 3, 3) {
            header.disable_dir_page = AutoBool::try_read_from_io(&mut reader)?;
            header.disable_program_group_page = AutoBool::try_read_from_io(&mut reader)?;
        }
        if version >= 5.5 {
            header.uninstall_display_size = reader.read_u64::<LE>()?;
        } else if version >= (5, 3, 6) {
            header.uninstall_display_size = u64::from(reader.read_u32::<LE>()?);
        }

        if version.is_blackbox() {
            reader.read_u8()?;
        }

        header.flags |= Self::read_flags(&mut reader, version)?;
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
        if version < 1.3 {
            header.license_text = Some(PascalString::read_sized_decoded(
                &mut reader,
                license_size,
                WINDOWS_1252,
            )?);
            header.info_before = Some(PascalString::read_sized_decoded(
                &mut reader,
                info_before_size,
                WINDOWS_1252,
            )?);
            header.info_after = Some(PascalString::read_sized_decoded(
                &mut reader,
                info_after_size,
                WINDOWS_1252,
            )?);
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
            if version < 3 => HeaderFlags::ALWAYS_CREATE_UNINSTALL_ICON,
            if version < (1, 3, 6) => HeaderFlags::OVERWRITE_UNINSTALL_REG_ENTRIES,
            if version < (5, 6, 1) => HeaderFlags::CHANGES_ASSOCIATIONS,
            if ((1, 3, 0)..(5, 3, 8)).contains(&version) => HeaderFlags::CREATE_UNINSTALL_REG_KEY,
            if version >= (1, 3, 1) => HeaderFlags::USE_PREVIOUS_APP_DIR,
            if version >= (1, 3, 3) && version < (6, 4, 0, 1) => HeaderFlags::BACK_COLOR_HORIZONTAL,
            if version >= (1, 3, 10) => HeaderFlags::USE_PREVIOUS_GROUP,
            if version >= (1, 3, 20) => HeaderFlags::UPDATE_UNINSTALL_LOG_APP_NAME,
            if version >= 2 || (version.is_isx() && version >= (1, 3, 10)) => HeaderFlags::USE_PREVIOUS_SETUP_TYPE,
            if version >= 2 => [
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
            if version >= 3 => [
                HeaderFlags::USER_INFO_PAGE,
                HeaderFlags::USE_PREVIOUS_USER_INFO,
            ],
            if version >= (3, 0, 1) => HeaderFlags::UNINSTALL_RESTART_COMPUTER,
            if version >= (3, 0, 3) => HeaderFlags::RESTART_IF_NEEDED_BY_RUN,
            if version >= 4 || (version.is_isx() && version >= (3, 0, 3)) => HeaderFlags::SHOW_TASKS_TREE_LINES,
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
            if version >= 6 => [
                HeaderFlags::APP_NAME_HAS_CONSTS,
                HeaderFlags::USE_PREVIOUS_PRIVILEGES,
                HeaderFlags::WIZARD_RESIZABLE,
            ],
            if version >= 6.3 => HeaderFlags::UNINSTALL_LOGGING
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

    pub fn decode(&mut self, codepage: &'static Encoding) {
        self.app_name
            .as_mut()
            .map(|app_name| app_name.decode(codepage));
        self.app_versioned_name
            .as_mut()
            .map(|versioned_name| versioned_name.decode(codepage));
        self.app_id.as_mut().map(|app_id| app_id.decode(codepage));
        self.app_copyright
            .as_mut()
            .map(|copyright| copyright.decode(codepage));
        self.app_publisher
            .as_mut()
            .map(|publisher| publisher.decode(codepage));
        self.app_publisher_url
            .as_mut()
            .map(|publisher_url| publisher_url.decode(codepage));
        self.app_support_phone
            .as_mut()
            .map(|support_phone| support_phone.decode(codepage));
        self.app_support_url
            .as_mut()
            .map(|support_url| support_url.decode(codepage));
        self.app_updates_url
            .as_mut()
            .map(|updates_url| updates_url.decode(codepage));
        self.app_version
            .as_mut()
            .map(|version| version.decode(codepage));
        self.default_dir_name
            .as_mut()
            .map(|dir_name| dir_name.decode(codepage));
        self.default_group_name
            .as_mut()
            .map(|group_name| group_name.decode(codepage));
        self.uninstall_icon_name
            .as_mut()
            .map(|icon_name| icon_name.decode(codepage));
        self.base_filename
            .as_mut()
            .map(|filename| filename.decode(codepage));
        self.uninstall_files_dir
            .as_mut()
            .map(|files_dir| files_dir.decode(codepage));
        self.uninstall_name
            .as_mut()
            .map(|uninstall_name| uninstall_name.decode(codepage));
        self.uninstall_icon
            .as_mut()
            .map(|icon| icon.decode(codepage));
        self.app_mutex.as_mut().map(|mutex| mutex.decode(codepage));
        self.default_user_name
            .as_mut()
            .map(|user_name| user_name.decode(codepage));
        self.default_user_organisation
            .as_mut()
            .map(|organisation| organisation.decode(codepage));
        self.default_serial
            .as_mut()
            .map(|serial| serial.decode(codepage));
        self.app_readme_file
            .as_mut()
            .map(|readme_file| readme_file.decode(codepage));
        self.app_contact
            .as_mut()
            .map(|contact| contact.decode(codepage));
        self.app_comments
            .as_mut()
            .map(|comments| comments.decode(codepage));
        self.app_modify_path
            .as_mut()
            .map(|modify_path| modify_path.decode(codepage));
        self.create_uninstall_registry_key
            .as_mut()
            .map(|registry_key| registry_key.decode(codepage));
        self.uninstallable
            .as_mut()
            .map(|uninstallable| uninstallable.decode(codepage));
        self.close_applications_filter
            .as_mut()
            .map(|filter| filter.decode(codepage));
        self.setup_mutex
            .as_mut()
            .map(|mutex| mutex.decode(codepage));
        self.changes_environment
            .as_mut()
            .map(|environment| environment.decode(codepage));
        self.changes_associations
            .as_mut()
            .map(|associations| associations.decode(codepage));
        self.architectures_allowed_expr
            .as_mut()
            .map(|expr| expr.decode(codepage));
        self.architectures_install_in_64_bit_mode_expr
            .as_mut()
            .map(|expr| expr.decode(codepage));
        self.close_applications_filter_excludes
            .as_mut()
            .map(|excludes| excludes.decode(codepage));
        self.license_text.as_mut().map(|text| text.decode(codepage));
        self.info_before.as_mut().map(|info| info.decode(codepage));
        self.info_after.as_mut().map(|info| info.decode(codepage));
        self.uninstaller_signature
            .as_mut()
            .map(|signature| signature.decode(codepage));
    }

    /// Returns the Product Code as it would appear in Windows Registry.
    #[must_use]
    pub fn product_code(&self) -> Option<String> {
        /// Inno tags '_is1' onto the end of the App ID to create the Uninstall registry key
        const IS1_SUFFIX: &str = "_is1";

        let app_id = self.app_id()?;

        let product_code = if app_id.starts_with("{{") {
            &app_id[1..]
        } else {
            &app_id
        };

        Some(format!("{product_code}{IS1_SUFFIX}"))
    }

    /// Returns the name of the application..
    #[doc(alias = "AppName")]
    #[must_use]
    #[inline]
    pub fn app_name(&self) -> Option<&str> {
        self.app_name.as_ref().map(PascalString::as_str)
    }

    /// Returns the versioned name of the application.
    #[doc(alias = "AppVerName")]
    #[must_use]
    #[inline]
    pub fn app_versioned_name(&self) -> Option<&str> {
        self.app_versioned_name.as_ref().map(PascalString::as_str)
    }

    /// Returns the ID of the application.
    #[doc(alias = "AppId")]
    #[must_use]
    #[inline]
    pub fn app_id(&self) -> Option<&str> {
        self.app_id.as_ref().map(PascalString::as_str)
    }

    /// Returns the copyright of the application.
    #[doc(alias = "AppCopyright")]
    #[must_use]
    #[inline]
    pub fn app_copyright(&self) -> Option<&str> {
        self.app_copyright.as_ref().map(PascalString::as_str)
    }

    /// Returns the publisher of the application.
    #[doc(alias = "AppPublisher")]
    #[must_use]
    #[inline]
    pub fn app_publisher(&self) -> Option<&str> {
        self.app_publisher.as_ref().map(PascalString::as_str)
    }

    /// Returns the URL of the publisher of the application.
    #[doc(alias = "AppPublisherURL")]
    #[must_use]
    #[inline]
    pub fn app_publisher_url(&self) -> Option<&str> {
        self.app_publisher_url.as_ref().map(PascalString::as_str)
    }

    /// Returns the support phone number for the application.
    #[doc(alias = "AppSupportPhone")]
    #[must_use]
    #[inline]
    pub fn app_support_phone(&self) -> Option<&str> {
        self.app_support_phone.as_ref().map(PascalString::as_str)
    }

    /// Returns the support URL for the application.
    #[doc(alias = "AppSupportURL")]
    #[must_use]
    #[inline]
    pub fn app_support_url(&self) -> Option<&str> {
        self.app_support_url.as_ref().map(PascalString::as_str)
    }

    /// Returns the updates URL for the application.
    #[doc(alias = "AppUpdatesURL")]
    #[must_use]
    #[inline]
    pub fn app_updates_url(&self) -> Option<&str> {
        self.app_updates_url.as_ref().map(PascalString::as_str)
    }

    /// Returns the version of the application.
    #[doc(alias = "AppVersion")]
    #[must_use]
    #[inline]
    pub fn app_version(&self) -> Option<&str> {
        self.app_version.as_ref().map(PascalString::as_str)
    }

    /// Returns the default directory name for the application.
    #[doc(alias = "DefaultDirName")]
    #[must_use]
    #[inline]
    pub fn default_dir_name(&self) -> Option<&str> {
        self.default_dir_name.as_ref().map(PascalString::as_str)
    }

    /// Returns the default group name for the application.
    #[doc(alias = "DefaultGroupName")]
    #[must_use]
    #[inline]
    pub fn default_group_name(&self) -> Option<&str> {
        self.default_group_name.as_ref().map(PascalString::as_str)
    }

    /// Returns the uninstall icon name for the application.
    #[doc(alias = "UninstallIconName")]
    #[must_use]
    #[inline]
    pub fn uninstall_icon_name(&self) -> Option<&str> {
        self.uninstall_icon_name.as_ref().map(PascalString::as_str)
    }

    /// Returns the base filename for the application.
    #[doc(alias = "BaseFileName")]
    #[must_use]
    #[inline]
    pub fn base_filename(&self) -> Option<&str> {
        self.base_filename.as_ref().map(PascalString::as_str)
    }

    /// Returns the uninstallation files directory for the application.
    #[doc(alias = "UninstallFilesDir")]
    #[must_use]
    #[inline]
    pub fn uninstall_files_dir(&self) -> Option<&str> {
        self.uninstall_files_dir.as_ref().map(PascalString::as_str)
    }

    /// Returns the uninstaller name for the application.
    #[doc(alias = "UninstallName")]
    #[must_use]
    #[inline]
    pub fn uninstall_name(&self) -> Option<&str> {
        self.uninstall_name.as_ref().map(PascalString::as_str)
    }

    /// Returns the uninstaller icon for the application.
    #[doc(alias = "UninstallIcon")]
    #[must_use]
    #[inline]
    pub fn uninstall_icon(&self) -> Option<&str> {
        self.uninstall_icon.as_ref().map(PascalString::as_str)
    }

    /// Returns the application mutex name.
    #[doc(alias = "AppMutex")]
    #[must_use]
    #[inline]
    pub fn app_mutex(&self) -> Option<&str> {
        self.app_mutex.as_ref().map(PascalString::as_str)
    }

    /// Returns the default user info name for the application.
    #[doc(alias = "DefaultUserInfoName")]
    #[must_use]
    #[inline]
    pub fn default_user_name(&self) -> Option<&str> {
        self.default_user_name.as_ref().map(PascalString::as_str)
    }

    /// Returns the default user organization for the application.
    #[doc(alias = "DefaultUserInfoOrg")]
    #[must_use]
    #[inline]
    pub fn default_user_organisation(&self) -> Option<&str> {
        self.default_user_organisation
            .as_ref()
            .map(PascalString::as_str)
    }

    /// Returns the default serial number for the application.
    #[doc(alias = "DefaultUserInfoSerial")]
    #[must_use]
    #[inline]
    pub fn default_serial(&self) -> Option<&str> {
        self.default_serial.as_ref().map(PascalString::as_str)
    }

    /// Returns the name of the application ReadMe file.
    #[doc(alias = "AppReadmeFile")]
    #[must_use]
    #[inline]
    pub fn app_readme_file(&self) -> Option<&str> {
        self.app_readme_file.as_ref().map(PascalString::as_str)
    }
}

fn password_salt<const LEN: usize>(reader: &mut impl io::Read) -> io::Result<String> {
    let mut password_salt_buf = [0; LEN];
    reader.read_exact(&mut password_salt_buf)?;
    Ok(String::from_utf8_lossy(&password_salt_buf).into_owned())
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Header")
            .field("Flags", &self.flags)
            .field("AppName", &self.app_name())
            .field("AppVerName", &self.app_versioned_name())
            .field("AppId", &self.app_id())
            .field("AppCopyright", &self.app_copyright())
            .field("AppPublisher", &self.app_publisher())
            .field("AppPublisherURL", &self.app_publisher_url())
            .field("AppSupportPhone", &self.app_support_phone())
            .field("AppSupportURL", &self.app_support_url())
            .field("AppUpdatesURL", &self.app_updates_url())
            .field("AppVersion", &self.app_version())
            .field("DefaultDirName", &self.default_dir_name())
            .field("DefaultGroupName", &self.default_group_name())
            .field("UninstallIconName", &self.uninstall_icon_name())
            .field("BaseFileName", &self.base_filename())
            .field("UninstallFilesDir", &self.uninstall_files_dir())
            .field("UninstallName", &self.uninstall_name())
            .field("UninstallIcon", &self.uninstall_icon())
            .field("AppMutex", &self.app_mutex())
            .field("DefaultUserInfoName", &self.default_user_name())
            .field("DefaultUserInfoOrg", &self.default_user_organisation())
            .field("DefaultUserInfoSerial", &self.default_serial())
            .field("AppReadmeFile", &self.app_readme_file())
            .field("AppContact", &self.app_contact)
            .field("AppComments", &self.app_comments)
            .field("AppModifyPath", &self.app_modify_path)
            .field("CreateUninstallRegKey", &self.create_uninstall_registry_key)
            .field("Uninstallable", &self.uninstallable)
            .field("CloseApplicationsFilter", &self.close_applications_filter)
            .field("SetupMutex", &self.setup_mutex)
            .field("ChangesEnvironment", &self.changes_environment)
            .field("ChangesAssociations", &self.changes_associations)
            .field("ArchitecturesAllowed", &self.architectures_allowed)
            .field("ArchitecturesDisallowed", &self.architectures_disallowed)
            .field(
                "ArchitecturesInstallIn64BitMode",
                &self.architectures_install_in_64_bit_mode,
            )
            .field(
                "CloseApplicationsFilterExcludes",
                &self.close_applications_filter_excludes,
            )
            .field("LicenseText", &self.license_text)
            .field("InfoBefore", &self.info_before)
            .field("InfoAfter", &self.info_after)
            .field("UninstallerSignature", &self.uninstaller_signature)
            // Skip compiled code
            // Skip lead bytes
            .field("NumLanguageEntries", &self.language_count)
            .field("NumCustomMessageEntries", &self.message_count)
            .field("NumPermissionEntries", &self.permission_count)
            .field("NumTypeEntries", &self.type_count)
            .field("NumComponentEntries", &self.component_count)
            .field("NumTaskEntries", &self.task_count)
            .field("NumDirEntries", &self.directory_count)
            .field("NumFileEntries", &self.file_count)
            .field("NumFileLocationEntries", &self.data_entry_count)
            .field("NumIconEntries", &self.icon_count)
            .field("NumIniEntries", &self.ini_entry_count)
            .field("NumRegistryEntries", &self.registry_entry_count)
            .field("NumUninstallDeleteEntries", &self.delete_entry_count)
            .field("NumUninstallRunEntries", &self.uninstall_delete_entry_count)
            .field("RunEntryCount", &self.run_entry_count)
            .field("UninstallRunEntryCount", &self.uninstall_run_entry_count)
            .field("BackColor", &self.back_color)
            .field("BackColor2", &self.back_color2)
            .field("ImageBackColor", &self.image_back_color)
            .field("SmallImageBackColor", &self.small_image_back_color)
            .field("WizardStyle", &self.wizard_style)
            .field("WizardResizePercentX", &self.wizard_resize_percent_x)
            .field("WizardResizePercentY", &self.wizard_resize_percent_y)
            .field("ImageAlphaFormat", &self.image_alpha_format)
            // Skip password salt
            .field("ExtraDiskSpaceRequired", &self.extra_disk_space_required)
            .field("SlicesPerDisk", &self.slices_per_disk)
            .field("InstallVerbosity", &self.install_verbosity)
            .field("UninstallLogMode", &self.uninstall_log_mode)
            .field("UninstallStyle", &self.uninstall_style)
            .field("DirExistsWarning", &self.dir_exists_warning)
            .field("PrivilegesRequired", &self.privileges_required)
            .field(
                "PrivilegesRequiredOverridesAllowed",
                &self.privileges_required_overrides_allowed,
            )
            .field("ShowLanguageDialog", &self.show_language_dialog)
            .field("LanguageDetection", &self.language_detection)
            .field("Compression", &self.compression)
            .field(
                "SignedUninstallerOriginalSize",
                &self.signed_uninstaller_original_size,
            )
            .field(
                "SignedUninstallerHeaderChecksum",
                &self.signed_uninstaller_header_checksum,
            )
            .field("DisableDirPage", &self.disable_dir_page)
            .field("DisableProgramGroupPage", &self.disable_program_group_page)
            .field("UninstallDisplaySize", &self.uninstall_display_size)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::Header;
    use crate::string::PascalString;

    #[test]
    fn product_code() {
        let header = Header {
            app_id: Some(PascalString::from(
                "{{31AA9DE2-36A2-4FB7-921F-865D4B0657D5}",
            )),
            ..Default::default()
        };

        assert_eq!(
            header.product_code().as_deref(),
            Some("{31AA9DE2-36A2-4FB7-921F-865D4B0657D5}_is1")
        );
    }
}
