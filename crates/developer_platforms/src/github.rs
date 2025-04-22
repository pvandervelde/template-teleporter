//! Implements the `DeveloperPlatform` trait for GitHub using the `octocrab` crate.

use crate::{
    auth::{create_github_app_client, create_github_installation_client},
    error::{PlatformError, Result},
    types::{
        RepositoryInfo, TemplateCategory, TemplateChange, TemplateMetadata, TemplatePath,
        UpdateResult,
    },
    DeveloperPlatform,
    utils::calculate_checksum, // Added import for checksum utility
};
use async_trait::async_trait;
use log::{debug, error, info, instrument, warn}; // Added error, instrument
use octocrab::{models::{repos::RepoCommit, pulls::PullRequest, git::{Blob, Commit, Tree, TreeEntry}}, params::{self, git::CreateTree, pulls::Sort}, Octocrab}; // Added git models and params
use serde::Deserialize; // For parsing config file

// TODO: Move config parsing to core or a dedicated config module?
// For now, define needed structs here.
#[derive(Deserialize, Debug)]
struct MasterConfig {
    #[serde(default)]
    categories: std::collections::HashMap<String, CategoryConfig>,
    #[serde(default)]
    repositories: std::collections::HashMap<String, RepositoryTargetConfig>,
}

#[derive(Deserialize, Debug)]
struct CategoryConfig {
    description: Option<String>,
    files: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct RepositoryTargetConfig {
    category: String,
}


/// Configuration required to initialize the GitHub client.
#[derive(Clone, Debug)] // Clone needed if client holds it
pub struct GitHubConfig {
    pub app_id: u64,
    pub private_key_pem: String,
    pub master_repository_owner: String,
    pub master_repository_name: String,
    // Add other config like base URL for GitHub Enterprise if needed
}

/// Client for interacting with GitHub as a developer platform.
#[derive(Debug)] // Avoid Clone unless necessary and safe (Octocrab might not be easily cloneable)
pub struct GitHubClient {
    config: GitHubConfig,
    /// Octocrab client authenticated as the GitHub App itself.
    app_client: Octocrab,
}

impl GitHubClient {
    /// Creates a new `GitHubClient`.
    ///
    /// Initializes authentication with GitHub using the provided App ID and private key.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration containing App credentials and master repo details.
    ///
    /// # Returns
    ///
    /// A `Result` containing the initialized `GitHubClient` or a `PlatformError`.
    pub async fn new(config: GitHubConfig) -> Result<Self> {
        info!(
            "Initializing GitHubClient for App ID: {}",
            config.app_id
        );
        let app_client =
            create_github_app_client(config.app_id, &config.private_key_pem).await?;
        Ok(Self { config, app_client })
    }

    /// Helper function to get an Octocrab client authenticated for a specific installation.
    /// Caches clients based on installation ID? Maybe later.
    async fn get_installation_client(&self, installation_id: u64) -> Result<Octocrab> {
        // Pass None for repo context as this client might be used for multiple repos
        create_github_installation_client(&self.app_client, installation_id, None, None).await
    }

    /// Helper function to fetch and parse the master configuration file.
    async fn get_master_config(&self) -> Result<MasterConfig> {
        debug!(
            "Fetching master config from {}/{}",
            self.config.master_repository_owner, self.config.master_repository_name
        );

        // Need installation ID for the master repo. How do we get this?
        // Option 1: Assume a fixed installation ID for the master repo (passed in config?)
        // Option 2: Find installation ID dynamically based on repo owner/name (requires app auth)
        // Let's assume we need to find it dynamically for now.

        let installation = self
            .app_client
            .apps()
            .get_repository_installation(&self.config.master_repository_owner, &self.config.master_repository_name)
            .await
            .map_err(|e| PlatformError::ApiError(format!("Failed to get installation for master repo: {}", e)))?;

        let installation_client = self.get_installation_client(installation.id.into_inner()).await?;


        let content_items = installation_client
            .repos(&self.config.master_repository_owner, &self.config.master_repository_name)
            .get_content()
            .path("template-teleporter.toml")
            // .r#ref("main") // TODO: Use default branch or make configurable
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("Failed to fetch template-teleporter.toml: {}", e)))?;

        let first_item = content_items.items.into_iter().next().ok_or_else(|| {
            PlatformError::ConfigError("template-teleporter.toml not found in master repo".to_string())
        })?;

        let content_base64 = first_item.content.ok_or_else(|| {
            PlatformError::ConfigError(
                "template-teleporter.toml content is empty or not available".to_string(),
            )
        })?;

