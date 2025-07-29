use core::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Checksum {
    Adler32(u32),
    Crc32(u32),
    Md5([u8; 16]),
    Sha1([u8; 20]),
    Sha256([u8; 32]),
    Check([u8; 4]),
}

struct Md5<'a>(&'a [u8; 16]);

impl fmt::Debug for Md5<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Md5<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.0 {
            write!(f, "{byte:02X}")?;
        }
        Ok(())
    }
}

struct Sha1<'a>(&'a [u8; 20]);

impl fmt::Debug for Sha1<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Sha1<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.0 {
            write!(f, "{byte:02X}")?;
        }
        Ok(())
    }
}

struct Sha256<'a>(&'a [u8; 32]);

impl fmt::Debug for Sha256<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Sha256<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.0 {
            write!(f, "{byte:02X}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adler32(adler32) => f.debug_tuple("Adler32").field(adler32).finish(),
            Self::Crc32(crc32) => f.debug_tuple("Crc32").field(crc32).finish(),
            Self::Md5(md5) => f.debug_tuple("MD5").field(&Md5(md5)).finish(),
            Self::Sha1(sha1) => f.debug_tuple("SHA1").field(&Sha1(sha1)).finish(),
            Self::Sha256(sha256) => f.debug_tuple("SHA256").field(&Sha256(sha256)).finish(),
            Self::Check(check) => f.debug_tuple("Check").field(check).finish(),
        }
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adler32(adler32) => write!(f, "{adler32}"),
            Self::Crc32(crc32) => write!(f, "{crc32}"),
            Self::Md5(md5) => write!(f, "{}", Md5(md5)),
            Self::Sha1(sha1) => write!(f, "{}", Sha1(sha1)),
            Self::Sha256(sha256) => write!(f, "{}", Sha256(sha256)),
            Self::Check(check) => write!(f, "{:?}", u32::from_le_bytes(*check)),
        }
    }
}

impl Default for Checksum {
    fn default() -> Self {
        Self::Adler32(0)
    }
}
