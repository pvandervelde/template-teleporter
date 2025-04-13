use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Authentication failed: {0}")]
    AuthError(String),
    #[error("API rate limit exceeded")]
    RateLimitExceeded,
    #[error("Repository not found: {org}/{name}")]
    RepoNotFound { org: String, name: String },
    #[error("Template path not found: {0}")]
    TemplateNotFound(String),
    #[error("Category not found: {0:?}")]
    CategoryNotFound(String),
    #[error("Invalid content encoding/decoding: {0}")]
    InvalidContent(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Network or API error: {0}")]
    ApiError(String),
    #[error("Operation failed: {0}")]
    OperationFailed(String), // General failure
    #[error("Webhook verification failed")]
    WebhookVerificationFailed,
    #[error("Underlying platform error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