        let content_bytes = base64::decode(content_base64.replace('\n', "")).map_err(|e| {
            PlatformError::InvalidContent(format!(
                "Failed to decode template-teleporter.toml base64 content: {}",
                e
            ))
        })?;

        let content_str = String::from_utf8(content_bytes).map_err(|e| {
            PlatformError::InvalidContent(format!(
                "template-teleporter.toml content is not valid UTF-8: {}",
                e
            ))
        })?;

        let config: MasterConfig = toml::from_str(&content_str).map_err(|e| {
            PlatformError::ConfigError(format!(
                "Failed to parse template-teleporter.toml: {}",
                e
            ))
        })?;

        Ok(config)
    }
}

#[async_trait]
impl DeveloperPlatform for GitHubClient {
    /// Get all defined template categories from the master configuration file.
    #[instrument(skip(self))]
    async fn list_categories(&self) -> Result<Vec<TemplateCategory>> {
        let master_config = self.get_master_config().await?;
        let categories = master_config
            .categories
            .keys()
            .map(|k| TemplateCategory(k.clone()))
            .collect();
        Ok(categories)
    }

    /// List all target repositories configured to use a specific template category.
    #[instrument(skip(self), fields(category = ?category))]
    async fn list_repos_by_category(
        &self,
        category: &TemplateCategory,
    ) -> Result<Vec<RepositoryInfo>> {
         let master_config = self.get_master_config().await?;
         let mut repos = Vec::new();

         for (repo_full_name, repo_config) in master_config.repositories {
             if repo_config.category == category.0 {
                 // Split "owner/name" into parts
                 let parts: Vec<&str> = repo_full_name.split('/').collect();
                 if parts.len() == 2 {
                     // TODO: How to get the default branch efficiently?
                     // Need to make another API call per repo, or assume 'main'/'master'?
                     // For now, let's placeholder it. This might be expensive.
                     warn!("Default branch lookup not implemented yet for {}. Using placeholder.", repo_full_name);
                     repos.push(RepositoryInfo {
                         org: parts[0].to_string(),
                         name: parts[1].to_string(),
                         default_branch: "main".to_string(), // Placeholder!
                     });
                 } else {
                     warn!("Invalid repository name format in config: {}", repo_full_name);
                 }
             }
         }
         Ok(repos)
    }


    // --- Placeholder implementations for other methods ---

    #[instrument(skip(self), fields(category = ?category, path = path))]
    async fn get_template(
        &self,
        category: &TemplateCategory,
        path: &TemplatePath,
    ) -> Result<Vec<u8>> {
        debug!(
            "Fetching template content for category '{}', path '{}' from master repo {}/{}",
            category.0, path, self.config.master_repository_owner, self.config.master_repository_name
        );

        // 1. Get installation ID for the master repo
        let installation = self
            .app_client
            .apps()
            .get_repository_installation(
                &self.config.master_repository_owner,
                &self.config.master_repository_name,
            )
            .await
            .map_err(|e| {
                PlatformError::ApiError(format!(
                    "Failed to get installation for master repo: {}",
                    e
                ))
            })?;
        let installation_id = installation.id.into_inner();
        debug!("Found installation ID {} for master repo", installation_id);

        // 2. Get installation client for master repo
        let installation_client = self.get_installation_client(installation_id).await?;

        // 3. Construct full path within the master repo
        let full_path = format!("templates/{}/{}", category.0, path);
        debug!("Constructed full path in master repo: {}", full_path);

        // 4. Fetch content using the installation client
        // TODO: Determine the correct ref (default branch?) dynamically or from config
        let content_items_result = installation_client
            .repos(
                &self.config.master_repository_owner,
                &self.config.master_repository_name,
            )
            .get_content()
            .path(&full_path)
            // .r#ref("main") // Assuming 'main' for now, needs to be dynamic
            .send()
            .await;

        let content_items = match content_items_result {
            Ok(items) => items,
            Err(octocrab::Error::GitHub { source, .. })
                if source.status_code == http::StatusCode::NOT_FOUND =>
            {
                warn!("Template not found at path '{}': {}", full_path, source.message);
                return Err(PlatformError::TemplateNotFound(path.clone()));
            }
            Err(e) => {
                error!("Failed to fetch template content for path '{}': {}", full_path, e);
                return Err(PlatformError::ApiError(format!(
                    "Failed to fetch template content '{}': {}",
                    full_path, e
                )));
            }
        };


        // 5. Extract and decode base64 content
        let first_item = content_items.items.into_iter().next().ok_or_else(|| {
            warn!("No content item found for path '{}', though API call succeeded.", full_path);
            PlatformError::TemplateNotFound(path.clone()) // Treat as not found if no item returned
        })?;

        let content_base64 = first_item.content.ok_or_else(|| {
            PlatformError::InvalidContent(format!(
                "Content is empty or not available for template path '{}'",
                full_path
            ))
        })?;

        let content_bytes = base64::decode(content_base64.replace('\n', "")).map_err(|e| {
            PlatformError::InvalidContent(format!(
                "Failed to decode base64 content for template path '{}': {}",
                full_path, e
            ))
        })?;

        info!("Successfully fetched template content for path '{}'", full_path);
        Ok(content_bytes)
    }

