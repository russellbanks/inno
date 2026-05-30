mod block;
pub mod chunk;
pub mod crc32;
#[cfg(feature = "extract")]
pub mod data_chunk;
mod decoder;
mod ext;
pub mod stream;

pub use ext::ReadBytesExt;
