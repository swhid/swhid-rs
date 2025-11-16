#![cfg(feature = "git")]

use assert_fs::prelude::*;
use git2::{Repository, Signature, Time};

use swhid::git::*;
use swhid::release::{Release, ReleaseTargetType};
use swhid::revision::Revision;
use swhid::snapshot::{Branch, BranchTarget, Snapshot};
use swhid::ObjectType;

fn bs(s: &'static str) -> Box<[u8]> {
    s.as_bytes().into()
}

fn oid_to_array(oid: git2::Oid) -> [u8; 20] {
    oid.as_bytes()
        .try_into()
        .expect("Unexpected tree_oid length")
}

#[test]
fn test_revision_swhid() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();

    // Create content
    let mut index = repo.index().unwrap();
    let file_path = tmp.child("test.txt");
    file_path.write_str("test content").unwrap();

    // Create directory
    index
        .add_path(file_path.path().strip_prefix(tmp.path()).unwrap())
        .unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree_hash = hex::decode("0efb37b28c53c7e4fbd253bb04a4df14008f63fe")
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(oid_to_array(tree_oid), tree_hash);
    let tree = repo.find_tree(tree_oid).unwrap();

    // Create commit
    let sig = Signature::new("Test User", "test@example.com", &Time::new(1763027354, 60)).unwrap();
    let commit_oid = repo
        .commit(
            Some("refs/heads/main"),
            &sig,
            &sig,
            "Test commit",
            &tree,
            &[],
        )
        .unwrap();

    let rev = revision_from_git(&repo, &commit_oid).unwrap();
    assert_eq!(
        rev,
        Revision {
            directory: tree_hash,
            parents: Vec::new(),
            author: bs("Test User <test@example.com>"),
            author_timestamp: 1763027354,
            author_timestamp_offset: bs("+0100"),
            committer: bs("Test User <test@example.com>"),
            committer_timestamp: 1763027354,
            committer_timestamp_offset: bs("+0100"),
            extra_headers: Vec::new(),
            message: Some(bs("Test commit")),
        }
    );

    // With the pragmatic approach, SWHID = Git commit OID directly
    // This ensures GPG signatures are preserved (they're part of the Git object)
    let swhid = revision_swhid(&repo, &commit_oid).unwrap();
    
    // Verify SWHID matches Git OID (this is the key for signed object support)
    assert_eq!(swhid.digest_bytes(), commit_oid.as_bytes());
    assert_eq!(swhid.to_string(), format!("swh:1:rev:{}", commit_oid));
}

#[test]
fn test_release_swhid() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();

    // Create content
    let mut index = repo.index().unwrap();
    let file_path = tmp.child("test.txt");
    file_path.write_str("test content").unwrap();

    // Create directory
    index
        .add_path(file_path.path().strip_prefix(tmp.path()).unwrap())
        .unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree_hash = hex::decode("0efb37b28c53c7e4fbd253bb04a4df14008f63fe")
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(oid_to_array(tree_oid), tree_hash);
    let tree = repo.find_tree(tree_oid).unwrap();

    // Create tag
    let sig = Signature::new("Test User", "test@example.com", &Time::new(1763027354, 60)).unwrap();
    let tag_oid = repo
        .tag(
            "v1.0",
            &tree.into_object(),
            &sig,
            "Test tag",
            /* force= */ false,
        )
        .unwrap();

    let rev = release_from_git(&repo, &tag_oid).unwrap();
    assert_eq!(
        rev,
        Release {
            object: tree_hash,
            object_type: ReleaseTargetType::Directory,
            name: bs("v1.0"),
            author: Some(bs("Test User <test@example.com>")),
            author_timestamp: Some(1763027354),
            author_timestamp_offset: Some(bs("+0100")),
            extra_headers: Vec::new(),
            message: Some(bs("Test tag")),
        }
    );

    // With the pragmatic approach, SWHID = Git tag OID directly
    // This ensures GPG signatures are preserved (they're part of the Git object)
    let swhid = release_swhid(&repo, &tag_oid).unwrap();
    
    // Verify SWHID matches Git OID (this is the key for signed object support)
    assert_eq!(swhid.digest_bytes(), tag_oid.as_bytes());
    assert_eq!(swhid.to_string(), format!("swh:1:rel:{}", tag_oid));
}

