//! Reassembling the files in a GOG.com installer.
//!
//! GOG builds its installers so that no single entry is larger than a fixed
//! part size. Every payload file is stored in the temporary directory under a
//! content-addressed name, and the name it is meant to end up with appears
//! nowhere in the file entry itself:
//!
//! ```text
//! destination      {tmp}\43\c4\43c42d97cd4395cafc5b251378f5be5a
//! before_install   before_install('5538b1...', 'LOCO.EXE', 1)
//! after_install    after_install('5538b1...', 1533091, 3117056)
//! ```
//!
//! The id is the MD5 of the finished file. That entry is the first part; the
//! rest follow and carry no `before_install` of their own. Each part is also a
//! zlib stream in its own right, which the entry's own compression filter does
//! not mention, so reassembly means decompressing before concatenating.
//!
//! Without this an installer reads as a few thousand files with hexadecimal
//! names, none of them usable.
//!
//! Behind the `gog` feature, which implies `extract`.

use std::io::{self, Read, Seek, Write};

use flate2::read::ZlibDecoder;
use md5::{Digest, Md5 as Md5Hasher};

use crate::entry::File;
use crate::entry::checksum::{ChecksumMismatchError, Md5};
use crate::iterator::StreamingFiles;

/// A file the installer produces, and the entries it is made from.
///
/// Not every file is split: an installer mixes content-addressed parts with
/// ordinary entries carrying their own destination, and both appear here.
/// Locomotion's `Data/plugin.dat` and its saved games are the second kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallerFile {
    root: Option<String>,
    path: String,
    /// The MD5 the installer recorded, over the reassembled and decompressed
    /// contents, so it checks every step at once. `None` for a file stored
    /// whole; one reassembled from parts always has a digest.
    checksum: Option<Md5>,
    parts: Vec<usize>,
    compressed: bool,
}

impl InstallerFile {
    /// Returns the path the file is meant to be written to, relative to
    /// [`root`], with `/` separators.
    ///
    /// [`root`]: Self::root
    #[must_use]
    #[inline]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the directory constant the file is written under, without its
    /// braces: `app` for the installed program, `tmp` for the files the
    /// installer only needs while it runs, and so on.
    ///
    /// The path alone is not unique: GOG's installers write their icon under
    /// two constants, and keeping only the path merges them into one file.
    /// Files split into parts have no constant.
    #[must_use]
    #[inline]
    pub fn root(&self) -> Option<&str> {
        self.root.as_deref()
    }

    /// Returns the indices, into the installer's file entries, of the parts
    /// this file is split across, in the order they must be concatenated.
    #[must_use]
    #[inline]
    pub fn parts(&self) -> &[usize] {
        &self.parts
    }

    /// Writes the finished file to `out`, decompressing each part on the way,
    /// and checks the result against the recorded MD5.
    ///
    /// `parts` must be in the order [`parts`] lists them. Nothing can tell a
    /// swap from the right order except the digest, where it reads as
    /// corruption.
    ///
    /// Every part is held at once, so this costs the size of the file. Use
    /// [`assembly`] to feed them one at a time instead. Returns the bytes
    /// written; flushing `out` is the caller's job.
    ///
    /// [`assembly`]: Self::assembly
    ///
    /// # Errors
    ///
    /// [`io::ErrorKind::InvalidInput`] if the number of parts does not match,
    /// the decoder's error if a part is not the zlib stream it should be,
    /// [`io::ErrorKind::InvalidData`] on a checksum mismatch, and whatever
    /// `out` returns.
    ///
    /// A file that fails its checksum has already been written: the bytes
    /// reach `out` before the digest is known, so a caller that cares must be
    /// able to discard them.
    ///
    /// [`parts`]: Self::parts
    pub fn assemble_into<W: Write>(
        &self,
        parts: &[impl AsRef<[u8]>],
        out: &mut W,
    ) -> io::Result<u64> {
        let mut assembly = self.assembly();
        for part in parts {
            assembly.add(part.as_ref(), out)?;
        }
        assembly.finish()
    }

    /// Begins assembling this file one part at a time.
    ///
    /// Unlike [`assemble_into`], which takes every part at once, this holds
    /// only what the decoder needs, so the cost does not grow with the file.
    /// That is what lets a caller feed it from an iterator lending one reader
    /// at a time.
    ///
    /// [`assemble_into`]: Self::assemble_into
    #[must_use]
    pub fn assembly(&self) -> Assembly<'_> {
        Assembly {
            file: self,
            hasher: self.checksum.map(|_| Md5Hasher::new()),
            added: 0,
            written: 0,
        }
    }
}

