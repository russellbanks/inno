/*!
Inno is a read-only parser for [Inno Setup](https://jrsoftware.org/isinfo.php) installers (.exe).
It reads the installer structures so you can inspect an installer without actually running it.

This crate focuses on correctness across a wide range of Inno Setup versions and provides
strongly-typed access to sections like Languages, Files, Tasks, Components, Registry entries, and
more. It does not execute any installer logic.

# Getting started

Add this to your `Cargo.toml`:

```toml
[dependencies]
inno = "0.3"
```

Then, open an installer and inspect its contents:

```no_run
use std::fs::File;
use inno::Inno;

fn main() -> Result<(), inno::error::InnoError> {
    let file = File::open("path/to/setup.exe")?;
    let inno = Inno::new(file)?;

    println!("Inno Setup version: {}", inno.version());

    if let Some(name) = inno.header.app_name() {
        println!("App name: {name}");
    }

    println!("Languages ({}):", inno.languages().len());
    for lang in inno.languages() {
        println!("- {}", lang.name());
    }

    Ok(())
}
```

## Optional Features

The following are a list of [Cargo features][cargo-features] that can be enabled or disabled:

- **chrono**: Enables converting a file's created at time to a [`DateTime<UTC>`].
- **jiff**: Enables converting a file's created at time to a [`Timestamp`].

# What this crate provides

- A high-level [`Inno`] struct representing a parsed installer, including:
  - [`Inno::version`]: the detected installer version and variant (Unicode/ANSI, ISX, etc.).
  - [`Inno::header`]: top-level metadata and configuration (app name, publisher, compression, wizard
    settings, etc.).
  - Collections for languages, messages, permissions, types, components, tasks, directories, files,
    icons, INI entries, registry entries, and run entries.
  - [`Inno::file_locations`]: low-level data/file location entries for consumers that want to
    implement extraction.
- Safe decoding of text according to the installer’s Unicode/ANSI mode and language codepage.
- Version-aware parsing that accounts for structural changes between Inno Setup releases.

This crate does not extract files by itself, but it exposes enough information
(e.g., file locations, checksums, compression type) for downstream tools to do so.
See the `innex` CLI in this repository for a reference consumer.

# Supported versions

Parsing is supported up to Inno Setup 6.6.x. Newer installers may work but can introduce format
changes. In that case, you will get [`InnoError::UnsupportedVersion`].

# Features

- LZMA/BZip2/Deflate detection through header metadata.
- Optional static LZMA linking via the `lzma-static` feature for consumers that
  need it:

  ```toml
  [dependencies]
  inno = { version = "0.2", features = ["lzma-static"] }
  ```

# Error handling

All fallible operations return [`InnoError`]. Typical errors include:

- Not an Inno installer file.
- Unsupported (too-new) installer version.
- I/O errors while reading.
- Unexpected data at the end of a header stream (corruption or truncated file).

# Notes on text decoding

In Unicode installers, text is always read as UTF-16LE. In ANSI installers,
this crate picks a codepage based on the language table, preferring
Windows-1252 when no explicit match is found, to maximize compatibility with
older installers.

# Minimum Supported Rust Version (MSRV)

This crate is tested with Rust 1.88 or newer. Newer Rust versions are generally recommended.

# Acknowledgements

- innoextract: <https://github.com/dscharrer/innoextract>
- Inno Setup: <https://jrsoftware.org/isinfo.php>

[cargo-features]: https://doc.rust-lang.org/stable/cargo/reference/manifest.html#the-features-section
[`DateTime<Utc>`]: https://docs.rs/chrono/latest/chrono/struct.DateTime.html
[`Timestamp`]: https://docs.rs/jiff/latest/jiff/struct.Timestamp.html
*/

#![doc(html_root_url = "https://docs.rs/inno")]
#![allow(dead_code)]

mod compression;
mod encryption;
pub mod entry;
pub mod error;
pub mod header;
mod loader;
mod lzma_stream_header;
mod pe;
mod read;
pub mod string;
pub mod version;
mod wizard;

use std::{
    io,
    io::{Read, Seek, SeekFrom},
};

use encoding_rs::{UTF_16LE, WINDOWS_1252};
use encryption::EncryptionHeader;
use entry::{
    Component, DeleteEntry, Directory, File, FileLocation, ISSigKey, Icon, Ini, Language, Message,
    MessageEntry, Permission, RegistryEntry, RunEntry, Task, Type,
};
use error::{HeaderStream, InnoError};
pub use header::Header;
use itertools::Itertools;
use loader::SetupLoader;
use read::{ReadBytesExt, stream::InnoStreamReader};
use version::{InnoVersion, windows_version::WindowsVersionRange};
pub use wizard::Wizard;
pub use zerocopy;

