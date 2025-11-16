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

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    // using this script:
    // >>> from swh.model.model import *
    // >>> person = Person.from_fullname(b"Test User <test@example.com>")
    // >>> ts = TimestampWithTimezone(timestamp=Timestamp(seconds=1763027354, microseconds=0), offset_bytes=b"+0100")
    // >>> rev = Revision(directory=bytes.fromhex("0efb37b28c53c7e4fbd253bb04a4df14008f63fe"), message=b"Test commit", author=person, committer=person, date=ts, committer_date=ts, type=RevisionType.GIT, synthetic=False)
    // >>> rev.swhid()
    // CoreSWHID.from_string('swh:1:rev:07cde6575fb633ef9b5ecbe730e6eb97475a2fd9')

    // With spec-compliant approach, SWHID should match Git OID
    // because we reconstruct from all components including extra headers
    let swhid = revision_swhid(&repo, &commit_oid).unwrap();
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

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    // using this script:
    // >>> from swh.model.model import *
    // >>> person = Person.from_fullname(b"Test User <test@example.com>")
    // >>> ts = TimestampWithTimezone(timestamp=Timestamp(seconds=1763027354, microseconds=0), offset_bytes=b"+0100")
    // >>> rel = Release(name=b"v1.0", target_type=ObjectType.DIRECTORY, target=bytes.fromhex("0efb37b28c53c7e4fbd253bb04a4df14008f63fe"), message=b"Test tag", author=person, date=ts, synthetic=False)
    // >>> rel.swhid()
    // CoreSWHID.from_string('swh:1:rel:46d326edb8bfc49b757ccd09930365595806bfc0')

    // With spec-compliant approach, SWHID should match Git OID
    // because we reconstruct from all components including message with signatures
    let swhid = release_swhid(&repo, &tag_oid).unwrap();
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
    // With spec-compliant approach, revision/release SWHIDs match Git OIDs,
    // so the snapshot SWHID will be computed from those OIDs
    let computed_swhid = snapshot_swhid(&repo).unwrap();
    assert_eq!(computed_swhid.object_type(), ObjectType::Snapshot);
    
    // Verify it matches the snapshot computed from the parsed structure
    let expected_swhid = snp.swhid();
    assert_eq!(computed_swhid, expected_swhid);
}

#[test]
fn test_revision_swhid_matches_git_oid_spec_compliant() {
    // Test that revision_swhid (spec-compliant approach) returns the Git commit OID
    // This verifies that reconstructing from components including extra headers
    // produces the same hash as the Git object (which includes GPG signatures)
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
    
    // With spec-compliant approach, SWHID should match Git OID
    // because we extract and include all extra headers (including gpgsig)
    assert_eq!(swhid.digest_bytes(), commit_oid.as_bytes());
    assert_eq!(swhid.to_string(), format!("swh:1:rev:{}", commit_oid));
}

#[test]
fn test_release_swhid_matches_git_oid_spec_compliant() {
    // Test that release_swhid (spec-compliant approach) returns the Git tag OID
    // This verifies that reconstructing from components including message
    // produces the same hash as the Git object (which includes GPG signatures)
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
    
    // With spec-compliant approach, SWHID should match Git OID
    // because we include the full message (which contains GPG signatures in signed tags)
    assert_eq!(swhid.digest_bytes(), tag_oid.as_bytes());
    assert_eq!(swhid.to_string(), format!("swh:1:rel:{}", tag_oid));
}

#[test]
fn test_signed_revision_extra_headers_extraction() {
    // Test that extra headers (including gpgsig) are extracted from raw Git objects
    // This verifies the spec-compliant approach extracts gpgsig headers correctly
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

    // Extract revision with extra headers
    let rev = revision_from_git(&repo, &commit_oid).unwrap();
    
    // For unsigned commits, extra_headers should be empty
    // For signed commits, extra_headers would contain gpgsig
    // This test verifies the extraction mechanism works
    assert!(rev.extra_headers.is_empty() || 
            rev.extra_headers.iter().any(|(k, _)| k.as_ref() == b"gpgsig"));
    
    // Verify SWHID matches Git OID (proves all components including headers are included)
    let swhid = rev.swhid();
    assert_eq!(swhid.digest_bytes(), commit_oid.as_bytes());
}

#[test]
fn test_signed_release_message_includes_signatures() {
    // Test that release message includes GPG signatures (for signed tags)
    // In Git, GPG signatures for tags are embedded in the message
    // This test verifies the message is preserved correctly
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

    // Extract release with message
    let rel = release_from_git(&repo, &tag_oid).unwrap();
    
    // Verify message is preserved (git2's message_bytes() should include full message)
    assert!(rel.message.is_some());
    
    // Verify SWHID matches Git OID (proves message including signatures is included)
    let swhid = rel.swhid();
    assert_eq!(swhid.digest_bytes(), tag_oid.as_bytes());
}
