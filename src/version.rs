use std::{cmp::Ordering, fmt, io};

use bitflags::bitflags;

use crate::error::InnoError;

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct VersionFlags: u8 {
        const UNICODE = 1;
        const ISX = 1 << 1;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq)]
pub struct InnoVersion {
    major: u8,
    minor: u8,
    patch: u8,
    revision: u8,
    variant: VersionFlags,
}

impl InnoVersion {
    const RAW_LEN: usize = 1 << 6;

    #[must_use]
    #[inline]
    pub const fn new(major: u8, minor: u8, patch: u8, revision: u8) -> Self {
        Self {
            major,
            minor,
            patch,
            revision,
            variant: VersionFlags::empty(),
        }
    }

    #[must_use]
    #[inline]
    pub const fn new_with_variant(
        major: u8,
        minor: u8,
        patch: u8,
        revision: u8,
        variant: VersionFlags,
    ) -> Self {
        Self {
            major,
            minor,
            patch,
            revision,
            variant,
        }
    }

    pub fn read_from<R>(mut src: R) -> Result<Self, InnoError>
    where
        R: io::Read,
    {
        let mut raw_version = [0; Self::RAW_LEN];

        src.read_exact(&mut raw_version)?;

        Self::from_raw_version(&raw_version).ok_or_else(|| {
            InnoError::UnknownVersion(String::from_utf8_lossy(&raw_version).into_owned())
        })
    }

    #[must_use]
    pub fn from_raw_version(mut raw_version: &[u8]) -> Option<Self> {
        const ISX: &[u8; 3] = b"ISX";
        const INNO_SETUP_EXTENSIONS: &[u8; 21] = b"Inno Setup Extensions";

        // Trim trailing null bytes
        if let Some(null_pos) = raw_version.iter().rposition(|&byte| byte != b'\0') {
            raw_version = &raw_version[..=null_pos];
        }

        // Extract the version within the parentheses
        let (version, remaining) = if let Some(start) =
            raw_version.iter().position(|&byte| byte == b'(')
            && let Some(end) = raw_version[start..].iter().position(|&byte| byte == b')')
        {
            (
                &raw_version[start + 1..start + end],
                &raw_version[start + end + 1..],
            )
        } else {
            return None;
        };

        // Split the version string into its components by a `.`
        let mut parts = version
            .split(|&byte| byte == b'.')
            .filter_map(|s| std::str::from_utf8(s).ok()?.parse::<u8>().ok());

        let inno_version = Self::new(
            parts.next()?,
            parts.next()?,
            parts.next()?,
            parts.next().unwrap_or_default(),
        );

        // Inno Setup 6.3.0 and above is always only Unicode
        if inno_version >= (6, 3, 0) {
            return Some(Self {
                variant: VersionFlags::UNICODE,
                ..inno_version
            });
        }

        let mut flags = VersionFlags::empty();

        // Check for a Unicode "(u)" flag within parentheses
        if let Some(u_start) = remaining.iter().position(|&byte| byte == b'(')
            && let Some(u_end) = remaining[u_start..].iter().position(|&byte| byte == b')')
            && remaining[u_start + 1..u_start + u_end].eq_ignore_ascii_case(b"u")
        {
            flags |= VersionFlags::UNICODE;
        }

        // Check for "ISX" or "Inno Setup Extensions"
        if remaining.windows(ISX.len()).any(|window| window == ISX)
            || remaining
                .windows(INNO_SETUP_EXTENSIONS.len())
                .any(|window| window == INNO_SETUP_EXTENSIONS)
        {
            flags |= VersionFlags::ISX;
        }

        Some(Self {
            variant: flags,
            ..inno_version
        })
    }

    #[must_use]
    #[inline]
    pub const fn is_unicode(&self) -> bool {
        self.variant.contains(VersionFlags::UNICODE)
    }

    #[must_use]
    #[inline]
    pub const fn is_isx(&self) -> bool {
        self.variant.contains(VersionFlags::ISX)
    }

