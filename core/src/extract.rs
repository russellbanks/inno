use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read, Seek},
    path::Path,
};

use crate::{
    Inno,
    entry::{CompressionFilter, File, FileLocation},
    error::InnoError,
    read::{data_chunk::DataChunkReader, filter::apply_filter},
};

/// A file to extract, pairing the logical file entry with its location metadata.
struct ExtractEntry<'a> {
    file: &'a File,
    location: &'a FileLocation,
    loc_index: u32,
}

impl Inno {
    /// Extract all files to the given destination directory.
    ///
    /// The `reader` must be the same file used to construct this `Inno` instance.
    /// Files are grouped by chunk to avoid redundant decompression.
    ///
    /// Paths from the installer (e.g. `{app}\bin\foo.dll`) have the leading
    /// `{...}\` prefix stripped and backslashes normalized to the platform separator.
    pub fn extract_all<R: Read + Seek>(
        &self,
        reader: &mut R,
        dest: &Path,
    ) -> Result<(), InnoError> {
        self.extract_all_with_progress(reader, dest, |_, _| {})
    }

    /// Extract all files with a progress callback.
    ///
    /// The callback receives `(files_extracted, total_files)` after each file is written.
    pub fn extract_all_with_progress<R, F>(
        &self,
        reader: &mut R,
        dest: &Path,
        mut on_progress: F,
    ) -> Result<(), InnoError>
    where
        R: Read + Seek,
        F: FnMut(usize, usize),
    {
        let data_offset = self.setup_loader.data_offset().unsigned_abs();

        // Build entries and group by chunk start_offset for efficient extraction
        let mut chunks: BTreeMap<u64, Vec<ExtractEntry<'_>>> = BTreeMap::new();

        for file in self.files() {
            let loc_index = file.location();
            if loc_index as usize >= self.file_locations().len() {
                continue;
            }
            let location = &self.file_locations()[loc_index as usize];
            let chunk_key = location.chunk().start_offset();
            chunks.entry(chunk_key).or_default().push(ExtractEntry {
                file,
                location,
                loc_index,
            });
        }

        // Sort entries within each chunk by file offset for sequential reading
        for entries in chunks.values_mut() {
            entries.sort_by_key(|e| (e.location.file().offset(), e.loc_index));
        }

        let total_files: usize = chunks.values().map(|e| e.len()).sum();
        let mut files_done: usize = 0;

        // Extract each chunk
        for entries in chunks.values() {
            let chunk = &entries[0].location.chunk();
            let mut chunk_reader = DataChunkReader::new(&mut *reader, data_offset, chunk)?;

            let mut pos: u64 = 0;
            // Cache data for locations shared by multiple files
            let mut last_loc_index: Option<u32> = None;
            let mut last_data: Vec<u8> = Vec::new();

            for entry in entries {
                let file_meta = entry.location.file();

                // If this is the same location as the previous entry, reuse cached data
                let data = if last_loc_index == Some(entry.loc_index) {
                    last_data.clone()
                } else {
                    let target_offset = file_meta.offset();

                    // Skip to the file's position within the decompressed chunk
                    if target_offset > pos {
                        io::copy(
                            &mut (&mut chunk_reader).take(target_offset - pos),
                            &mut io::sink(),
                        )?;
                        pos = target_offset;
                    }

                    // Read the file data
                    let mut raw = vec![0u8; file_meta.size() as usize];
                    chunk_reader.read_exact(&mut raw)?;
                    pos += file_meta.size();

                    // Apply instruction filter first
                    let filter = file_meta.compression_filter();
                    apply_filter(&mut raw, filter);

                    // Handle ZlibFilter
                    if filter == CompressionFilter::ZlibFilter {
                        let mut decoded = Vec::new();
                        flate2::read::ZlibDecoder::new(&raw[..]).read_to_end(&mut decoded)?;
                        raw = decoded;
                    }

                    // Verify checksum (computed on data after filter is applied)
                    file_meta.checksum().verify(&raw)?;

                    last_loc_index = Some(entry.loc_index);
                    last_data = raw.clone();
                    raw
                };

                // Resolve destination path
                let Some(dest_path) = entry.file.destination() else {
                    continue;
                };
                let rel_path = resolve_inno_path(dest_path);
                if rel_path.is_empty() {
                    continue;
                }

                let full_path = dest.join(&rel_path);
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&full_path, &data)?;
                files_done += 1;
                on_progress(files_done, total_files);
            }
        }