#[derive(Debug)]
pub struct Inno {
    pub setup_loader: SetupLoader,
    version: InnoVersion,
    encryption_header: Option<EncryptionHeader>,
    pub header: Header,
    languages: Vec<Language>,
    messages: Vec<MessageEntry>,
    permissions: Vec<Permission>,
    type_entries: Vec<Type>,
    components: Vec<Component>,
    tasks: Vec<Task>,
    directories: Vec<Directory>,
    is_sig_keys: Vec<ISSigKey>,
    files: Vec<File>,
    icons: Vec<Icon>,
    ini_entries: Vec<Ini>,
    registry_entries: Vec<RegistryEntry>,
    delete_entries: Vec<DeleteEntry>,
    uninstall_delete_entries: Vec<DeleteEntry>,
    run_entries: Vec<RunEntry>,
    uninstall_run_entries: Vec<RunEntry>,
    wizard: Wizard,
    file_locations: Vec<FileLocation>,
}

impl Inno {
    /// The maximum supported Inno Version by this library.
    ///
    /// Inno Setup versions newer than this version are likely to have breaking changes where the
    /// changes have not yet been implemented into this library.
    pub const MAX_SUPPORTED_VERSION: InnoVersion = InnoVersion::new(6, 6, u8::MAX, u8::MAX);

