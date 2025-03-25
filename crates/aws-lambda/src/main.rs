use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};
use octocrab::{Octocrab, models};
use base64::{Engine as _, engine::general_purpose};
use serde_json::{json, Value};
use http::header::HeaderMap;
use futures::stream::{self, StreamExt};
use std::env;

// Configuration constants
const MASTER_REPO_OWNER: &str = "your-org";
const MASTER_REPO_NAME: &str = "template-masters";
const CONFIG_PATH: &str = "config.yaml";

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    categories: Vec<Category>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Category {
    name: String,
    topics: Vec<String>,
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebhookPayload {
    repository: Repository,
    #[serde(default)]
    commits: Vec<Commit>,
    #[serde(rename = "ref")]
    git_ref: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Repository {
    name: String,
    full_name: String,
    owner: Owner,
    default_branch: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Owner {
    login: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Commit {
    added: Vec<String>,
    modified: Vec<String>,
    removed: Vec<String>,
    message: String,
    committer: Option<Committer>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Committer {
    date: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TemplateFile {
    path: String,
    sha: String,
    content: Option<String>,
    encoding: Option<String>,
}

// Main Lambda handler
async fn function_handler(event: LambdaEvent<ApiGatewayProxyRequest>) -> Result<ApiGatewayProxyResponse, Error> {
    // Extract webhook data
    let payload = event.payload;
    let headers = payload.headers;
    let body = payload.body.unwrap_or_default();

    // Parse event type from headers
    let event_type = headers.get("X-GitHub-Event")
        .or_else(|| headers.get("x-github-event"))
        .map(|h| h.to_string())
        .unwrap_or_default();

    // Parse webhook payload
    let webhook_payload: WebhookPayload = serde_json::from_str(&body)?;

    // Initialize GitHub client
    let octocrab = get_authenticated_client().await?;

    // Process based on event type
    match event_type.as_str() {
        "push" => {
            if is_template_push(&webhook_payload) {
                sync_templates(&octocrab, &webhook_payload).await?;
            }
        },
        "repository" | "repository.edited" => {
            check_and_apply_templates(&octocrab, &webhook_payload.repository).await?;
        },
        _ => {
            println!("Ignoring unsupported event type: {}", event_type);
        }
    }

    // Return success response
    Ok(ApiGatewayProxyResponse {
        status_code: 200,
        headers: Default::default(),
        multi_value_headers: Default::default(),
        body: Some(json!({"message": "Processed successfully"}).to_string()),
        is_base64_encoded: Some(false),
    })
}

// Initialize the GitHub client with app authentication
async fn get_authenticated_client() -> Result<Octocrab, Error> {
    // In production, you'd use a proper GitHub App JWT authentication
    // This is simplified for example purposes
    let token = env::var("GITHUB_APP_TOKEN")?;
    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()?;

    Ok(octocrab)
}

// Check if a push event affected template files
fn is_template_push(payload: &WebhookPayload) -> bool {
    if payload.repository.full_name != format!("{}/{}", MASTER_REPO_OWNER, MASTER_REPO_NAME) {
        return false;
    }

    // Check if the push is to the main branch
    if let Some(ref_name) = &payload.git_ref {
        if !ref_name.ends_with(&format!("refs/heads/{}", payload.repository.default_branch)) {
            return false;
        }
    }

    // Check if any commits modified template files
    payload.commits.iter().any(|commit| {
        commit.added.iter().any(|file| file.starts_with("templates/")) ||
        commit.modified.iter().any(|file| file.starts_with("templates/")) ||
        commit.removed.iter().any(|file| file.starts_with("templates/"))
    })
}

// Main function to sync templates to repositories
async fn sync_templates(octocrab: &Octocrab, payload: &WebhookPayload) -> Result<(), Error> {
    println!("Starting template sync process");

    // Get configuration
    let config = fetch_configuration(octocrab).await?;

    // Find which categories were updated
    let updated_categories = find_updated_categories(&payload.commits, &config);

    if updated_categories.is_empty() {
        println!("No template categories were updated");
        return Ok(());
    }

    println!("Found {} updated categories", updated_categories.len());

    // For each updated category, find matching repositories and update them
    for category in updated_categories {
        println!("Processing category: {}", category.name);
        let repos = find_repositories_by_category(octocrab, &category).await?;

        println!("Found {} matching repositories", repos.len());

        for repo in repos {
            match update_repository_templates(octocrab, &repo, &category).await {
                Ok(_) => println!("Successfully updated templates for {}", repo.full_name),
                Err(e) => println!("Error updating templates for {}: {}", repo.full_name, e),
            }
        }
    }

    Ok(())
}

// Fetch configuration from master repository
async fn fetch_configuration(octocrab: &Octocrab) -> Result<Config, Error> {
    let content = octocrab
        .repos(MASTER_REPO_OWNER, MASTER_REPO_NAME)
        .get_content()
        .path(CONFIG_PATH)
        .send()
        .await?;

    // Get the first item (the file)
    let file = content.items.first().ok_or_else(|| Error::from("Config file not found"))?;

    // Decode content
    let content = if let Some(content) = &file.content {
        let content = content.replace("\n", "");
        general_purpose::STANDARD.decode(content)?
    } else {
        return Err(Error::from("Empty config file"));
    };

    // Parse YAML
    let config: Config = serde_yaml::from_slice(&content)?;

    Ok(config)
}

// Find which categories were updated based on commit changes
fn find_updated_categories(commits: &[Commit], config: &Config) -> Vec<Category> {
    let mut changed_paths = HashSet::new();

    // Collect all changed file paths
    for commit in commits {
        for path in &commit.added {
            changed_paths.insert(path);
        }
        for path in &commit.modified {
            changed_paths.insert(path);
        }
        for path in &commit.removed {
            changed_paths.insert(path);
        }
    }

    // Find categories that match changed paths
    config.categories
        .iter()
        .filter(|category| {
            let category_prefix = &category.path;
            changed_paths.iter().any(|path| path.starts_with(category_prefix))
        })
        .cloned()
        .collect()
}

// Find repositories that match a category based on topics
async fn find_repositories_by_category(octocrab: &Octocrab, category: &Category) -> Result<Vec<Repository>, Error> {
    let mut all_repos = Vec::new();
    let mut repo_ids = HashSet::new();

    // Search repos for each topic
    for topic in &category.topics {
        let query = format!("topic:{} org:{}", topic, MASTER_REPO_OWNER);

        let search = octocrab.search()
            .repositories(&query)
            .send()
            .await?;

        for repo in search.items {
            // Use string formatting to re-create our Repository struct
            // This is a bit of a hack but works for our example
            let repo_json = serde_json::to_value(&repo)?;
            if let Ok(our_repo) = serde_json::from_value::<Repository>(repo_json) {
                let repo_id = format!("{}", repo.id);
                if !repo_ids.contains(&repo_id) {
                    repo_ids.insert(repo_id);
                    all_repos.push(our_repo);
                }
            }
        }
    }

    Ok(all_repos)
}

// Update templates in a repository
async fn update_repository_templates(octocrab: &Octocrab, repo: &Repository, category: &Category) -> Result<(), Error> {
    println!("Updating templates for {}", repo.full_name);

    // Get template files for this category
    let template_files = get_template_files(octocrab, category).await?;

    // Create a branch for the changes
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    let branch_name = format!("template-sync-{}", timestamp);

    let branch_created = match create_branch(octocrab, repo, &branch_name).await {
        Ok(_) => true,
        Err(e) => {
            println!("Failed to create branch: {}", e);
            false
        }
    };

    if !branch_created {
        return Err(Error::from("Failed to create branch"));
    }

    let mut updated_any = false;

    // Update each template file
    for template_file in &template_files {
        let target_path = template_file.path.replace(&format!("{}/", category.path), "");

        // Check if the file exists and is customized
        let existing_file = octocrab
            .repos(&repo.owner.login, &repo.name)
            .get_content()
            .path(&target_path)
            .send()
            .await;

        let should_update = match existing_file {
            Ok(content) => {
                // Check if file exists and if it's been customized
                let file = content.items.first().ok_or_else(|| Error::from("File not found"))?;
                !is_template_customized(octocrab, repo, file, template_file).await?
            },
            Err(_) => {
                // File doesn't exist, so create it
                true
            }
        };

        if should_update {
            match create_or_update_file(octocrab, repo, &target_path, template_file, &branch_name).await {
                Ok(_) => {
                    println!("Updated {} in {}", target_path, repo.full_name);
                    updated_any = true;
                },
                Err(e) => println!("Error updating {}: {}", target_path, e),
            }
        } else {
            println!("Skipping customized template {} in {}", target_path, repo.full_name);
        }
    }

    // Create a PR if any files were updated
    if updated_any {
        match create_pull_request(octocrab, repo, category, &branch_name).await {
            Ok(_) => println!("Created PR in {}", repo.full_name),
            Err(e) => println!("Error creating PR in {}: {}", repo.full_name, e),
        }
    } else {
        // Clean up branch if no updates were made
        println!("No files were updated, cleaning up branch");
        let _ = octocrab
            .repos(&repo.owner.login, &repo.name)
            .git
            .delete_reference(&format!("heads/{}", branch_name))
            .await;
    }

    Ok(())
}

// Get all template files for a category
async fn get_template_files(octocrab: &Octocrab, category: &Category) -> Result<Vec<TemplateFile>, Error> {
    let mut template_files = Vec::new();

    // Helper function to recursively fetch files
    async fn fetch_files_recursively(
        octocrab: &Octocrab,
        path: &str,
        template_files: &mut Vec<TemplateFile>
    ) -> Result<(), Error> {
        let content = octocrab
            .repos(MASTER_REPO_OWNER, MASTER_REPO_NAME)
            .get_content()
            .path(path)
            .send()
            .await?;

        for item in content.items {
            if item.r#type == Some("file".to_string()) {
                template_files.push(TemplateFile {
                    path: item.path.unwrap_or_default(),
                    sha: item.sha.unwrap_or_default(),
                    content: None,
                    encoding: None,
                });
            } else if item.r#type == Some("dir".to_string()) {
                if let Some(item_path) = &item.path {
                    fetch_files_recursively(octocrab, item_path, template_files).await?;
                }
            }
        }

        Ok(())
    }

    fetch_files_recursively(octocrab, &category.path, &mut template_files).await?;

    Ok(template_files)
}

// Check if a template has been customized
async fn is_template_customized(
    octocrab: &Octocrab,
    repo: &Repository,
    existing_file: &models::content::ContentItem,
    master_template: &TemplateFile
) -> Result<bool, Error> {
    // Get the path from the existing file
    let file_path = existing_file.path.as_ref().ok_or_else(|| Error::from("No path found"))?;

    // Check commit history for this file
    let commits = octocrab
        .repos(&repo.owner.login, &repo.name)
        .list_commits()
        .path(file_path)
        .send()
        .await?;

    // Check if we've updated this file before
    let app_commits: Vec<_> = commits.items.iter()
        .filter(|commit| {
            if let Some(message) = &commit.commit.message {
                message.contains("[Template Sync]")
            } else {
                false
            }
        })
        .collect();

    if !app_commits.is_empty() {
        // Get the last time we updated it
        let last_app_commit = &app_commits[0];

        if let Some(last_date) = &last_app_commit.commit.committer.date {
            let last_update = DateTime::parse_from_rfc3339(last_date)
                .map_err(|_| Error::from("Invalid date format"))?;

            // Check if there are any commits after our last update
            let custom_commits = commits.items.iter()
                .filter(|commit| {
                    if let (Some(message), Some(date)) = (&commit.commit.message, &commit.commit.committer.date) {
                        if let Ok(commit_date) = DateTime::parse_from_rfc3339(date) {
                            return commit_date > last_update && !message.contains("[Template Sync]");
                        }
                    }
                    false
                })
                .count();

            return Ok(custom_commits > 0);
        }
    }

    // Get content from master template
    let master_content = octocrab
        .repos(MASTER_REPO_OWNER, MASTER_REPO_NAME)
        .get_content()
        .path(&master_template.path)
        .send()
        .await?;

    let master_item = master_content.items.first()
        .ok_or_else(|| Error::from("Master template file not found"))?;

    // Compare content
    let master_content = if let Some(content) = &master_item.content {
        let content = content.replace("\n", "");
        String::from_utf8(general_purpose::STANDARD.decode(content)?)?
    } else {
        return Err(Error::from("Empty master template"));
    };

    let existing_content = if let Some(content) = &existing_file.content {
        let content = content.replace("\n", "");
        String::from_utf8(general_purpose::STANDARD.decode(content)?)?
    } else {
        return Err(Error::from("Empty existing file"));
    };

    Ok(master_content != existing_content)
}

// Create or update a file
async fn create_or_update_file(
    octocrab: &Octocrab,
    repo: &Repository,
    target_path: &str,
    template_file: &TemplateFile,
    branch_name: &str
) -> Result<(), Error> {
    // Get the template content
    let master_content = octocrab
        .repos(MASTER_REPO_OWNER, MASTER_REPO_NAME)
        .get_content()
        .path(&template_file.path)
        .send()
        .await?;

    let master_item = master_content.items.first()
        .ok_or_else(|| Error::from("Master template file not found"))?;

    let content = if let Some(content) = &master_item.content {
        content.replace("\n", "")
    } else {
        return Err(Error::from("Empty master template"));
    };

    // Check if file exists to get SHA
    let existing_sha = match octocrab
        .repos(&repo.owner.login, &repo.name)
        .get_content()
        .path(target_path)
        .send()
        .await
    {
        Ok(content) => {
            content.items.first()
                .and_then(|item| item.sha.clone())
        },
        Err(_) => None,
    };

    // Create or update file
    octocrab
        .repos(&repo.owner.login, &repo.name)
        .create_or_update_file(
            target_path,
            &format!("[Template Sync] Update {}", target_path),
            &content,
        )
        .branch(branch_name)
        .sha(existing_sha.as_deref())
        .send()
        .await?;

    Ok(())
}

// Create a branch for changes
async fn create_branch(octocrab: &Octocrab, repo: &Repository, branch_name: &str) -> Result<(), Error> {
    // Get default branch reference
    let default_ref = octocrab
        .repos(&repo.owner.login, &repo.name)
        .get_ref(&format!("heads/{}", repo.default_branch))
        .await?;

    // Create new branch
    octocrab
        .repos(&repo.owner.login, &repo.name)
        .create_ref(&format!("refs/heads/{}", branch_name), &default_ref.object.sha)
        .await?;

    Ok(())
}

// Create a PR for the changes
async fn create_pull_request(
    octocrab: &Octocrab,
    repo: &Repository,
    category: &Category,
    branch_name: &str
) -> Result<(), Error> {
    octocrab
        .pulls(&repo.owner.login, &repo.name)
        .create(&format!("[Template Sync] Update templates for {}", category.name),
                branch_name,
                &repo.default_branch)
        .body(&format!(
            "This PR updates the issue and PR templates to match the latest version from the master templates repository.\n\n\
            ## Changes\n\
            - Updated templates from the {} category\n\n\
            This PR was automatically generated by the Template Sync GitHub App.",
            category.name
        ))
        .send()
        .await?;

    Ok(())
}

// Check and apply templates to newly created or updated repository
async fn check_and_apply_templates(octocrab: &Octocrab, repository: &Repository) -> Result<(), Error> {
    // Get configuration
    let config = fetch_configuration(octocrab).await?;

    // Get repository topics
    let topics_response = octocrab
        .repos(&repository.owner.login, &repository.name)
        .list_topics()
        .await?;

    let repo_topics = topics_response.names;

    // Find matching category
    let matching_category = config.categories.iter()
        .find(|category| {
            category.topics.iter().any(|topic| repo_topics.contains(topic))
        })
        .cloned();

    if let Some(category) = matching_category {
        println!("Repository {} matches category {}", repository.full_name, category.name);
        update_repository_templates(octocrab, repository, &category).await?;
    } else {
        println!("Repository {} doesn't match any template category", repository.full_name);
    }

    Ok(())
}

// Lambda entry point
#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(function_handler)).await
}