/// One file being reassembled, part by part.
///
/// Each part is decompressed and hashed as it is written, so nothing is held
/// whole and the digest costs no second pass. [`finish`] is what checks it.
///
/// [`finish`]: Self::finish
pub struct Assembly<'file> {
    file: &'file InstallerFile,
    hasher: Option<Md5Hasher>,
    added: usize,
    written: u64,
}

impl Assembly<'_> {
    /// Writes the next part to `out`, decompressing it on the way.
    ///
    /// # Errors
    ///
    /// [`io::ErrorKind::InvalidInput`] if this is one part more than the file
    /// has, the decoder's error if the part is not the zlib stream it should
    /// be, and whatever `out` returns.
    pub fn add<R: Read, W: Write>(&mut self, part: R, out: &mut W) -> io::Result<u64> {
        if self.added == self.file.parts.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "{} is {} parts, given more",
                    self.file.path,
                    self.file.parts.len()
                ),
            ));
        }

        let mut sink = Verifying {
            inner: out,
            hasher: self.hasher.as_mut(),
        };

        let written = if self.file.compressed {
            io::copy(&mut ZlibDecoder::new(part), &mut sink)?
        } else {
            io::copy(&mut io::BufReader::new(part), &mut sink)?
        };

        self.added += 1;
        self.written += written;

        Ok(written)
    }

    /// Checks the finished file against the recorded MD5, and that every part
    /// arrived. Returns the bytes written.
    ///
    /// # Errors
    ///
    /// [`io::ErrorKind::InvalidInput`] if parts are missing, and
    /// [`io::ErrorKind::InvalidData`] on a checksum mismatch.
    pub fn finish(self) -> io::Result<u64> {
        if self.added != self.file.parts.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "{} is {} parts, given {}",
                    self.file.path,
                    self.file.parts.len(),
                    self.added
                ),
            ));
        }

        if let (Some(expected), Some(hasher)) = (self.file.checksum, self.hasher) {
            let actual = hasher.finalize().0;
            if expected != actual {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    ChecksumMismatchError::new_md5(expected.into_inner(), actual),
                ));
            }
        }

        Ok(self.written)
    }
}

/// The files a GOG.com installer produces, one reader's worth at a time.
///
/// [`files`] says what an installer is meant to produce; this reads it. The
/// parts a file is split across are pulled from the installer in order and
/// written straight out, so nothing proportional to a file or a part is held.
///
/// Two steps rather than one, because a caller needs the file's name before it
/// has anywhere to put the bytes: [`next`] says what is coming and [`write_into`]
/// writes it.
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let mut inno = inno::Inno::open(std::path::Path::new("setup.exe"))?;
/// let mut files = inno.gog_files(|_| true);
/// while let Some(file) = files.next() {
///     let path = file.path().to_owned();
///     let mut out = std::fs::File::create(path)?;
///     files.write_into(&mut out)?;
/// }
/// # Ok(()) }
/// ```
///
/// [`files`]: files
/// [`next`]: Self::next
/// [`write_into`]: Self::write_into
pub struct GogFiles<'reader, R: Read + Seek> {
    entries: StreamingFiles<'reader, R>,
    plan: Vec<InstallerFile>,
    /// How far through `plan` [`next`](Self::next) has reported.
    position: usize,
    /// Parts of the reported file still to come, so abandoning one skips them.
    remaining: usize,
}

impl<'reader, R: Read + Seek> GogFiles<'reader, R> {
    pub(crate) fn new(entries: StreamingFiles<'reader, R>, plan: Vec<InstallerFile>) -> Self {
        Self {
            entries,
            plan,
            position: 0,
            remaining: 0,
        }
    }

    /// Returns the number of files still to be reported.
    #[must_use]
    pub fn len(&self) -> usize {
        self.plan.len() - self.position
    }

    /// Returns `true` if there are no files left.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reports the next file, which [`write_into`](Self::write_into) then
    /// writes. Whatever the last one left unwritten is skipped here.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<&InstallerFile> {
        // Skipping an abandoned file, so a failure to read what it left is
        // not this caller's problem: the next `write_into` surfaces anything
        // that really broke the reader.
        while self.remaining > 0 {
            drop(self.entries.next()?);
            self.remaining -= 1;
        }

