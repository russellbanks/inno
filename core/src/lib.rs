/*!
A library for reading and parsing Inno Setup installers.

# Usage

This crate is [on crates.io](https://crates.io/crates/inno) and can be used by adding `inno` to your
dependencies in your project's `Cargo.toml`:

```toml
[dependencies]
inno = "0.2"
```

# Examples

## Basic usage

```no_run
use inno::Inno;
use std::fs::File;

fn main() -> Result<(), inno::error::InnoError> {
    let file = File::open("path/to/your/setup.exe")?;
    let inno = Inno::new(file)?;

    println!("Inno Setup version: {}", inno.version);
    println!("Installer languages: {:?}", inno.languages);

    Ok(())
}
*/

#![doc(html_root_url = "https://docs.rs/inno")]
#![allow(dead_code)]

mod compression;
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
use entry::{
    Component, DeleteEntry, Directory, File, Icon, Ini, Language, Message, Permission,
    RegistryEntry, RunEntry, Task, Type,
};
use error::InnoError;
use header::Header;
use itertools::Itertools;
use loader::SetupLoader;
use read::{ReadBytesExt, stream::InnoStreamReader};
use version::{InnoVersion, windows_version::WindowsVersionRange};
use wizard::Wizard;
pub use zerocopy;

use crate::{entry::FileLocation, error::HeaderStream};

const MAX_SUPPORTED_VERSION: InnoVersion = InnoVersion::new(6, 4, u8::MAX, u8::MAX);

#[derive(Debug)]
pub struct Inno {
    pub setup_loader: SetupLoader,
    pub version: InnoVersion,
    pub header: Header,
    pub languages: Vec<Language>,
    pub messages: Vec<Message>,
    pub permissions: Vec<Permission>,
    pub type_entries: Vec<Type>,
    pub components: Vec<Component>,
    pub tasks: Vec<Task>,
    pub directories: Vec<Directory>,
    pub files: Vec<File>,
    pub icons: Vec<Icon>,
    pub ini_entries: Vec<Ini>,
    pub registry_entries: Vec<RegistryEntry>,
    pub delete_entries: Vec<DeleteEntry>,
    pub uninstall_delete_entries: Vec<DeleteEntry>,
    pub run_entries: Vec<RunEntry>,
    pub uninstall_run_entries: Vec<RunEntry>,
    pub wizard: Wizard,
    pub data_entries: Vec<FileLocation>,
}

impl Inno {
    pub fn new<R>(mut reader: R) -> Result<Self, InnoError>
    where
        R: Read + Seek,
    {
        let setup_loader =
            SetupLoader::read_from(&mut reader).map_err(|_| InnoError::NotInnoFile)?;

        // Seek to Inno header
        reader.seek(SeekFrom::Start(setup_loader.header_offset().unsigned_abs()))?;

        let inno_version = InnoVersion::read(&mut reader)?;

        if inno_version > MAX_SUPPORTED_VERSION {
            return Err(InnoError::UnsupportedVersion(inno_version));
        }

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
            .map(|_| Message::read(&mut reader, &languages, codepage))
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
            header,
            languages,
            messages,
            permissions,
            type_entries,
            components,
            tasks,
            directories,
            files,
            icons,
            ini_entries,
            registry_entries,
            delete_entries,
            uninstall_delete_entries,
            run_entries,
            uninstall_run_entries,
            wizard,
            data_entries: file_locations,
        })
    }

    /// Returns the primary language of the installer, if available.
    #[must_use]
    #[inline]
    pub fn primary_language(&self) -> Option<&Language> {
        self.languages.first()
    }
}
