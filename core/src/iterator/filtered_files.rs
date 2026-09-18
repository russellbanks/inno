//! Two backends for the same iterator.
//!
//! The buffered one reads each file into a `Vec` and is what this crate has
//! always done. The streaming one is [`StreamingFiles`] with a `Vec` on the
//! end, so the two paths cannot drift apart, at the cost of re-reading a
//! location a second entry names instead of keeping the bytes it already had.
//! Which is why the buffered one is still the default: that trade is worth
//! measuring on a real installer before it is made for everyone.

#[cfg(not(feature = "streaming-filtered-files"))]
mod buffered {
    use std::{
        borrow::Cow,
        collections::{BTreeMap, VecDeque},
        io,
        io::{Read, Seek},
    };

    use crate::{
        Inno, Source,
        error::{InnoError, InnoResult},
        iterator::{ExtractEntry, files_reader::FilesReader},
        read::Embedded,
    };

    pub struct FilteredFilesIterator<'reader, R: Read + Seek> {
        reader: FilesReader<'reader, R>,
        chunks: BTreeMap<u64, VecDeque<ExtractEntry>>,
        entries: VecDeque<ExtractEntry>,
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
            let mut chunks = BTreeMap::<_, Vec<ExtractEntry>>::new();

            for (index, file) in inno.file_entries().iter().enumerate() {
                let Some(location) = inno.file_locations().get(file.location() as usize) else {
                    continue;
                };

                let extract_entry = ExtractEntry::new(index, file.clone(), *location);

                if predicate(&extract_entry) {
                    chunks
                        .entry(location.chunk().start_offset())
                        .or_default()
                        .push(extract_entry);
                }
            }

            // Read order within a chunk is by position, so that the reader only
            // ever moves forward. A sequence rather than a set: two entries can
            // name one location, and a set keyed on this order would treat the
            // second as a duplicate and drop it.
            let chunks = chunks
                .into_iter()
                .map(|(offset, mut entries)| {
                    entries.sort_by_key(|entry| {
                        (
                            entry.file_location().file().offset(),
                            entry.location_index(),
                        )
                    });
                    (offset, VecDeque::from(entries))
                })
                .collect();

            let data_offset = inno
                .inner
                .setup_loader
                .data_offset()
                .try_into()
                .unwrap_or_else(|_| unreachable!());

            let source = match inno.slices.as_mut() {
                Some(slices) => Source::Slices(slices),
                None => Source::Embedded(Embedded::new(&mut inno.reader, data_offset)),
            };

            Self {
                reader: FilesReader::Source(Some(source)),
                entries: VecDeque::new(),
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
            let entry = if let Some(entry) = self.entries.pop_front() {
                entry
            } else {
                self.entries = self.chunks.pop_first().map(|(_, entries)| entries)?;

                let entry = self.entries.pop_front()?;

                if let Err(err) = self.reader.reinitialize(entry.file_location().chunk()) {
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
            let remaining =
                self.chunks.values().map(VecDeque::len).sum::<usize>() + self.entries.len();
            (remaining, Some(remaining))
        }
    }

    impl<R: Read + Seek> ExactSizeIterator for FilteredFilesIterator<'_, R> {}
}

#[cfg(not(feature = "streaming-filtered-files"))]
pub use buffered::FilteredFilesIterator;

#[cfg(feature = "streaming-filtered-files")]
mod streamed {
    use std::io::{self, Read, Seek};

    use crate::{
        Inno,
        entry::checksum::ChecksumMismatchError,
        error::{InnoError, InnoResult},
        iterator::{ExtractEntry, StreamingFiles},
    };

    /// [`Read`] can only carry an [`io::Error`], but this iterator has always
    /// reported a mismatch as [`InnoError::ChecksumMismatch`], which callers match
    /// on to tell a corrupt file from a failed read.
    fn to_inno_error(error: io::Error) -> InnoError {
        match error.downcast::<ChecksumMismatchError>() {
            Ok(inner) => InnoError::ChecksumMismatch {
                location: "extracted file",
                inner,
            },
            Err(error) => InnoError::Io(error),
        }
    }

    pub struct FilteredFilesIterator<'reader, R: Read + Seek> {
        files: StreamingFiles<'reader, R>,
    }

    impl<'reader, R: Read + Seek> FilteredFilesIterator<'reader, R> {
        pub fn new<P>(inno: &'reader mut Inno<R>, predicate: P) -> Self
        where
            P: FnMut(&ExtractEntry) -> bool,
        {
            Self {
                files: StreamingFiles::new(inno, predicate),
            }
        }
    }

    impl<R: Read + Seek> Iterator for FilteredFilesIterator<'_, R> {
        type Item = InnoResult<(ExtractEntry, Vec<u8>)>;

        fn next(&mut self) -> Option<Self::Item> {
            let (entry, mut reader) = match self.files.next()? {
                Ok(pair) => pair,
                Err(error) => return Some(Err(error)),
            };

            // An exact fit, so `read_to_end` probes for the end rather than
            // doubling. That probe is also the read which checks the checksum.
            let size = usize::try_from(entry.file_location().file().size()).unwrap_or(0);
            let mut data = Vec::with_capacity(size);
            if let Err(error) = reader.read_to_end(&mut data) {
                return Some(Err(to_inno_error(error)));
            }

            Some(Ok((entry, data)))
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let remaining = self.files.len();
            (remaining, Some(remaining))
        }
    }

    impl<R: Read + Seek> ExactSizeIterator for FilteredFilesIterator<'_, R> {}

    #[cfg(test)]
    mod tests {
        use std::io;

        use crate::{
            entry::checksum::Checksum, error::InnoError, iterator::streaming::mismatch_to_io,
        };

        /// Rebuilding this iterator on the streaming reader must not turn a
        /// corrupt file into an `Io` error.
        #[test]
        fn a_checksum_mismatch_keeps_its_own_variant() {
            let mut hasher = Checksum::new_crc32(crc32fast::hash(b"expected")).hasher();
            hasher.update(b"actual");
            let error = super::to_inno_error(mismatch_to_io(hasher.finish().unwrap_err()));

            assert!(matches!(
                error,
                InnoError::ChecksumMismatch {
                    location: "extracted file",
                    ..
                }
            ));
            assert!(
                error
                    .to_string()
                    .starts_with("Inno Setup checksum mismatch reading extracted file.")
            );
        }

        #[test]
        fn any_other_failure_stays_an_io_error() {
            let error = super::to_inno_error(io::Error::from(io::ErrorKind::UnexpectedEof));
            assert!(matches!(error, InnoError::Io(_)));
        }
    }
}

#[cfg(feature = "streaming-filtered-files")]
pub use streamed::FilteredFilesIterator;
