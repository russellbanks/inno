use std::io;

use encoding_rs::Encoding;

use crate::{read::ReadBytesExt, version::InnoVersion};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Condition {
    components: Option<String>,
    tasks: Option<String>,
    languages: Option<String>,
    check: Option<String>,
    after_install: Option<String>,
    before_install: Option<String>,
}

impl Condition {
    pub fn read<R>(
        mut reader: R,
        codepage: &'static Encoding,
        version: InnoVersion,
    ) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut condition = Self::default();

        if version >= 2 || (version.is_isx() && version >= (1, 3, 8)) {
            condition.components = reader.read_decoded_pascal_string(codepage)?;
        }

        if version >= 2 || (version.is_isx() && version >= (1, 3, 17)) {
            condition.tasks = reader.read_decoded_pascal_string(codepage)?;
        }

        if version >= (4, 0, 1) {
            condition.languages = reader.read_decoded_pascal_string(codepage)?;
        }

        if version >= 4 || (version.is_isx() && version >= (1, 3, 24)) {
            condition.check = reader.read_decoded_pascal_string(codepage)?;
        }

        if version >= 4.1 {
            condition.after_install = reader.read_decoded_pascal_string(codepage)?;
            condition.before_install = reader.read_decoded_pascal_string(codepage)?;
        }

        Ok(condition)
    }

    /// Returns the components as a string slice.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        self.components.as_deref()
    }

    /// Returns the tasks as a string slice.
    #[must_use]
    #[inline]
    pub fn tasks(&self) -> Option<&str> {
        self.tasks.as_deref()
    }

    /// Returns the languages as a string slice.
    #[must_use]
    #[inline]
    pub fn languages(&self) -> Option<&str> {
        self.languages.as_deref()
    }

    /// Returns the check condition as a string slice.
    #[must_use]
    #[inline]
    pub fn check(&self) -> Option<&str> {
        self.check.as_deref()
    }

    /// Returns the after install condition as a string slice.
    #[must_use]
    #[inline]
    pub fn after_install(&self) -> Option<&str> {
        self.after_install.as_deref()
    }

    /// Returns the before install condition as a string slice.
    #[must_use]
    #[inline]
    pub fn before_install(&self) -> Option<&str> {
        self.before_install.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use encoding_rs::UTF_8;

    use super::Condition;
    use crate::version::InnoVersion;

    /// A pascal string as the reader expects it: a little-endian length,
    /// then the bytes.
    fn pascal(value: &str) -> Vec<u8> {
        let mut out = u32::try_from(value.len()).unwrap().to_le_bytes().to_vec();
        out.extend_from_slice(value.as_bytes());
        out
    }

    #[test]
    fn the_six_strings_land_in_the_order_the_format_writes_them() {
        // after_install really does come before before_install on the wire.
        // Swapping them still parses and still produces two plausible
        // strings, so the mistake would be a quiet one.
        let mut data = Vec::new();
        for field in [
            "components",
            "tasks",
            "languages",
            "check",
            "after",
            "before",
        ] {
            data.extend_from_slice(&pascal(field));
        }

        let condition =
            Condition::read(data.as_slice(), UTF_8, InnoVersion::new(5, 6, 2, 0)).unwrap();

        assert_eq!(condition.as_str(), Some("components"));
        assert_eq!(condition.tasks(), Some("tasks"));
        assert_eq!(condition.languages(), Some("languages"));
        assert_eq!(condition.check(), Some("check"));
        assert_eq!(condition.after_install(), Some("after"));
        assert_eq!(condition.before_install(), Some("before"));
    }
}
