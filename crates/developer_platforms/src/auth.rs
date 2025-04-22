//! Handles authentication logic for GitHub using `octocrab`.

use crate::error::{PlatformError, Result};
use jsonwebtoken::EncodingKey;
use log::{debug, error, info, instrument}; // Added instrument for consistency if tracing is used later
use octocrab::{models::AppId, Octocrab};

/// Creates an `Octocrab` client authenticated as a GitHub App.
///
/// This uses the App ID and private key to authenticate directly with GitHub
/// via `octocrab`. This client can then be used to get installation-specific clients.
///
/// # Arguments
///
/// * `app_id` - The ID of the GitHub App.
/// * `private_key_pem` - The PEM-encoded private key for the GitHub App.
///
/// # Returns
///
/// A `Result` containing an `Octocrab` client authenticated as the App,
/// or a `PlatformError`.
#[instrument(skip(private_key_pem), fields(app_id = app_id))]
pub async fn create_github_app_client(app_id: u64, private_key_pem: &str) -> Result<Octocrab> {
    // Create an encoding key from the PEM-encoded private key bytes.
    let key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes()).map_err(|e| {
        PlatformError::ConfigurationError(format!(
            "Failed to parse private key PEM: {}. Ensure it's a valid RSA private key.",
            e
        ))
    })?;

    // Build the Octocrab client authenticated as the GitHub App.
    let octocrab = Octocrab::builder().app(AppId(app_id), key).build().map_err(|e| {
        PlatformError::AuthenticationError(format!("Failed to build Octocrab App client: {}", e))
    })?;

    info!("Successfully created Octocrab client authenticated as GitHub App ID: {}", app_id);

    // Optional: Verify app authentication by fetching app details
    // match octocrab.current().app().await {
    //     Ok(app_details) => info!("Successfully verified authentication for App: {}", app_details.name),
    //     Err(e) => {
    //         error!("Failed to verify app authentication: {}", e);
    //         return Err(PlatformError::AuthenticationError(format!("Failed to verify app auth: {}", e)));
    //     }
    // }

    Ok(octocrab)
}

/// Creates an `Octocrab` client authenticated for a specific GitHub App installation.
///
/// Takes an existing App-authenticated `Octocrab` client and an installation ID,
/// requests an installation access token from GitHub, and returns a new `Octocrab`
/// client authenticated with that token.
///
/// # Arguments
///
/// * `app_client` - An `Octocrab` client authenticated as the GitHub App.
/// * `installation_id` - The ID of the specific GitHub App installation.
/// * `repository_owner` - (Optional) Owner context for logging.
/// * `repository_name` - (Optional) Repository context for logging.
///
/// # Returns
///
/// A `Result` containing an `Octocrab` client authenticated for the installation,
/// or a `PlatformError`.
#[instrument(skip(app_client), fields(installation_id = installation_id))]
pub async fn create_github_installation_client(
    app_client: &Octocrab,
    installation_id: u64,
    repository_owner: Option<&str>, // Optional context for logging
    repository_name: Option<&str>,  // Optional context for logging
) -> Result<Octocrab> {
    debug!(
        "Attempting to get installation token for installation_id: {}",
        installation_id
    );

    // Use the app client to get an installation-specific client and token.
    let (installation_client, _token_info) = app_client // Renamed from api_with_token for clarity
        .installation_and_token(installation_id.into())
        .await
        .map_err(|e| {
            error!(
                "Failed to create installation access token for installation_id {}: {}",
                installation_id, e
            );
            // Map octocrab error to PlatformError
            match e {
                octocrab::Error::GitHub { source, .. } => {
                    if source.message.contains("rate limit exceeded") {
                        PlatformError::RateLimitExceeded
                    } else {
                        PlatformError::AuthenticationError(format!(
                            "GitHub API error getting installation token: {}",
                            source.message
                        ))
                    }
                }
                _ => PlatformError::AuthenticationError(format!(
                    "Failed to get installation token: {}",
                    e
                )),
            }
        })?;

    info!(
        "Successfully created Octocrab client for installation_id: {}",
        installation_id
    );
    if let (Some(owner), Some(repo)) = (repository_owner, repository_name) {
        debug!("Context: {}/{}", owner, repo);
    }

    Ok(installation_client)
}