#[test]
fn test_snapshot_swhid() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();

    // Create content
    let mut index = repo.index().unwrap();
    let file_path = tmp.child("test.txt");
    file_path.write_str("test content").unwrap();

    // Create directory
    index
        .add_path(file_path.path().strip_prefix(tmp.path()).unwrap())
        .unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree_hash = hex::decode("0efb37b28c53c7e4fbd253bb04a4df14008f63fe")
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(oid_to_array(tree_oid), tree_hash);
    let tree = repo.find_tree(tree_oid).unwrap();

    // Add reference directly to a tree
    repo.reference(
        "refs/heads/tree-branch",
        tree_oid,
        /* force: */ false,
        "log message",
    )
    .unwrap();

    // Create commit
    let sig = Signature::new("Test User", "test@example.com", &Time::new(1763027354, 60)).unwrap();
    let commit_oid = repo
        .commit(
            Some("refs/heads/main"),
            &sig,
            &sig,
            "Test commit",
            &tree,
            &[],
        )
        .unwrap();
    let commit_hash = hex::decode("07cde6575fb633ef9b5ecbe730e6eb97475a2fd9")
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(oid_to_array(commit_oid), commit_hash);

    // Create tag
    let tag_oid = repo
        .tag(
            "v1.0",
            &tree.into_object(),
            &sig,
            "Test tag",
            /* force: */ false,
        )
        .unwrap();
    let tag_hash = hex::decode("46d326edb8bfc49b757ccd09930365595806bfc0")
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(oid_to_array(tag_oid), tag_hash);

    let snp = snapshot_from_git(&repo).unwrap();
    // Snapshot includes HEAD pointing to main, plus all other branches
    // Branches are sorted by name, so HEAD comes first
    let expected_branches = vec![
        Branch {
            name: bs("HEAD"),
            target: BranchTarget::Alias(Some(bs("refs/heads/main"))),
        },
        Branch {
            name: bs("refs/heads/main"),
            target: BranchTarget::Revision(Some(commit_hash)),
        },
        Branch {
            name: bs("refs/heads/tree-branch"),
            target: BranchTarget::Directory(Some(tree_hash)),
        },
        Branch {
            name: bs("refs/tags/v1.0"),
            target: BranchTarget::Release(Some(tag_hash)),
        },
    ];
    assert_eq!(snp, Snapshot::new(expected_branches).unwrap());

    // Verify snapshot SWHID computes correctly
    // Note: With the pragmatic approach, revision/release SWHIDs match Git OIDs,
    // so the snapshot SWHID will be computed from those OIDs
    let computed_swhid = snapshot_swhid(&repo).unwrap();
    assert_eq!(computed_swhid.object_type(), ObjectType::Snapshot);
    
    // Verify it matches the snapshot computed from the parsed structure
    let expected_swhid = snp.swhid();
    assert_eq!(computed_swhid, expected_swhid);
}

#[test]
fn test_revision_swhid_matches_git_oid() {
    // Test that revision_swhid returns the Git commit OID directly
    // This ensures GPG signatures are included (they're part of the Git object)
    let tmp = assert_fs::TempDir::new().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();

    let mut index = repo.index().unwrap();
    let file_path = tmp.child("test.txt");
    file_path.write_str("test content").unwrap();

    index
        .add_path(file_path.path().strip_prefix(tmp.path()).unwrap())
        .unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();

    let sig = Signature::new("Test User", "test@example.com", &Time::new(1763027354, 60)).unwrap();
    let commit_oid = repo
        .commit(
            Some("refs/heads/main"),
            &sig,
            &sig,
            "Test commit",
            &tree,
            &[],
        )
        .unwrap();

    let swhid = revision_swhid(&repo, &commit_oid).unwrap();
    
    // The SWHID should match the Git commit OID
    assert_eq!(swhid.digest_bytes(), commit_oid.as_bytes());
    assert_eq!(swhid.to_string(), format!("swh:1:rev:{}", commit_oid));
}

