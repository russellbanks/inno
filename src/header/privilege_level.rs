use std::io;

use zerocopy::{Immutable, KnownLayout, TryFromBytes};

use super::HeaderFlags;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Immutable, KnownLayout, TryFromBytes)]
#[repr(u8)]
pub enum PrivilegeLevel {
    #[default]
    None = 0,
    PowerUser = 1,
    Admin = 2,
    Lowest = 3,
}

impl PrivilegeLevel {
    pub fn try_read_from_io<R>(mut src: R) -> io::Result<Self>
    where
        Self: Sized,
        R: io::Read,
    {
        let mut buf = [0; size_of::<Self>()];
        src.read_exact(&mut buf)?;
        Self::try_read_from_bytes(&buf)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
    }
}

impl From<HeaderFlags> for PrivilegeLevel {
    fn from(flags: HeaderFlags) -> Self {
        if flags.contains(HeaderFlags::ADMIN_PRIVILEGES_REQUIRED) {
            Self::Admin
        } else {
            Self::None
        }
    }
}