        let file = self.plan.get(self.position)?;
        self.position += 1;
        self.remaining = file.parts().len();

        Some(file)
    }

    /// Writes the file [`next`](Self::next) reported to `out`, decompressing
    /// each part on the way, and checks the result against the recorded MD5.
    /// Returns the bytes written; flushing `out` is the caller's job.
    ///
    /// A file that fails its checksum has already been written: the bytes reach
    /// `out` before the digest is known, so a caller that cares must be able to
    /// discard them.
    ///
    /// # Errors
    ///
    /// [`io::ErrorKind::NotConnected`] if no file has been reported,
    /// [`io::ErrorKind::InvalidData`] on a checksum mismatch or if the
    /// installer yields a part other than the one expected, and whatever `out`
    /// or the reader returns.
    pub fn write_into<W: Write>(&mut self, out: &mut W) -> io::Result<u64> {
        let Some(file) = self
            .position
            .checked_sub(1)
            .and_then(|at| self.plan.get(at))
        else {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "no file has been reported",
            ));
        };

        let mut assembly = file.assembly();

        for &expected in file.parts() {
            let Some(result) = self.entries.next() else {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!("{} is short of its parts", file.path),
                ));
            };

            let (entry, reader) = result.map_err(io::Error::other)?;

            // The reader hands entries back in the order the installer records
            // them, and a file's parts are consecutive in that order, so this
            // holds for anything Inno Setup built. Refused rather than
            // assembled out of order, where it would read as a bad checksum.
            if entry.index() != expected {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "{} wanted entry {expected} and the installer gave {}",
                        file.path,
                        entry.index()
                    ),
                ));
            }

            assembly.add(reader, out)?;
            self.remaining -= 1;
        }

        assembly.finish()
    }
}

/// Reads an id as an MD5, accepting either case. Anything that is not exactly
/// 32 hexadecimal characters is not a digest.
fn parse_md5(id: &str) -> Option<Md5> {
    if id.len() != 32 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    // `from_str_radix` accepts a leading sign, so `"+a"` would read as 10.
    // The hex-digit check above is what rules that out.
    let mut bytes = [0u8; 16];
    let (pairs, _) = id.as_bytes().as_chunks::<2>();
    for (byte, pair) in bytes.iter_mut().zip(pairs) {
        let text = std::str::from_utf8(pair).ok()?;
        *byte = u8::from_str_radix(text, 16).ok()?;
    }
    Some(Md5::new(bytes))
}

/// Feeds a digest with what it writes, so the check costs no second pass and
/// no second copy. Only the bytes `inner` accepted are hashed.
struct Verifying<'writer, W> {
    inner: &'writer mut W,
    hasher: Option<&'writer mut Md5Hasher>,
}

impl<W: Write> Write for Verifying<'_, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buf)?;
        if let Some(hasher) = self.hasher.as_mut() {
            hasher.update(&buf[..written]);
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Returns the files a GOG.com installer is meant to produce.
///
/// An installer that does not use GOG's scheme has no `before_install`
/// markers and produces an empty result, which is how a caller tells the two
/// apart: there is nothing to reassemble and the entries' own destinations
/// are already the answer.
#[must_use]
pub fn files(entries: &[File]) -> Vec<InstallerFile> {
    plan(entries.iter().map(|entry| Entry {
        before_install: entry.condition().before_install(),
        after_install: entry.condition().after_install(),
        destination: entry.destination(),
    }))
}

/// What [`plan`] needs to know about one file entry.
struct Entry<'a> {
    before_install: Option<&'a str>,
    after_install: Option<&'a str>,
    destination: Option<&'a str>,
}

