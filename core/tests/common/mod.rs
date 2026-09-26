use bytes::Bytes;
use reqwest::blocking;
use semver::Version;

/// Downloads the specified Inno Setup into memory, returning its bytes.
///
/// # Errors
///
/// Returns a [`reqwest::Error`] if the request fails or the server returns a non-success status
/// code.
///
/// # Panics
///
/// Panics if `version` is not a valid semantic version.
pub fn download_inno_version(version: &str) -> reqwest::Result<Bytes> {
    let semver = Version::parse(version).unwrap();
    let Version {
        major,
        minor,
        patch,
        ref pre,
        ..
    } = semver;

    let url = if major >= 6 {
        format!(
            "https://github.com/jrsoftware/issrc/releases/download/is-{major}_{minor}_{patch}{new}/innosetup-{version}.exe",
            new = if semver == Version::new(6, 0, 3) || semver == Version::new(6, 0, 4) {
                "-2"
            } else if major == 7 && minor == 0 && patch == 0 && pre.starts_with("preview") {
                if pre.starts_with("preview-1") {
                    "_0"
                } else if pre.starts_with("preview-2") {
                    "_1"
                } else if pre.starts_with("preview-3") {
                    "_2"
                } else {
                    ""
                }
            } else {
                ""
            }
        )
    } else {
        format!(
            "https://files.jrsoftware.org/is/{major}/{name}-{version}.exe",
            major = semver.major,
            name = if semver < Version::new(5, 5, 9) {
                "isetup"
            } else {
                "innosetup"
            },
        )
    };

    blocking::get(url)?.error_for_status()?.bytes()
}
