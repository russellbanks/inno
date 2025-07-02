use std::io;

use flate2::read::ZlibDecoder;
use liblzma::read::XzDecoder;

pub enum Decoder<R: io::Read> {
    Stored(R),
    Zlib(ZlibDecoder<R>),
    LZMA1(XzDecoder<R>),
}

impl<R: io::Read> io::Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stored(reader) => reader.read(buf),
            Self::Zlib(reader) => reader.read(buf),
            Self::LZMA1(reader) => reader.read(buf),
        }
    }
}