/// The part of [`files`] that does not need a whole installer to test: each
/// entry's `before_install` script and its own destination, in order.
fn plan<'a>(entries: impl Iterator<Item = Entry<'a>>) -> Vec<InstallerFile> {
    let mut files: Vec<InstallerFile> = Vec::new();
    // Not always the last file: an ordinary entry can sit between two parts.
    let mut open: Option<usize> = None;
    let mut remaining = 0_usize;

    for (index, entry) in entries.enumerate() {
        if let Some(start) = entry.before_install.and_then(start_of_file) {
            // Begins a new file even if the last one is short of the parts it
            // asked for, so an overrunning count cannot swallow this entry.
            files.push(InstallerFile {
                root: None,
                path: start.path,
                checksum: Some(start.checksum),
                parts: vec![index],
                compressed: true,
            });
            open = Some(files.len() - 1);
            remaining = start.parts.saturating_sub(1);
            continue;
        }

        // A part says so itself; counting off whatever follows would swallow
        // an ordinary entry sitting between two parts.
        if remaining > 0
            && is_part(entry.after_install)
            && let Some(current) = open.and_then(|at| files.get_mut(at))
        {
            current.parts.push(index);
            remaining -= 1;
            continue;
        }

        // Stored and named the ordinary way. An entry with nowhere to go is
        // not a file at all.
        if let Some((root, path)) = entry.destination.map(split_root)
            && !path.is_empty()
        {
            files.push(InstallerFile {
                root,
                path,
                checksum: None,
                parts: vec![index],
                compressed: false,
            });
        }
    }

    files
}

/// Whether an entry's `after_install` marks it as one part of a file.
fn is_part(after_install: Option<&str>) -> bool {
    after_install.is_some_and(|script| {
        call_arguments(script, "after_install").is_some()
            || call_arguments(script, "after_install_dependency").is_some()
    })
}

struct Start {
    path: String,
    checksum: Md5,
    parts: usize,
}

/// Reads `before_install('<id>', '<path>', <parts>)`, and the `_dependency`
/// spelling used for the redistributables GOG ships alongside the game.
fn start_of_file(script: &str) -> Option<Start> {
    let arguments = call_arguments(script, "before_install")
        .or_else(|| call_arguments(script, "before_install_dependency"))?;

    let path = arguments.get(1)?;
    if path.is_empty() {
        return None;
    }

    // A missing or unreadable count means one part; dropping the file would
    // lose it over a detail that only matters once it is large enough to split.
    let parts = arguments
        .get(2)
        .and_then(|count| count.parse::<usize>().ok())
        .unwrap_or(1)
        .max(1);

    Some(Start {
        path: normalize(path),
        checksum: parse_md5(arguments.first()?)?,
        parts,
    })
}

/// Turns a Windows path from the installer script into the form the rest of
/// this crate uses for destinations.
fn normalize(path: &str) -> String {
    path.replace('\\', "/")
}

/// Splits `{app}\\Data\\x.dat` into its directory constant and the path
/// under it. A destination with no constant keeps its whole path and has no
/// root.
fn split_root(destination: &str) -> (Option<String>, String) {
    let Some(rest) = destination.strip_prefix('{') else {
        return (None, normalize(destination));
    };

    let Some(end) = rest.find('}') else {
        return (None, normalize(destination));
    };

    // A root names a directory a caller will join onto its own. Inno's other
    // constants -- `{code:...}`, `{%VAR|default}`, `{reg:...}` -- are not
    // names, and `..` is not one either, so the destination keeps its braces
    // and is reported as having no root rather than being joined blindly.
    let root = &rest[..end];
    if root.is_empty() || !root.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        return (None, normalize(destination));
    }

    let path = rest[end + 1..]
        .strip_prefix('\\')
        .or_else(|| rest[end + 1..].strip_prefix('/'))
        .unwrap_or(&rest[end + 1..]);

    (Some(root.to_string()), normalize(path))
}

/// Reads the arguments of `name(...)` out of a fragment of the Pascal that
/// Inno Setup scripts are written in.
///
/// Returns `None` unless the fragment is a call to exactly that function, so
/// that `before_install_dependency` is not mistaken for `before_install`.
/// Arguments are returned with their quotes removed and `''` unescaped;
/// unquoted ones, which is how the numbers appear, are returned as written.
fn call_arguments(code: &str, name: &str) -> Option<Vec<String>> {
    let code = code.trim_start();
    let rest = code.strip_prefix(name)?;
    let rest = rest.trim_start();
    let mut characters = rest.strip_prefix('(')?.chars().peekable();

    let mut arguments = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut started = false;

    while let Some(character) = characters.next() {
        match character {
            '\'' if quoted => {
                // Pascal escapes a quote by doubling it.
                if characters.peek() == Some(&'\'') {
                    characters.next();
                    current.push('\'');
                } else {
                    quoted = false;
                }
            }
            '\'' => {
                quoted = true;
                started = true;
            }
            _ if quoted => current.push(character),
            ',' => {
                arguments.push(std::mem::take(&mut current).trim().to_string());
                started = false;
            }
            ')' => {
                if started || !current.trim().is_empty() {
                    arguments.push(current.trim().to_string());
                }
                return Some(arguments);
            }
            _ => {
                current.push(character);
                started = true;
            }
        }
    }

    // Unterminated: the call was cut off, so nothing here can be trusted.
    None
}

