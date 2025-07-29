use std::fmt;

use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Default, Eq, PartialEq)]
    pub struct DataEntryFlags: u16 {
        const VERSION_INFO_VALID = 1;
        const VERSION_INFO_NOT_VALID = 1 << 1;
        const TIMESTAMP_IN_UTC = 1 << 2;
        const IS_UNINSTALLER_EXE = 1 << 3;
        const CALL_INSTRUCTION_OPTIMIZED = 1 << 4;
        const TOUCH = 1 << 5;
        const CHUNK_ENCRYPTED = 1 << 6;
        const CHUNK_COMPRESSED = 1 << 7;
        const SOLID_BREAK = 1 << 8;
        const SIGN = 1 << 9;
        const SIGN_ONCE = 1 << 10;

        // ~~~ Obsolete flags~~~

        const BZIPPED = 1 << 15;
    }
}

impl fmt::Debug for DataEntryFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            f.write_str("None")
        } else {
            bitflags::parser::to_writer(self, f)
        }
    }
}
