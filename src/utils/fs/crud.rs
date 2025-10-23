use std::path::Path;
use tokio::fs::write;

pub async fn write_file(path: &Path, data: &[u8]) -> std::io::Result<()> { 
    write(path, data).await?;
    Ok(())
}
