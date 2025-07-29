use super::{Checksum, CompressionFilter};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct File {
    /// Offset of the file within the decompressed chunk.
    pub(crate) offset: u64,

    /// Pre-filter size of the file in the decompressed chunk.
    pub(crate) size: u64,

    /// Checksum for the file.
    pub(crate) checksum: Checksum,

    /// Additional filter used before compression.
    pub(crate) compression_filter: CompressionFilter,
}
