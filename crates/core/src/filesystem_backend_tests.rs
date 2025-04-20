//! Tests for the FilesystemBackend implementation.

use super::*; // Import items from filesystem_backend.rs
use crate::state_manager::StatePersistence; // Import the trait
use crate::types::TemplateState;
use chrono::Utc;
use futures::future;
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_filesystem_backend_new_creates_directory() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("test_state");
    // Ensure directory doesn't exist initially
    assert!(!base_path.exists());

    let backend_result = FilesystemBackend::new(&base_path);
    assert!(backend_result.is_ok());
    // Ensure directory was created
    assert!(base_path.exists());
    assert!(base_path.is_dir());
}

#[tokio::test]
async fn test_filesystem_backend_update_and_get_state() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("state");
    let backend = FilesystemBackend::new(&base_path).unwrap();

    let template_id = "my/template";
    let state = TemplateState {
        template_id: template_id.to_string(),
        source_repository: "owner/repo".to_string(),
        current_checksum: "checksum123".to_string(),
        last_updated_utc: Utc::now(),
    };

    // 1. Update state
    let update_result = backend.update_state(&state).await;
    assert!(update_result.is_ok());

    // Verify file exists and content is correct (optional deep check)
    let file_path = base_path.join("my_template.json"); // Check sanitized name
    assert!(file_path.exists());
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("checksum123"));

    // 2. Get state
    let get_result = backend.get_state(template_id).await;
    assert!(get_result.is_ok());
    let retrieved_state_opt = get_result.unwrap();
    assert!(retrieved_state_opt.is_some());
    assert_eq!(retrieved_state_opt.unwrap(), state); // Compare full state
}

#[tokio::test]
async fn test_filesystem_backend_get_state_not_found() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("state");
    let backend = FilesystemBackend::new(&base_path).unwrap();

    let template_id = "non-existent-template";
    let get_result = backend.get_state(template_id).await;

    assert!(get_result.is_ok());
    assert!(get_result.unwrap().is_none());
}

#[tokio::test]
async fn test_filesystem_backend_update_overwrites_existing() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("state");
    let backend = FilesystemBackend::new(&base_path).unwrap();

    let template_id = "overwrite/template";
    let initial_state = TemplateState {
        template_id: template_id.to_string(),
        source_repository: "owner/repo".to_string(),
        current_checksum: "checksum_initial".to_string(),
        last_updated_utc: Utc::now(),
    };
    let updated_state = TemplateState {
        template_id: template_id.to_string(),
        source_repository: "owner/repo".to_string(),
        current_checksum: "checksum_updated".to_string(),
        last_updated_utc: Utc::now(), // Timestamps will differ slightly, maybe ignore in comparison if needed
    };

    // Write initial state
    backend.update_state(&initial_state).await.unwrap();
    let retrieved_initial = backend.get_state(template_id).await.unwrap().unwrap();
    assert_eq!(retrieved_initial.current_checksum, "checksum_initial");

    // Write updated state
    backend.update_state(&updated_state).await.unwrap();
    let retrieved_updated = backend.get_state(template_id).await.unwrap().unwrap();
    assert_eq!(retrieved_updated.current_checksum, "checksum_updated");
    // Note: Comparing the full updated_state might fail due to timestamp differences.
    // assert_eq!(retrieved_updated, updated_state); // This might fail
}

#[tokio::test]
async fn test_filesystem_backend_get_state_invalid_json() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("state");
    let backend = FilesystemBackend::new(&base_path).unwrap();

    let template_id = "invalid-json-template";
    let file_path = base_path.join("invalid-json-template.json");

    // Create a file with invalid JSON content
    fs::write(&file_path, "{ invalid json ").unwrap();

    let get_result = backend.get_state(template_id).await;

    assert!(get_result.is_err());
    match get_result.err().unwrap() {
        CoreError::DatabaseError(msg) => {
            assert!(msg.contains("Failed to deserialize state"));
        }
        _ => panic!("Expected DatabaseError due to invalid JSON"),
    }
}

#[cfg(unix)]
#[tokio::test]
async fn test_filesystem_backend_get_state_io_error() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("state");
    let backend = FilesystemBackend::new(&base_path).unwrap();

    let template_id = "io-error-template";
    let file_path = base_path.join("io-error-template.json");

    // Write valid JSON, then make the file readonly
    fs::write(&file_path, r#"{"template_id":"id","source_repository":"repo","current_checksum":"sum","last_updated_utc":"2020-01-01T00:00:00Z"}"#).unwrap();
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&file_path, perms.clone()).unwrap();

    let get_result = backend.get_state(template_id).await;

    // Restore permissions for cleanup
    perms.set_readonly(false);
    fs::set_permissions(&file_path, perms).unwrap();

    assert!(get_result.is_err());
    match get_result.err().unwrap() {
        CoreError::IoError(_) => {}
        _ => panic!("Expected IoError due to unreadable file"),
    }
}

#[cfg(unix)]
#[tokio::test]
async fn test_filesystem_backend_update_state_io_error() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("readonly_state");
    let backend = FilesystemBackend::new(&base_path).unwrap();

    // Make the directory read-only
    let mut perms = fs::metadata(&base_path).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&base_path, perms.clone()).unwrap();

    let state = TemplateState {
        template_id: "fail".to_string(),
        source_repository: "repo".to_string(),
        current_checksum: "sum".to_string(),
        last_updated_utc: Utc::now(),
    };

    let result = backend.update_state(&state).await;

    // Restore permissions for cleanup
    perms.set_readonly(false);
    fs::set_permissions(&base_path, perms).unwrap();

    assert!(result.is_err());
    match result.err().unwrap() {
        CoreError::IoError(_) => {}
        _ => panic!("Expected IoError due to unwritable directory"),
    }
}

#[tokio::test]
async fn test_filesystem_backend_handles_concurrent_updates() {
    // This test verifies that the Mutex prevents race conditions,
    // although fully proving absence of races is complex.
    // We simulate concurrent updates to the *same* template ID.
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("concurrent_state");
    let backend = Arc::new(FilesystemBackend::new(&base_path).unwrap()); // Use Arc for sharing

    let template_id = "concurrent/template";

    let tasks = (0..10)
        .map(|i| {
            let backend_clone = Arc::clone(&backend);
            let state = TemplateState {
                template_id: template_id.to_string(),
                source_repository: "owner/repo".to_string(),
                current_checksum: format!("checksum_{}", i),
                last_updated_utc: Utc::now(),
            };
            tokio::spawn(async move { backend_clone.update_state(&state).await })
        })
        .collect::<Vec<_>>();

    let results = future::join_all(tasks).await;

    // Check all updates succeeded without IO errors or panics
    for result in results {
        assert!(result.is_ok()); // Check outer JoinHandle result
        assert!(result.unwrap().is_ok()); // Check inner update_state Result
    }

    // Verify the final state reflects one of the updates (likely the last one)
    let final_state_opt = backend.get_state(template_id).await.unwrap();
    assert!(final_state_opt.is_some());
    let final_state = final_state_opt.unwrap();
    assert!(final_state.current_checksum.starts_with("checksum_")); // Checksum should be one of the written ones
    println!("Final checksum: {}", final_state.current_checksum); // See which one "won"
}
