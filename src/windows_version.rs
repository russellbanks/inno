use std::io;

use zerocopy::LE;

use super::InnoVersion;
use crate::ReadBytesExt;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
struct Version {
    major: u8,
    minor: u8,
    build: u16,
}

impl Version {
    fn read_from<R>(src: &mut R, inno_version: InnoVersion) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut version = Self::default();
        if inno_version >= (1, 3, 19) {
            version.build = src.read_u16::<LE>()?;
        }
        version.minor = src.read_u8()?;
        version.major = src.read_u8()?;
        Ok(version)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
struct ServicePack {
    major: u8,
    minor: u8,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
struct WindowsVersion {
    pub win_version: Version,
    pub nt_version: Version,
    pub nt_service_pack: ServicePack,
}

impl WindowsVersion {
    pub fn read_from<R>(src: &mut R, version: InnoVersion) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut windows_version = Self {
            win_version: Version::read_from(src, version)?,
            nt_version: Version::read_from(src, version)?,
            ..Self::default()
        };

        if version >= (1, 3, 19) {
            windows_version.nt_service_pack.minor = src.read_u8()?;
            windows_version.nt_service_pack.major = src.read_u8()?;
        }

        Ok(windows_version)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct WindowsVersionRange {
    begin: WindowsVersion,
    end: WindowsVersion,
}

impl WindowsVersionRange {
    pub fn read_from<R>(src: &mut R, version: InnoVersion) -> io::Result<Self>
    where
        R: io::Read,
    {
        Ok(Self {
            begin: WindowsVersion::read_from(src, version)?,
            end: WindowsVersion::read_from(src, version)?,
        })
    }
}
