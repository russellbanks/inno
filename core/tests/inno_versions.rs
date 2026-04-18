use std::{error::Error, io::Cursor};

use bytes::Bytes;
use inno::{Inno, version::InnoVersion};
use reqwest::blocking;
use rstest::rstest;
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
fn download_inno_version(version: &str) -> reqwest::Result<Bytes> {
    let semver = Version::parse(version).unwrap();
    let Version {
        major,
        minor,
        patch,
        ..
    } = semver;

    let url = if major >= 6 {
        format!(
            "https://github.com/jrsoftware/issrc/releases/download/is-{major}_{minor}_{patch}{new}/innosetup-{version}.exe",
            new = if semver == Version::new(6, 0, 3) || semver == Version::new(6, 0, 4) {
                "-2"
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

#[rstest]
#[ignore]
fn inno_versions(
    #[values(
        "5.0.0-beta",
        "5.0.1-beta",
        "5.0.2-beta",
        "5.0.3-beta",
        "5.0.4-beta",
        "5.0.5-beta",
        "5.0.6",
        "5.0.7",
        "5.0.8",
        "5.1.0-beta",
        "5.1.1-beta",
        "5.1.2-beta",
        "5.1.3-beta",
        "5.1.4",
        "5.1.5",
        "5.1.6",
        "5.1.7",
        "5.1.8",
        "5.1.9",
        "5.2.0",
        "5.2.1",
        "5.2.2",
        "5.2.3",
        "5.3.0-beta",
        "5.3.1-beta",
        "5.3.2-beta",
        "5.3.3",
        "5.3.4",
        "5.3.5",
        "5.3.6",
        "5.3.7",
        "5.3.8",
        "5.3.9",
        "5.3.10",
        "5.3.11",
        "5.4.0",
        "5.4.1",
        "5.4.2",
        "5.4.3",
        "5.5.0",
        "5.5.1",
        "5.5.2",
        "5.5.3",
        "5.5.4",
        "5.5.5",
        "5.5.6",
        "5.5.6-unicode",
        "5.5.7",
        "5.5.7-unicode",
        "5.5.8",
        "5.5.8-unicode",
        "6.0.0-beta",
        "6.0.1-beta",
        "6.0.2",
        "6.0.3",
        "6.0.4",
        "6.0.5",
        "6.1.0-beta",
        "6.1.1-beta",
        "6.1.2",
        "6.2.0",
        "6.2.1",
        "6.2.2",
        "6.3.0",
        "6.3.1",
        "6.3.2",
        "6.3.3",
        "6.4.0",
        "6.4.1",
        "6.4.2",
        "6.4.3",
        "6.5.0",
        "6.5.1",
        "6.5.2",
        "6.5.3",
        "6.5.4"
    )]
    version: &str,
) -> Result<(), Box<dyn Error>> {
    let inno_bytes = download_inno_version(version)?;

    Inno::new(Cursor::new(inno_bytes))?;

    Ok(())
}

#[test]
#[ignore]
fn inno_6_6_0() -> Result<(), Box<dyn Error>> {
    let inno_bytes = download_inno_version("6.6.0")?;

    let inno = Inno::new(Cursor::new(inno_bytes))?;

    assert_eq!(inno.version(), InnoVersion::new(6, 6, 0, 0));
    assert!(inno.version().is_unicode());

    assert_eq!(inno.header.wizard_image_opacity(), None);

    assert!(!inno.wizard().images_dynamic_dark().is_empty());
    assert!(!inno.wizard().small_images_dynamic_dark().is_empty());

    Ok(())
}

#[test]
#[ignore]
fn inno_6_6_1() -> Result<(), Box<dyn Error>> {
    let inno_bytes = download_inno_version("6.6.1")?;

    let inno = Inno::new(Cursor::new(inno_bytes))?;

    assert_eq!(inno.version(), InnoVersion::new(6, 6, 1, 0));
    assert!(inno.version().is_unicode());

    assert_eq!(inno.header.wizard_image_opacity(), Some(u8::MAX));

    Ok(())
}

#[test]
#[ignore]
fn inno_6_7_0() -> Result<(), Box<dyn Error>> {
    let inno_bytes = download_inno_version("6.7.0")?;
    let inno = Inno::new(Cursor::new(inno_bytes))?;

    assert_eq!(inno.version(), InnoVersion::new(6, 7, 0, 0));
    assert!(inno.version().is_unicode());

    assert_eq!(
        inno.header.use_previous_app_dir(),
        Some("not PortableCheck")
    );
    assert_eq!(inno.header.use_previous_group(), Some("yes"));
    assert_eq!(inno.header.use_previous_setup_type(), Some("yes"));
    assert_eq!(inno.header.use_previous_tasks(), Some("yes"));
    assert_eq!(inno.header.use_previous_user_info(), Some("yes"));

    Ok(())
}

#[test]
#[ignore]
fn inno_6_7_1() -> Result<(), Box<dyn Error>> {
    let inno_bytes = download_inno_version("6.7.1")?;
    let inno = Inno::new(Cursor::new(inno_bytes))?;

    assert_eq!(inno.version(), InnoVersion::new(6, 7, 0, 0));
    assert!(inno.version().is_unicode());

    // 6.7.0 parses without these, but 6.7.1 fails
    assert!(inno.wizard().back_images().is_empty());
    assert!(inno.wizard().back_images_dynamic_dark().is_empty());

    Ok(())
}
