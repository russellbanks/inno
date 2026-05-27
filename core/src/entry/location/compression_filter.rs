use std::{borrow::Cow, io, io::Read};

use flate2::read::ZlibDecoder;
use zerocopy::TryFromBytes;

use super::instruction::Instruction;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompressionFilter {
    #[default]
    NoFilter,
    InstructionFilter4108,
    InstructionFilter5200,
    InstructionFilter5309,
    ZlibFilter,
}

impl CompressionFilter {
    /// Returns `true` if this compression filter is [`NoFilter`].
    ///
    /// [`NoFilter`]: Self::NoFilter
    #[must_use]
    #[inline]
    pub const fn is_no_filter(&self) -> bool {
        matches!(self, Self::NoFilter)
    }

    /// Returns `true` if this compression filter is [`ZlibFilter`].
    ///
    /// [`ZlibFilter`]: Self::ZlibFilter
    #[must_use]
    #[inline]
    pub const fn is_zlib(&self) -> bool {
        matches!(self, Self::ZlibFilter)
    }

    /// Converts relative addresses in x86/x64 CALL and JMP instructions to absolute addresses for older Inno Setup versions.
    ///
    /// This modifies `data` in-place.
    fn decode_4108(data: &mut [u8]) {
        let mut position = 0;
        while position + size_of::<Instruction>() < data.len() {
            let Ok(instruction) = Instruction::try_mut_from_bytes(
                &mut data[position..position + size_of::<Instruction>()],
            ) else {
                position += 1;
                continue;
            };

            let base_address = (position + size_of::<Instruction>()) as u32;

            // Change the address to be relative to the beginning of the next instruction
            instruction.address = instruction
                .address()
                .wrapping_sub(base_address)
                .to_le_bytes();

            position += size_of::<Instruction>();
        }
    }

    /// Converts relative addresses in x86/x64 CALL and JMP instructions to
    /// absolute addresses if `ENCODE` IS `true`, or the inverse if `ENCODE` is
    /// false.
    ///
    /// This modifies `data` in-place.
    fn transform_call_instructions<const ENCODE: bool, const FLIP_HIGH_BYTE: bool>(
        data: &mut [u8],
    ) {
        // https://github.com/jrsoftware/issrc/blob/is-6_7_3/Projects/Src/Compression.Base.pas#L172

        const BLOCK_SIZE: usize = 64 * 1024; // 64KiB

        let mut position = 0;
        while position + size_of::<Instruction>() < data.len() {
            // Does it appear to be a CALL or JMP instruction with a relative 32-bit address?
            let Ok(instruction) = Instruction::try_mut_from_bytes(
                &mut data[position..position + size_of::<Instruction>()],
            ) else {
                position += 1;
                continue;
            };

            // Check that the instruction doesn't span a block boundary
            if position % BLOCK_SIZE > BLOCK_SIZE - size_of::<Instruction>() {
                position += 1;
                continue;
            }

            // If the address' sign extension is not 0x00 or 0xFF, it's not a CALL or JMP
            if !matches!(instruction.sign_extension(), u8::MIN | u8::MAX) {
                position += size_of::<Instruction>();
                continue;
            }

            // Get the base and relative address as 24-bit integers
            let base_address = (position + size_of::<Instruction>()) as u32 & 0x00FF_FFFF;
            let mut relative_address = instruction.address() & 0x00FF_FFFF;

            if !ENCODE {
                // Change the address to be relative to the beginning of the next instruction
                relative_address = relative_address.wrapping_sub(base_address);
            }

            // For a slightly higher compression ratio, Inno Setup >= 5.3.0.9
            // wants the resulting high byte to be 0x00 for both forward and
            // backward jumps. The high byte of the original relative address is
            // likely to be the sign extension of bit 23, so if bit 23 is set,
            // toggle all bits in the high byte.
            if FLIP_HIGH_BYTE && (relative_address & (1 << 23)) != 0 {
                instruction.address[3] = !instruction.address[3];
            }

            if ENCODE {
                // Change the address to be relative to the beginning of the buffer
                relative_address = relative_address.wrapping_add(base_address);
            }

            instruction.address[..3].copy_from_slice(&relative_address.to_le_bytes()[..3]);

            position += size_of::<Instruction>();
        }
    }

    /// Apply the inverse compression filter to extracted file data.
    ///
    /// Inno Setup applies instruction filters before compression to improve the
    /// compression ratio of executables. This function reverses those transforms.
    ///
    /// * If the compression filter is [`InstructionFilter4108`],
    ///   [`InstructionFilter5200`], [`InstructionFilter5309`], `data` is
    ///   modified in-place and the returned [`Cow`] is a reference to the data.
    /// * If the compression filter is [`ZlibFilter`], `data` is not modified
    ///   and the returned [`Cow`] is the owned decompressed data.
    /// * If the compression filter is [`NoFilter`], `data` is not modified
    ///   and the returned [`Cow`] is a reference to the data.
    ///
    /// [`ZlibFilter`]: Self::ZlibFilter
    pub fn decode(self, data: &mut [u8]) -> io::Result<Cow<'_, [u8]>> {
        match self {
            Self::NoFilter => {}
            Self::InstructionFilter4108 => Self::decode_4108(data),
            Self::InstructionFilter5200 => Self::transform_call_instructions::<false, false>(data),
            Self::InstructionFilter5309 => Self::transform_call_instructions::<false, true>(data),
            Self::ZlibFilter => {
                // Create a buffer that is at least the size of the compressed data
                let mut decompressed = Vec::with_capacity(data.len());
                ZlibDecoder::new(&*data).read_to_end(&mut decompressed)?;
                return Ok(Cow::Owned(decompressed));
            }
        }

        Ok(Cow::Borrowed(data))
    }
}

#[cfg(test)]
mod tests {
    use super::CompressionFilter;

    #[test]
    fn no_filter_is_noop() {
        let mut data = [0xE8, 0x01, 0x02, 0x03, 0x04];
        let original = data;

        CompressionFilter::NoFilter.decode(&mut data).unwrap();

        assert_eq!(data, original);
    }

    #[test]
    fn filter_4108_no_call() {
        /// No-operation opcode
        const NOP: u8 = 0x90;

        let mut data = [NOP; 4];
        let original = data;

        CompressionFilter::InstructionFilter4108
            .decode(&mut data)
            .unwrap();

        assert_eq!(data, original);
    }

    #[test]
    fn filter_5200_no_transform_non_sign_extended() {
        // High byte is 0x42, not 0x00 or 0xFF - should not transform
        let mut data = [0xE8, 0x10, 0x20, 0x30, 0x42];
        let original = data;

        CompressionFilter::InstructionFilter5200
            .decode(&mut data)
            .unwrap();

        assert_eq!(data, original);
    }
}
