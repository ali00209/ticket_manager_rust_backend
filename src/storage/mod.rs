pub mod local;

use async_trait::async_trait;

/// Storage backend trait for file operations.
///
/// Implement this trait to add support for different storage backends
/// (local filesystem, S3, GCS, etc.).
#[async_trait]
pub trait Storage: Send + Sync {
    /// Store a file and return the storage path.
    async fn put(&self, path: &str, data: &[u8]) -> Result<String, String>;

    /// Retrieve file data by storage path.
    async fn get(&self, path: &str) -> Result<Vec<u8>, String>;

    /// Delete a file by storage path.
    async fn delete(&self, path: &str) -> Result<(), String>;

    /// Check if a file exists.
    async fn exists(&self, path: &str) -> Result<bool, String>;
}
