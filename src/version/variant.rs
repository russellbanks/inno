use core::fmt;

use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Default, Eq, PartialEq)]
    pub struct VersionVariant: u8 {
        const UNICODE = 1;
        const ISX = 1 << 1;
    }
}

impl VersionVariant {
    /// Returns `true` if the variant has a Unicode flag.
    #[must_use]
    #[inline]
    pub const fn is_unicode(&self) -> bool {
        self.contains(Self::UNICODE)
    }

    /// Returns `true` if the variant has an ISX flag.
    #[must_use]
    #[inline]
    pub const fn is_isx(&self) -> bool {
        self.contains(Self::ISX)
    }
}

impl fmt::Debug for VersionVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            f.write_str("None")
        } else {
            bitflags::parser::to_writer(self, f)
        }
    }
}
