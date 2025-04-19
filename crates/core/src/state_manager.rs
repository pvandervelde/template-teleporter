//! Defines the `StatePersistence` trait for abstracting state storage
//! and the `StateManager` struct which uses this trait.

use crate::types::{Result, TemplateState}; // Removed AppConfig as it's not directly needed
use async_trait::async_trait;
use std::fmt; // Import fmt for custom Debug implementation

#[cfg(test)]
#[path = "state_manager_tests.rs"]
mod tests;

/// Trait defining the interface for state persistence backends.
///
/// This trait abstracts the underlying storage mechanism (e.g., DynamoDB, CosmosDB, filesystem)
/// allowing the core logic to remain agnostic of the specific database implementation.
/// Implementers must be `Send` and `Sync` to be usable in async contexts.
#[async_trait]
pub trait StatePersistence: Send + Sync {
    /// Retrieves the state for a given template ID from the backend.
    ///
    /// # Arguments
    /// * `template_id` - The unique identifier of the template state to retrieve.
    ///
    /// # Returns
    /// A `Result` containing `Some(TemplateState)` if found, `None` if not found,
    /// or a `CoreError::DatabaseError` if the backend operation fails.
    async fn get_state(&self, template_id: &str) -> Result<Option<TemplateState>>;

    /// Saves or updates the state for a template in the backend.
    ///
    /// If the state for the given `template_id` already exists, it should be overwritten.
    /// If it does not exist, it should be created.
    ///
    /// # Arguments
    /// * `state` - The `TemplateState` object to save or update.
    ///
    /// # Returns
    /// An empty `Result` on success, or a `CoreError::DatabaseError` if the backend operation fails.
    async fn update_state(&self, state: &TemplateState) -> Result<()>;

    // Potentially add methods for initialization or configuration if needed later
    // async fn initialize(&self) -> Result<()>;
}

// Implement Debug manually for the trait object
impl fmt::Debug for dyn StatePersistence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StatePersistence").finish_non_exhaustive()
    }
}

/// Manages state persistence logic by delegating to a backend.
///
/// This struct holds an instance of a type implementing the `StatePersistence` trait
/// and uses it to perform state management operations (get, update). It acts as
/// an intermediary between the core application logic and the specific storage backend.
pub struct StateManager {
    /// The backend implementation responsible for actual storage operations.
    /// Stored as a boxed trait object for dynamic dispatch.
    backend: Box<dyn StatePersistence>,
}

// Manual Debug implementation for StateManager because Box<dyn Trait> is not Debug
impl fmt::Debug for StateManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateManager")
            .field("backend", &"Box<dyn StatePersistence>") // Don't print the actual backend
            .finish()
    }
}

impl StateManager {
    /// Creates a new `StateManager` instance with a specific persistence backend.
    ///
    /// The provided `backend` must implement the `StatePersistence` trait.
    ///
    /// # Arguments
    /// * `backend` - A `Box` containing an implementation of the `StatePersistence` trait.
    ///
    /// # Returns
    /// A new `StateManager` instance.
    pub fn new(backend: Box<dyn StatePersistence>) -> Self {
        Self { backend }
    }

    /// Retrieves the state for a given template ID by delegating to the configured backend.
    ///
    /// # Arguments
    /// * `template_id` - The unique identifier of the template state to retrieve.
    ///
    /// # Returns
    /// A `Result` containing `Some(TemplateState)` if found, `None` if not found,
    /// or a `CoreError` if the backend operation fails.
    pub async fn get_state(&self, template_id: &str) -> Result<Option<TemplateState>> {
        self.backend.get_state(template_id).await
    }

    /// Saves or updates the state for a template by delegating to the configured backend.
    ///
    /// # Arguments
    /// * `state` - The `TemplateState` object to save or update.
    ///
    /// # Returns
    /// An empty `Result` on success, or a `CoreError` if the backend operation fails.
    pub async fn update_state(&self, state: &TemplateState) -> Result<()> {
        self.backend.update_state(state).await
    }
}
