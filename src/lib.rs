#![allow(dead_code)]

mod compression;
mod encoding;
pub mod entry;
pub mod error;
pub mod header;
mod loader;
mod lzma_stream_header;
mod pe;
mod read;
pub mod version;
mod windows_version;
mod wizard;

use std::{
    io,
    io::{Read, Seek, SeekFrom},
};

use encoding_rs::{UTF_16LE, WINDOWS_1252};
use entry::{
    Component, Directory, File, Icon, Ini, Language, Message, Permission, Registry, Task, Type,
};
use error::InnoError;
use header::Header;
use itertools::Itertools;
use loader::SetupLoader;
use read::{ReadBytesExt, block::InnoBlockReader};
use version::InnoVersion;
use windows_version::WindowsVersionRange;
use wizard::Wizard;
pub use zerocopy;

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
    pub registries: Vec<Registry>,
}

impl Inno {
    pub fn new<R>(mut reader: R) -> Result<Self, InnoError>
    where
        R: Read + Seek,
    {
        let setup_loader = SetupLoader::read_from(&mut reader)?;

        // Seek to Inno header
        reader.seek(SeekFrom::Start(setup_loader.header_offset.into()))?;

        let inno_version = InnoVersion::read_from(&mut reader)?;

        if inno_version > MAX_SUPPORTED_VERSION {
            return Err(InnoError::UnsupportedVersion(inno_version));
        }

        let mut reader = InnoBlockReader::get(&mut reader, inno_version)?;

        let mut codepage = if inno_version.is_unicode() {
            UTF_16LE
        } else {
            WINDOWS_1252
        };

        let header = Header::read_from(&mut reader, codepage, inno_version)?;

        let languages = (0..header.language_count)
            .map(|_| Language::read_from(&mut reader, codepage, &inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        if !inno_version.is_unicode() {
            codepage = languages
                .iter()
                .map(|language| language.codepage)
                .find_or_first(|&codepage| codepage == WINDOWS_1252)
                .unwrap_or(WINDOWS_1252);
        }

        if inno_version < (4, 0, 0) {
            Wizard::read_from(&mut reader, inno_version, &header)?;
        }

        let messages = (0..header.message_count)
            .map(|_| Message::read_from(&mut reader, &languages, codepage))
            .collect::<io::Result<Vec<_>>>()?;

        let permissions = (0..header.permission_count)
            .map(|_| Permission::read_from(&mut reader, codepage))
            .collect::<io::Result<Vec<_>>>()?;

        let type_entries = (0..header.type_count)
            .map(|_| Type::read_from(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let components = (0..header.component_count)
            .map(|_| Component::read_from(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let tasks = (0..header.task_count)
            .map(|_| Task::read_from(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let directories = (0..header.directory_count)
            .map(|_| Directory::read_from(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let files = (0..header.file_count)
            .map(|_| File::read_from(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let icons = (0..header.icon_count)
            .map(|_| Icon::read_from(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let ini_entries = (0..header.ini_entry_count)
            .map(|_| Ini::read_from(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

        let registries = (0..header.registry_entry_count)
            .map(|_| Registry::read_from(&mut reader, codepage, inno_version))
            .collect::<io::Result<Vec<_>>>()?;

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
            registries,
        })
    }
}