    pub fn new<R>(mut reader: R) -> Result<Self, InnoError>
    where
        R: Read + Seek,
    {
        let setup_loader =
            SetupLoader::read_from(&mut reader).map_err(|_| InnoError::NotInnoFile)?;

        // Seek to Inno header
        reader.seek(SeekFrom::Start(setup_loader.header_offset().unsigned_abs()))?;

        let inno_version = InnoVersion::read(&mut reader)?;

        if inno_version > Self::MAX_SUPPORTED_VERSION {
            return Err(InnoError::UnsupportedVersion(inno_version));
        }

        let encryption_header = if inno_version >= 6.5 {
            Some(EncryptionHeader::read(&mut reader, inno_version)?)
        } else {
            None
        };

        let mut reader = InnoStreamReader::new(&mut reader, inno_version)?;

        let mut header = Header::read(&mut reader, inno_version)?;

        let languages = (0..header.language_count())
            .map(|_| Language::read(&mut reader, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let codepage = if inno_version.is_unicode() {
            UTF_16LE
        } else {
            languages
                .iter()
                .map(Language::codepage)
                .find_or_first(|&codepage| codepage == WINDOWS_1252)
                .unwrap_or(WINDOWS_1252)
        };

        header.decode(codepage);

        let mut wizard = if inno_version < 4 {
            Wizard::read(&mut reader, &header, inno_version)?
        } else {
            Wizard::default()
        };

        let messages = (0..header.custom_message_count())
            .map(|_| MessageEntry::read(&mut reader, &languages, codepage))
            .collect::<io::Result<Vec<_>>>()?;

        let permissions = (0..header.permission_count())
            .map(|_| Permission::read(&mut reader))
            .collect::<io::Result<Vec<_>>>()?;

        let type_entries = (0..header.type_count())
            .map(|_| Type::read(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let components = (0..header.component_count())
            .map(|_| Component::read(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let tasks = (0..header.task_count())
            .map(|_| Task::read(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let directories = (0..header.directory_count())
            .map(|_| Directory::read(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let is_sig_keys = (0..header.is_sig_keys_count())
            .map(|_| ISSigKey::read(&mut reader, codepage))
            .collect::<io::Result<Vec<_>>>()?;

        let files = (0..header.file_count())
            .map(|_| File::read(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let icons = (0..header.icon_count())
            .map(|_| Icon::read(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let ini_entries = (0..header.ini_entry_count())
            .map(|_| Ini::read(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let registry_entries = (0..header.registry_entry_count())
            .map(|_| RegistryEntry::read(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let delete_entries = (0..header.install_delete_entry_count())
            .map(|_| DeleteEntry::read(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let uninstall_delete_entries = (0..header.uninstall_delete_entry_count())
            .map(|_| DeleteEntry::read(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let run_entries = (0..header.run_entry_count())
            .map(|_| RunEntry::read(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let uninstall_run_entries = (0..header.uninstall_run_entry_count())
            .map(|_| RunEntry::read(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        if inno_version >= 4 {
            wizard = Wizard::read(&mut reader, &header, inno_version)?;
        }

        // Check that the reader is at the end of the primary header stream
        if !reader.is_end_of_stream() {
            return Err(InnoError::UnexpectedExtraData(HeaderStream::Primary));
        }

        // Reset the block reader for the secondary header stream
        reader = reader.reset()?;

        let file_locations = (0..header.file_location_entry_count())
            .map(|_| FileLocation::read(&mut reader, &header, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        if !reader.is_end_of_stream() {
            return Err(InnoError::UnexpectedExtraData(HeaderStream::Secondary));
        }

        Ok(Self {
            setup_loader,
            version: inno_version,
            encryption_header,
            header,
            languages,
            messages,
            permissions,
            type_entries,
            components,
            tasks,
            directories,
            is_sig_keys,
            files,
            icons,
            ini_entries,
            registry_entries,
            delete_entries,
            uninstall_delete_entries,
            run_entries,
            uninstall_run_entries,
            wizard,
            file_locations,
        })
    }

    /// Returns the Inno Setup version.
    #[must_use]
    #[inline]
    pub const fn version(&self) -> InnoVersion {
        self.version
    }

    /// Returns the encryption header, if any.
    #[must_use]
    pub fn encryption_header(&self) -> Option<&EncryptionHeader> {
        self.encryption_header
            .as_ref()
            .or_else(|| self.header.encryption_header())
    }

    /// Returns the primary language of the installer, if available.
    #[must_use]
    #[inline]
    pub const fn primary_language(&self) -> Option<&Language> {
        self.languages().first()
    }

    /// Returns the languages as a slice.
    #[must_use]
    #[inline]
    pub const fn languages(&self) -> &[Language] {
        self.languages.as_slice()
    }

    /// Returns the message entries as a slice.
    #[must_use]
    #[inline]
    pub const fn message_entries(&self) -> &[MessageEntry] {
        self.messages.as_slice()
    }

    pub fn messages(&self) -> impl Iterator<Item = Message<'_, '_>> {
        self.messages
            .iter()
            .map(|message| Message::new(message, self.languages()))
    }

    /// Returns the permission entries as a slice.
    #[must_use]
    #[inline]
    pub const fn permissions(&self) -> &[Permission] {
        self.permissions.as_slice()
    }

    /// Returns the type entries as a slice.
    #[must_use]
    #[inline]
    pub const fn type_entries(&self) -> &[Type] {
        self.type_entries.as_slice()
    }

    /// Returns the component entries as a slice.
    #[must_use]
    #[inline]
    pub const fn components(&self) -> &[Component] {
        self.components.as_slice()
    }

    /// Returns the task entries as a slice.
    #[must_use]
    #[inline]
    pub const fn tasks(&self) -> &[Task] {
        self.tasks.as_slice()
    }

    /// Returns the directory entries as a slice.
    #[must_use]
    #[inline]
    pub const fn directories(&self) -> &[Directory] {
        self.directories.as_slice()
    }

    /// Returns the IS Sig Key entries as a slice.
    #[must_use]
    #[inline]
    pub const fn is_sig_keys(&self) -> &[ISSigKey] {
        self.is_sig_keys.as_slice()
    }

    /// Returns the file entries as a slice.
    #[must_use]
    #[inline]
    pub const fn files(&self) -> &[File] {
        self.files.as_slice()
    }

    /// Returns the icon entries as a slice.
    #[must_use]
    #[inline]
    pub const fn icons(&self) -> &[Icon] {
        self.icons.as_slice()
    }

    /// Returns the ini entries as a slice.
    #[must_use]
    #[inline]
    pub const fn ini_entries(&self) -> &[Ini] {
        self.ini_entries.as_slice()
    }

    /// Returns the registry entries a slice.
    #[must_use]
    #[inline]
    pub const fn registry_entries(&self) -> &[RegistryEntry] {
        self.registry_entries.as_slice()
    }

    /// Returns the delete entries as a slice.
    #[must_use]
    #[inline]
    pub const fn delete_entries(&self) -> &[DeleteEntry] {
        self.delete_entries.as_slice()
    }

    /// Returns the uninstall delete entries as a slice.
    #[must_use]
    #[inline]
    pub const fn uninstall_delete_entries(&self) -> &[DeleteEntry] {
        self.uninstall_delete_entries.as_slice()
    }

    /// Returns the run entries as a slice.
    #[must_use]
    #[inline]
    pub const fn run_entries(&self) -> &[RunEntry] {
        self.run_entries.as_slice()
    }

    /// Returns the uninstall run entries as a slice.
    #[must_use]
    #[inline]
    pub const fn uninstall_run_entries(&self) -> &[RunEntry] {
        self.uninstall_run_entries.as_slice()
    }

    /// Returns a reference to the [`Wizard`].
    #[must_use]
    #[inline]
    pub const fn wizard(&self) -> &Wizard {
        &self.wizard
    }

    /// Returns the file locations entries as a slice.
    #[must_use]
    #[inline]
    pub const fn file_locations(&self) -> &[FileLocation] {
        self.file_locations.as_slice()
    }
}
