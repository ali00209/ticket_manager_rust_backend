use async_trait::async_trait;
use std::path::Path;

use super::Storage;

/// Local filesystem storage backend.
///
/// Stores files on the local disk. Suitable for development and small deployments.
/// For production, consider implementing the Storage trait for S3 or similar.
pub struct LocalStorage {
    base_dir: String,
}

impl LocalStorage {
    /// Create a new local storage instance.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for file storage
    pub fn new(base_dir: &str) -> Self {
        // Ensure base directory exists
        std::fs::create_dir_all(base_dir).ok();
        Self {
            base_dir: base_dir.to_string(),
        }
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn put(&self, path: &str, data: &[u8]) -> Result<String, String> {
        let full_path = format!("{}/{}", self.base_dir, path);

        // Create parent directories
        if let Some(parent) = Path::new(&full_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        std::fs::write(&full_path, data)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(path.to_string())
    }

    async fn get(&self, path: &str) -> Result<Vec<u8>, String> {
        let full_path = format!("{}/{}", self.base_dir, path);
        std::fs::read(&full_path).map_err(|e| format!("Failed to read file: {}", e))
    }

    async fn delete(&self, path: &str) -> Result<(), String> {
        let full_path = format!("{}/{}", self.base_dir, path);
        std::fs::remove_file(&full_path).map_err(|e| format!("Failed to delete file: {}", e))
    }

    async fn exists(&self, path: &str) -> Result<bool, String> {
        let full_path = format!("{}/{}", self.base_dir, path);
        Ok(Path::new(&full_path).exists())
    }
}