    #[must_use]
    pub fn is_blackbox(&self) -> bool {
        const BLACKBOX_VERSIONS: [InnoVersion; 3] = [
            InnoVersion::new(5, 3, 10, 0),
            InnoVersion::new(5, 4, 2, 0),
            InnoVersion::new(5, 5, 0, 0),
        ];

        self.is_unicode() && BLACKBOX_VERSIONS.contains(self)
    }
}

impl fmt::Display for InnoVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major, self.minor, self.patch, self.revision
        )?;

        if self.is_unicode() {
            write!(f, " (u)")?;
        }

        Ok(())
    }
}

impl PartialEq for InnoVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.revision == other.revision
    }
}

impl PartialOrd for InnoVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InnoVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
            .then_with(|| self.revision.cmp(&other.revision))
    }
}

impl PartialEq<(u8, u8, u8, u8)> for InnoVersion {
    fn eq(&self, &(major, minor, patch, revision): &(u8, u8, u8, u8)) -> bool {
        self == &Self::new(major, minor, patch, revision)
    }
}

impl PartialEq<(u8, u8, u8)> for InnoVersion {
    fn eq(&self, &(major, minor, patch): &(u8, u8, u8)) -> bool {
        self == &Self::new(major, minor, patch, 0)
    }
}

impl PartialEq<(u8, u8)> for InnoVersion {
    fn eq(&self, &(major, minor): &(u8, u8)) -> bool {
        self == &Self::new(major, minor, 0, 0)
    }
}

impl PartialEq<u8> for InnoVersion {
    fn eq(&self, &major: &u8) -> bool {
        self == &Self::new(major, 0, 0, 0)
    }
}

impl PartialEq<InnoVersion> for (u8, u8, u8, u8) {
    fn eq(
        &self,
        &InnoVersion {
            major,
            minor,
            patch,
            revision,
            ..
        }: &InnoVersion,
    ) -> bool {
        self == &(major, minor, patch, revision)
    }
}

impl PartialEq<InnoVersion> for (u8, u8, u8) {
    fn eq(
        &self,
        &InnoVersion {
            major,
            minor,
            patch,
            revision,
            ..
        }: &InnoVersion,
    ) -> bool {
        (self.0, self.1, self.2, 0) == (major, minor, patch, revision)
    }
}

impl PartialEq<InnoVersion> for (u8, u8) {
    fn eq(
        &self,
        &InnoVersion {
            major,
            minor,
            patch,
            revision,
            ..
        }: &InnoVersion,
    ) -> bool {
        (self.0, self.1, 0, 0) == (major, minor, patch, revision)
    }
}

impl PartialEq<InnoVersion> for u8 {
    fn eq(
        &self,
        &InnoVersion {
            major,
            minor,
            patch,
            revision,
            ..
        }: &InnoVersion,
    ) -> bool {
        (*self, 0, 0, 0) == (major, minor, patch, revision)
    }
}

impl PartialOrd<(u8, u8, u8, u8)> for InnoVersion {
    fn partial_cmp(&self, &(major, minor, patch, revision): &(u8, u8, u8, u8)) -> Option<Ordering> {
        self.partial_cmp(&Self::new(major, minor, patch, revision))
    }
}

impl PartialOrd<(u8, u8, u8)> for InnoVersion {
    fn partial_cmp(&self, &(major, minor, patch): &(u8, u8, u8)) -> Option<Ordering> {
        self.partial_cmp(&Self::new(major, minor, patch, 0))
    }
}

impl PartialOrd<(u8, u8)> for InnoVersion {
    fn partial_cmp(&self, &(major, minor): &(u8, u8)) -> Option<Ordering> {
        self.partial_cmp(&Self::new(major, minor, 0, 0))
    }
}

impl PartialOrd<u8> for InnoVersion {
    fn partial_cmp(&self, &major: &u8) -> Option<Ordering> {
        self.partial_cmp(&Self::new(major, 0, 0, 0))
    }
}

impl PartialOrd<InnoVersion> for (u8, u8, u8, u8) {
    fn partial_cmp(
        &self,
        &InnoVersion {
            major,
            minor,
            patch,
            revision,
            ..
        }: &InnoVersion,
    ) -> Option<Ordering> {
        self.partial_cmp(&(major, minor, patch, revision))
    }
}

