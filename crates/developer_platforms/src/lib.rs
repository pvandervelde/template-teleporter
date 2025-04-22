//! # Template Teleporter Developer Platforms Crate
//!
//! This crate provides traits and implementations for interacting with various
//! developer platforms (e.g., GitHub) as part of the Template Teleporter system.
//!
//! It defines the core `DeveloperPlatform` trait, shared data structures, and
//! platform-specific clients.

// Module declarations (following recommended order)
pub mod error;
pub mod types;
pub mod auth; // Authentication logic, e.g., GitHub App JWT
pub mod github; // GitHub specific implementation
pub mod utils; // Utility functions

// Re-export key items for easier use by consumers
pub use error::PlatformError;
pub use types::{RepoInfo, TemplateCategory, TemplateChange, TemplateMetadata, TemplatePath, UpdateResult};

use async_trait::async_trait;

/// Trait defining the interface for interacting with a developer platform.
///
/// This allows the core logic to remain platform-agnostic.
#[async_trait]
pub trait DeveloperPlatform: Send + Sync {
    /// Get all defined template categories from the master configuration.
    async fn list_categories(&self) -> Result<Vec<TemplateCategory>, PlatformError>;

    /// Get template content from the master repository for a specific category and path.
    async fn get_template(
        &self,
        category: &TemplateCategory,
        path: &TemplatePath,
    ) -> Result<Vec<u8>, PlatformError>;

    /// List all template metadata (path, checksum, last updated) for a given category
    /// in the master repository.
    async fn list_templates(
        &self,
        category: &TemplateCategory,
    ) -> Result<Vec<TemplateMetadata>, PlatformError>;

    /// List all target repositories configured to use a specific template category.
    /// This likely involves reading configuration from the master repo or a central store.
    async fn list_repos_by_category(
        &self,
        category: &TemplateCategory,
    ) -> Result<Vec<RepoInfo>, PlatformError>;

    /// Determine which templates within a category have changed in the master repository
    /// since a given commit SHA.
    async fn get_updated_templates(
        &self,
        category: &TemplateCategory,
        since_commit: &str,
    ) -> Result<Vec<TemplateChange>, PlatformError>;

    /// Apply template changes to a target repository: create a branch, commit changes,
    /// create a pull request, and return the PR details.
    async fn update_repo(
        &self,
        repo: &RepoInfo,
        changes: &[TemplateChange],
    ) -> Result<UpdateResult, PlatformError>;
}
