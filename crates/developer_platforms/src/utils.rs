//! Utility functions for the developer_platforms crate.

use crate::error::Result; // Use the crate's Result type
use sha2::{Digest, Sha256};

/// Calculates the SHA-256 checksum for the given input data.
///
/// # Arguments
///
/// * `data` - A byte slice representing the data to checksum.
///
/// # Returns
///
/// A `Result` containing the hex-encoded SHA-256 checksum string or a `PlatformError`.
/// Note: This function itself doesn't currently return an error, but uses Result
/// for consistency with potential future fallible operations.
pub fn calculate_checksum(data: &[u8]) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let hex_string = hex::encode(result);

    // Return Ok result
    Ok(hex_string)
}

// TODO: Add other helper functions as needed (e.g., for crypto, string manipulation).
