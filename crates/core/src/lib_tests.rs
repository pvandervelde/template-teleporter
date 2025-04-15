use super::*;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

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
fn test_parse_config() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("config.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(
        file,
        "---\ndatabase_type: dynamodb\ndatabase_endpoint: http://localhost:8000\ntable_name: TemplateState"
    )
    .unwrap();

    let config = parse_config(&file_path).unwrap();
    assert_eq!(config.database_type, "dynamodb");
    assert_eq!(config.database_endpoint.unwrap(), "http://localhost:8000");
    assert_eq!(config.table_name, "TemplateState");
}

#[test]
fn debug_raw_hash_bytes() {
    let data = b"test data";
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    println!("Raw hash bytes: {:?}", result);
}
