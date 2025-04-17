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
    #[error("Platform interaction error: {0}")]
    PlatformError(String),
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
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub database_type: DatabaseType,
    pub database_endpoint: Option<String>,
    pub table_name: String,
}

/// Supported database types for state management.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    Dynamodb,
    Cosmosdb,
}

/// Calculates the SHA-256 checksum for the given input data.
///
/// # Arguments
/// * `data` - A byte slice representing the data to checksum.
///
/// # Returns
/// A `Result` containing the hex-encoded SHA-256 checksum string or a `CoreError`.
pub fn calculate_checksum(data: &[u8]) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Parses the application configuration from a specified YAML file path.
///
/// # Arguments
/// * `config_path` - Path to the configuration file.
///
/// # Returns
/// A `Result` containing the parsed `AppConfig` or a `CoreError`.
pub fn parse_config(config_path: &Path) -> Result<AppConfig> {
    let file = std::fs::File::open(config_path)?;
    let config: AppConfig = serde_yaml::from_reader(file)?;
    Ok(config)
}

use async_trait::async_trait;

/// Manages state persistence for templates.
pub struct StateManager {
    table_name: String,
    // Placeholder for DB client, e.g., DynamoDbClient or CosmosDbClient
    // db_client: ...,
}

impl StateManager {
    /// Creates a new StateManager instance based on configuration.
    pub async fn new(config: &AppConfig) -> Result<Self> {
        // TODO: Initialize DB client based on config.database_type
        Ok(Self {
            table_name: config.table_name.clone(),
            // db_client: ...
        })
    }

    /// Retrieves the state for a given template ID. Returns Ok(None) if not found.
    pub async fn get_state(&self, template_id: &str) -> Result<Option<TemplateState>> {
        // TODO: Implement using serde_dynamo or specific SDK
        // Placeholder implementation
        Err(CoreError::DatabaseError(
            "get_state not yet implemented".to_string(),
        ))
    }

    /// Saves or updates the state for a template.
    pub async fn update_state(&self, state: &TemplateState) -> Result<()> {
        // TODO: Implement using serde_dynamo or specific SDK
        // Placeholder implementation
        Err(CoreError::DatabaseError(
            "update_state not yet implemented".to_string(),
        ))
    }
}