#[test]
fn test_release_swhid_matches_git_oid() {
    // Test that release_swhid returns the Git tag OID directly
    // This ensures GPG signatures are included (they're part of the Git object)
    let tmp = assert_fs::TempDir::new().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();

    let mut index = repo.index().unwrap();
    let file_path = tmp.child("test.txt");
    file_path.write_str("test content").unwrap();

    index
        .add_path(file_path.path().strip_prefix(tmp.path()).unwrap())
        .unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();

    let sig = Signature::new("Test User", "test@example.com", &Time::new(1763027354, 60)).unwrap();
    let tag_oid = repo
        .tag(
            "v1.0",
            &tree.into_object(),
            &sig,
            "Test tag",
            false,
        )
        .unwrap();

    let swhid = release_swhid(&repo, &tag_oid).unwrap();
    
    // The SWHID should match the Git tag OID
    assert_eq!(swhid.digest_bytes(), tag_oid.as_bytes());
    assert_eq!(swhid.to_string(), format!("swh:1:rel:{}", tag_oid));
}

#[test]
fn test_signed_revision_swhid_principle() {
    // This test documents the expected behavior for signed revisions:
    // The SWHID should be the Git commit OID, which includes GPG signatures.
    // 
    // Note: Creating GPG-signed commits programmatically requires GPG setup.
    // This test verifies the principle: SWHID = Git OID for all commits.
    //
    // For actual signed commits, the Git OID will be different from an
    // unsigned commit with the same content, because the gpgsig header
    // is part of the object hash.
    let tmp = assert_fs::TempDir::new().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();

    let mut index = repo.index().unwrap();
    let file_path = tmp.child("test.txt");
    file_path.write_str("test content").unwrap();

    index
        .add_path(file_path.path().strip_prefix(tmp.path()).unwrap())
        .unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();

    let sig = Signature::new("Test User", "test@example.com", &Time::new(1763027354, 60)).unwrap();
    let commit_oid = repo
        .commit(
            Some("refs/heads/main"),
            &sig,
            &sig,
            "Test commit message",
            &tree,
            &[],
        )
        .unwrap();

    let swhid = revision_swhid(&repo, &commit_oid).unwrap();
    
    // Verify SWHID matches Git OID (this is the key principle for signed objects)
    assert_eq!(swhid.digest_bytes(), commit_oid.as_bytes());
    
    // The Git OID includes all object content, including any gpgsig headers
    // that would be present in a signed commit. By using the OID directly,
    // we ensure GPG signatures are preserved.
}

#[test]
fn test_signed_release_swhid_principle() {
    // This test documents the expected behavior for signed releases:
    // The SWHID should be the Git tag OID, which includes GPG signatures.
    //
    // Note: Creating GPG-signed tags programmatically requires GPG setup.
    // This test verifies the principle: SWHID = Git OID for all tags.
    //
    // For actual signed tags, the GPG signature is embedded in the tag message,
    // and the Git OID includes the entire message (including signature).
    let tmp = assert_fs::TempDir::new().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();

    let mut index = repo.index().unwrap();
    let file_path = tmp.child("test.txt");
    file_path.write_str("test content").unwrap();

    index
        .add_path(file_path.path().strip_prefix(tmp.path()).unwrap())
        .unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();

    let sig = Signature::new("Test User", "test@example.com", &Time::new(1763027354, 60)).unwrap();
    let tag_oid = repo
        .tag(
            "v1.0.0",
            &tree.into_object(),
            &sig,
            "Release message\n\nThis would contain GPG signature in real signed tags.",
            false,
        )
        .unwrap();

    let swhid = release_swhid(&repo, &tag_oid).unwrap();
    
    // Verify SWHID matches Git OID (this is the key principle for signed objects)
    assert_eq!(swhid.digest_bytes(), tag_oid.as_bytes());
    
    // The Git OID includes all tag content, including the message which would
    // contain GPG signatures in signed tags. By using the OID directly,
    // we ensure GPG signatures are preserved.
}
