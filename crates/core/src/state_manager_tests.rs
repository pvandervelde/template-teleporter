use super::*; // Import items from state_manager.rs
use crate::types::{CoreError, Result, TemplateState}; // Import necessary types
use async_trait::async_trait;
use chrono::Utc;
use mockall::mock;

// Create a mock implementation of the StatePersistence trait
mock! {
    pub StatePersistenceBackend {} // Name it differently from the trait
    #[async_trait]
    impl StatePersistence for StatePersistenceBackend {
        async fn get_state(&self, template_id: &str) -> Result<Option<TemplateState>>;
        async fn update_state(&self, state: &TemplateState) -> Result<()>;
    }
}

#[tokio::test]
async fn test_state_manager_get_state_found() {
    let template_id = "test-template";
    let expected_state = TemplateState {
        template_id: template_id.to_string(),
        source_repository: "test/repo".to_string(),
        current_checksum: "checksum123".to_string(),
        last_updated_utc: Utc::now(),
    };

    let mut mock_backend = MockStatePersistenceBackend::new();
    let state_clone = expected_state.clone(); // Clone for the closure

    // Expect get_state to be called once with "test-template"
    mock_backend
        .expect_get_state()
        .with(mockall::predicate::eq(template_id))
        .times(1)
        .returning(move |_| Ok(Some(state_clone.clone()))); // Return the cloned state

    // Create StateManager with the mock backend
    let state_manager = StateManager::new(Box::new(mock_backend));

    // Call the method under test
    let result = state_manager.get_state(template_id).await;

    // Assertions
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(expected_state));
}

#[tokio::test]
async fn test_state_manager_get_state_not_found() {
    let template_id = "not-found-template";
    let mut mock_backend = MockStatePersistenceBackend::new();

    // Expect get_state to be called once and return Ok(None)
    mock_backend
        .expect_get_state()
        .with(mockall::predicate::eq(template_id))
        .times(1)
        .returning(|_| Ok(None));

    let state_manager = StateManager::new(Box::new(mock_backend));
    let result = state_manager.get_state(template_id).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
}

#[tokio::test]
async fn test_state_manager_get_state_error() {
    let template_id = "error-template";
    let mut mock_backend = MockStatePersistenceBackend::new();

    // Expect get_state to be called once and return an error
    mock_backend
        .expect_get_state()
        .with(mockall::predicate::eq(template_id))
        .times(1)
        .returning(|_| Err(CoreError::DatabaseError("Simulated DB error".to_string())));

    let state_manager = StateManager::new(Box::new(mock_backend));
    let result = state_manager.get_state(template_id).await;

    assert!(result.is_err());
    match result.err().unwrap() {
        CoreError::DatabaseError(msg) => assert_eq!(msg, "Simulated DB error"),
        _ => panic!("Expected DatabaseError"),
    }
}

#[tokio::test]
async fn test_state_manager_update_state_success() {
    let state_to_update = TemplateState {
        template_id: "update-template".to_string(),
        source_repository: "test/repo".to_string(),
        current_checksum: "new_checksum".to_string(),
        last_updated_utc: Utc::now(),
    };

    let mut mock_backend = MockStatePersistenceBackend::new();
    let state_clone = state_to_update.clone();

    // Expect update_state to be called once with the correct state
    mock_backend
        .expect_update_state()
        .withf(move |state| state == &state_clone) // Use withf for complex comparisons
        .times(1)
        .returning(|_| Ok(())); // Return success

    let state_manager = StateManager::new(Box::new(mock_backend));
    let result = state_manager.update_state(&state_to_update).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_state_manager_update_state_error() {
    let state_to_update = TemplateState {
        template_id: "update-error-template".to_string(),
        source_repository: "test/repo".to_string(),
        current_checksum: "error_checksum".to_string(),
        last_updated_utc: Utc::now(),
    };

    let mut mock_backend = MockStatePersistenceBackend::new();
    let state_clone = state_to_update.clone();

    // Expect update_state to be called once and return an error
    mock_backend
        .expect_update_state()
        .withf(move |state| state == &state_clone)
        .times(1)
        .returning(|_| {
            Err(CoreError::DatabaseError(
                "Simulated update error".to_string(),
            ))
        });

    let state_manager = StateManager::new(Box::new(mock_backend));
    let result = state_manager.update_state(&state_to_update).await;

    assert!(result.is_err());
    match result.err().unwrap() {
        CoreError::DatabaseError(msg) => assert_eq!(msg, "Simulated update error"),
        _ => panic!("Expected DatabaseError"),
    }
}
