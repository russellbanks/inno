use std::borrow::Cow;

use inno::{Header, version::InnoVersion};
use ratatui::{
    buffer::Buffer,
    layout::{
        Constraint::{Fill, Length},
        Flex, Rect,
    },
    widgets::{Row, Table, Widget},
};

use super::emoji::Emoji;
use crate::scroll::TableScroll;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Summary<'a> {
    header: &'a Header,
    version: InnoVersion,
    scroll: TableScroll,
    rows: Vec<Row<'a>>,
}

impl<'a> Summary<'a> {
    #[must_use]
    pub fn new(header: &'a Header, version: InnoVersion) -> Self {
        let rows = rows(header, version);
        Self {
            header,
            version,
            scroll: TableScroll::new(rows.len()),
            rows,
        }
    }

    pub const fn scroll_mut(&mut self) -> &mut TableScroll {
        &mut self.scroll
    }
}

fn rows(header: &'_ Header, version: InnoVersion) -> Vec<Row<'_>> {
    let mut rows = Vec::from([Row::new([
        Cow::Borrowed("Inno Setup version"),
        Cow::Owned(version.to_string()),
    ])]);
    if let Some(name) = header.app_name() {
        rows.push(Row::new(["App name", name]));
    }
    if let Some(version) = header.app_version() {
        rows.push(Row::new(["Version", version]));
    }
    if let Some(app_versioned_name) = header.app_versioned_name() {
        rows.push(Row::new(["Versioned name", app_versioned_name]));
    }
    if let Some(id) = header.app_id() {
        rows.push(Row::new(["ID", id]));
    }
    if let Some(copyright) = header.app_copyright() {
        rows.push(Row::new(["Copyright", copyright]));
    }
    if let Some(publisher) = header.app_publisher() {
        rows.push(Row::new(["Publisher", publisher]));
    }
    if let Some(publisher_url) = header.app_publisher_url() {
        rows.push(Row::new(["Publisher URL", publisher_url]));
    }
    if let Some(support_phone) = header.app_support_phone() {
        rows.push(Row::new(["Support phone", support_phone]));
    }
    if let Some(support_url) = header.app_support_url() {
        rows.push(Row::new(["Support URL", support_url]));
    }
    if let Some(updates_url) = header.app_updates_url() {
        rows.push(Row::new(["Updates URL", updates_url]));
    }
    if let Some(default_directory) = header.default_dir_name() {
        rows.push(Row::new(["Default directory", default_directory]));
    }
    if let Some(default_group_name) = header.default_group_name() {
        rows.push(Row::new(["Default group name", default_group_name]));
    }
    if let Some(icon_name) = header.uninstall_icon_name() {
        rows.push(Row::new(["Uninstall icon", icon_name]));
    }
    if let Some(mutex) = header.app_mutex() {
        rows.push(Row::new(["Mutex", mutex]));
    }
    if let Some(default_user_name) = header.default_user_name() {
        rows.push(Row::new(["Default user name", default_user_name]));
    }
    if let Some(default_user_organization) = header.default_user_organization() {
        rows.push(Row::new([
            "Default user organization",
            default_user_organization,
        ]));
    }
    if let Some(default_serial) = header.default_serial() {
        rows.push(Row::new(["Default serial", default_serial]));
    }
    if let Some(readme_file) = header.app_readme_file() {
        rows.push(Row::new(["ReadMe", readme_file]));
    }
    if let Some(contact) = header.app_contact() {
        rows.push(Row::new(["Contact", contact]));
    }
    if let Some(comments) = header.app_comments() {
        rows.push(Row::new(["Comments", comments]));
    }
    if let Some(modify_path) = header.app_modify_path() {
        rows.push(Row::new(["Modify path", modify_path]));
    }
    rows.push(Row::new([
        "Creates uninstall registry key",
        header.create_uninstall_registry_key().emoji(),
    ]));
    rows.push(Row::new([
        "Is uninstallable",
        header.is_uninstallable().emoji(),
    ]));
    rows.push(Row::new([
        "Close applications filter",
        header.close_applications_filter().emoji(),
    ]));
    if let Some(setup_mutex) = header.setup_mutex() {
        rows.push(Row::new(["Setup mutex", setup_mutex]));
    }
    rows.push(Row::new([
        "Changes environment",
        header.changes_environment().emoji(),
    ]));
    rows.push(Row::new([
        "Changes associations",
        header.changes_associations().emoji(),
    ]));
    rows.push(Row::new([
        Cow::Borrowed("Architectures allowed"),
        Cow::Owned(header.architectures_allowed().to_string()),
    ]));
    rows.push(Row::new([
        Cow::Borrowed("Architectures disallowed"),
        Cow::Owned(header.architectures_disallowed().to_string()),
    ]));
    if let Some(close_application_filter_excludes) = header.close_applications_filter_excludes() {
        rows.push(Row::new([
            "Close application filter excludes",
            close_application_filter_excludes,
        ]));
    }
    if let Some(uninstaller_signature) = header.uninstaller_signature() {
        rows.push(Row::new(["Uninstaller signature", uninstaller_signature]));
    }
    rows.push(Row::new(["Wizard style", header.wizard_style().as_str()]));
    rows.push(Row::new([
        Cow::Borrowed("Wizard size percent"),
        Cow::Owned(header.wizard_size_percent().to_string()),
    ]));

    rows.push(Row::new([
        "Wizard image alpha format",
        header.wizard_image_alpha_format().as_str(),
    ]));

    rows.push(Row::new([
        Cow::Borrowed("Wizard image background color"),
        Cow::Owned(header.image_background_color().to_string()),
    ]));

    rows.push(Row::new([
        Cow::Borrowed("Wizard small image background color"),
        Cow::Owned(header.small_image_background_color().to_string()),
    ]));

    rows.push(Row::new([
        Cow::Borrowed("Wizard image dark background color"),
        Cow::Owned(header.image_dynamic_background_color().to_string()),
    ]));

    if let Some(opacity) = header.wizard_image_opacity() {
        rows.push(Row::new([
            Cow::Borrowed("Wizard image opacity"),
            Cow::Owned(opacity.to_string()),
        ]));
    }

    rows.push(Row::new([
        Cow::Borrowed("Wizard small image dark background color"),
        Cow::Owned(header.small_image_dynamic_background_color().to_string()),
    ]));

    rows.push(Row::new([
        Cow::Borrowed("Extra disk space required"),
        Cow::Owned(header.extra_disk_space_required().to_string()),
    ]));

    rows.push(Row::new([
        Cow::Borrowed("Slices per disk"),
        Cow::Owned(header.extra_disk_space_required().to_string()),
    ]));

    rows.push(Row::new([
        "Install verbosity",
        header.install_verbosity().as_str(),
    ]));
    rows.push(Row::new([
        "Uninstall log mode",
        header.uninstall_log_mode().as_str(),
    ]));
    rows.push(Row::new([
        "Uninstall style",
        header.uninstall_style().as_str(),
    ]));
    rows.push(Row::new([
        "Directory exists warning",
        header.directory_exists_warning().emoji(),
    ]));
    rows.push(Row::new([
        "Privileges required",
        header.privileges_required().as_str(),
    ]));
    rows.push(Row::new([
        "Privileges required overrides",
        header.privileges_required().as_str(),
    ]));
    rows.push(Row::new([
        "Show language dialog",
        header.show_language_dialog().emoji(),
    ]));
    rows.push(Row::new([
        "Language detection method",
        header.language_detection_method().as_str(),
    ]));
    rows.push(Row::new(["Compression", header.compression().as_str()]));

    rows.push(Row::new([
        Cow::Borrowed("Signed uninstaller original size"),
        Cow::Owned(header.signed_uninstaller_original_size().to_string()),
    ]));

    rows.push(Row::new([
        "Directory page disabled",
        header.is_directory_page_disabled().emoji(),
    ]));

    rows.push(Row::new([
        "Program group page disabled",
        header.is_program_group_page_disabled().emoji(),
    ]));

    rows.push(Row::new([
        Cow::Borrowed("Uninstall display size"),
        Cow::Owned(header.uninstall_display_size().to_string()),
    ]));

    rows
}

impl Widget for &mut Summary<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.scroll.render(
            Table::new(self.rows.iter().cloned(), [Length(40), Fill(1)]).flex(Flex::Start),
            ["Name", "Value"],
            "Summary",
            area,
            buf,
        );
    }
}
