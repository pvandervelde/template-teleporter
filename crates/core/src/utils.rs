//! Provides utility functions for the core library, such as checksum calculation
//! and configuration parsing.

use crate::types::{AppConfig, CoreError, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

// Test module declaration for lib.rs itself
#[cfg(test)]
#[path = "utils_tests.rs"]
mod tests;

/// Calculates the SHA-256 checksum for the given input data.
///
/// Uses the `sha2` crate for the hashing algorithm and `hex` crate for encoding the result.
///
/// # Arguments
/// * `data` - A byte slice representing the data to checksum.
///
/// # Returns
/// A `Result` containing the lowercase hex-encoded SHA-256 checksum string,
/// or a `CoreError::ChecksumFailure` if hashing fails (though this is unlikely with SHA-256).
///
/// # Examples
/// ```
/// // Assuming this code is run within the context where calculate_checksum is available
/// # use template_teleporter_core::{calculate_checksum, Result};
/// # fn run() -> Result<()> {
/// let data = b"hello world";
/// let checksum = calculate_checksum(data)?;
/// assert_eq!(checksum, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
/// # Ok(())
/// # }
/// ```
pub fn calculate_checksum(data: &[u8]) -> Result<String> {
    // Note: Sha256::new() is infallible. Error handling here is minimal.
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    // hex::encode is also generally infallible for standard byte arrays.
    Ok(hex::encode(result))
}

/// Parses the application configuration from a specified YAML file path.
///
/// Reads the file at the given path and attempts to deserialize it into an `AppConfig` struct
/// using `serde_yaml`. Also performs basic validation.
///
/// # Arguments
/// * `config_path` - Path to the YAML configuration file.
///
/// # Returns
/// A `Result` containing the parsed `AppConfig` on success, or a `CoreError` if:
///   - The file cannot be opened (`CoreError::IoError`).
///   - The file content is not valid YAML or doesn't match the `AppConfig` structure (`CoreError::ConfigParseError`).
///   - A required configuration value is missing or invalid (`CoreError::MissingConfiguration`).
///
/// # Errors
/// Returns `CoreError::IoError` if the file cannot be opened.
/// Returns `CoreError::ConfigParseError` if YAML parsing fails.
/// Returns `CoreError::MissingConfiguration` if `table_name` is empty.
pub fn parse_config(config_path: &Path) -> Result<AppConfig> {
    // Open the file, propagating IO errors.
    let file = std::fs::File::open(config_path)?;
    // Parse the YAML, mapping serde_yaml errors to our CoreError::ConfigParseError.
    let config: AppConfig =
        serde_yaml::from_reader(file).map_err(|e| CoreError::ConfigParseError { source: e })?;

    // Basic validation: Ensure table_name is not empty
    if config.table_name.is_empty() {
        return Err(CoreError::MissingConfiguration(
            "table_name cannot be empty".to_string(),
        ));
    }

    Ok(config)
}
