use std::io::{Read, Result};

use encoding_rs::Encoding;

use crate::{encoding::InnoValue, version::InnoVersion};

#[derive(Clone, Debug, Default)]
pub struct Condition {
    pub components: Option<String>,
    pub tasks: Option<String>,
    pub languages: Option<String>,
    pub check: Option<String>,
    pub after_install: Option<String>,
    pub before_install: Option<String>,
}

impl Condition {
    pub fn read_from<R: Read>(
        mut src: R,
        codepage: &'static Encoding,
        version: InnoVersion,
    ) -> Result<Self> {
        let mut condition = Self::default();

        if version >= (2, 0, 0) || (version.is_isx() && version >= (1, 3, 8)) {
            condition.components = InnoValue::string_from(&mut src, codepage)?;
        }

        if version >= (2, 0, 0) || (version.is_isx() && version >= (1, 3, 17)) {
            condition.tasks = InnoValue::string_from(&mut src, codepage)?;
        }

        if version >= (4, 0, 1) {
            condition.languages = InnoValue::string_from(&mut src, codepage)?;
        }

        if version >= (4, 0, 0) || (version.is_isx() && version >= (1, 3, 24)) {
            condition.check = InnoValue::string_from(&mut src, codepage)?;
        }

        if version >= (4, 1, 0) {
            condition.after_install = InnoValue::string_from(&mut src, codepage)?;
            condition.before_install = InnoValue::string_from(&mut src, codepage)?;
        }

        Ok(condition)
    }
}
