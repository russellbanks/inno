use std::{fmt, io};

use encoding_rs::Encoding;

use crate::encoding::InnoValue;

#[derive(Clone, Debug, Default)]
pub struct Permission(String);

impl Permission {
    pub fn read_from<R: io::Read>(reader: &mut R, codepage: &'static Encoding) -> io::Result<Self> {
        InnoValue::string_from(reader, codepage)
            .map(Option::unwrap_or_default)
            .map(Permission)
    }

    /// Extracts a string slice containing the entire `Permission`.
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for Permission {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