        Ok(())
    }

    /// Extract a single file by its index in [`Inno::files()`].
    ///
    /// The `reader` must be the same file used to construct this `Inno` instance.
    pub fn extract_file<R: Read + Seek>(
        &self,
        reader: &mut R,
        file_index: usize,
        dest: &Path,
    ) -> Result<(), InnoError> {
        let file = &self.files()[file_index];
        let loc_index = file.location() as usize;

        if loc_index >= self.file_locations().len() {
            return Err(InnoError::FileLocationOutOfBounds {
                index: file.location(),
                max: self.file_locations().len(),
            });
        }

        let location = &self.file_locations()[loc_index];
        let data_offset = self.setup_loader.data_offset().unsigned_abs();
        let chunk = location.chunk();

        let mut chunk_reader = DataChunkReader::new(&mut *reader, data_offset, &chunk)?;

        // Skip to the file's position
        let file_meta = location.file();
        if file_meta.offset() > 0 {
            io::copy(
                &mut (&mut chunk_reader).take(file_meta.offset()),
                &mut io::sink(),
            )?;
        }

        // Read file data
        let mut data = vec![0u8; file_meta.size() as usize];
        chunk_reader.read_exact(&mut data)?;

        // Apply instruction filter first
        let filter = file_meta.compression_filter();
        apply_filter(&mut data, filter);

        // Handle ZlibFilter
        if filter == CompressionFilter::ZlibFilter {
            let mut decoded = Vec::new();
            flate2::read::ZlibDecoder::new(&data[..]).read_to_end(&mut decoded)?;
            data = decoded;
        }

        // Verify checksum (computed on data after filter is applied)
        file_meta.checksum().verify(&data)?;

        // Resolve destination path
        let dest_str = file.destination().unwrap_or("unknown");
        let rel_path = resolve_inno_path(dest_str);
        let file_name = if rel_path.is_empty() {
            dest_str
        } else {
            &rel_path
        };

        let full_path = dest.join(file_name);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&full_path, &data)?;

        Ok(())
    }
}

/// Resolve an Inno Setup path by stripping the `{...}\` prefix and normalizing
/// backslashes to forward slashes.
///
/// Examples:
/// - `{app}\bin\foo.dll` -> `bin/foo.dll`
/// - `{sys}\bar.dll` -> `bar.dll`
/// - `plain\path.txt` -> `plain/path.txt`
fn resolve_inno_path(path: &str) -> String {
    let stripped = if path.starts_with('{') {
        // Strip the `{...}\` or `{.../` prefix
        path.find('}')
            .map(|i| {
                let rest = &path[i + 1..];
                rest.strip_prefix('\\')
                    .or_else(|| rest.strip_prefix('/'))
                    .unwrap_or(rest)
            })
            .unwrap_or(path)
    } else {
        path
    };

    stripped.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_app_path() {
        assert_eq!(resolve_inno_path(r"{app}\bin\foo.dll"), "bin/foo.dll");
    }

    #[test]
    fn resolve_sys_path() {
        assert_eq!(resolve_inno_path(r"{sys}\bar.dll"), "bar.dll");
    }

    #[test]
    fn resolve_plain_path() {
        assert_eq!(resolve_inno_path(r"plain\path.txt"), "plain/path.txt");
    }

    #[test]
    fn resolve_empty_after_prefix() {
        assert_eq!(resolve_inno_path("{app}"), "");
    }
}
