//! Tests for the gitoxide backend (git_gix module).
//!
//! Uses the git CLI for repo creation instead of git2, so this test file
//! has no dependency on the `git` feature. Cross-validation tests that
//! compare both backends are gated on `#[cfg(all(feature = "git", feature = "gitoxide"))]`.

#![cfg(feature = "gitoxide")]

use std::collections::HashMap;
use std::process::Command;

use assert_fs::prelude::*;
use gix::ObjectId;

use swhid::git_gix::*;

/// Create a git repo with a single file and commit using the git CLI.
/// Returns (gix::Repository, commit ObjectId).
fn make_test_repo_cli(
    tmp: &assert_fs::TempDir,
    file_name: &str,
    file_content: &str,
    author_name: &str,
    author_email: &str,
    timestamp: i64,
    tz_offset_str: &str,
    message: &str,
) -> (gix::Repository, ObjectId) {
    let dir = tmp.path();

    // git init
    let status = Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("HOME", dir)
        .output()
        .expect("git must be installed to run these tests");
    assert!(status.status.success(), "git init failed: {:?}", status);

    // Write file
    let file_path = tmp.child(file_name);
    file_path.write_str(file_content).unwrap();

    // git add
    let status = Command::new("git")
        .args(["add", file_name])
        .current_dir(dir)
        .output()
        .expect("git add failed");
    assert!(status.status.success());

    // git commit with controlled timestamp
    let date_str = format!("{} {}", timestamp, tz_offset_str);
    let status = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", author_name)
        .env("GIT_AUTHOR_EMAIL", author_email)
        .env("GIT_AUTHOR_DATE", &date_str)
        .env("GIT_COMMITTER_NAME", author_name)
        .env("GIT_COMMITTER_EMAIL", author_email)
        .env("GIT_COMMITTER_DATE", &date_str)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("HOME", dir)
        .output()
        .expect("git commit failed");
    assert!(
        status.status.success(),
        "git commit failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );

    // Get HEAD commit hash
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .expect("git rev-parse failed");
    assert!(output.status.success());
    let hash = String::from_utf8(output.stdout).unwrap().trim().to_string();

    let repo = gix::open(dir).unwrap();
    let oid = ObjectId::from_hex(hash.as_bytes()).unwrap();
    (repo, oid)
}

#[test]
fn test_revision_swhid_gix() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let (repo, commit_oid) = make_test_repo_cli(
        &tmp,
        "test.txt",
        "test content",
        "Test User",
        "test@example.com",
        1763027354,
        "+0100",
        "Initial commit\n",
    );

    let swhid = revision_swhid(&repo, &commit_oid, &mut HashMap::new()).unwrap();
    assert_eq!(swhid.object_type().as_tag(), "rev");
    let s = swhid.to_string();
    assert!(s.starts_with("swh:1:rev:"));
    assert_eq!(s.len(), "swh:1:rev:".len() + 40);
}

#[test]
fn test_snapshot_swhid_gix() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let (repo, _) = make_test_repo_cli(
        &tmp,
        "hello.txt",
        "hello world",
        "Test User",
        "test@example.com",
        1763027354,
        "+0100",
        "Initial commit\n",
    );

    let swhid = snapshot_swhid(&repo).unwrap();
    let s = swhid.to_string();
    assert!(s.starts_with("swh:1:snp:"));
    assert_eq!(s.len(), "swh:1:snp:".len() + 40);
}

#[test]
fn test_open_repo_gix() {
    let tmp = assert_fs::TempDir::new().unwrap();
    // Init with git CLI
    let status = Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("HOME", tmp.path())
        .output()
        .unwrap();
    assert!(status.status.success());

    let repo = open_repo(tmp.path());
    assert!(repo.is_ok());
}

#[test]
fn test_open_repo_gix_invalid() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let repo = open_repo(tmp.path());
    assert!(repo.is_err());
}

#[test]
fn test_get_head_commit_gix() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let (repo, expected_oid) = make_test_repo_cli(
        &tmp,
        "test.txt",
        "content",
        "Test User",
        "test@example.com",
        1763027354,
        "+0100",
        "Initial commit\n",
    );

    let head_oid = get_head_commit(&repo).unwrap();
    assert_eq!(head_oid, expected_oid);
}

/// Cross-validate: both backends produce the same revision SWHID.
#[test]
#[cfg(feature = "git")]
fn test_revision_matches_git2_backend() {
    use swhid::git as git2_mod;

    let tmp = assert_fs::TempDir::new().unwrap();
    let (repo_gix, commit_oid_gix) = make_test_repo_cli(
        &tmp,
        "cross.txt",
        "cross-validation content",
        "Cross User",
        "cross@example.com",
        1700000000,
        "+0000",
        "cross test\n",
    );

    // Compute with gitoxide backend
    let swhid_gix = revision_swhid(&repo_gix, &commit_oid_gix, &mut HashMap::new()).unwrap();

    // Compute with git2 backend
    let repo_g2 = git2::Repository::open(tmp.path()).unwrap();
    let commit_oid_g2 = git2::Oid::from_str(&commit_oid_gix.to_string()).unwrap();
    let swhid_git2 =
        git2_mod::revision_swhid(&repo_g2, &commit_oid_g2, &mut HashMap::new()).unwrap();

    assert_eq!(
        swhid_git2.to_string(),
        swhid_gix.to_string(),
        "git2 and gitoxide backends must produce identical revision SWHIDs"
    );
}

/// Cross-validate snapshot SWHIDs between backends.
#[test]
#[cfg(feature = "git")]
fn test_snapshot_matches_git2_backend() {
    use swhid::git as git2_mod;

    let tmp = assert_fs::TempDir::new().unwrap();
    let (repo_gix, _) = make_test_repo_cli(
        &tmp,
        "snap.txt",
        "snapshot content",
        "Snap User",
        "snap@example.com",
        1700000000,
        "+0000",
        "snap commit\n",
    );

    let swhid_gix = snapshot_swhid(&repo_gix).unwrap();

    let repo_g2 = git2::Repository::open(tmp.path()).unwrap();
    let swhid_git2 = git2_mod::snapshot_swhid(&repo_g2).unwrap();

    assert_eq!(
        swhid_git2.to_string(),
        swhid_gix.to_string(),
        "git2 and gitoxide backends must produce identical snapshot SWHIDs"
    );
}