#[cfg(test)]
mod tests {
    use md5::Digest;

    use super::{Entry, InstallerFile, Md5, call_arguments, io, plan};

    /// An entry that starts a file: its script names it.
    fn start(before_install: &str) -> Entry<'_> {
        Entry {
            before_install: Some(before_install),
            after_install: Some("after_install('id', 1, 2)"),
            destination: Some("{tmp}/ab/cd\\abcd"),
        }
    }

    /// A further part of the file before it.
    fn part<'a>() -> Entry<'a> {
        Entry {
            before_install: None,
            after_install: Some("after_install('id', 1, 2)"),
            destination: Some("{tmp}/ab/cd\\abcd"),
        }
    }

    /// An ordinary entry, stored and named the usual way.
    fn plain(destination: &str) -> Entry<'_> {
        Entry {
            before_install: None,
            after_install: None,
            destination: Some(destination),
        }
    }

    /// An entry with nothing at all on it.
    fn nothing<'a>() -> Entry<'a> {
        Entry {
            before_install: None,
            after_install: None,
            destination: None,
        }
    }

    #[test]
    fn a_call_gives_up_its_arguments() {
        let arguments =
            call_arguments("before_install('5538b1', 'LOCO.EXE', 1)", "before_install").unwrap();
        assert_eq!(arguments, ["5538b1", "LOCO.EXE", "1"]);
    }

    #[test]
    fn a_different_function_with_the_same_prefix_is_not_a_match() {
        // `before_install_dependency` starts with `before_install`, so a
        // prefix match files every redistributable under the wrong name.
        assert!(
            call_arguments(
                "before_install_dependency('14afcf', '__redist\\ISI\\x.exe', 1)",
                "before_install"
            )
            .is_none()
        );
    }

    #[test]
    fn a_quote_inside_an_argument_survives() {
        let arguments = call_arguments(
            "before_install('id', 'Sawyer''s Loco.exe', 1)",
            "before_install",
        )
        .unwrap();
        assert_eq!(arguments[1], "Sawyer's Loco.exe");
    }

    #[test]
    fn a_call_that_was_cut_off_is_not_half_read() {
        assert!(call_arguments("before_install('id', 'LOCO.EXE'", "before_install").is_none());
    }

    #[test]
    fn a_single_part_file_is_one_entry() {
        let files = plan(
            [start(
                "before_install('a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1', 'LOCO.EXE', 1)",
            )]
            .into_iter(),
        );
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path(), "LOCO.EXE");
        assert_eq!(files[0].parts(), [0]);
    }

    #[test]
    fn the_entries_after_a_split_file_are_its_remaining_parts() {
        // Locomotion's Manual.pdf is three parts, and only the first says so.
        let files = plan(
            [
                start("before_install('a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1', 'Manual.pdf', 3)"),
                part(),
                part(),
                start("before_install('b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2', 'pskill.exe', 1)"),
            ]
            .into_iter(),
        );

        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path(), "Manual.pdf");
        assert_eq!(files[0].parts(), [0, 1, 2]);
        assert_eq!(files[1].path(), "pskill.exe");
        assert_eq!(files[1].parts(), [3]);
    }

    /// A root is joined onto the caller's own directory, so only a bare
    /// constant is reported as one.
    #[test]
    fn a_root_that_could_escape_the_destination_is_not_reported_as_one() {
        for destination in [
            "{..\\..}\\x.dat",
            "{code:GetDir}\\x.dat",
            "{%HOME|/tmp}\\x.dat",
        ] {
            let files = plan([plain(destination)].into_iter());
            assert_eq!(files[0].root(), None, "{destination} named a root");
        }

        let files = plan([plain("{app}\\x.dat")].into_iter());
        assert_eq!(files[0].root(), Some("app"));
        assert_eq!(files[0].path(), "x.dat");
    }

    #[test]
    fn a_backslash_path_becomes_a_normal_one() {
        let files = plan(
            [start(
                r"before_install('a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1', 'Data\20s1.dat', 1)",
            )]
            .into_iter(),
        );
        assert_eq!(files[0].path(), "Data/20s1.dat");
    }

    #[test]
    fn entries_before_the_first_named_file_belong_to_nothing() {
        let files = plan(
            [
                nothing(),
                nothing(),
                start("before_install('a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1', 'a.txt', 1)"),
            ]
            .into_iter(),
        );
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].parts(), [2]);
    }

    #[test]
    fn an_ordinary_entry_between_two_parts_is_not_swallowed_as_one() {
        let files = plan(
            [
                start("before_install('a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1', 'split.bin', 2)"),
                plain(r"{app}\innocent.txt"),
                part(),
            ]
            .into_iter(),
        );

        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path(), "split.bin");
        assert_eq!(files[0].parts(), [0, 2], "the real second part is entry 2");
        assert_eq!(files[1].path(), "innocent.txt");
    }

    #[test]
    fn a_part_count_that_overruns_does_not_eat_the_next_file() {
        let files = plan(
            [
                start("before_install('a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1', 'greedy.bin', 9)"),
                part(),
                start("before_install('b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2', 'next.bin', 1)"),
            ]
            .into_iter(),
        );

        assert_eq!(files.len(), 2);
        assert_eq!(files[0].parts(), [0, 1]);
        assert_eq!(files[1].path(), "next.bin");
        assert_eq!(files[1].parts(), [2]);
    }

    #[test]
    fn a_part_count_of_zero_still_leaves_the_entry_it_names() {
        let files = plan(
            [
                start("before_install('a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1', 'odd.bin', 0)"),
                part(),
            ]
            .into_iter(),
        );
        assert_eq!(files[0].parts(), [0]);
    }

    #[test]
    fn the_checksum_of_the_finished_file_is_kept() {
        let files = plan(
            [start(
                "before_install('5538B198F731ABA14ABAF401DDDDF13F', 'LOCO.EXE', 1)",
            )]
            .into_iter(),
        );
        assert_eq!(
            files[0].checksum,
            Some(Md5::new([
                0x55, 0x38, 0xb1, 0x98, 0xf7, 0x31, 0xab, 0xa1, 0x4a, 0xba, 0xf4, 0x01, 0xdd, 0xdd,
                0xf1, 0x3f,
            ])),
            "written upper case by some installers"
        );
    }

    #[test]
    fn an_id_that_does_not_decode_is_not_a_file_start() {
        for id in [
            "id",
            "",
            "5538b198f731aba14abaf401ddddf13",
            "zz38b198f731aba14abaf401ddddf13f",
            "+a+a+a+a+a+a+a+a+a+a+a+a+a+a+a+a",
        ] {
            let files =
                plan([start(&format!("before_install('{id}', 'LOCO.EXE', 1)"))].into_iter());
            assert_eq!(files.len(), 1, "{id}");
            assert_eq!(files[0].path(), "ab/cd/abcd", "{id} names the destination");
            assert_eq!(files[0].root(), Some("tmp"), "{id}");
            assert_eq!(files[0].checksum, None, "{id}");
        }
    }

    #[test]
    fn an_undecodable_id_inside_an_open_split_file_is_swallowed_as_a_part() {
        // Unlike the case above: with a file open, the part-absorption branch
        // claims the entry before the destination branch runs.
        let files = plan(
            [
                start("before_install('a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1', 'big.bin', 3)"),
                Entry {
                    before_install: Some("before_install('id', 'other.exe', 1)"),
                    after_install: Some("after_install('id', 1, 2)"),
                    destination: Some(r"{app}\other.exe"),
                },
            ]
            .into_iter(),
        );

        assert_eq!(files.len(), 1, "other.exe never becomes a file of its own");
        assert_eq!(files[0].path(), "big.bin");
        assert_eq!(
            files[0].parts(),
            [0, 1],
            "entry 1's bytes fold into big.bin instead"
        );
    }

    #[test]
    fn a_file_reassembled_from_parts_always_carries_a_checksum() {
        let files = plan(
            [
                start("before_install('5538b198f731aba14abaf401ddddf13f', 'LOCO.EXE', 1)"),
                plain(r"{app}\Data\plugin.dat"),
            ]
            .into_iter(),
        );
        assert!(files[0].checksum.is_some(), "reassembled from parts");
        assert_eq!(files[1].checksum, None, "stored whole");
    }

    /// A `before_install` naming the digest of what the finished file will be,
    /// so a test that checks the contents is not fighting the verification.
    fn start_for(content: &[u8], path: &str, parts: usize) -> String {
        let digest: String = ::md5::Md5::digest(content)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        format!("before_install('{digest}', '{path}', {parts})")
    }

    /// A digest that decodes, for tests that fail before it is ever compared.
    const A_DIGEST: &str = "5538b198f731aba14abaf401ddddf13f";

    /// Assembles into a `Vec`, which is what every test here wants to inspect.
    fn assembled(file: &InstallerFile, parts: &[&[u8]]) -> io::Result<Vec<u8>> {
        let mut out = Vec::new();
        file.assemble_into(parts, &mut out)?;
        Ok(out)
    }

    /// The streaming path must agree with the all-at-once one byte for byte,
    /// or a caller's choice between them changes what it gets.
    #[test]
    fn a_file_fed_one_part_at_a_time_gives_the_same_bytes() {
        let original = b"the whole file, in two halves";
        let (first, second) = original.split_at(12);
        let files = plan([start(&start_for(original, "OUT.DAT", 2)), part()].into_iter());

        let mut streamed = Vec::new();
        let mut assembly = files[0].assembly();
        assembly.add(zlib(first).as_slice(), &mut streamed).unwrap();
        assembly
            .add(zlib(second).as_slice(), &mut streamed)
            .unwrap();
        let written = assembly.finish().unwrap();

        assert_eq!(streamed, original);
        assert_eq!(written, original.len() as u64);
        assert_eq!(
            streamed,
            assembled(&files[0], &[&zlib(first), &zlib(second)]).unwrap()
        );
    }

    #[test]
    fn an_assembly_that_is_short_of_parts_is_refused() {
        let original = b"two parts were promised";
        let files = plan([start(&start_for(original, "OUT.DAT", 2)), part()].into_iter());

        let mut out = Vec::new();
        let mut assembly = files[0].assembly();
        assembly.add(zlib(original).as_slice(), &mut out).unwrap();

        assert_eq!(
            assembly.finish().unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn an_assembly_given_more_parts_than_the_file_has_is_refused() {
        let original = b"one part only";
        let files = plan([start(&start_for(original, "OUT.DAT", 1))].into_iter());

        let mut out = Vec::new();
        let mut assembly = files[0].assembly();
        assembly.add(zlib(original).as_slice(), &mut out).unwrap();

        assert_eq!(
            assembly
                .add(zlib(original).as_slice(), &mut out)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }

    /// The bytes are already written when the digest is checked, exactly as on
    /// the streaming reader, so the caller has to be able to discard them.
    #[test]
    fn an_assembly_whose_checksum_does_not_match_fails_at_the_end() {
        let files = plan([start(&start_for(b"what the digest says", "OUT.DAT", 1))].into_iter());

        let mut out = Vec::new();
        let mut assembly = files[0].assembly();
        assembly
            .add(zlib(b"something else entirely").as_slice(), &mut out)
            .unwrap();

        assert!(
            !out.is_empty(),
            "the bytes reach the caller before the check"
        );
        assert_eq!(
            assembly.finish().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    fn zlib(bytes: &[u8]) -> Vec<u8> {
        use std::io::Write;
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn a_part_round_trips_through_the_zlib_layer() {
        let original = b"LOCO.EXE would be here";
        let files = plan([start(&start_for(original, "x.bin", 1))].into_iter());
        assert_eq!(assembled(&files[0], &[&zlib(original)]).unwrap(), original);
    }

    #[test]
    fn a_part_that_is_not_a_zlib_stream_is_an_error_not_a_panic() {
        let files = plan([start(&format!("before_install('{A_DIGEST}', 'x.bin', 1)"))].into_iter());
        assert!(assembled(&files[0], &[b"not compressed at all"]).is_err());
    }

    #[test]
    fn an_entry_that_is_not_part_of_a_split_file_is_a_file_of_its_own() {
        // Real game data in Locomotion: Data/plugin.dat and the saved games.
        let files = plan(
            [
                start("before_install('a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1', 'LOCO.EXE', 1)"),
                plain(r"{app}\Data\plugin.dat"),
            ]
            .into_iter(),
        );

        assert_eq!(files.len(), 2);
        assert_eq!(files[1].path(), "Data/plugin.dat");
        assert_eq!(files[1].parts(), [1]);
        assert_eq!(files[1].checksum, None);
    }

    #[test]
    fn a_plain_entry_is_not_decompressed_on_the_way_out() {
        // Only GOG's own parts carry the zlib layer.
        let files = plan([plain("{app}/readme.txt")].into_iter());
        assert_eq!(
            assembled(&files[0], &[b"plain bytes"]).unwrap(),
            b"plain bytes"
        );
    }

    #[test]
    fn an_entry_with_nowhere_to_go_is_not_a_file() {
        let files = plan([nothing(), plain("{app}")].into_iter());
        assert!(files.is_empty());
    }

    #[test]
    fn assembling_the_wrong_number_of_parts_is_refused() {
        let files = plan(
            [start(&format!(
                "before_install('{A_DIGEST}', 'split.bin', 3)"
            ))]
            .into_iter(),
        );
        assert!(assembled(&files[0], &[b"only one"]).is_err());
    }

    #[test]
    fn parts_are_joined_in_the_order_they_are_given() {
        let files = plan([start(&start_for(b"first second", "split.bin", 2)), part()].into_iter());
        let first = zlib(b"first ");
        let second = zlib(b"second");
        assert_eq!(
            assembled(&files[0], &[&first, &second]).unwrap(),
            b"first second"
        );
    }

    #[test]
    fn the_number_of_bytes_written_is_reported() {
        let original = b"twelve bytes";
        let files = plan([start(&start_for(original, "x.bin", 1))].into_iter());
        let mut out = Vec::new();
        assert_eq!(
            files[0]
                .assemble_into(&[&zlib(original)], &mut out)
                .unwrap(),
            12
        );
    }

    #[test]
    fn a_file_whose_checksum_does_not_match_is_refused() {
        let files = plan(
            [start(&format!(
                "before_install('{A_DIGEST}', 'LOCO.EXE', 1)"
            ))]
            .into_iter(),
        );
        let error = assembled(&files[0], &[&zlib(b"not what the digest says")]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn the_bytes_written_before_a_mismatch_are_left_where_they_were_put() {
        // Inherent to streaming, and documented: the caller discards them.
        let files = plan(
            [start(&format!(
                "before_install('{A_DIGEST}', 'LOCO.EXE', 1)"
            ))]
            .into_iter(),
        );
        let mut out = Vec::new();
        assert!(
            files[0]
                .assemble_into(&[&zlib(b"wrong")], &mut out)
                .is_err()
        );
        assert_eq!(out, b"wrong");
    }

    #[test]
    fn two_files_with_one_name_under_different_roots_stay_two_files() {
        // GOG ships its icon under both {app} and {tmp}.
        let files = plan([plain(r"{app}\goggame.ico"), plain(r"{tmp}\goggame.ico")].into_iter());

        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path(), files[1].path());
        assert_ne!(files[0].root(), files[1].root());
    }

    #[test]
    fn a_split_file_has_no_root_of_its_own() {
        // Named relative to where the installer unpacks, so there is no
        // constant to report; inventing one collides with the plain entry an
        // installer sometimes has for the same file.
        let files = plan(
            [start(
                "before_install('a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1', 'LOCO.EXE', 1)",
            )]
            .into_iter(),
        );
        assert_eq!(files[0].root(), None);
        assert_eq!(files[0].path(), "LOCO.EXE");
    }

    #[test]
    fn a_destination_with_no_constant_keeps_its_whole_path() {
        let files = plan([plain(r"plain\path.txt")].into_iter());
        assert_eq!(files[0].root(), None);
        assert_eq!(files[0].path(), "plain/path.txt");
    }

    #[test]
    fn an_installer_that_is_not_gogs_is_read_as_plain_entries() {
        // What a plain Inno Setup installer looks like.
        let files = plan([plain(r"{app}\a.txt"), plain(r"{tmp}\dir\b.txt")].into_iter());
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.parts().len() == 1));
        assert_eq!(files[1].path(), "dir/b.txt");
        assert_eq!(files[1].root(), Some("tmp"));
    }
}
