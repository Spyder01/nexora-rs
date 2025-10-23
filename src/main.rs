mod models;
mod utils;
mod storage_engine;

use std::path::Path;

use models::file_layout::{NexoraFile, NexoraHeader, PAGE_SIZE, NexoraFooter, OffsetTableChunk};
use utils::fs::crud::{write_file};
use storage_engine::engine::StorageEngine;

#[tokio::main]
async fn main() {
    let nexora_file = NexoraFile::default();

    let nexora_file_path = Path::new("./test.nexora");
    
    let data = nexora_file.serialize();
    match write_file(nexora_file_path, &data).await {
        Ok(_) => {},
        Err(e) => {
            println!("{:?}", e);
        },
    };

    let mut buf = [0u8; PAGE_SIZE];
    buf.copy_from_slice(&data[..PAGE_SIZE]);
    
    let header = NexoraHeader::deserialize(buf);
    println!("{:?}", header);
    
    let mut start = header.footer_offset as usize;
    buf.copy_from_slice(&data[start..start+PAGE_SIZE]);
    let footer = NexoraFooter::deserialize(buf);
    println!("{:?}", footer);
    
    start = footer.name_table_offset.base_chunk_offset as usize;
    buf.copy_from_slice(&data[start..start+PAGE_SIZE]);
    println!("{:?}", OffsetTableChunk::deserialize(&buf));

    let storage_engine = StorageEngine::load("test.nexora").await.unwrap();
    println!("{:?}", storage_engine.file_layout.footer)
}
