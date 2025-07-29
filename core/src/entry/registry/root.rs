use std::fmt;

use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Immutable, KnownLayout, TryFromBytes)]
#[repr(u32)]
pub enum RegRoot {
    #[default]
    HKeyClassesRoot = 0,
    HKeyCurrentUser = 1u32.to_le(),
    HKeyLocalMachine = 2u32.to_le(),
    HKeyUsers = 3u32.to_le(),
    HKeyPerformanceData = 4u32.to_le(),
    HKeyCurrentConfig = 5u32.to_le(),
    HKeyDynamicData = 6u32.to_le(),
    Unset = 7u32.to_le(),
}

impl fmt::Display for RegRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HKeyClassesRoot => f.write_str("HKEY_CLASSES_ROOT"),
            Self::HKeyCurrentUser => f.write_str("HKEY_CURRENT_USER"),
            Self::HKeyLocalMachine => f.write_str("HKEY_LOCAL_MACHINE"),
            Self::HKeyUsers => f.write_str("HKEY_USERS"),
            Self::HKeyPerformanceData => f.write_str("HKEY_PERFORMANCE_DATA"),
            Self::HKeyCurrentConfig => f.write_str("HKEY_CURRENT_CONFIG"),
            Self::HKeyDynamicData => f.write_str("HKEY_DYNAMIC_DATA"),
            Self::Unset => f.write_str("Unset"),
        }
    }
}
