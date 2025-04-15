use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use thiserror::Error;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

/// Custom error types for the core library.
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Database connection or operation error: {0}")]
    DatabaseError(String),
    #[error("Checksum calculation failed: {0}")]
    ChecksumFailure(String),
    #[error("Configuration parsing error: {source}")]
    ConfigParseError {
        #[from]
        source: serde_yaml::Error,
    },
    #[error("Template validation failed: {0}")]
    TemplateValidation(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Missing configuration value: {0}")]
    MissingConfiguration(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;

/// Represents the state of a managed template.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TemplateState {
    pub template_id: String,
    pub source_repository: String,
    pub current_checksum: String,
    pub last_updated_utc: DateTime<Utc>,
}

/// Represents the application's configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AppConfig {
    pub database_type: String,
    pub database_endpoint: Option<String>,
    pub table_name: String,
}

/// Calculates the SHA-256 checksum for the given input data.
pub fn calculate_checksum(data: &[u8]) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    println!("Input data: {:?}", data);
    println!("Hashed result: {:?}", result);
    Ok(hex::encode(result))
}

/// Parses the application configuration from a specified YAML file path.
pub fn parse_config(config_path: &Path) -> Result<AppConfig> {
    let file = std::fs::File::open(config_path)?;
    let config: AppConfig = serde_yaml::from_reader(file)?;
    Ok(config)
}
