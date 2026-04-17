use std::io::{Read, Result};

pub struct LzmaStreamHeader;

impl LzmaStreamHeader {
    /// Parses the 5-byte raw LZMA1 properties header used by Inno Setup's LZMA1
    /// streams (same layout as the `.lzma` format): a single properties byte
    /// encoding lc/lp/pb, followed by a little-endian `u32` dictionary size.
    pub fn read<R>(src: &mut R) -> Result<(u8, u32)>
    where
        R: Read,
    {
        let mut properties = [0; 5];
        src.read_exact(&mut properties)?;
        let props = properties[0];
        let dict_size = u32::from_le_bytes([
            properties[1],
            properties[2],
            properties[3],
            properties[4],
        ]);
        Ok((props, dict_size))
    }
}
