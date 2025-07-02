use std::io;

use encoding_rs::Encoding;
use zerocopy::LE;

use super::Language;
use crate::{ReadBytesExt, encoding::InnoValue};

#[derive(Clone, Debug, Default)]
pub struct Message {
    pub name: String,
    pub value: String,
    pub language_index: i32,
}

impl Message {
    pub fn read_from<R>(
        mut src: R,
        languages: &[Language],
        codepage: &'static Encoding,
    ) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut message = Self {
            name: InnoValue::string_from(&mut src, codepage)?.unwrap_or_default(),
            ..Self::default()
        };

        let value = InnoValue::encoded_from(&mut src)?.unwrap_or_default();

        message.language_index = src.read_i32::<LE>()?;

        let mut codepage = codepage;
        if message.language_index >= 0 {
            if let Some(language) = usize::try_from(message.language_index)
                .ok()
                .and_then(|index| languages.get(index))
            {
                codepage = language.codepage;
            }
        }

        message.value = value.into_string(codepage);

        Ok(message)
    }
}
