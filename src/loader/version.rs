use std::{cmp::Ordering, fmt};

use super::{InnoVersion, SIGNATURE_LEN};

pub const KNOWN_SETUP_LOADER_VERSIONS: [SetupLoaderVersion; 7] = [
    SetupLoaderVersion {
        signature: *b"rDlPtS02\x87eVx",
        version: InnoVersion::new(1, 2, 10, 0),
    },
    SetupLoaderVersion {
        signature: *b"rDlPtS04\x87eVx",
        version: InnoVersion::new(4, 0, 0, 0),
    },
    SetupLoaderVersion {
        signature: *b"rDlPtS05\x87eVx",
        version: InnoVersion::new(4, 0, 3, 0),
    },
    SetupLoaderVersion {
        signature: *b"rDlPtS06\x87eVx",
        version: InnoVersion::new(4, 0, 10, 0),
    },
    SetupLoaderVersion {
        signature: *b"rDlPtS07\x87eVx",
        version: InnoVersion::new(4, 1, 6, 0),
    },
    SetupLoaderVersion {
        signature: *b"rDlPtS\xCD\xE6\xD7{\x0B*",
        version: InnoVersion::new(5, 1, 5, 0),
    },
    SetupLoaderVersion {
        signature: *b"nS5W7dT\x83\xAA\x1B\x0Fj",
        version: InnoVersion::new(5, 1, 5, 0),
    },
];

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct SetupLoaderVersion {
    pub signature: [u8; SIGNATURE_LEN],
    pub version: InnoVersion,
}

impl SetupLoaderVersion {
    fn signature(&self) -> String {
        self.signature
            .into_iter()
            .flat_map(std::ascii::escape_default)
            .map(char::from)
            .collect()
    }
}

impl PartialEq<InnoVersion> for SetupLoaderVersion {
    fn eq(&self, other: &InnoVersion) -> bool {
        self.version.eq(other)
    }
}

impl PartialEq<(u8, u8, u8)> for SetupLoaderVersion {
    fn eq(&self, other: &(u8, u8, u8)) -> bool {
        self.version.eq(other)
    }
}

impl PartialOrd<InnoVersion> for SetupLoaderVersion {
    fn partial_cmp(&self, other: &InnoVersion) -> Option<Ordering> {
        self.version.partial_cmp(other)
    }
}

impl PartialOrd<(u8, u8, u8)> for SetupLoaderVersion {
    fn partial_cmp(&self, other: &(u8, u8, u8)) -> Option<Ordering> {
        self.version.partial_cmp(other)
    }
}

impl fmt::Debug for SetupLoaderVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SetupLoaderVersion")
            .field("signature", &self.signature())
            .field("version", &self.version)
            .finish()
    }
}
