//! The streaming reader against real installers, which is the only place the
//! chunk, filter and checksum layers meet.

#![cfg(feature = "extract")]

use std::io::Read;

mod common;

/// Entry order is what a caller pairing these against the installer's file
/// entries relies on, so it is asserted rather than left to the sort that
/// happens to produce it.
#[test]
#[ignore = "downloads an installer; run with --ignored"]
fn entries_arrive_in_the_order_the_installer_records_them() {
    let bytes = common::download_inno_version("6.6.0").expect("download");
    let mut cursor = std::io::Cursor::new(&bytes);
    let mut inno = inno::Inno::new(&mut cursor).expect("parse");

    let mut seen = Vec::new();
    let mut files = inno.streaming_files(|_| true);
    while let Some(result) = files.next() {
        let (entry, _) = result.expect("entry");
        seen.push(entry.index());
    }

    let mut sorted = seen.clone();
    sorted.sort_unstable();
    assert_eq!(seen, sorted, "entries did not arrive in entry order");
}

/// Every entry must come out whole. The per-file checksums the installer
/// records are the only ground truth not produced by this crate, and the reader
/// validates them as it reads, so a clean read is one the installer agreed to.
#[test]
#[ignore = "downloads an installer; run with --ignored"]
fn every_entry_of_an_installer_matches_its_recorded_checksum() {
    assert_eq!(extract_every_entry("6.6.0"), 117);
}

/// Before Inno Setup 5.2 the instruction filter had no block boundary rule, so
/// an instruction can straddle one. Only carrying those bytes into the next
/// block decodes it; a scan that restarts per block fails the recorded checksum.
#[test]
#[ignore = "downloads an installer; run with --ignored"]
fn a_pre_5_2_instruction_filter_is_decoded_correctly() {
    assert_eq!(extract_every_entry("5.1.9"), 67);
}

/// Extracts every entry of the given Inno Setup release, returning how many
/// there were, and panicking if any of them failed to read.
fn extract_every_entry(version: &str) -> usize {
    let bytes = common::download_inno_version(version).expect("download");
    let mut cursor = std::io::Cursor::new(&bytes);
    let mut inno = inno::Inno::new(&mut cursor).expect("open");

    let mut extracted = 0;
    let mut errors = Vec::new();
    for result in inno.filtered_files(|_| true) {
        match result {
            Ok(_) => extracted += 1,
            Err(error) => errors.push(error.to_string()),
        }
    }

    assert!(errors.is_empty(), "{version}: {errors:#?}");

    extracted
}

/// Abandoning a reader part way must not derail the entry after it.
#[test]
#[ignore = "downloads an installer; run with --ignored"]
fn a_partially_read_entry_does_not_break_the_next_one() {
    let bytes = common::download_inno_version("6.6.0").expect("download");

    let mut expected = Vec::new();
    {
        let mut cursor = std::io::Cursor::new(&bytes);
        let mut inno = inno::Inno::new(&mut cursor).expect("open");
        for result in inno.filtered_files(|_| true) {
            expected.push(result.expect("read").1);
        }
    }
    assert!(expected.len() >= 2, "need at least two files to test this");

    let mut cursor = std::io::Cursor::new(&bytes);
    let mut inno = inno::Inno::new(&mut cursor).expect("open");
    let mut files = inno.streaming_files(|_| true);

    // Read one byte of the first entry, then drop it.
    {
        let (_entry, mut reader) = files.next().expect("first").expect("read");
        let mut one = [0u8; 1];
        let _ = reader.read(&mut one);
    }

    let (_entry, mut reader) = files.next().expect("second").expect("read");
    let mut data = Vec::new();
    reader.read_to_end(&mut data).expect("stream");
    assert_eq!(data, expected[1], "the second entry's bytes are wrong");
}

/// `len` is what the CLI's progress bar used from `ExactSizeIterator`.
#[test]
#[ignore = "downloads an installer; run with --ignored"]
fn len_counts_the_entries_that_match() {
    let bytes = common::download_inno_version("6.6.0").expect("download");
    let mut cursor = std::io::Cursor::new(&bytes);
    let mut inno = inno::Inno::new(&mut cursor).expect("open");

    let all = inno.streaming_files(|_| true).len();
    let none = inno.streaming_files(|_| false).len();

    assert!(all > 0);
    assert_eq!(none, 0);
    assert!(inno.streaming_files(|_| false).is_empty());
}

/// A location used by more than one entry cannot be streamed twice, so the
/// second entry rewinds the chunk and reads it again.
#[test]
#[ignore = "downloads an installer; run with --ignored"]
fn a_shared_location_gives_both_entries_the_same_bytes() {
    use std::collections::BTreeMap;

    let bytes = common::download_inno_version("5.1.9").expect("download");
    let mut cursor = std::io::Cursor::new(&bytes);
    let mut inno = inno::Inno::new(&mut cursor).expect("parse");

    let mut counts: BTreeMap<u32, usize> = BTreeMap::new();
    for entry in inno.file_entries() {
        *counts.entry(entry.location()).or_default() += 1;
    }
    let shared: Vec<u32> = counts
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(location, _)| *location)
        .collect();
    assert!(!shared.is_empty(), "5.1.9 has no shared location to test");

    let mut seen: BTreeMap<u32, Vec<u8>> = BTreeMap::new();
    let mut yielded = 0;
    let mut files = inno.streaming_files(|entry| shared.contains(&entry.location_index()));
    while let Some(result) = files.next() {
        let (entry, mut reader) = result.expect("read");
        let mut data = Vec::new();
        reader.read_to_end(&mut data).expect("stream");
        yielded += 1;
        if let Some(first) = seen.get(&entry.location_index()) {
            assert_eq!(first, &data, "a shared location gave two different results");
        } else {
            seen.insert(entry.location_index(), data);
        }
    }

    // Or the test passes having compared nothing.
    assert!(
        yielded > seen.len(),
        "{yielded} entries over {} locations: none was read twice",
        seen.len()
    );
}
