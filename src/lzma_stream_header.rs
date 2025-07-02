use std::io::{Error, ErrorKind, Read, Result};

use liblzma::stream::{Filters, Stream};

pub struct LzmaStreamHeader;

impl LzmaStreamHeader {
    pub fn read_from<R>(src: &mut R) -> Result<Stream>
    where
        R: Read,
    {
        let mut properties = [0; 5];
        src.read_exact(&mut properties)?;

        let mut filters = Filters::new();
        filters.lzma1_properties(&properties)?;

        Stream::new_raw_decoder(&filters).map_err(|error| Error::new(ErrorKind::InvalidData, error))
    }
}
