use std::fs;
use std::path::Path;
use tempfile::TempDir;
use swhid::{Content, Directory, SwhidComputer, ObjectType};

/// Test helper to create a temporary directory with specific structure
struct TestDir {
    temp_dir: TempDir,
}

impl TestDir {
    fn new() -> Self {
        Self {
            temp_dir: TempDir::new().unwrap(),
        }
    }

    fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    fn create_file(&self, name: &str, content: &[u8]) {
        fs::write(self.path().join(name), content).unwrap();
    }

    fn create_executable(&self, name: &str, content: &[u8]) {
        let file_path = self.path().join(name);
        fs::write(&file_path, content).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&file_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&file_path, perms).unwrap();
        }
    }

    fn create_subdir(&self, name: &str) -> std::path::PathBuf {
        let dir_path = self.path().join(name);
        fs::create_dir(&dir_path).unwrap();
        dir_path
    }

    fn create_symlink(&self, name: &str, target: &str) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(target, self.path().join(name)).unwrap();
        }
    }
}

#[test]
fn test_content_hash_basic() {
    let test_dir = TestDir::new();
    test_dir.create_file("test.txt", b"Hello, World!");
    
    let content = Content::from_file(test_dir.path().join("test.txt")).unwrap();
    let swhid = content.swhid();
    
    // Known hash for "Hello, World!" content (matches Python swh identify)
    assert_eq!(swhid.hash(), &hex::decode("b45ef6fec89518d314f546fd6c3025367b721684").unwrap()[..]);
    assert_eq!(swhid.object_type(), ObjectType::Content);
}

#[test]
fn test_content_hash_empty() {
    let test_dir = TestDir::new();
    test_dir.create_file("empty.txt", b"");
    
    let content = Content::from_file(test_dir.path().join("empty.txt")).unwrap();
    let swhid = content.swhid();
    
    // Known hash for empty content
    assert_eq!(swhid.hash(), &hex::decode("e69de29bb2d1d6434b8b29ae775ad8c2e48c5391").unwrap()[..]);
}