impl PartialOrd<InnoVersion> for (u8, u8, u8) {
    fn partial_cmp(
        &self,
        &InnoVersion {
            major,
            minor,
            patch,
            revision,
            ..
        }: &InnoVersion,
    ) -> Option<Ordering> {
        (self.0, self.1, self.2, 0).partial_cmp(&(major, minor, patch, revision))
    }
}

impl PartialOrd<InnoVersion> for (u8, u8) {
    fn partial_cmp(
        &self,
        &InnoVersion {
            major,
            minor,
            patch,
            revision,
            ..
        }: &InnoVersion,
    ) -> Option<Ordering> {
        (self.0, self.1, 0, 0).partial_cmp(&(major, minor, patch, revision))
    }
}

impl PartialOrd<InnoVersion> for u8 {
    fn partial_cmp(
        &self,
        &InnoVersion {
            major,
            minor,
            patch,
            revision,
            ..
        }: &InnoVersion,
    ) -> Option<Ordering> {
        (*self, 0, 0, 0).partial_cmp(&(major, minor, patch, revision))
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use rstest::rstest;

    use super::{InnoVersion, VersionFlags};

    #[rstest]
    #[case(b"", InnoVersion::new(0, 0, 0, 0))]
    #[case(b"Inno Setup Setup Data (1.3.3)", InnoVersion::new(1, 3, 3, 0))]
    #[case(
        b"Inno Setup Setup Data (1.3.12) with ISX (1.3.12.1)",
        InnoVersion::new_with_variant(1, 3, 12, 0, VersionFlags::ISX)
    )]
    #[case(
        b"Inno Setup Setup Data (3.0.3) with ISX (3.0.0)",
        InnoVersion::new_with_variant(3, 0, 3, 0, VersionFlags::ISX)
    )]
    #[case(
        b"My Inno Setup Extensions Setup Data (3.0.4)",
        InnoVersion::new(3, 0, 4, 0)
    )]
    #[case(
        b"My Inno Setup Extensions Setup Data (3.0.6.1)",
        InnoVersion::new(3, 0, 6, 1)
    )]
    #[case(b"Inno Setup Setup Data (5.3.10)", InnoVersion::new(5, 3, 10, 0))]
    #[case(
        b"Inno Setup Setup Data (5.3.10) (u)",
        InnoVersion::new_with_variant(5, 3, 10, 0, VersionFlags::UNICODE)
    )]
    #[case(
        b"Inno Setup Setup Data (5.5.7) (U)",
        InnoVersion::new_with_variant(5, 5, 7, 0, VersionFlags::UNICODE)
    )]
    #[case(b"Inno Setup Setup Data (5.6.0)", InnoVersion::new(5, 6, 0, 0))]
    #[case(
        b"Inno Setup Setup Data (5.6.0) (u)",
        InnoVersion::new_with_variant(5, 6, 0, 0, VersionFlags::UNICODE)
    )]
    #[case(
        b"Inno Setup Setup Data (6.1.0) (u)",
        InnoVersion::new_with_variant(6, 1, 0, 0, VersionFlags::UNICODE)
    )]
    #[case(
        b"Inno Setup Setup Data (6.2.0) (u)",
        InnoVersion::new_with_variant(6, 2, 0, 0, VersionFlags::UNICODE)
    )]
    #[case(
        b"Inno Setup Setup Data (6.3.0)",
        InnoVersion::new_with_variant(6, 3, 0, 0, VersionFlags::UNICODE)
    )]
    #[case(
        b"Inno Setup Setup Data (6.4.0.1)",
        InnoVersion::new_with_variant(6, 4, 0, 1, VersionFlags::UNICODE)
    )]
    fn inno_version_from_bytes(#[case] input: &[u8], #[case] expected_inno_version: InnoVersion) {
        assert_eq!(
            InnoVersion::from_raw_version(input).unwrap_or_default(),
            expected_inno_version
        );
    }

    #[test]
    fn inno_version_equality() {
        let version = InnoVersion::new(1, 2, 3, 4);
        let unicode_version = InnoVersion::new_with_variant(1, 2, 3, 4, VersionFlags::UNICODE);
        let isx_version = InnoVersion::new_with_variant(1, 2, 3, 4, VersionFlags::ISX);

        // Check that version flags aren't included in comparison
        assert_eq!(version, unicode_version);
        assert_eq!(version, isx_version);
        assert_eq!(unicode_version, isx_version);

        // Check that comparison equality returns the same as normal equality
        assert_eq!(version.cmp(&unicode_version), Ordering::Equal);
        assert_eq!(version.cmp(&isx_version), Ordering::Equal);
        assert_eq!(unicode_version.cmp(&isx_version), Ordering::Equal);
    }

    #[test]
    fn inno_version_tuple_equality() {
        // Check equality of `PartialEq<(u8, X, X, X)> for InnoVersion`
        assert_eq!(InnoVersion::new(1, 2, 3, 4), (1, 2, 3, 4));
        assert_eq!(InnoVersion::new(1, 2, 3, 0), (1, 2, 3));
        assert_eq!(InnoVersion::new(1, 2, 0, 0), (1, 2));
        assert_eq!(InnoVersion::new(1, 0, 0, 0), 1);

        // Check inequality of `PartialEq<(u8, X, X, X)> for InnoVersion`
        assert_ne!(InnoVersion::new(1, 2, 3, 4), (4, 3, 2, 1));
        assert_ne!(InnoVersion::new(1, 2, 3, 4), (1, 2, 3));
        assert_ne!(InnoVersion::new(1, 2, 3, 4), (1, 2));
        assert_ne!(InnoVersion::new(1, 2, 3, 4), 1);

        // Check that `PartialCmp<(u8, X, X, X) for InnoVersion` returns the same equality
        assert_eq!(
            InnoVersion::new(1, 2, 3, 4).partial_cmp(&(1, 2, 3, 4)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            InnoVersion::new(1, 2, 3, 0).partial_cmp(&(1, 2, 3)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            InnoVersion::new(1, 2, 0, 0).partial_cmp(&(1, 2)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            InnoVersion::new(1, 0, 0, 0).partial_cmp(&1),
            Some(Ordering::Equal)
        );

        // Check equality of `PartialEq<InnoVersion> for (u8, X, X, X)`
        assert_eq!((1, 2, 3, 4), InnoVersion::new(1, 2, 3, 4));
        assert_eq!((1, 2, 3), InnoVersion::new(1, 2, 3, 0));
        assert_eq!((1, 2), InnoVersion::new(1, 2, 0, 0));
        assert_eq!(1, InnoVersion::new(1, 0, 0, 0));

        // Check inequality of `PartialEq<InnoVersion> for (u8, X, X, X)`
        assert_ne!((1, 2, 3, 4), InnoVersion::new(4, 3, 2, 1));
        assert_ne!((1, 2, 3), InnoVersion::new(1, 2, 3, 4));
        assert_ne!((1, 2), InnoVersion::new(1, 2, 3, 4));
        assert_ne!(1, InnoVersion::new(1, 2, 3, 4));

        // Check that `PartialCmp<InnoVersion> for (u8, X, X, X)` returns the same equality
        assert_eq!(
            (1, 2, 3, 4).partial_cmp(&InnoVersion::new(1, 2, 3, 4)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            (1, 2, 3).partial_cmp(&InnoVersion::new(1, 2, 3, 0)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            (1, 2).partial_cmp(&InnoVersion::new(1, 2, 0, 0)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            1.partial_cmp(&InnoVersion::new(1, 0, 0, 0)),
            Some(Ordering::Equal)
        );
    }

    #[test]
    fn inno_version_comparison() {
        let version = InnoVersion::new(1, 2, 3, 4);

        assert!(version < InnoVersion::new(1, 2, 3, 5));
        assert!(version > InnoVersion::new(1, 2, 3, 3));

        assert!(version < (1, 2, 3, 5));
        assert!(version > (1, 2, 3, 3));

        assert!(version > (1, 2, 3));
        assert!(version < (1, 2, 4));
    }
}
