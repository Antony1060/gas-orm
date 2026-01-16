use std::path::Path;

pub async fn list_dir(path: impl AsRef<Path>) -> Result<Vec<tokio::fs::DirEntry>, std::io::Error> {
    let mut entries = tokio::fs::read_dir(path).await?;

    let mut result = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        result.push(entry);
    }

    Ok(result)
}
