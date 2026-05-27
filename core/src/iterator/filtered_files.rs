use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    io,
    io::{Read, Seek},
};

use crate::{
    Inno,
    error::{InnoError, InnoResult},
    iterator::ExtractEntry,
    read::{chunk::Chunk, data_chunk::DataChunkReader},
};

enum FilesReader<'reader, R: Read + Seek> {
    Source(Option<&'reader mut R>),
    Chunk(Option<DataChunkReader<&'reader mut R>>),
}

impl<R: Read + Seek> FilesReader<'_, R> {
    pub fn to_source_mut(&mut self) -> &mut Self {
        if let Self::Chunk(reader) = self
            && let Some(reader) = reader.take()
        {
            let reader = reader.into_inner().into_inner();
            *self = FilesReader::Source(Some(reader));
        }

        self
    }

    pub fn to_chunk_mut(&mut self, data_offset: u64, chunk: &Chunk) -> InnoResult<&mut Self> {
        if let Self::Source(reader) = self
            && let Some(reader) = reader.take()
        {
            let chunk_reader = DataChunkReader::new(reader, data_offset, chunk)?;
            *self = FilesReader::Chunk(Some(chunk_reader));
        }

        Ok(self)
    }

    pub fn reinitialize(&mut self, data_offset: u64, chunk: &Chunk) -> InnoResult<&mut Self> {
        self.to_source_mut();
        self.to_chunk_mut(data_offset, chunk)
    }
}

impl<R: Read + Seek> Read for FilesReader<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Source(Some(reader)) => reader.read(buf),
            Self::Chunk(Some(reader)) => reader.read(buf),
            _ => unreachable!(),
        }
    }
}

pub struct FilteredFilesIterator<'reader, R: Read + Seek> {
    reader: FilesReader<'reader, R>,
    data_offset: u64,
    chunks: BTreeMap<u64, BTreeSet<ExtractEntry>>,
    entries: BTreeSet<ExtractEntry>,
    current_position: u64,
    previous_location_index: Option<u32>,
    data: Vec<u8>,
}

impl<'reader, R: Read + Seek> FilteredFilesIterator<'reader, R> {
    pub fn new<P>(inno: &'reader mut Inno<R>, mut predicate: P) -> Self
    where
        P: FnMut(&ExtractEntry) -> bool,
    {
        // Group entries by their chunk start offset to allow for sequential extraction
        let mut chunks = BTreeMap::<_, BTreeSet<_>>::new();

        for file in inno.file_entries() {
            let Some(location) = inno.file_locations().get(file.location() as usize) else {
                continue;
            };

            let extract_entry = ExtractEntry::new(file.clone(), *location);

            if predicate(&extract_entry) {
                chunks
                    .entry(location.chunk().start_offset())
                    .or_default()
                    .insert(extract_entry);
            }
        }

        Self {
            reader: FilesReader::Source(Some(&mut inno.reader)),
            data_offset: inno
                .inner
                .setup_loader
                .data_offset()
                .try_into()
                .unwrap_or_else(|_| unreachable!()),
            entries: BTreeSet::new(),
            chunks,
            current_position: 0,
            previous_location_index: None,
            data: Vec::new(),
        }
    }
}

impl<R: Read + Seek> Iterator for FilteredFilesIterator<'_, R> {
    type Item = InnoResult<(ExtractEntry, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = if let Some(entry) = self.entries.pop_first() {
            entry
        } else {
            self.entries = self.chunks.pop_first().map(|(_, entries)| entries)?;

            let entry = self.entries.pop_first()?;

            if let Err(err) = self
                .reader
                .reinitialize(self.data_offset, entry.file_location().chunk())
            {
                return Some(Err(err));
            }

            entry
        };

        // If this is the same location as the previous entry, reuse the cached data
        if self
            .previous_location_index
            .is_some_and(|index| index == entry.location_index())
        {
            return Some(Ok((entry, self.data.clone())));
        }

        let file_metadata = entry.file_location().file();
        let target_offset = file_metadata.offset();

        // Skip to the file's position within the compressed chunk
        if self.current_position < target_offset {
            if let Err(err) = io::copy(
                &mut self
                    .reader
                    .by_ref()
                    .take(target_offset - self.current_position),
                &mut io::sink(),
            ) {
                return Some(Err(err.into()));
            }

            self.current_position = target_offset;
        }

        // Resize the data buffer, reusing any existing allocated capacity
        self.data.resize(file_metadata.size() as usize, 0);

        // Read the file data
        if let Err(err) = self.reader.read_exact(&mut self.data) {
            return Some(Err(err.into()));
        }
        self.current_position += file_metadata.size();

        // Apply instruction filter first
        match file_metadata.compression_filter().decode(&mut self.data) {
            Ok(Cow::Owned(decompressed)) => self.data = decompressed,
            Ok(Cow::Borrowed(_)) => {}
            Err(err) => return Some(Err(err.into())),
        }

        // Validate the checksum (computed on data after filter is applied)
        if let Err(inner) = file_metadata.validate_checksum(&self.data) {
            return Some(Err(InnoError::ChecksumMismatch {
                location: "extracted file",
                inner,
            }));
        }

        self.previous_location_index = Some(entry.location_index());

        Some(Ok((entry, self.data.clone())))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.chunks.values().map(BTreeSet::len).sum::<usize>() + self.entries.len();
        (remaining, Some(remaining))
    }
}

impl<R: Read + Seek> ExactSizeIterator for FilteredFilesIterator<'_, R> {}
