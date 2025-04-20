//! Defines the core data types, error types, and the standard `Result` type for the crate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Custom error types encompassing potential failures within the core library.
#[derive(Error, Debug)]
pub enum CoreError {
    /// Errors originating from the state persistence backend (e.g., database connection, operation failure).
    #[error("Database connection or operation error: {0}")]
    DatabaseError(String),

    /// Error during SHA-256 checksum calculation.
    #[error("Checksum calculation failed: {0}")]
    ChecksumFailure(String),

    /// Error parsing the application configuration file (e.g., YAML format error).
    #[error("Configuration parsing error: {source}")]
    ConfigParseError {
        #[from]
        source: serde_yaml::Error,
    },

    /// Error indicating a template failed validation checks (specific checks TBD).
    #[error("Template validation failed: {0}")]
    TemplateValidation(String),

    /// General I/O errors (e.g., reading config file, potentially template files later).
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error when a required configuration value is missing.
    #[error("Missing configuration value: {0}")]
    MissingConfiguration(String),

    /// Errors originating from interactions with external developer platforms (e.g., GitHub API errors).
    #[error("Platform interaction error: {0}")]
    PlatformError(String),
}

/// A specialized `Result` type for the core library, using `CoreError` as the error type.
pub type Result<T> = std::result::Result<T, CoreError>;

/// Represents the persisted state of a managed template, tracked by the application.
///
/// This struct holds information about the template's source, its last known checksum,
/// and when it was last updated by the system.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TemplateState {
    /// Unique identifier for the template (e.g., file path within the master repo).
    #[serde(rename = "templateId")]
    pub template_id: String,

    /// The source repository where the master version of the template resides.
    #[serde(rename = "sourceRepository")]
    pub source_repository: String,

    /// The SHA-256 checksum of the template content the last time it was successfully processed or updated.
    #[serde(rename = "currentChecksum")]
    pub current_checksum: String,

    /// Timestamp (UTC) when the template state was last updated in the persistence layer.
    #[serde(rename = "lastUpdatedUtc")]
    pub last_updated_utc: DateTime<Utc>,
}

/// Represents the application's configuration settings, typically loaded from a file.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")] // Consistent config naming
pub struct AppConfig {
    /// Optional endpoint override for the database (e.g., for local testing).
    pub database_endpoint: Option<String>,

    /// The name of the table or container used for storing `TemplateState`.
    pub table_name: String,
}
