use crate::entry::CompressionFilter;

/// Apply the inverse compression filter to extracted file data.
///
/// Inno Setup applies instruction filters before compression to improve the
/// compression ratio of executables. This function reverses those transforms.
pub fn apply_filter(data: &mut [u8], filter: CompressionFilter) {
    match filter {
        CompressionFilter::NoFilter => {}
        CompressionFilter::InstructionFilter4108 => decode_4108(data),
        CompressionFilter::InstructionFilter5200 => decode_5200(data, false),
        CompressionFilter::InstructionFilter5309 => decode_5200(data, true),
        CompressionFilter::ZlibFilter => {
            // ZlibFilter is handled separately in the extraction pipeline
            // by wrapping with a ZlibDecoder before this function is called.
        }
    }
}

/// InstructionFilter4108: used for Inno Setup < 5.2.0.
///
/// Simple stateful byte-by-byte transform that reverses CALL (0xE8) and JMP (0xE9)
/// address translation.
fn decode_4108(data: &mut [u8]) {
    let mut addr: u32 = 0;
    let mut addr_bytes_left: usize = 0;
    let mut addr_offset: u32 = 5;

    for byte in data.iter_mut() {
        if addr_bytes_left == 0 {
            if *byte == 0xE8 || *byte == 0xE9 {
                addr = (!addr_offset).wrapping_add(1); // = -(addr_offset as i32) as u32
                addr_bytes_left = 4;
            }
        } else {
            addr = addr.wrapping_add(*byte as u32);
            *byte = (addr & 0xFF) as u8;
            addr >>= 8;
            addr_bytes_left -= 1;
        }
        addr_offset += 1;
    }
}

/// InstructionFilter5200 / InstructionFilter5309: used for Inno Setup >= 5.2.0.
///
/// Block-based filter (64KiB blocks). When `flip_high_byte` is true, this is
/// the 5309 variant which additionally flips the high byte for backward jumps.
fn decode_5200(data: &mut [u8], flip_high_byte: bool) {
    const BLOCK_SIZE: usize = 0x10000;

    let mut i = 0;
    while i < data.len() {
        let byte = data[i];
        i += 1;

        if byte != 0xE8 && byte != 0xE9 {
            continue;
        }

        // Check that the instruction doesn't span a block boundary
        let block_offset = (i - 1) % BLOCK_SIZE;
        let block_size_left = BLOCK_SIZE - block_offset;
        if block_size_left < 5 {
            continue;
        }

        // Need 4 bytes for the address
        if i + 4 > data.len() {
            break;
        }

        let high_byte = data[i + 3];

        // Only transform if high byte is 0x00 or 0xFF (sign-extended address)
        if high_byte != 0x00 && high_byte != 0xFF {
            i += 4;
            continue;
        }

        // Read the 24-bit value
        let mut rel = data[i] as u32 | ((data[i + 1] as u32) << 8) | ((data[i + 2] as u32) << 16);

        // Subtract the current offset (position after all 4 address bytes, low 24 bits)
        let addr = (i + 4) as u32 & 0x00FF_FFFF;
        rel = rel.wrapping_sub(addr);

        // Write back
        data[i] = rel as u8;
        data[i + 1] = (rel >> 8) as u8;
        data[i + 2] = (rel >> 16) as u8;

        // 5309 variant: flip high byte if bit 23 of result is set
        if flip_high_byte && (rel & 0x0080_0000) != 0 {
            data[i + 3] = !data[i + 3];
        }

        i += 4;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_filter_is_noop() {
        let mut data = vec![0xE8, 0x01, 0x02, 0x03, 0x04];
        let original = data.clone();
        apply_filter(&mut data, CompressionFilter::NoFilter);
        assert_eq!(data, original);
    }

    #[test]
    fn filter_4108_no_call() {
        let mut data = vec![0x90, 0x90, 0x90, 0x90]; // NOP NOP NOP NOP
        let original = data.clone();
        decode_4108(&mut data);
        assert_eq!(data, original);
    }

    #[test]
    fn filter_5200_no_transform_non_sign_extended() {
        // High byte is 0x42, not 0x00 or 0xFF -- should not transform
        let mut data = vec![0xE8, 0x10, 0x20, 0x30, 0x42];
        let original = data.clone();
        decode_5200(&mut data, false);
        assert_eq!(data, original);
    }
}
