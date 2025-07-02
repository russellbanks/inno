use std::{fmt, io};

use encoding_rs::{Encoding, UTF_16LE, WINDOWS_1252};
use zerocopy::LE;

use crate::{encoding::InnoValue, read::ReadBytesExt, version::InnoVersion};

#[derive(Clone)]
pub struct Language {
    pub internal_name: Option<String>,
    pub name: Option<String>,
    pub dialog_font: Option<String>,
    pub title_font: Option<String>,
    pub welcome_font: Option<String>,
    pub copyright_font: Option<String>,
    pub data: Option<String>,
    pub license_text: Option<String>,
    pub info_before: Option<String>,
    pub info_after: Option<String>,
    pub id: u32,
    pub codepage: &'static Encoding,
    pub dialog_font_size: u32,
    pub dialog_font_standard_height: u32,
    pub title_font_size: u32,
    pub welcome_font_size: u32,
    pub copyright_font_size: u32,
    pub right_to_left: bool,
}

impl Language {
    pub fn read_from<R>(
        mut src: R,
        codepage: &'static Encoding,
        version: &InnoVersion,
    ) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut language = Self::default();

        if *version >= (4, 0, 0) {
            language.internal_name = InnoValue::string_from(&mut src, codepage)?;
        }

        language.name = InnoValue::string_from(&mut src, codepage)?;
        language.dialog_font = InnoValue::string_from(&mut src, codepage)?;
        language.title_font = InnoValue::string_from(&mut src, codepage)?;
        language.welcome_font = InnoValue::string_from(&mut src, codepage)?;
        language.copyright_font = InnoValue::string_from(&mut src, codepage)?;

        if *version >= (4, 0, 0) {
            language.data = InnoValue::string_from(&mut src, codepage)?;
        }

        if *version >= (4, 0, 1) {
            language.license_text = InnoValue::string_from(&mut src, codepage)?;
            language.info_before = InnoValue::string_from(&mut src, codepage)?;
            language.info_after = InnoValue::string_from(&mut src, codepage)?;
        }

        language.id = src.read_u32::<LE>()?;

        if *version < (4, 2, 2) {
            language.codepage = u16::try_from(language.id)
                .ok()
                .and_then(codepage::to_encoding)
                .unwrap_or(WINDOWS_1252);
        } else if !version.is_unicode() {
            let codepage = src.read_u32::<LE>()?;
            language.codepage = (codepage != 0)
                .then(|| u16::try_from(codepage).ok().and_then(codepage::to_encoding))
                .flatten()
                .unwrap_or(WINDOWS_1252);
        } else {
            if *version < (5, 3, 0) {
                src.read_u32::<LE>()?;
            }
            language.codepage = UTF_16LE;
        }

        language.dialog_font_size = src.read_u32::<LE>()?;

        if *version < (4, 1, 0) {
            language.dialog_font_standard_height = src.read_u32::<LE>()?;
        }

        language.title_font_size = src.read_u32::<LE>()?;
        language.welcome_font_size = src.read_u32::<LE>()?;
        language.copyright_font_size = src.read_u32::<LE>()?;

        if *version >= (5, 2, 3) {
            language.right_to_left = src.read_u8()? != 0;
        }

        Ok(language)
    }
}

impl fmt::Debug for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Language")
            .field("internal_name", &self.internal_name)
            .field("name", &self.name)
            .field("dialog_font", &self.dialog_font)
            .field("title_font", &self.title_font)
            .field("welcome_font", &self.welcome_font)
            .field("copyright_font", &self.copyright_font)
            .field("license_text", &self.license_text)
            .field("info_before", &self.info_before)
            .field("info_after", &self.info_after)
            .field("id", &self.id)
            .field("codepage", &self.codepage)
            .field("dialog_font_size", &self.dialog_font_size)
            .field(
                "dialog_font_standard_height",
                &self.dialog_font_standard_height,
            )
            .field("title_font_size", &self.title_font_size)
            .field("welcome_font_size", &self.welcome_font_size)
            .field("copyright_font_size", &self.copyright_font_size)
            .field("right_to_left", &self.right_to_left)
            .finish_non_exhaustive()
    }
}

impl Default for Language {
    fn default() -> Self {
        Self {
            internal_name: None,
            name: None,
            dialog_font: None,
            title_font: None,
            welcome_font: None,
            copyright_font: None,
            data: None,
            license_text: None,
            info_before: None,
            info_after: None,
            id: 0,
            codepage: WINDOWS_1252,
            dialog_font_size: 0,
            dialog_font_standard_height: 0,
            title_font_size: 0,
            welcome_font_size: 0,
            copyright_font_size: 0,
            right_to_left: false,
        }
    }
}
