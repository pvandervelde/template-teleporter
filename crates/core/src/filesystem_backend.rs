//! Implements a simple `StatePersistence` backend using the local filesystem.
//! State is stored as JSON files within a specified base directory.

use crate::state_manager::StatePersistence;
use crate::types::{CoreError, Result, TemplateState};
use async_trait::async_trait;
use std::fs;

#[cfg(test)]
#[path = "filesystem_backend_tests.rs"]
mod tests;

use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc; // Using Arc for potential future sharing needs, though Mutex might be needed for concurrent writes
use tokio::sync::Mutex; // Use tokio's Mutex for async locking

/// A state persistence backend that stores `TemplateState` as JSON files
/// in a specified directory on the local filesystem.
#[derive(Debug)]
pub struct FilesystemBackend {
    base_path: PathBuf,
    // Using Mutex to prevent race conditions if multiple operations happen concurrently
    // on the same file system backend instance. Arc allows sharing the Mutex.
    lock: Arc<Mutex<()>>,
}

impl FilesystemBackend {
    /// Creates a new `FilesystemBackend`.
    ///
    /// Ensures the base directory exists, creating it if necessary.
    ///
    /// # Arguments
    /// * `base_path` - The path to the directory where state files will be stored.
    ///
    /// # Returns
    /// A `Result` containing the new `FilesystemBackend` or a `CoreError::IoError` if the
    /// directory cannot be created.
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&path).map_err(CoreError::IoError)?;
        Ok(Self {
            base_path: path,
            lock: Arc::new(Mutex::new(())),
        })
    }

    /// Constructs the full path for a state file based on the template ID.
    fn get_file_path(&self, template_id: &str) -> PathBuf {
        // Basic sanitization: replace common path separators to prevent directory traversal issues.
        // A more robust solution might involve hashing the ID or stricter validation.
        let sanitized_id = template_id.replace(['/', '\\', ':', '*'], "_");
        self.base_path.join(format!("{}.json", sanitized_id))
    }
}

#[async_trait]
impl StatePersistence for FilesystemBackend {
    /// Retrieves the state for a given template ID by reading its corresponding JSON file.
    async fn get_state(&self, template_id: &str) -> Result<Option<TemplateState>> {
        let file_path = self.get_file_path(template_id);
        let _guard = self.lock.lock().await; // Lock for read operation consistency

        match tokio::fs::read_to_string(&file_path).await {
            Ok(content) => {
                let state: TemplateState = serde_json::from_str(&content).map_err(|e| {
                    CoreError::DatabaseError(format!(
                        "Failed to deserialize state for {}: {}",
                        template_id, e
                    ))
                })?;
                Ok(Some(state))
            }
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(CoreError::IoError(e)),
        }
    }

    /// Saves or updates the state for a template by writing it as JSON to the corresponding file.
    async fn update_state(&self, state: &TemplateState) -> Result<()> {
        let file_path = self.get_file_path(&state.template_id);
        let _guard = self.lock.lock().await; // Lock for write operation

        let content = serde_json::to_string_pretty(state).map_err(|e| {
            CoreError::DatabaseError(format!(
                "Failed to serialize state for {}: {}",
                state.template_id, e
            ))
        })?;

        // Write to a temporary file first, then rename to make the update more atomic.
        let temp_path = file_path.with_extension("json.tmp");

        let mut temp_file = fs::File::create(&temp_path).map_err(CoreError::IoError)?;
        temp_file
            .write_all(content.as_bytes())
            .map_err(CoreError::IoError)?;
        temp_file.sync_all().map_err(CoreError::IoError)?; // Ensure data is flushed to disk

        fs::rename(&temp_path, &file_path).map_err(CoreError::IoError)?;

        Ok(())
    }
}