    #[instrument(skip(self), fields(category = ?category))]
    async fn list_templates(
        &self,
        category: &TemplateCategory,
    ) -> Result<Vec<TemplateMetadata>> {
        debug!("Listing templates for category '{}' from master config", category.0);

        // 1. Get master config
        let master_config = self.get_master_config().await?;

        // 2. Find category entry
        let category_config = master_config
            .categories
            .get(&category.0)
            .ok_or_else(|| PlatformError::CategoryNotFound(category.clone()))?;

        // 3. Create TemplateMetadata for each file listed in the config
        let mut metadata_list = Vec::new();
        for file_path in &category_config.files {
            // TODO: Implement accurate checksum calculation.
            // This currently requires fetching each file's content via get_template.
            // Consider optimizing later, perhaps by fetching directory listings or using Git APIs.
            let checksum_placeholder = "TODO_CHECKSUM".to_string();
            warn!(
                "Using placeholder checksum for template '{}' in category '{}'",
                file_path, category.0
            );

            // TODO: Implement accurate last_updated timestamp fetching.
            // This requires complex Git history analysis (e.g., finding the last commit
            // that modified templates/{category.0}/{file_path}).
            let last_updated_placeholder = chrono::Utc::now(); // Placeholder
            warn!(
                "Using placeholder last_updated timestamp for template '{}' in category '{}'",
                file_path, category.0
            );


            metadata_list.push(TemplateMetadata {
                path: file_path.clone(),
                checksum: checksum_placeholder,
                last_updated: last_updated_placeholder,
            });
        }

        info!(
            "Found {} templates listed in config for category '{}'",
            metadata_list.len(),
            category.0
        );
        Ok(metadata_list)
    }

    #[instrument(skip(self), fields(category = ?category, since_commit = since_commit))]
    async fn get_updated_templates(
        &self,
        category: &TemplateCategory,
        since_commit: &str,
    ) -> Result<Vec<TemplateChange>> {
        debug!(
            "Getting updated templates for category '{}' since commit '{}'",
            category.0, since_commit
        );

        // 1. Get installation ID and client for the master repo
        let installation = self
            .app_client
            .apps()
            .get_repository_installation(
                &self.config.master_repository_owner,
                &self.config.master_repository_name,
            )
            .await
            .map_err(|e| {
                PlatformError::ApiError(format!(
                    "Failed to get installation for master repo: {}",
                    e
                ))
            })?;
        let installation_id = installation.id.into_inner();
        let installation_client = self.get_installation_client(installation_id).await?;

        // 2. Use GitHub compare API
        // TODO: Determine default branch dynamically instead of hardcoding "main"
        let default_branch = "main";
        let comparison_result = installation_client
            .repos(
                &self.config.master_repository_owner,
                &self.config.master_repository_name,
            )
            .compare_commits(since_commit, default_branch)
            .send()
            .await;

        let comparison = match comparison_result {
            Ok(comp) => comp,
            Err(e) => {
                error!(
                    "Failed to compare commits {}...{} in master repo: {}",
                    since_commit, default_branch, e
                );
                return Err(PlatformError::ApiError(format!(
                    "Failed to compare commits: {}",
                    e
                )));
            }
        };

        let mut changes = Vec::new();
        let category_prefix = format!("templates/{}/", category.0);

        // 3. Filter changed files based on category path prefix
        if let Some(files) = comparison.files {
            for file in files {
                if file.filename.starts_with(&category_prefix) {
                    // Ensure file status indicates a modification or addition
                    if file.status == "modified" || file.status == "added" || file.status == "renamed" { // Handle renames?
                        // Strip the prefix to get the path relative to the category
                        let relative_path = file.filename.strip_prefix(&category_prefix).unwrap_or(&file.filename).to_string();

                        debug!("Detected change in template: {}", relative_path);

                        // 4a. Get new content
                        // Use the existing get_template method for consistency
                        // Note: This makes an extra API call per changed file. Could optimize later
                        // by getting content directly from commit data if available/reliable.
                        let content_result = self.get_template(category, &relative_path).await;

                        match content_result {
                            Ok(content) => {
                                // 4a. Calculate new checksum
                                // Calculate new checksum using the utility function
                                let new_checksum = calculate_checksum(&content)?;
                                debug!("Calculated new checksum for '{}': {}", relative_path, new_checksum);


                                // 4b. Determine old checksum (requires state from core)
                                let old_checksums = Vec::new(); // Placeholder
                                warn!("Old checksums not available for changed template '{}' - requires state", relative_path);


                                // 4c. Create TemplateChange struct
                                changes.push(TemplateChange {
                                    path: relative_path,
                                    old_checksums,
                                    new_checksum,
                                    content,
                                });
                            }
                            Err(PlatformError::TemplateNotFound(_)) => {
                                // If get_template returns NotFound, it might mean the file was deleted
                                // in the range, but the compare API listed it differently. Log and skip.
                                warn!("Changed file '{}' reported by compare API but not found by get_template. Skipping.", relative_path);
                            }
                            Err(e) => {
                                // Log error fetching content but continue processing other files
                                error!("Failed to fetch content for changed template '{}': {}", relative_path, e);
                            }
                        }
                    } else if file.status == "removed" {
                         warn!("Template '{}' was removed. Handling removal is not yet implemented.", file.filename);
                         // TODO: How should removals be represented in TemplateChange or handled?
                    }
                }
            }
        } else {
            debug!("No files changed in the comparison range.");
        }

        info!(
            "Found {} updated templates for category '{}' since commit '{}'",
            changes.len(),
            category.0,
            since_commit
        );
        Ok(changes)
    }

