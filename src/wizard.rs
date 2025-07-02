use std::io::{Read, Result};

use zerocopy::LE;

use crate::{
    encoding::InnoValue,
    header::{Compression, Header, HeaderFlags},
    read::ReadBytesExt,
    version::InnoVersion,
};

#[derive(Debug, Default)]
pub struct Wizard {
    images: Vec<Vec<u8>>,
    small_images: Vec<Vec<u8>>,
    decompressor_dll: Option<Vec<u8>>,
    decrypt_dll: Option<Vec<u8>>,
}

impl Wizard {
    pub fn read_from<R: Read>(mut src: R, version: InnoVersion, header: &Header) -> Result<Self> {
        let mut wizard = Self {
            images: Self::read_images(&mut src, version)?,
            ..Self::default()
        };

        if version >= (2, 0, 0) || version.is_isx() {
            wizard.small_images = Self::read_images(&mut src, version)?;
        }

        if header.compression == Compression::BZip2
            || (header.compression == Compression::LZMA1 && version == (4, 1, 5))
            || (header.compression == Compression::Zlib && version >= (4, 2, 6))
        {
            wizard.decompressor_dll = InnoValue::raw_from(&mut src)?;
        }

        if header.flags.contains(HeaderFlags::ENCRYPTION_USED) {
            wizard.decrypt_dll = InnoValue::raw_from(&mut src)?;
        }

        Ok(wizard)
    }

    fn read_images<R: Read>(mut reader: R, version: InnoVersion) -> Result<Vec<Vec<u8>>> {
        let count = if version >= (5, 6, 0) {
            reader.read_u32::<LE>()?
        } else {
            1
        };

        let mut images = (0..count)
            .filter_map(|_| InnoValue::raw_from(&mut reader).transpose())
            .collect::<Result<Vec<_>>>()?;

        if version < (5, 6, 0) && images.first().is_some_and(Vec::is_empty) {
            images.clear();
        }

        Ok(images)
    }
}
