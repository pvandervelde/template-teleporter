use super::*;
use chrono::Utc;

#[test]
fn test_template_category_new() {
    let category = TemplateCategory::new("saas_rust".to_string());
    assert_eq!(category.name(), "saas_rust");
}

#[test]
fn test_template_metadata_new() {
    let metadata = TemplateMetadata::new(
        "/path/to/template".to_string(),
        "checksum123".to_string(),
        Utc::now(),
    );
    assert_eq!(metadata.path(), "/path/to/template");
    assert_eq!(metadata.checksum(), "checksum123");
}

#[test]
fn test_repo_info_new() {
    let repo_info = RepoInfo::new("org".to_string(), "repo".to_string(), "main".to_string());
    assert_eq!(repo_info.org(), "org");
    assert_eq!(repo_info.name(), "repo");
    assert_eq!(repo_info.default_branch(), "main");
}

#[test]
fn test_template_change_new() {
    let change = TemplateChange::new(
        "/path/to/template".to_string(),
        vec!["old_checksum1".to_string(), "old_checksum2".to_string()],
        "new_checksum".to_string(),
        vec![1, 2, 3],
    );
    assert_eq!(change.path(), "/path/to/template");
    assert_eq!(change.new_checksum(), "new_checksum");
    assert_eq!(change.content(), &vec![1, 2, 3]);
    assert_eq!(change.old_checksum_count(), 2);
    assert_eq!(
        change.old_checksum_at(0),
        Some(&"old_checksum1".to_string())
    );
    assert_eq!(
        change.old_checksum_at(1),
        Some(&"old_checksum2".to_string())
    );
    assert!(change
        .old_checksums()
        .eq(vec!["old_checksum1", "old_checksum2"].iter()));
}

#[test]
fn test_update_result_new() {
    let result = UpdateResult::new(
        "https://pr.url".to_string(),
        42,
        vec!["/path/to/file".to_string()],
    );
    assert_eq!(result.pr_url(), "https://pr.url");
    assert_eq!(result.pr_number(), 42);
    assert_eq!(result.updated_files(), &vec!["/path/to/file".to_string()]);
}
