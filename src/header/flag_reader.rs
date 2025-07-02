use std::{io, ops::BitOrAssign};

use super::ReadBytesExt;

/// Represents a reader for loading a flag set where the possible flags are not known at
/// compile-time.
///
/// The flags are stored as packed bitfields, with 1 byte for every 8 flags.
/// 3-byte bitfields are padded to 4 bytes.
pub struct FlagReader<'reader, F, R> {
    reader: &'reader mut R,
    flags: F,
    /// The bit position within the current byte (0..=7)
    bit_pos: u8,
    /// Buffer for current 8-bit chunk
    current_byte: u8,
    /// Number of bytes read (used to determine if padding is needed)
    bytes_read: usize,
}

impl<'reader, F, R> FlagReader<'reader, F, R>
where
    F: BitOrAssign + Copy + Default,
    R: io::Read,
{
    pub fn new(reader: &'reader mut R) -> Self {
        Self {
            reader,
            flags: F::default(),
            bit_pos: 0,
            current_byte: 0,
            bytes_read: 0,
        }
    }

    pub fn add<I>(&mut self, flags: I) -> io::Result<()>
    where
        I: IntoIterator<Item = F>,
    {
        for flag in flags {
            if self.next_bit()? {
                self.flags |= flag;
            }
        }

        Ok(())
    }

    pub fn finalize(self) -> io::Result<F> {
        // 3-byte bitfields are padded to 4 bytes
        if self.bytes_read == 3 {
            self.reader.read_u8()?;
        }

        Ok(self.flags)
    }

    #[allow(
        clippy::cast_possible_truncation,
        reason = "u8::BITS will always fit in a u8"
    )]
    fn next_bit(&mut self) -> io::Result<bool> {
        // Check if the current bit position is on a byte boundary
        if self.bit_pos.is_multiple_of(u8::BITS as u8) {
            // Read a byte as the backing buffer for the bit flags
            self.current_byte = self.reader.read_u8()?;

            // Reset the bit position
            self.bit_pos = 0;

            self.bytes_read += 1;
        }

        let bit = (self.current_byte >> self.bit_pos) & 1 != 0;
        self.bit_pos += 1;

        Ok(bit)
    }
}

pub mod read_flags {
    macro_rules! read_flags {
        ($reader_init:expr, $(,)?) => {{
            let mut flag_reader = crate::header::flag_reader::FlagReader::new($reader_init);
            flag_reader.finalize()
        }};

        ($reader_init:expr, [$($flags:expr),+ $(,)?]) => {{
            let mut flag_reader = crate::header::flag_reader::FlagReader::new($reader_init);
            flag_reader.add([$($flags),+])?;
            flag_reader.finalize()
        }};

        ($reader_init:expr, [$($flags:expr),+ $(,)?], $($rest:tt)*) => {{
            let mut flag_reader = crate::header::flag_reader::FlagReader::new($reader_init);
            flag_reader.add([$($flags),+])?;
            crate::header::flag_reader::read_flags::read_flags_internal!(flag_reader, $($rest)*)
        }};

        ($reader_init:expr, if $cond:expr => $flag:expr) => {{
            let mut flag_reader = crate::header::flag_reader::FlagReader::new($reader_init);
            if $cond {
                flag_reader.add($flag)?;
            }
            flag_reader.finalize()
        }};

        ($reader_init:expr, if $cond:expr => $flag:expr, $($rest:tt)*) => {{
            let mut flag_reader = crate::header::flag_reader::FlagReader::new($reader_init);
            if $cond {
                flag_reader.add($flag)?;
            }
            crate::header::flag_reader::read_flags::::read_flags_internal!(flag_reader, $($rest)*)
        }};

        ($reader_init:expr, $flag:expr) => {{
            let mut flag_reader = crate::header::flag_reader::FlagReader::new($reader_init);
            flag_reader.add($flag)?;
            flag_reader.finalize()
        }};

        ($reader_init:expr, $flag:expr, $($rest:tt)*) => {{
            let mut flag_reader = crate::header::flag_reader::FlagReader::new($reader_init);
            flag_reader.add($flag)?;
            crate::header::flag_reader::read_flags::read_flags_internal!(flag_reader, $($rest)*)
        }};
    }

    macro_rules! read_flags_internal {
        ($reader:expr) => {
            $reader.finalize()
        };

        ($reader:expr, [$($flags:expr),+ $(,)?]) => {{
            $reader.add([$($flags),+])?;
            $reader.finalize()
        }};

        ($reader:expr, [$($flags:expr),+ $(,)?], $($rest:tt)*) => {{
            $reader.add([$($flags),+])?;
            crate::header::flag_reader::read_flags::read_flags_internal!($reader, $($rest)*)
        }};

        ($reader:expr, if $cond:expr => $flag:expr) => {{
            if $cond {
                $reader.add($flag)?;
            }
            $reader.finalize()
        }};

        ($reader:expr, if $cond:expr => $flag:expr, $($rest:tt)*) => {{
            if $cond {
                $reader.add($flag)?;
            }
            crate::header::flag_reader::read_flags::read_flags_internal!($reader, $($rest)*)
        }};

        ($reader:expr, $flag:expr) => {{
            $reader.add($flag)?;
            $reader.finalize()
        }};

        ($reader:expr, $flag:expr, $($rest:tt)*) => {{
            $reader.add($flag)?;
            crate::header::flag_reader::read_flags::read_flags_internal!($reader, $($rest)*)
        }};
    }

    pub(crate) use read_flags;
    pub(crate) use read_flags_internal;
}

#[cfg(test)]
mod tests {
    use bitflags::bitflags;

    use super::FlagReader;

    bitflags! {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        struct TestFlags: u8 {
            const A = 1;
            const B = 1 << 1;
            const C = 1 << 2;
            const D = 1 << 3;
            const E = 1 << 4;
            const F = 1 << 5;
            const G = 1 << 6;
            const H = 1 << 7;
        }
    }

    #[test]
    fn read_flags() {
        let mut data = &[0b1001_0101_u8][..];

        let mut reader = FlagReader::new(&mut data);

        reader.add(TestFlags::all()).unwrap();

        let flags = reader.finalize().unwrap();

        assert_eq!(
            flags,
            TestFlags::A | TestFlags::C | TestFlags::E | TestFlags::H
        );
    }
}
