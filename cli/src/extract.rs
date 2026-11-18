use std::{
    ffi::OsString,
    fs::{self, File},
    io::{self, BufWriter, Read, Seek, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::Context;
use indicatif::{ProgressBar, ProgressStyle};
use inno::{
    Inno,
    path::{ByName, PathError, RelativePath},
};
use regex::Regex;

/// Extracts the installer's files into `destination`, or only those whose extracted path, such as
/// `app/bin/foo.dll`, matches `filter` if one is given. Each constant in a destination becomes a
/// directory of its own, so files installed to `{app}` and `{sys}` cannot collide.
///
/// Each file is streamed to disk rather than held in memory, and is written under a temporary
/// name first, so a file that fails its checksum is not left behind half-written.
///
/// # Errors
///
/// Returns an error if a file cannot be read from the installer, fails its checksum, or cannot be
/// written.
pub fn run<R: Read + Seek>(
    inno: &mut Inno<R>,
    destination: &Path,
    filter: Option<&Regex>,
) -> anyhow::Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("creating {}", destination.display()))?;

    // The filter matches the path the file is extracted to, such as `app/bin/foo.dll`.
    let mut files = inno.streaming_files(|entry| {
        filter.is_none_or(|pattern| {
            entry
                .file()
                .destination()
                .and_then(|file_destination| file_destination.to_relative_path(ByName).ok())
                .is_some_and(|path| pattern.is_match(&path.to_string()))
        })
    });

    let progress = ProgressBar::new(files.len() as u64)
        .with_style(
            ProgressStyle::default_bar()
                .template("{spinner:.cyan} [{bar:40.green/black}] {pos}/{len} {msg}")?
                .progress_chars("───"),
        )
        .with_message(format!("Extracting files to {}", destination.display()));

    while let Some(file) = files.next() {
        let (entry, mut reader) = file?;

        if let Some(relative_path) = output_path(entry.file(), &progress) {
            let path = relative_path.join_onto(destination);
            progress.set_message(relative_path.to_string());
            write_file(&path, &mut reader, entry.file_location().file_time().into())
                .with_context(|| format!("extracting {}", path.display()))?;
        }

        progress.inc(1);
    }

    progress.finish();
    Ok(())
}

/// Returns where a file is extracted to, relative to the output directory, or `None` to skip it.
/// A file whose destination is not a safe relative path is skipped with a message saying why.
fn output_path(file: &inno::entry::File, progress: &ProgressBar) -> Option<RelativePath> {
    let file_destination = file.destination()?;
    match file_destination.to_relative_path(ByName) {
        Ok(relative_path) => Some(relative_path),
        Err(PathError::Empty) => None,
        Err(error) => {
            progress.println(format!("Skipping {file_destination}: {error}"));
            None
        }
    }
}

/// Streams `reader` into the file at `path`, creating its directory if needed, and sets its
/// modified time.
///
/// The data is written to a `.part` file next to `path` and renamed into place only once it has
/// all been read, so an error partway through, such as a checksum mismatch, leaves no file at
/// `path`. The `.part` file is removed on error.
fn write_file(path: &Path, reader: &mut impl Read, modified: SystemTime) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let part = part_path(path);
    let written = File::create(&part).and_then(|file| {
        let mut writer = BufWriter::new(file);
        io::copy(reader, &mut writer)?;
        writer.flush()?;
        let file = writer
            .into_inner()
            .map_err(io::IntoInnerError::into_error)?;
        file.set_modified(modified)
    });

    match written {
        Ok(()) => fs::rename(&part, path),
        Err(error) => {
            let _ = fs::remove_file(&part);
            Err(error)
        }
    }
}

/// Returns `path` with `.part` appended to its file name.
fn part_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().map(OsString::from).unwrap_or_default();
    name.push(".part");
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{self, Read},
        path::{Path, PathBuf},
        time::{Duration, SystemTime},
    };

    use super::{part_path, write_file};

    /// A fresh, empty directory for one test.
    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("innex-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Reads `data`, then fails, as a file that fails its checksum does.
    struct FailingReader<'a> {
        data: &'a [u8],
    }

    impl Read for FailingReader<'_> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.data.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "checksum mismatch",
                ));
            }
            let read = self.data.read(buf)?;
            Ok(read)
        }
    }

    fn files_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn writes_file_and_sets_modified_time() {
        let dir = test_dir("write");
        let path = dir.join("sub").join("file.txt");
        let modified = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);

        write_file(&path, &mut &b"hello"[..], modified).unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"hello");
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
        assert_eq!(files_in(&dir.join("sub")), ["file.txt"]);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failed_read_leaves_nothing_behind() {
        let dir = test_dir("fail");
        let path = dir.join("file.txt");

        let error = write_file(
            &path,
            &mut FailingReader { data: b"partial" },
            SystemTime::now(),
        )
        .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(files_in(&dir).is_empty());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn replaces_an_existing_file() {
        let dir = test_dir("replace");
        let path = dir.join("file.txt");
        fs::write(&path, b"old contents").unwrap();

        write_file(&path, &mut &b"new"[..], SystemTime::now()).unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"new");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn part_path_appends_to_file_name() {
        assert_eq!(
            part_path(Path::new("out/app/foo.dll")),
            Path::new("out/app/foo.dll.part")
        );
    }
}