#[test]
fn test_content_hash_large() {
    let test_dir = TestDir::new();
    let large_content = vec![b'a'; 10000];
    test_dir.create_file("large.txt", &large_content);
    
    let content = Content::from_file(test_dir.path().join("large.txt")).unwrap();
    let swhid = content.swhid();
    
    // Verify it's a valid SHA1 hash
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_directory_hash_single_file() {
    let test_dir = TestDir::new();
    test_dir.create_file("file.txt", b"test content");
    
    let mut dir = Directory::from_disk(test_dir.path(), &[]).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), ObjectType::Directory);
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_directory_hash_multiple_files() {
    let test_dir = TestDir::new();
    test_dir.create_file("file1.txt", b"content 1");
    test_dir.create_file("file2.txt", b"content 2");
    
    let mut dir = Directory::from_disk(test_dir.path(), &[]).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), ObjectType::Directory);
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_directory_hash_with_subdirectories() {
    let test_dir = TestDir::new();
    test_dir.create_file("root.txt", b"root content");
    
    let subdir = test_dir.create_subdir("subdir");
    fs::write(subdir.join("subfile.txt"), b"sub content").unwrap();
    
    let mut dir = Directory::from_disk(test_dir.path(), &[]).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), ObjectType::Directory);
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_directory_hash_with_executable() {
    let test_dir = TestDir::new();
    test_dir.create_file("normal.txt", b"normal content");
    test_dir.create_executable("script.sh", b"#!/bin/bash\necho hello");
    
    let mut dir = Directory::from_disk(test_dir.path(), &[]).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), ObjectType::Directory);
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_directory_hash_with_symlinks() {
    let test_dir = TestDir::new();
    test_dir.create_file("target.txt", b"target content");
    test_dir.create_symlink("link.txt", "target.txt");
    
    let mut dir = Directory::from_disk(test_dir.path(), &[]).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), ObjectType::Directory);
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_directory_hash_exclude_patterns() {
    let test_dir = TestDir::new();
    test_dir.create_file("include.txt", b"include content");
    test_dir.create_file("exclude.tmp", b"exclude content");
    
    let mut dir = Directory::from_disk(test_dir.path(), &["*.tmp".to_string()]).unwrap();
    let swhid = dir.swhid();
    
    assert_eq!(swhid.object_type(), ObjectType::Directory);
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_swhid_computer_basic() {
    let computer = SwhidComputer::new();
    
    let content = b"test content";
    let swhid = computer.compute_content_swhid(content).unwrap();
    
    assert_eq!(swhid.object_type(), ObjectType::Content);
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_swhid_computer_file() {
    let test_dir = TestDir::new();
    test_dir.create_file("test.txt", b"file content");
    
    let computer = SwhidComputer::new();
    let swhid = computer.compute_file_swhid(test_dir.path().join("test.txt")).unwrap();
    
    assert_eq!(swhid.object_type(), ObjectType::Content);
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_swhid_computer_directory() {
    let test_dir = TestDir::new();
    test_dir.create_file("test.txt", b"file content");
    
    let computer = SwhidComputer::new();
    let swhid = computer.compute_directory_swhid(test_dir.path()).unwrap();
    
    assert_eq!(swhid.object_type(), ObjectType::Directory);
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_swhid_computer_auto_detect_file() {
    let test_dir = TestDir::new();
    test_dir.create_file("test.txt", b"file content");
    
    let computer = SwhidComputer::new();
    let swhid = computer.compute_swhid(test_dir.path().join("test.txt")).unwrap();
    
    assert_eq!(swhid.object_type(), ObjectType::Content);
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_swhid_computer_auto_detect_directory() {
    let test_dir = TestDir::new();
    test_dir.create_file("test.txt", b"file content");
    
    let computer = SwhidComputer::new();
    let swhid = computer.compute_swhid(test_dir.path()).unwrap();
    
    assert_eq!(swhid.object_type(), ObjectType::Directory);
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_swhid_computer_symlink_default() {
    let test_dir = TestDir::new();
    test_dir.create_file("target.txt", b"target content");
    test_dir.create_symlink("link.txt", "target.txt");
    
    let computer = SwhidComputer::new();
    let swhid = computer.compute_swhid(test_dir.path().join("link.txt")).unwrap();
    
    // Should hash the symlink target string, not the target file
    assert_eq!(swhid.object_type(), ObjectType::Content);
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_swhid_computer_symlink_dereference() {
    let test_dir = TestDir::new();
    test_dir.create_file("target.txt", b"target content");
    test_dir.create_symlink("link.txt", "target.txt");
    
    let computer = SwhidComputer::new().with_follow_symlinks(true);
    let swhid = computer.compute_swhid(test_dir.path().join("link.txt")).unwrap();
    
    // Should hash the target file content
    assert_eq!(swhid.object_type(), ObjectType::Content);
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_swhid_computer_exclude_patterns() {
    let test_dir = TestDir::new();
    test_dir.create_file("include.txt", b"include content");
    test_dir.create_file("exclude.tmp", b"exclude content");
    
    let computer = SwhidComputer::new().with_exclude_patterns(&["*.tmp".to_string()]);
    let swhid = computer.compute_directory_swhid(test_dir.path()).unwrap();
    
    assert_eq!(swhid.object_type(), ObjectType::Directory);
    assert_eq!(swhid.hash().len(), 20);
}

#[test]
fn test_swhid_verification() {
    let test_dir = TestDir::new();
    test_dir.create_file("test.txt", b"test content");
    
    let computer = SwhidComputer::new();
    let swhid = computer.compute_file_swhid(test_dir.path().join("test.txt")).unwrap();
    
    let is_valid = computer.verify_swhid(test_dir.path().join("test.txt"), &swhid.to_string()).unwrap();
    assert!(is_valid);
}

#[test]
fn test_swhid_verification_mismatch() {
    let test_dir = TestDir::new();
    test_dir.create_file("test.txt", b"test content");
    
    let computer = SwhidComputer::new();
    let wrong_swhid = "swh:1:cnt:0000000000000000000000000000000000000000";
    
    let is_valid = computer.verify_swhid(test_dir.path().join("test.txt"), wrong_swhid).unwrap();
    assert!(!is_valid);
}
