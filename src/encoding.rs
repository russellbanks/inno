use std::io;

use encoding_rs::Encoding;
use zerocopy::LE;

use crate::ReadBytesExt;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InnoValue(Vec<u8>);

impl InnoValue {
    pub fn raw_from<R>(mut src: R) -> io::Result<Option<Vec<u8>>>
    where
        R: io::Read,
    {
        let length = src.read_u32::<LE>()?;
        if length == 0 {
            return Ok(None);
        }
        let mut buf = vec![0; length as usize];
        src.read_exact(&mut buf)?;
        Ok(Some(buf))
    }

    pub fn encoded_from<R>(src: R) -> io::Result<Option<Self>>
    where
        R: io::Read,
    {
        Self::raw_from(src).map(|opt_raw| opt_raw.map(Self))
    }

    pub fn string_from<R>(src: R, codepage: &'static Encoding) -> io::Result<Option<String>>
    where
        R: io::Read,
    {
        Self::encoded_from(src).map(|opt_value| opt_value.map(|value| value.into_string(codepage)))
    }

    pub fn sized_string_from<R>(
        mut src: R,
        length: u32,
        codepage: &'static Encoding,
    ) -> io::Result<Option<String>>
    where
        R: io::Read,
    {
        if length == 0 {
            return Ok(None);
        }
        let mut buf = vec![0; length as usize];
        src.read_exact(&mut buf)?;
        Ok(Some(codepage.decode(&buf).0.into_owned()))
    }

    pub fn into_string(self, codepage: &'static Encoding) -> String {
        codepage.decode(&self.0).0.into_owned()
    }
}
