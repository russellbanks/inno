mod encryption;

pub use encryption::Encryption;

use crate::header::Compression;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Chunk {
    pub(crate) first_slice: u32,
    pub(crate) last_slice: u32,
    pub(crate) sort_offset: u32,
    pub(crate) offset: u32,
    pub(crate) size: u64,
    pub(crate) compression: Compression,
    pub(crate) encryption: Encryption,
}
