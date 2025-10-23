use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use thiserror::Error;

use crate::models::file_layout::{
    NexoraFile, NexoraFooter, NexoraHeader, PAGE_SIZE,
    OffsetTableChunk, OffsetItem, INVALID_OFFSET,
};

#[derive(Debug, Error)]
pub enum CorruptedFileError {
    #[error("Invalid magic value in file header")]
    InvalidMagicValue,

    #[error("Offset value is Invalid")]
    InvalidOffsetValue,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error occurred due to: {0}")]
    Io(#[from] io::Error),

    #[error("Corrupted file format due to: {0:?}")]
    Corrupted(#[from] CorruptedFileError),
}

#[derive(Debug)]
pub struct StorageEngine {
    pub file_path: String,
    pub file_layout: NexoraFile,
    pub file_handle: File,
}

impl StorageEngine {
    pub async fn new(file_path: &str) -> Result<Self, StorageError> {
        let file_handle = File::open(file_path).await?;
        Ok(Self {
            file_layout: NexoraFile::default(),
            file_path: file_path.to_string(),
            file_handle,
        })
    }

    pub async fn load(file_path: &str) -> Result<Self, StorageError> {
        let mut engine = Self::new(file_path).await?;

        let mut buffer = [0u8; 6];
        engine.file_handle.read_exact(&mut buffer).await?;
        engine.file_handle.seek(SeekFrom::Start(0)).await?;

        if !NexoraHeader::verify_magic(buffer) {
            return Err(CorruptedFileError::InvalidMagicValue.into());
        }

        let mut raw_header = [0u8; PAGE_SIZE];
        engine.file_handle.read_exact(&mut raw_header).await?;
        let header = NexoraHeader::deserialize(raw_header);
        engine.file_layout.header = header;

        engine.file_handle.seek(SeekFrom::Start(header.footer_offset)).await?;
        let mut raw_footer = [0u8; PAGE_SIZE];
        engine.file_handle.read_exact(&mut raw_footer).await?;
        engine.file_layout.footer = NexoraFooter::deserialize(raw_footer);

        engine.file_handle.seek(SeekFrom::Start(0)).await?;
        Ok(engine)
    }

    /// Reads an offset table chunk from the file at a given offset.
    pub async fn read_offset_table(&mut self, offset: u64) -> Result<OffsetTableChunk, StorageError> {
        if offset == INVALID_OFFSET {
            return Err(CorruptedFileError::InvalidOffsetValue.into());
        }

        let mut raw_chunk = [0u8; PAGE_SIZE];
        self.file_handle.seek(SeekFrom::Start(offset)).await?;
        self.file_handle.read_exact(&mut raw_chunk).await?;
        let chunk = OffsetTableChunk::deserialize(&raw_chunk);
        Ok(chunk)
    }

    /// Writes an offset table chunk to disk at the given offset.
    async fn log_offset_chunk(&mut self, chunk: &OffsetTableChunk, offset: u64) -> Result<(), StorageError> {
        let buf = chunk.serialize();
        self.file_handle.seek(SeekFrom::Start(offset)).await?;
        self.file_handle.write_all(&buf).await?;
        self.file_handle.seek(SeekFrom::Start(0)).await?;
        self.file_handle.flush().await?;
        
        self.log_footer_chunk();
        Ok(())
    }

    /// Log footer val
    async fn log_footer_chunk(&mut self) -> Result<(), StorageError> {
        let buf = self.file_layout.footer.serialize();
        self.file_handle.seek(SeekFrom::Start(self.file_layout.header.footer_offset)).await?;
        self.file_handle.write_all(&buf).await?;
        self.file_handle.seek(SeekFrom::Start(0)).await?;
        self.file_handle.flush().await?;
        Ok(())
    }

    /// Get new offset table space
    fn get_new_offset_table_space(&mut self) -> u64 {
        let new_chunk_path = self.file_layout.header.footer_offset;
        self.file_layout.header.footer_offset += PAGE_SIZE as u64;

        new_chunk_path
    }

    /// Inserts a new OffsetItem into the linked list of offset table chunks.
    pub async fn insert_offset_item(&mut self, mut offset: u64, offset_item: OffsetItem) -> Result<(), StorageError> {
        if offset == INVALID_OFFSET {
            return Err(CorruptedFileError::InvalidOffsetValue.into());
        }

        loop {
            // Load current chunk
            let mut chunk = self.read_offset_table(offset).await?;

            // Find first empty slot
            if (chunk.nb_items as usize) < chunk.offset_items.len() {
                chunk.offset_items[chunk.nb_items as usize] = offset_item;
                chunk.nb_items += 1;

                // Write it back
                self.log_offset_chunk(&chunk, offset).await?;
                return Ok(());
            }

            // If current chunk is full, go to next
            if chunk.next_chunk == INVALID_OFFSET {
                // Create a new chunk
                let mut new_chunk = OffsetTableChunk::default();
                new_chunk.previous_chunk = offset;
                new_chunk.offset_items[0] = offset_item;
                new_chunk.nb_items = 1;

                // Determine file size to place new chunk at EOF
                let new_offset = self.get_new_offset_table_space();
                chunk.next_chunk = new_offset;

                // Write updated current chunk
                self.log_offset_chunk(&chunk, offset).await?;

                // Write new chunk
                self.log_offset_chunk(&new_chunk, new_offset).await?;
                return Ok(());
            }

            // Move to next chunk
            offset = chunk.next_chunk;
        }
    }

    pub async fn close(&mut self) -> io::Result<()> {
        self.file_handle.flush().await
    }
}
