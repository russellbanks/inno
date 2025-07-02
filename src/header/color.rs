use std::fmt;

use zerocopy::{FromBytes, Immutable, KnownLayout, LittleEndian, U32};

#[derive(Clone, Copy, Default, Eq, PartialEq, FromBytes, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct Color(U32<LittleEndian>);

impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{:06X}", self.0)
    }
}
