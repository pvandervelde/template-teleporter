use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

mod errors;

pub use errors::PlatformError;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

// Placeholder for potential path abstraction
pub type TemplatePath = String;

/// Represents a category of templates.
///
/// # Example
/// ```rust,no_run
/// use template_teleporter_developer_platforms::TemplateCategory;
/// let category = TemplateCategory::new("saas_rust".to_string());
/// assert_eq!(category.name(), "saas_rust");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateCategory(String);

impl TemplateCategory {
    /// Creates a new `TemplateCategory`.
    pub fn new(name: String) -> Self {
        Self(name)
    }

    /// Returns the name of the template category.
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// Metadata about a template, including its path, checksum, and last updated timestamp.
///
/// # Example
/// ```rust,no_run
/// use template_teleporter_developer_platforms::TemplateMetadata;
/// use chrono::Utc;
/// let metadata = TemplateMetadata::new("/path/to/template".to_string(), "checksum123".to_string(), Utc::now());
/// assert_eq!(metadata.path(), "/path/to/template");
/// assert_eq!(metadata.checksum(), "checksum123");
/// ```
#[derive(Debug, Clone)]
pub struct TemplateMetadata {
    path: TemplatePath,
    checksum: String,
    last_updated: DateTime<Utc>,
}

impl TemplateMetadata {
    /// Creates a new `TemplateMetadata`.
    pub fn new(path: TemplatePath, checksum: String, last_updated: DateTime<Utc>) -> Self {
        Self {
            path,
            checksum,
            last_updated,
        }
    }

    /// Returns the path of the template.
    pub fn path(&self) -> &TemplatePath {
        &self.path
    }

    /// Returns the checksum of the template.
    pub fn checksum(&self) -> &str {
        &self.checksum
    }

    /// Returns the last updated timestamp of the template.
    pub fn last_updated(&self) -> &DateTime<Utc> {
        &self.last_updated
    }
}

/// Information about a repository, including its organization, name, and default branch.
///
/// # Example
/// ```rust,no_run
/// use template_teleporter_developer_platforms::RepoInfo;
/// let repo_info = RepoInfo::new("org".to_string(), "repo".to_string(), "main".to_string());
/// assert_eq!(repo_info.org(), "org");
/// assert_eq!(repo_info.name(), "repo");
/// assert_eq!(repo_info.default_branch(), "main");
/// ```
#[derive(Debug, Clone)]
pub struct RepoInfo {
    org: String,
    name: String,
    default_branch: String,
}

impl RepoInfo {
    /// Creates a new `RepoInfo`.
    pub fn new(org: String, name: String, default_branch: String) -> Self {
        Self {
            org,
            name,
            default_branch,
        }
    }

    /// Returns the organization of the repository.
    pub fn org(&self) -> &str {
        &self.org
    }

    /// Returns the name of the repository.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the default branch of the repository.
    pub fn default_branch(&self) -> &str {
        &self.default_branch
    }
}

/// Represents a change to a template, including its path, old checksums, new checksum, and content.
///
/// # Example
/// ```rust,no_run
/// use template_teleporter_developer_platforms::TemplateChange;
/// let change = TemplateChange::new(
///     "/path/to/template".to_string(),
///     vec!["old_checksum1".to_string(), "old_checksum2".to_string()],
///     "new_checksum".to_string(),
///     vec![1, 2, 3],
/// );
/// assert_eq!(change.path(), "/path/to/template");
/// assert_eq!(change.new_checksum(), "new_checksum");
/// assert_eq!(change.content(), &vec![1, 2, 3]);
/// ```
#[derive(Debug, Clone)]
pub struct TemplateChange {
    path: TemplatePath,
    old_checksum: Vec<String>,
    new_checksum: String,
    content: Vec<u8>,
}

impl TemplateChange {
    /// Creates a new `TemplateChange`.
    pub fn new(
        path: TemplatePath,
        old_checksum: Vec<String>,
        new_checksum: String,
        content: Vec<u8>,
    ) -> Self {
        Self {
            path,
            old_checksum,
            new_checksum,
            content,
        }
    }

    /// Returns the path of the template change.
    pub fn path(&self) -> &TemplatePath {
        &self.path
    }

    /// Returns an iterator over the old checksums of the template change.
    pub fn old_checksums(&self) -> impl Iterator<Item = &String> {
        self.old_checksum.iter()
    }

    /// Returns the old checksum at the specified index, if it exists.
    pub fn old_checksum_at(&self, index: usize) -> Option<&String> {
        self.old_checksum.get(index)
    }

    /// Returns the number of old checksums available.
    pub fn old_checksum_count(&self) -> usize {
        self.old_checksum.len()
    }

