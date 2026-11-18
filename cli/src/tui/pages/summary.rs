use std::borrow::Cow;

use inno::{Header, header::PrivilegesRequiredOverrides, version::InnoVersion};
use ratatui::layout::Flex;

use crate::tui::{
    emoji::Emoji,
    table::{Column, EntryTable},
};

/// One named value from the installer's header.
pub struct Field<'a> {
    name: &'static str,
    value: Cow<'a, str>,
}

pub fn table(header: &Header, version: InnoVersion) -> EntryTable<'_, Field<'_>> {
    EntryTable::owned(
        "Summary",
        fields(header, version),
        vec![
            Column::new("Name", |field: &Field| Cow::Borrowed(field.name)).fixed(),
            Column::new("Value", |field: &Field| Cow::Borrowed(&field.value)).fill(),
        ],
    )
    .unnumbered()
    .flex(Flex::Start)
}

#[derive(Default)]
struct Fields<'a>(Vec<Field<'a>>);

impl<'a> Fields<'a> {
    fn add(&mut self, name: &'static str, value: impl Into<Cow<'a, str>>) {
        self.0.push(Field {
            name,
            value: value.into(),
        });
    }

    /// Adds the field only if it has a value.
    fn optional(&mut self, name: &'static str, value: Option<&'a str>) {
        if let Some(value) = value {
            self.add(name, value);
        }
    }
}

fn fields(header: &Header, version: InnoVersion) -> Vec<Field<'_>> {
    let mut fields = Fields::default();
    fields.add("Inno Setup version", version.to_string());
    fields.optional("App name", header.app_name());
    fields.optional("Version", header.app_version());
    fields.optional("Versioned name", header.app_versioned_name());
    fields.optional("ID", header.app_id());
    fields.optional("Copyright", header.app_copyright());
    fields.optional("Publisher", header.app_publisher());
    fields.optional("Publisher URL", header.app_publisher_url());
    fields.optional("Support phone", header.app_support_phone());
    fields.optional("Support URL", header.app_support_url());
    fields.optional("Updates URL", header.app_updates_url());
    fields.optional("Default directory", header.default_dir_name());
    fields.optional("Default group name", header.default_group_name());
    fields.optional("Uninstall icon", header.uninstall_icon_name());
    fields.optional("Mutex", header.app_mutex());
    fields.optional("Default user name", header.default_user_name());
    fields.optional(
        "Default user organization",
        header.default_user_organization(),
    );
    fields.optional("Default serial", header.default_serial());
    fields.optional("ReadMe", header.app_readme_file());
    fields.optional("Contact", header.app_contact());
    fields.optional("Comments", header.app_comments());
    fields.optional("Modify path", header.app_modify_path());
    fields.add(
        "Creates uninstall registry key",
        header.create_uninstall_registry_key().emoji(),
    );
    fields.add("Is uninstallable", header.is_uninstallable().emoji());
    fields.add(
        "Close applications filter",
        header.close_applications_filter().emoji(),
    );
    fields.optional("Setup mutex", header.setup_mutex());
    fields.add("Changes environment", header.changes_environment().emoji());
    fields.add(
        "Changes associations",
        header.changes_associations().emoji(),
    );
    fields.add(
        "Architectures allowed",
        header.architectures_allowed().to_string(),
    );
    fields.add(
        "Architectures disallowed",
        header.architectures_disallowed().to_string(),
    );
    fields.optional(
        "Close application filter excludes",
        header.close_applications_filter_excludes(),
    );
    fields.optional("Uninstaller signature", header.uninstaller_signature());
    fields.add("Wizard style", header.wizard_style().as_str());
    fields.add(
        "Wizard size percent",
        header.wizard_size_percent().to_string(),
    );
    fields.add(
        "Wizard image alpha format",
        header.wizard_image_alpha_format().as_str(),
    );
    fields.add(
        "Wizard image background color",
        header.image_background_color().to_string(),
    );
    fields.add(
        "Wizard small image background color",
        header.small_image_background_color().to_string(),
    );
    fields.add(
        "Wizard image dark background color",
        header.image_dynamic_background_color().to_string(),
    );
    if let Some(opacity) = header.wizard_image_opacity() {
        fields.add("Wizard image opacity", opacity.to_string());
    }
    fields.add(
        "Wizard small image dark background color",
        header.small_image_dynamic_background_color().to_string(),
    );
    fields.add(
        "Extra disk space required",
        header.extra_disk_space_required().to_string(),
    );
    fields.add("Slices per disk", header.slices_per_disk().to_string());
    fields.add("Install verbosity", header.install_verbosity().as_str());
    fields.add("Uninstall log mode", header.uninstall_log_mode().as_str());
    fields.add("Uninstall style", header.uninstall_style().as_str());
    fields.add(
        "Directory exists warning",
        header.directory_exists_warning().emoji(),
    );
    fields.add("Privileges required", header.privileges_required().as_str());
    fields.add(
        "Privileges required overrides",
        privileges_required_overrides(header.privileges_required_overrides_allowed()),
    );
    fields.add(
        "Show language dialog",
        header.show_language_dialog().emoji(),
    );
    fields.add(
        "Language detection method",
        header.language_detection_method().as_str(),
    );
    fields.add("Compression", header.compression().as_str());
    fields.add(
        "Signed uninstaller original size",
        header.signed_uninstaller_original_size().to_string(),
    );
    fields.add(
        "Directory page disabled",
        header.is_directory_page_disabled().emoji(),
    );
    fields.add(
        "Program group page disabled",
        header.is_program_group_page_disabled().emoji(),
    );
    fields.add(
        "Uninstall display size",
        header.uninstall_display_size().to_string(),
    );
    fields.0
}

/// Formats the overrides as `COMMAND_LINE | DIALOG`, or `None` if there are none.
fn privileges_required_overrides(overrides: PrivilegesRequiredOverrides) -> String {
    if overrides.is_empty() {
        return String::from("None");
    }
    overrides
        .iter_names()
        .map(|(name, _)| name)
        .collect::<Vec<_>>()
        .join(" | ")
}
