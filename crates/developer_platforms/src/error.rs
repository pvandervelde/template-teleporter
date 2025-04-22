//! Defines the custom error type `PlatformError` for the developer_platforms crate.

use crate::types::{TemplateCategory, TemplatePath};
use thiserror::Error;

/// Represents errors that can occur during interactions with developer platforms.
#[derive(Error, Debug)]
pub enum PlatformError {
    /// Error during authentication (e.g., invalid credentials, token expiration).
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    /// The API rate limit for the platform was exceeded.
    #[error("API rate limit exceeded")]
    RateLimitExceeded,

    /// The specified repository could not be found.
    #[error("Repository not found: {org}/{name}")]
    RepositoryNotFound { org: String, name: String },

    /// The specified template path could not be found within the repository or category.
    #[error("Template path not found: {0}")]
    TemplateNotFound(TemplatePath),

    /// The specified template category is not defined or found.
    #[error("Category not found: {0:?}")]
    CategoryNotFound(TemplateCategory),

    /// Error related to content encoding (e.g., non-UTF8) or decoding (e.g., invalid base64).
    #[error("Invalid content encoding/decoding: {0}")]
    InvalidContent(String),

    /// Error originating from configuration issues (e.g., missing keys, invalid values).
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Generic error during an API call (e.g., network issues, unexpected response).
    #[error("Network or API error: {0}")]
    ApiError(String),

    /// A general failure during a platform operation not covered by other variants.
    #[error("Operation failed: {0}")]
    OperationFailed(String),

    /// Failed to verify the signature of an incoming webhook payload.
    #[error("Webhook verification failed")]
    WebhookVerificationFailed,

    /// Wrapper for underlying errors from platform SDKs or other dependencies.
    #[error("Underlying platform error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

// Define a standard Result type for this crate
pub type Result<T> = std::result::Result<T, PlatformError>;