    /// Returns the new checksum of the template change.
    pub fn new_checksum(&self) -> &str {
        &self.new_checksum
    }

    /// Returns the content of the template change.
    pub fn content(&self) -> &Vec<u8> {
        &self.content
    }
}

/// The result of updating a repository, including the pull request URL, number, and updated files.
///
/// # Example
/// ```rust,no_run
/// use template_teleporter_developer_platforms::UpdateResult;
/// let result = UpdateResult::new(
///     "https://pr.url".to_string(),
///     42,
///     vec!["/path/to/file".to_string()],
/// );
/// assert_eq!(result.pr_url(), "https://pr.url");
/// assert_eq!(result.pr_number(), 42);
/// ```
#[derive(Debug, Serialize)]
pub struct UpdateResult {
    pr_url: String,
    pr_number: u64,
    updated_files: Vec<TemplatePath>,
}

impl UpdateResult {
    /// Creates a new `UpdateResult`.
    pub fn new(pr_url: String, pr_number: u64, updated_files: Vec<TemplatePath>) -> Self {
        Self {
            pr_url,
            pr_number,
            updated_files,
        }
    }

    /// Returns the pull request URL of the update result.
    pub fn pr_url(&self) -> &str {
        &self.pr_url
    }

    /// Returns the pull request number of the update result.
    pub fn pr_number(&self) -> u64 {
        self.pr_number
    }

    /// Returns the updated files of the update result.
    pub fn updated_files(&self) -> &Vec<TemplatePath> {
        &self.updated_files
    }
}

/// A trait for interacting with developer platforms, such as GitHub.
///
/// This trait provides methods for listing categories, fetching templates, listing repositories,
/// detecting changes, and applying updates.
#[async_trait]
pub trait DeveloperPlatform: Send + Sync {
    /// Get all defined template categories from the master configuration.
    ///
    /// # Returns
    /// A `Result` containing a vector of `TemplateCategory` instances if successful, or a `PlatformError` otherwise.
    async fn list_categories(&self) -> Result<Vec<TemplateCategory>, PlatformError>;

    /// Get template content from the master repository for a specific category and path.
    ///
    /// # Parameters
    /// - `category`: A reference to the `TemplateCategory` to fetch the template from.
    /// - `path`: A reference to the `TemplatePath` specifying the location of the template.
    ///
    /// # Returns
    /// A `Result` containing the template content as a vector of bytes if successful, or a `PlatformError` otherwise.
    async fn get_template(
        &self,
        category: &TemplateCategory,
        path: &TemplatePath,
    ) -> Result<Vec<u8>, PlatformError>;

    /// List all template metadata (path, checksum, last updated) for a given category
    /// in the master repository.
    ///
    /// # Parameters
    /// - `category`: A reference to the `TemplateCategory` to list metadata for.
    ///
    /// # Returns
    /// A `Result` containing a vector of `TemplateMetadata` instances if successful, or a `PlatformError` otherwise.
    async fn list_templates(
        &self,
        category: &TemplateCategory,
    ) -> Result<Vec<TemplateMetadata>, PlatformError>;

    /// List all target repositories configured to use a specific template category.
    ///
    /// # Parameters
    /// - `category`: A reference to the `TemplateCategory` to list repositories for.
    ///
    /// # Returns
    /// A `Result` containing a vector of `RepoInfo` instances if successful, or a `PlatformError` otherwise.
    async fn list_repos_by_category(
        &self,
        category: &TemplateCategory,
    ) -> Result<Vec<RepoInfo>, PlatformError>;

    /// Determine which templates within a category have changed in the master repository
    /// since a given commit SHA.
    ///
    /// # Parameters
    /// - `category`: A reference to the `TemplateCategory` to check for updates.
    /// - `since_commit`: A string slice representing the commit SHA to compare against.
    ///
    /// # Returns
    /// A `Result` containing a vector of `TemplateChange` instances if successful, or a `PlatformError` otherwise.
    async fn get_updated_templates(
        &self,
        category: &TemplateCategory,
        since_commit: &str,
    ) -> Result<Vec<TemplateChange>, PlatformError>;

    /// Apply template changes to a target repository: create a branch, commit changes,
    /// create a pull request, and return the PR details.
    ///
    /// # Parameters
    /// - `repo`: A reference to the `RepoInfo` representing the target repository.
    /// - `changes`: A slice of `TemplateChange` instances representing the changes to apply.
    ///
    /// # Returns
    /// A `Result` containing an `UpdateResult` instance if successful, or a `PlatformError` otherwise.
    async fn update_repo(
        &self,
        repo: &RepoInfo,
        changes: &[TemplateChange],
    ) -> Result<UpdateResult, PlatformError>;
}
