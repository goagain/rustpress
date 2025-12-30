//! Local filesystem storage implementation
//! 
//! Stores files in a local directory and serves them via HTTP

use crate::storage::{StorageBackend, StorageResult};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Local filesystem storage backend
pub struct LocalStorage {
    /// Base directory where files are stored
    storage_dir: PathBuf,
    /// Base URL for accessing files (e.g., "http://localhost:3000/uploads")
    base_url: String,
}

impl LocalStorage {
    /// Create a new local storage instance
    /// 
    /// # Arguments
    /// * `storage_dir` - Directory path where files will be stored
    /// * `base_url` - Base URL for accessing files
    pub fn new(storage_dir: impl AsRef<Path>, base_url: String) -> Self {
        Self {
            storage_dir: storage_dir.as_ref().to_path_buf(),
            base_url,
        }
    }

    /// Ensure the storage directory exists
    pub async fn ensure_directory(&self) -> StorageResult<()> {
        tokio::fs::create_dir_all(&self.storage_dir).await?;
        Ok(())
    }

    /// Generate a unique file name
    fn generate_file_name(&self, original_name: &str) -> String {
        let extension = Path::new(original_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin");
        
        format!("{}.{}", Uuid::new_v4(), extension)
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn upload_file(
        &self,
        file_data: Vec<u8>,
        file_name: String,
        _content_type: String,
    ) -> StorageResult<String> {
        // Ensure directory exists
        self.ensure_directory().await?;

        // Generate unique file name
        let unique_name = self.generate_file_name(&file_name);
        let file_path = self.storage_dir.join(&unique_name);

        // Write file to disk
        tokio::fs::write(&file_path, file_data).await?;

        // Return public URL
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), unique_name);
        Ok(url)
    }

    async fn delete_file(&self, file_url: &str) -> StorageResult<()> {
        // Extract file name from URL
        let file_name = file_url
            .split('/')
            .last()
            .ok_or("Invalid file URL")?;
        
        let file_path = self.storage_dir.join(file_name);
        
        // Delete file if it exists
        if file_path.exists() {
            tokio::fs::remove_file(&file_path).await?;
        }
        
        Ok(())
    }

    fn get_base_url(&self) -> &str {
        &self.base_url
    }
}

