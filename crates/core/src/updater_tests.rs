//! Unit tests for TemplateUpdater in updater.rs

use super::*;
use crate::state_manager::StateManager;
use crate::types::{CoreError, Result, TemplateState};
use async_trait::async_trait;
use chrono::Utc;
use mockall::mock;
use std::sync::Arc;

// Mock StateManager using mockall
mock! {
    pub StateManager {
        #[async_trait]
        pub async fn get_state(&self, template_id: &str) -> Result<Option<TemplateState>>;
        #[async_trait]
        pub async fn update_state(&self, state: &TemplateState) -> Result<()>;
    }
    #[async_trait]
    impl StateManager {
        async fn get_state(&self, template_id: &str) -> Result<Option<TemplateState>>;
        async fn update_state(&self, state: &TemplateState) -> Result<()>;
    }
}

#[tokio::test]
async fn test_process_update_new_template() {
    let template_id = "template1";
    let source_repository = "repo1";
    let new_template_data = b"template content";
    let checksum = crate::utils::calculate_checksum(new_template_data).unwrap();

    let mut mock_manager = MockStateManager::new();
    // get_state returns Ok(None) to simulate new template
    mock_manager
        .expect_get_state()
        .withf(move |id| id == template_id)
        .times(1)
        .returning(|_| Ok(None));

    // update_state expects to be called with the new TemplateState
    mock_manager
        .expect_update_state()
        .withf(move |state| {
            state.template_id == template_id
                && state.source_repository == source_repository
                && state.current_checksum == checksum
        })
        .times(1)
        .returning(|_| Ok(()));

    let updater = TemplateUpdater::new(Arc::new(mock_manager));
    let result = updater
        .process_update(template_id, source_repository, new_template_data)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_process_update_unchanged_template() {
    let template_id = "template2";
    let source_repository = "repo2";
    let new_template_data = b"unchanged content";
    let checksum = crate::utils::calculate_checksum(new_template_data).unwrap();

    let state = TemplateState {
        template_id: template_id.to_string(),
        source_repository: source_repository.to_string(),
        current_checksum: checksum.clone(),
        last_updated_utc: Utc::now(),
    };

    let mut mock_manager = MockStateManager::new();
    // get_state returns Ok(Some(state)) with matching checksum
    mock_manager
        .expect_get_state()
        .withf(move |id| id == template_id)
        .times(1)
        .returning(move |_| Ok(Some(state.clone())));

    // update_state should NOT be called
    mock_manager.expect_update_state().times(0);

    let updater = TemplateUpdater::new(Arc::new(mock_manager));
    let result = updater
        .process_update(template_id, source_repository, new_template_data)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_process_update_changed_template() {
    let template_id = "template3";
    let source_repository = "repo3";
    let old_template_data = b"old content";
    let new_template_data = b"new content";
    let old_checksum = crate::utils::calculate_checksum(old_template_data).unwrap();
    let new_checksum = crate::utils::calculate_checksum(new_template_data).unwrap();

    let state = TemplateState {
        template_id: template_id.to_string(),
        source_repository: source_repository.to_string(),
        current_checksum: old_checksum.clone(),
        last_updated_utc: Utc::now(),
    };

    let mut mock_manager = MockStateManager::new();
    // get_state returns Ok(Some(state)) with non-matching checksum
    mock_manager
        .expect_get_state()
        .withf(move |id| id == template_id)
        .times(1)
        .returning(move |_| Ok(Some(state.clone())));

    // update_state expects to be called with the new TemplateState
    mock_manager
        .expect_update_state()
        .withf(move |state| {
            state.template_id == template_id
                && state.source_repository == source_repository
                && state.current_checksum == new_checksum
        })
        .times(1)
        .returning(|_| Ok(()));

    let updater = TemplateUpdater::new(Arc::new(mock_manager));
    let result = updater
        .process_update(template_id, source_repository, new_template_data)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_process_update_get_state_error() {
    let template_id = "template4";
    let source_repository = "repo4";
    let new_template_data = b"irrelevant";
    let error_msg = "simulated get_state error";

    let mut mock_manager = MockStateManager::new();
    // get_state returns an error
    mock_manager
        .expect_get_state()
        .withf(move |id| id == template_id)
        .times(1)
        .returning(move |_| Err(CoreError::DatabaseError(error_msg.to_string())));

    // update_state should NOT be called
    mock_manager.expect_update_state().times(0);

    let updater = TemplateUpdater::new(Arc::new(mock_manager));
    let result = updater
        .process_update(template_id, source_repository, new_template_data)
        .await;

    assert!(result.is_err());
    match result.err().unwrap() {
        CoreError::DatabaseError(msg) => assert_eq!(msg, error_msg),
        _ => panic!("Expected DatabaseError"),
    }
}

#[tokio::test]
async fn test_process_update_update_state_error() {
    let template_id = "template5";
    let source_repository = "repo5";
    let new_template_data = b"template data";
    let checksum = crate::utils::calculate_checksum(new_template_data).unwrap();
    let error_msg = "simulated update_state error";

    let mut mock_manager = MockStateManager::new();
    // get_state returns Ok(None) to simulate new template
    mock_manager
        .expect_get_state()
        .withf(move |id| id == template_id)
        .times(1)
        .returning(|_| Ok(None));

    // update_state returns an error
    mock_manager
        .expect_update_state()
        .withf(move |state| {
            state.template_id == template_id
                && state.source_repository == source_repository
                && state.current_checksum == checksum
        })
        .times(1)
        .returning(move |_| Err(CoreError::DatabaseError(error_msg.to_string())));

    let updater = TemplateUpdater::new(Arc::new(mock_manager));
    let result = updater
        .process_update(template_id, source_repository, new_template_data)
        .await;

    assert!(result.is_err());
    match result.err().unwrap() {
        CoreError::DatabaseError(msg) => assert_eq!(msg, error_msg),
        _ => panic!("Expected DatabaseError"),
    }
}

#[tokio::test]
async fn test_template_updater_new_and_debug() {
    // Test TemplateUpdater::new and Debug implementation
    let mock_manager = Arc::new(MockStateManager::new());
    let updater = TemplateUpdater::new(mock_manager);
    let debug_str = format!("{:?}", updater);
    assert!(debug_str.contains("TemplateUpdater"));
}
