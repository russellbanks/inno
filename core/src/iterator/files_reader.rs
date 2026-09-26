use std::io::{self, Read, Seek};

use crate::{
    error::InnoResult,
    read::{chunk::Chunk, data_chunk::DataChunkReader},
};

pub(super) enum FilesReader<'reader, R: Read + Seek> {
    Source(Option<&'reader mut R>),
    Chunk(Option<DataChunkReader<&'reader mut R>>),
}

impl<R: Read + Seek> FilesReader<'_, R> {
    pub(super) fn to_source_mut(&mut self) -> &mut Self {
        if let Self::Chunk(reader) = self
            && let Some(reader) = reader.take()
        {
            let reader = reader.into_inner().into_inner();
            *self = FilesReader::Source(Some(reader));
        }

        self
    }

    pub(super) fn to_chunk_mut(
        &mut self,
        data_offset: u64,
        chunk: &Chunk,
    ) -> InnoResult<&mut Self> {
        if let Self::Source(reader) = self
            && let Some(reader) = reader.take()
        {
            let chunk_reader = DataChunkReader::new(reader, data_offset, chunk)?;
            *self = FilesReader::Chunk(Some(chunk_reader));
        }

        Ok(self)
    }

    pub(super) fn reinitialize(
        &mut self,
        data_offset: u64,
        chunk: &Chunk,
    ) -> InnoResult<&mut Self> {
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
