//! An installer can store one payload once and install it under several names.
//! Inno Setup's own installers do it: ISCmplr.dll and ISCmplr.dls are one
//! location under two names.

use std::{cell::Cell, collections::BTreeMap, io::Cursor};

use common::download_inno_version;

mod common;

/// Every entry the predicate accepts must come back out.
///
/// `ExtractEntry`'s ordering once compared only `(offset, location_index)`,
/// which is equal for two entries sharing a location, so the `BTreeSet` that
/// collected them treated the second as a duplicate and dropped it. 5.1.9
/// yielded 66 of its 67 accepted entries.
#[test]
#[ignore = "downloads an installer; run with --ignored"]
fn every_accepted_entry_is_yielded() {
    let bytes = download_inno_version("5.1.9").expect("download the installer");
    let mut inno = inno::Inno::new(Cursor::new(bytes)).expect("parse the installer");

    // Refuse to pass vacuously: an installer with nothing shared proves nothing.
    let mut per_location: BTreeMap<u32, usize> = BTreeMap::new();
    for entry in inno.file_entries() {
        *per_location.entry(entry.location()).or_default() += 1;
    }
    let shared = per_location.values().filter(|count| **count > 1).count();
    assert!(
        shared > 0,
        "5.1.9 shares no location, so it cannot test this"
    );

    let accepted = Cell::new(0_usize);
    let mut yielded = 0_usize;
    for result in inno.filtered_files(|_| {
        accepted.set(accepted.get() + 1);
        true
    }) {
        result.expect("read an entry");
        yielded += 1;
    }

    assert_eq!(
        yielded,
        accepted.get(),
        "{} entries were accepted and {yielded} came back",
        accepted.get()
    );
}
