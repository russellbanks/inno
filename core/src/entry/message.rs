use std::io;

use encoding_rs::Encoding;
use zerocopy::LE;

use super::Language;
use crate::{ReadBytesExt, string::PascalString};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Message {
    name: Option<String>,
    value: Option<PascalString>,
    language_index: i32,
}

impl Message {
    pub fn read<R>(
        mut reader: R,
        languages: &[Language],
        codepage: &'static Encoding,
    ) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut message = Self {
            name: reader.read_decoded_pascal_string(codepage)?,
            ..Self::default()
        };

        message.value = reader.read_pascal_string()?;

        message.language_index = reader.read_i32::<LE>()?;

        let mut codepage = codepage;
        if let Ok(index) = usize::try_from(message.language_index)
            && let Some(language) = languages.get(index)
        {
            codepage = language.codepage();
        }

        if let Some(value) = &mut message.value {
            value.decode(codepage);
        }

        Ok(message)
    }

    /// Returns the name of the message as a string slice.
    #[must_use]
    #[inline]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the value of the message as a string slice.
    #[must_use]
    #[inline]
    pub fn value(&self) -> Option<&str> {
        self.value.as_ref().map(PascalString::as_str)
    }

    /// Returns the language index of the message.
    #[must_use]
    #[inline]
    pub const fn language_index(&self) -> i32 {
        self.language_index
    }
}
