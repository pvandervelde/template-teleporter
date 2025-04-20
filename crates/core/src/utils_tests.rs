use super::*;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir; // Added import for Sha256::new()

#[test]
fn test_calculate_checksum() {
    let data = b"test data";
    let checksum = calculate_checksum(data).unwrap();
    assert_eq!(
        checksum,
        "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9"
    );
}

#[test]
fn test_parse_config_with_enum() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("config.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(
        file,
        "---\ndatabaseEndpoint: http://localhost:8000\ntableName: TemplateState"
    )
    .unwrap();

    let config = parse_config(&file_path).unwrap();
    assert_eq!(
        config.database_endpoint.as_deref(),
        Some("http://localhost:8000")
    );
    assert_eq!(config.table_name, "TemplateState");
}

#[test]
fn test_parse_config_file_not_found() {
    use std::path::PathBuf;
    let path = PathBuf::from("this_file_should_not_exist.yaml");
    let result = parse_config(&path);
    assert!(result.is_err());
    match result.err().unwrap() {
        CoreError::IoError(_) => {}
        _ => panic!("Expected IoError for missing file"),
    }
}

#[test]
fn test_parse_config_invalid_yaml() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("invalid.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "not: valid: yaml: [").unwrap();

    let result = parse_config(&file_path);
    assert!(result.is_err());
    match result.err().unwrap() {
        CoreError::ConfigParseError { .. } => {}
        _ => panic!("Expected ConfigParseError for invalid YAML"),
    }
}

#[test]
fn test_parse_config_empty_table_name() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("empty_table.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(
        file,
        "---\ndatabaseEndpoint: http://localhost:8000\ntableName: \"\""
    )
    .unwrap();

    let result = parse_config(&file_path);
    assert!(result.is_err());
    match result.err().unwrap() {
        CoreError::MissingConfiguration(msg) => {
            assert!(msg.contains("table_name"));
        }
        _ => panic!("Expected MissingConfiguration for empty tableName"),
    }
}

#[test]
fn test_parse_config_missing_table_name() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("missing_table.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "---\ndatabaseEndpoint: http://localhost:8000").unwrap();

    let result = parse_config(&file_path);
    assert!(result.is_err());
    match result.err().unwrap() {
        CoreError::ConfigParseError { .. } => {}
        _ => panic!("Expected ConfigParseError for missing tableName"),
    }
}

#[test]
fn test_app_config_serialization() {
    let config = AppConfig {
        database_endpoint: Some("https://cosmos.example.com".to_string()),
        table_name: "StateTable".to_string(),
    };
    let yaml = serde_yaml::to_string(&config).unwrap();
    assert!(yaml.contains("databaseEndpoint: https://cosmos.example.com"));
    assert!(yaml.contains("tableName: StateTable"));

    let deserialized: AppConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(
        deserialized.database_endpoint.as_deref(),
        Some("https://cosmos.example.com")
    );
    assert_eq!(deserialized.table_name, "StateTable");
}

#[test]
fn test_core_error_platform_error() {
    let err = CoreError::PlatformError("platform failed".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("platform failed"));
}
