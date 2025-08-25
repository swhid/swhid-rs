use swhid::{SwhidComputer, Swhid, ObjectType, Content, Directory, QualifiedSwhid};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_content_swhid_conformance() {
    // Test that content SWHID matches expected format
    let data = b"Hello, World!";
    let content = Content::from_data(data.to_vec());
    let swhid = content.swhid();

    assert_eq!(swhid.object_type(), ObjectType::Content);
    assert_eq!(swhid.hash().len(), 20);

    // Verify SWHID format
    let swhid_str = swhid.to_string();
    assert!(swhid_str.starts_with("swh:1:cnt:"));
    assert_eq!(swhid_str.len(), 50); // swh:1:cnt: + 40 hex chars
}

#[test]
fn test_directory_swhid_conformance() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("file.txt"), b"test content").unwrap();

    let mut dir = Directory::from_disk(temp_dir.path(), &[], true).unwrap();
    let swhid = dir.swhid();

    assert_eq!(swhid.object_type(), ObjectType::Directory);
    assert_eq!(swhid.hash().len(), 20);

    // Verify SWHID format
    let swhid_str = swhid.to_string();
    assert!(swhid_str.starts_with("swh:1:dir:"));
    assert_eq!(swhid_str.len(), 50); // swh:1:dir: + 40 hex chars
}

#[test]
fn test_swhid_computer_basic() {
    let computer = SwhidComputer::new();

    // Test content computation
    let content_swhid = computer.compute_content_swhid(b"test").unwrap();
    assert_eq!(content_swhid.object_type(), ObjectType::Content);

    // Test file computation
    let temp_file = TempDir::new().unwrap();
    let file_path = temp_file.path().join("test.txt");
    fs::write(&file_path, b"test content").unwrap();

    let file_swhid = computer.compute_file_swhid(&file_path).unwrap();
    assert_eq!(file_swhid.object_type(), ObjectType::Content);

    // Test directory computation
    let dir_swhid = computer.compute_directory_swhid(temp_file.path()).unwrap();
    assert_eq!(dir_swhid.object_type(), ObjectType::Directory);
}

#[test]
fn test_swhid_parsing() {
    // Test valid SWHID parsing
    let valid_swhid = "swh:1:cnt:0000000000000000000000000000000000000000";
    let parsed = Swhid::from_string(valid_swhid).unwrap();
    assert_eq!(parsed.object_type(), ObjectType::Content);

    // Test invalid SWHID parsing
    assert!(Swhid::from_string("invalid").is_err());
    assert!(Swhid::from_string("swh:2:cnt:0000000000000000000000000000000000000000").is_err());
    assert!(Swhid::from_string("swh:1:invalid:0000000000000000000000000000000000000000").is_err());
    assert!(Swhid::from_string("swh:1:cnt:00000000000000000000000000000000000000").is_err());
}

#[test]
fn test_swhid_verification() {
    let computer = SwhidComputer::new();

    let temp_file = TempDir::new().unwrap();
    let file_path = temp_file.path().join("test.txt");
    fs::write(&file_path, b"test content").unwrap();

    // Compute SWHID
    let computed_swhid = computer.compute_file_swhid(&file_path).unwrap();

    // Verify it matches itself
    let is_valid = computer.verify_swhid(&file_path, &computed_swhid.to_string()).unwrap();
    assert!(is_valid);
}

#[test]
fn test_qualified_swhid_conformance() {
    // Test that qualified SWHID matches expected format
    let core = Swhid::new(ObjectType::Content, [0u8; 20]);
    let qualified = QualifiedSwhid::new(core)
        .with_origin("https://github.com/user/repo".to_string())
        .with_path(b"/src/main.rs".to_vec())
        .with_lines(10, Some(20));

    // Verify core SWHID properties
    assert_eq!(qualified.object_type(), ObjectType::Content);
    assert_eq!(qualified.hash().len(), 20);

    // Verify qualifiers
    assert_eq!(qualified.origin(), Some("https://github.com/user/repo"));
    assert_eq!(qualified.path(), Some(b"/src/main.rs".as_slice()));
    assert_eq!(qualified.lines(), Some((10, Some(20))));

    // Verify string format
    let qualified_str = qualified.to_string();
    assert!(qualified_str.starts_with("swh:1:cnt:0000000000000000000000000000000000000000"));
    assert!(qualified_str.contains(";origin=https://github.com/user/repo"));
    assert!(qualified_str.contains(";path=/src/main.rs"));
    assert!(qualified_str.contains(";lines=10-20"));
}

#[test]
fn test_qualified_swhid_parsing() {
    // Test parsing a complex qualified SWHID
    let s = "swh:1:cnt:0000000000000000000000000000000000000000;origin=https://github.com/user/repo;path=/src/main.rs;lines=10-20";
    let qualified = QualifiedSwhid::from_string(s).unwrap();

    assert_eq!(qualified.object_type(), ObjectType::Content);
    assert_eq!(qualified.origin(), Some("https://github.com/user/repo"));
    assert_eq!(qualified.path(), Some(b"/src/main.rs".as_slice()));
    assert_eq!(qualified.lines(), Some((10, Some(20))));

    // Test parsing with single line
    let s = "swh:1:cnt:0000000000000000000000000000000000000000;lines=15";
    let qualified = QualifiedSwhid::from_string(s).unwrap();
    assert_eq!(qualified.lines(), Some((15, None)));
}

#[test]
fn test_qualified_swhid_validation() {
    let core = Swhid::new(ObjectType::Content, [0u8; 20]);

    // Test valid visit qualifier (snapshot)
    let visit = Swhid::new(ObjectType::Snapshot, [1u8; 20]);
    let qualified = QualifiedSwhid::new(core.clone())
        .with_visit(visit).unwrap();
    assert!(qualified.visit().is_some());

    // Test invalid visit qualifier (content)
    let invalid_visit = Swhid::new(ObjectType::Content, [1u8; 20]);
    let result = QualifiedSwhid::new(core.clone())
        .with_visit(invalid_visit);
    assert!(result.is_err());

    // Test valid anchor qualifier (directory)
    let anchor = Swhid::new(ObjectType::Directory, [2u8; 20]);
    let qualified = QualifiedSwhid::new(core.clone())
        .with_anchor(anchor).unwrap();
    assert!(qualified.anchor().is_some());

    // Test invalid anchor qualifier (content)
    let invalid_anchor = Swhid::new(ObjectType::Content, [2u8; 20]);
    let result = QualifiedSwhid::new(core)
        .with_anchor(invalid_anchor);
    assert!(result.is_err());
}
