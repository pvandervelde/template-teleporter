//! Defines shared data structures used across the developer_platforms crate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents the path to a template file within a repository or category.
/// Using String for now, might become more complex later (e.g., PathBuf).
pub type TemplatePath = String;

/// Represents a named category for grouping templates.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateCategory(pub String);

/// Metadata associated with a template file in the master repository.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// The path to the template file relative to its category root.
    pub path: TemplatePath,
    /// The SHA-256 checksum of the template file content.
    pub checksum: String,
    /// The timestamp of the last commit that modified this template file.
    pub last_updated: DateTime<Utc>,
}

/// Information about a target repository.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// The organization or owner of the repository.
    pub org: String,
    /// The name of the repository.
    pub name: String,
    /// The default branch of the repository (e.g., "main", "master").
    pub default_branch: String,
}

/// Represents a change detected in a template file.
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateChange {
    /// The path to the changed template file.
    pub path: TemplatePath,
    /// Checksums of the file before the change, if known (e.g., from state).
    /// Storing multiple might help track history, but limiting for simplicity.
    pub old_checksums: Vec<String>, // Spec mentioned storing up to 10, using Vec for now.
    /// The new checksum of the file after the change.
    pub new_checksum: String,
    /// The new content of the template file.
    pub content: Vec<u8>,
}

/// Result returned after successfully updating a repository with template changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateResult {
    /// The URL of the created or updated pull request.
    pub pr_url: String,
    /// The number of the created or updated pull request.
    pub pr_number: u64,
    /// List of template paths that were updated in the pull request.
    pub updated_files: Vec<TemplatePath>,
}
