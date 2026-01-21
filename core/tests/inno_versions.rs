use std::{error::Error, io::Cursor};

use bytes::Bytes;
use inno::{Inno, version::InnoVersion};
use reqwest::blocking;
use rstest::rstest;

/// Downloads the specified Inno Setup into memory, returning its bytes.
fn download_inno_version(version: &str) -> reqwest::Result<Bytes> {
    let url = format!(
        "https://files.jrsoftware.org/is/{major}/{name}-{version}.exe",
        major = version.chars().next().unwrap(),
        name = if version < "5.5.9" || version == "6.3.3" {
            "isetup"
        } else {
            "innosetup"
        },
        version = version
    );

    blocking::get(url)?.error_for_status()?.bytes()
}

#[rstest]
#[ignore]
fn inno_versions(
    #[values(
        // "5.0.0-beta",
        // "5.0.1-beta",
        // "5.0.2-beta",
        // "5.0.3-beta",
        // "5.0.4-beta",
        // "5.0.5-beta",
        // "5.0.6",
        // "5.0.7",
        // "5.0.8",
        // "5.1.0-beta",
        // "5.1.1-beta",
        // "5.1.2-beta",
        // "5.1.3-beta",
        // "5.1.4",
        // "5.1.5",
        // "5.1.6",
        // "5.1.7",
        // "5.1.8",
        // "5.1.9",
        // "5.2.0",
        // "5.2.1",
        // "5.2.2",
        // "5.2.3",
        // "5.3.0-beta",
        // "5.3.1-beta",
        // "5.3.2-beta",
        // "5.3.3",
        // "5.3.4",
        // "5.3.5",
        // "5.3.6",
        // "5.3.7",
        // "5.3.8",
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
        "6.5.4",
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
