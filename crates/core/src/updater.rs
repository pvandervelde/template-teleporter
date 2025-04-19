//! Defines the `TemplateUpdater` struct, responsible for orchestrating the
//! template synchronization workflow.

use crate::state_manager::StateManager;
use crate::types::{Result, TemplateState};
use crate::utils::calculate_checksum;
use chrono::Utc;
use std::fmt; // Import fmt for custom Debug
use std::sync::Arc;

/// Handles the core logic for processing template updates.
///
/// This struct uses a `StateManager` (configured with a specific `StatePersistence` backend)
/// to check the current state of a template, compares checksums, and updates the state
/// if the template content has changed. It is designed to be agnostic of the specific
/// platform (like GitHub) and the specific database backend. Platform interaction logic
/// (e.g., creating Pull Requests) will be handled separately, potentially by injecting
/// platform-specific clients in the future.
pub struct TemplateUpdater {
    /// Shared access to the state manager, which handles persistence.
    state_manager: Arc<StateManager>,
    // TODO: Inject platform clients later
    // platform_clients: Vec<Box<dyn PlatformUpdater>>,
}

// Manual Debug implementation because StateManager is not Debug (due to Box<dyn Trait>)
impl fmt::Debug for TemplateUpdater {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TemplateUpdater")
            .field("state_manager", &"Arc<StateManager>") // Don't print the actual StateManager
            // Add other fields here if they are added later
            .finish()
    }
}

impl TemplateUpdater {
    /// Creates a new `TemplateUpdater` instance.
    ///
    /// # Arguments
    /// * `state_manager` - An `Arc`-wrapped `StateManager` instance configured with a
    ///   persistence backend implementing `StatePersistence`. The `Arc` allows the
    ///   `StateManager` to be shared if the `TemplateUpdater` needs to be cloned or shared
    ///   across threads.
    ///
    /// # Returns
    /// A new `TemplateUpdater` instance.
    pub fn new(state_manager: Arc<StateManager>) -> Self {
        // TODO: Initialize platform clients based on config later if needed
        Self { state_manager }
    }

    /// Processes a potential template update based on new content.
    ///
    /// This is the core workflow method. It performs the following steps:
    /// 1. Calculates the checksum of the `new_template_data`.
    /// 2. Retrieves the current `TemplateState` for the given `template_id` using the `StateManager`.
    /// 3. Compares the new checksum with the `current_checksum` in the retrieved state (if any).
    /// 4. If the checksums differ or no previous state exists (`needs_update` is true):
    ///    a. **(Placeholder)** In a full implementation, this is where interaction with platform clients
    ///    would occur to fetch target repository content, check for manual overrides, and potentially
    ///    create a Pull Request. This logic is currently omitted.
    ///    b. Creates a new `TemplateState` struct with the updated information (new checksum, current timestamp).
    ///    c. Saves the new state back to the persistence layer via the `StateManager`.
    /// 5. If the checksums match, it logs that no update is needed.
    ///
    /// # Arguments
    /// * `template_id` - The unique identifier for the template being processed (e.g., its path).
    /// * `source_repository` - The identifier of the source repository (used for recording in `TemplateState`).
    /// * `new_template_data` - The raw byte content of the new or current template version from the source.
    ///
    /// # Returns
    /// An empty `Result` on success, or a `CoreError` if any step (checksum calculation, state retrieval, state update) fails.
    ///
    /// # Errors
    /// Can return `CoreError::ChecksumFailure`, `CoreError::DatabaseError`.
    pub async fn process_update(
        &self,
        template_id: &str,
        source_repository: &str,
        new_template_data: &[u8],
    ) -> Result<()> {
        // Using tracing::info! or similar would be better than println! for real applications
        println!("Processing update for template: {}", template_id);

        // 1. Calculate checksum
        let new_checksum = calculate_checksum(new_template_data)?;
        println!("  New checksum: {}", new_checksum);

        // 2. Get current state
        let current_state_opt = self.state_manager.get_state(template_id).await?;

        // 3. Compare checksums
        let needs_update = match &current_state_opt {
            Some(current_state) => {
                println!("  Current checksum: {}", current_state.current_checksum);
                current_state.current_checksum != new_checksum
            }
            None => {
                println!("  No current state found.");
                true // New template
            }
        };

        if needs_update {
            println!("  Checksum mismatch or new template. Update required.");

            // 4. Platform Interaction (Placeholder)
            // TODO: Implement platform interaction logic here or in a separate method.
            // This would involve:
            // - Fetching target repo content via a platform client.
            // - Comparing target content checksum with current_state.current_checksum to detect manual changes.
            // - If no manual changes, creating a PR via the platform client.
            // - Handling potential CoreError::PlatformError from the client.
            // self.create_update_pr(template_id, new_template_data).await?;

            // 5. Create new state data
            let new_state = TemplateState {
                template_id: template_id.to_string(),
                source_repository: source_repository.to_string(),
                current_checksum: new_checksum,
                last_updated_utc: Utc::now(),
            };

            // 6. Save new state
            self.state_manager.update_state(&new_state).await?;
            println!("  State successfully updated for template: {}", template_id);
        } else {
            println!(
                "  No state change detected for template: {}. No update needed.",
                template_id
            );
        }

        Ok(())
    }

    // Placeholder for future platform interaction logic
    // async fn create_update_pr(&self, template_id: &str, data: &[u8]) -> Result<()> {
    //     println!("  (Placeholder) Would create PR for template: {}", template_id);
    //     // Logic to interact with platform clients would go here
    //     Ok(())
    // }
}
