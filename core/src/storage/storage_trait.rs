//! Storage trait definition
//! 
//! Defines the interface for storage backends (local filesystem, S3, etc.)

use async_trait::async_trait;
use std::error::Error;

/// Result type for storage operations
pub type StorageResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// Storage backend trait
/// 
/// This trait defines the interface for storing and retrieving files.
/// Implementations can be for local filesystem, S3, Azure Blob, etc.
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Upload a file and return its public URL
    /// 
    /// # Arguments
    /// * `file_data` - The file content as bytes
    /// * `file_name` - The original file name
    /// * `content_type` - The MIME type of the file
    /// 
    /// # Returns
    /// The public URL where the file can be accessed
    async fn upload_file(
        &self,
        file_data: Vec<u8>,
        file_name: String,
        content_type: String,
    ) -> StorageResult<String>;

    /// Delete a file by its URL or path
    /// 
    /// # Arguments
    /// * `file_url` - The URL or path of the file to delete
    async fn delete_file(&self, file_url: &str) -> StorageResult<()>;

    /// Get the base URL for accessing files
    /// 
    /// This is used to construct full URLs for uploaded files
    fn get_base_url(&self) -> &str;
}