    #[instrument(skip(self, changes), fields(repo = ?repo))]
    async fn update_repo(
        &self,
        repo: &RepositoryInfo,
        changes: &[TemplateChange],
    ) -> Result<UpdateResult> {
        info!(
            "Attempting to update repo {}/{} with {} template changes",
            repo.org,
            repo.name,
            changes.len()
        );

        if changes.is_empty() {
            warn!("update_repo called with no changes for {}/{}. Skipping.", repo.org, repo.name);
            // Or should this be an error? Returning OperationFailed for now.
            return Err(PlatformError::OperationFailed("No changes provided to update_repo".to_string()));
        }

        // 1. Find installation ID for the target repo
        let installation = self
            .app_client
            .apps()
            .get_repository_installation(&repo.org, &repo.name)
            .await
            .map_err(|e| {
                error!("Failed to get installation for target repo {}/{}: {}", repo.org, repo.name, e);
                PlatformError::ApiError(format!(
                    "Failed to get installation for target repo {}/{}: {}",
                    repo.org, repo.name, e
                ))
            })?;
        let installation_id = installation.id.into_inner();
        debug!("Found installation ID {} for target repo {}/{}", installation_id, repo.org, repo.name);


        // 2. Get installation client for target repo
        let installation_client = self.get_installation_client(installation_id).await?;


        // 3. Get default branch SHA
        // Use the default_branch provided in RepoInfo
        let default_branch_ref = format!("heads/{}", repo.default_branch);
        let ref_data = installation_client
            .git()
            .get_ref(&repo.org, &repo.name, &default_branch_ref)
            .await
            .map_err(|e| {
                error!("Failed to get ref {} for {}/{}: {}", default_branch_ref, repo.org, repo.name, e);
                PlatformError::ApiError(format!("Failed to get default branch ref: {}", e))
            })?;
        let default_branch_sha = ref_data.object.sha;
        debug!("Default branch '{}' SHA for {}/{}: {}", repo.default_branch, repo.org, repo.name, default_branch_sha);


        // 4. Define new branch name (e.g., template-teleporter-updates-timestamp)
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let new_branch_name = format!("template-teleporter-updates-{}", timestamp);
        let new_branch_ref = format!("refs/heads/{}", new_branch_name);


        // 5. Create new branch from default branch SHA using Git Refs API
        debug!("Creating new branch '{}' from SHA '{}'", new_branch_name, default_branch_sha);
        installation_client
            .git()
            .create_ref(&repo.org, &repo.name, &new_branch_ref, &default_branch_sha)
            .await
            .map_err(|e| {
                error!("Failed to create branch '{}' in {}/{}: {}", new_branch_name, repo.org, repo.name, e);
                PlatformError::ApiError(format!("Failed to create branch: {}", e))
            })?;
        info!("Successfully created branch '{}' in {}/{}", new_branch_name, repo.org, repo.name);


        // 6. Create commit with changes using Git Data API

        // 6a. Get the base commit to find the base tree SHA
        debug!("Getting base commit object for SHA: {}", default_branch_sha);
        let base_commit = installation_client
            .git()
            .get_commit(&repo.org, &repo.name, default_branch_sha.clone())
            .await
            .map_err(|e| PlatformError::ApiError(format!("Failed to get base commit: {}", e)))?;
        let base_tree_sha = base_commit.tree.sha;
        debug!("Base tree SHA: {}", base_tree_sha);


        // 6b. Create blobs for each change and collect tree entries
        let mut tree_entries = Vec::new();
        for change in changes {
            debug!("Creating blob for path: {}", change.path);
            let blob = installation_client
                .git()
                .create_blob(&repo.org, &repo.name, base64::encode(&change.content)) // Content needs to be base64 encoded
                .encoding("base64")
                .send()
                .await
                .map_err(|e| PlatformError::ApiError(format!("Failed to create blob for {}: {}", change.path, e)))?;

            debug!("Created blob SHA: {} for path: {}", blob.sha, change.path);

            tree_entries.push(
                CreateTree::Entry {
                    path: change.path.clone(),
                    mode: "100644".to_string(), // File mode
                    object_type: "blob".to_string(),
                    sha: Some(blob.sha),
                    content: None, // Content is in the blob, not directly in the tree entry
                }
            );
        }


        // 6c. Create a new tree object
        debug!("Creating new tree based on base tree SHA: {}", base_tree_sha);
        let new_tree = installation_client
            .git()
            .create_tree(&repo.org, &repo.name)
            .base_tree(&base_tree_sha)
            .tree(tree_entries)
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("Failed to create tree: {}", e)))?;
        debug!("Created new tree SHA: {}", new_tree.sha);


        // 6d. Create a new commit object
        let commit_message = format!(
            "chore: Apply template updates from template-teleporter ({})\n\n{}",
            timestamp,
            changes.iter().map(|c| format!("- Update {}", c.path)).collect::<Vec<_>>().join("\n")
        );
        debug!("Creating new commit with message: {}", commit_message);
        let new_commit = installation_client
            .git()
            .create_commit(&repo.org, &repo.name, &commit_message, &new_tree.sha)
            .parents(vec![default_branch_sha.clone()]) // Set parent commit
            // TODO: Set author/committer info?
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("Failed to create commit: {}", e)))?;
        let commit_sha = new_commit.sha; // Get the SHA of the newly created commit
        debug!("Created new commit SHA: {}", commit_sha);


        // 6e. Update the new branch ref to point to the new commit SHA
        debug!("Updating ref '{}' to point to commit SHA '{}'", new_branch_ref, commit_sha);
        installation_client
            .git()
            .update_ref(&repo.org, &repo.name, &new_branch_ref, &commit_sha)
            // .force(false) // Ensure we don't force push, though it shouldn't matter for a new branch
            .send()
            .await
            .map_err(|e| PlatformError::ApiError(format!("Failed to update ref {}: {}", new_branch_ref, e)))?;
        info!("Successfully updated branch '{}' to commit '{}'", new_branch_name, commit_sha);


        // 7. Create Pull Request
        let pr_title = format!("chore: Update templates from template-teleporter ({})", timestamp);
        let pr_body = format!(
            "Automated template updates applied by Template Teleporter.\n\nChanges applied:\n{}",
            changes.iter().map(|c| format!("- `{}`", c.path)).collect::<Vec<_>>().join("\n")
        );
        debug!("Creating PR in {}/{} from {} to {}", repo.org, repo.name, new_branch_name, repo.default_branch);

        let pull_request = installation_client
            .pulls(&repo.org, &repo.name)
            .create(&pr_title, &new_branch_name, &repo.default_branch)
            .body(&pr_body)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to create pull request in {}/{}: {}", repo.org, repo.name, e);
                PlatformError::ApiError(format!("Failed to create pull request: {}", e))
            })?;
        info!("Successfully created Pull Request #{} in {}/{}", pull_request.number, repo.org, repo.name);


        // 8. Return UpdateResult
        Ok(UpdateResult {
            pr_url: pull_request.html_url.map(|u| u.to_string()).unwrap_or_default(),
            pr_number: pull_request.number,
            updated_files: changes.iter().map(|c| c.path.clone()).collect(),
        })
    }
}
