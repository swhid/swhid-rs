#![cfg(feature = "git")]

use std::collections::HashMap;

use assert_fs::prelude::*;
use git2::{Repository, Signature, Time};

use swhid::git::*;
use swhid::release::{Release, ReleaseTargetType};
use swhid::revision::Revision;
use swhid::snapshot::{Branch, BranchTarget, Snapshot};

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

    let rev = revision_from_git(&repo, &commit_oid, &mut HashMap::new()).unwrap();
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

    let swhid = revision_swhid(&repo, &commit_oid, &mut HashMap::new()).unwrap();
    assert_eq!(
        swhid.to_string(),
        "swh:1:rev:07cde6575fb633ef9b5ecbe730e6eb97475a2fd9"
    );
}

#[test]
fn test_revision_negative_timezone() {
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
    let sig = Signature::new(
        "Test User",
        "test@example.com",
        &Time::new(1763027354, -420),
    )
    .unwrap();
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

    let rev = revision_from_git(&repo, &commit_oid, &mut HashMap::new()).unwrap();
    assert_eq!(
        rev,
        Revision {
            directory: tree_hash,
            parents: Vec::new(),
            author: bs("Test User <test@example.com>"),
            author_timestamp: 1763027354,
            author_timestamp_offset: bs("-0700"),
            committer: bs("Test User <test@example.com>"),
            committer_timestamp: 1763027354,
            committer_timestamp_offset: bs("-0700"),
            extra_headers: Vec::new(),
            message: Some(bs("Test commit")),
        }
    );

    let swhid = revision_swhid(&repo, &commit_oid, &mut HashMap::new()).unwrap();
    assert_eq!(
        swhid.to_string(),
        "swh:1:rev:927b3b4a8291765c874d26560da492de7b3d9091"
    );
}

#[test]
fn test_signed_revision_swhid() {
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
    let gpgsig = "-----BEGIN PGP SIGNATURE-----\nblah blah blah\n-----END PGP SIGNATURE-----";
    let buf = repo
        .commit_create_buffer(&sig, &sig, "Test commit", &tree, &[])
        .unwrap();
    let commit_oid = repo
        .commit_signed(
            buf.as_str().unwrap(),
            gpgsig,
            /* signature_field: */ None,
        )
        .unwrap();

    let rev = revision_from_git(&repo, &commit_oid, &mut HashMap::new()).unwrap();
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
            extra_headers: vec![(bs("gpgsig"), bs(gpgsig))],
            message: Some(bs("Test commit")),
        }
    );

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    // using this script:
    // >>> from swh.model.model import *
    // >>> person = Person.from_fullname(b"Test User <test@example.com>")
    // >>> ts = TimestampWithTimezone(timestamp=Timestamp(seconds=1763027354, microseconds=0), offset_bytes=b"+0100")
    // >>> rev = Revision(directory=bytes.fromhex("0efb37b28c53c7e4fbd253bb04a4df14008f63fe"), message=b"Test commit", author=person, committer=person, date=ts, committer_date=ts, extra_headers=((b"gpgsig", b"-----BEGIN PGP SIGNATURE-----\nblah blah blah\n-----END PGP SIGNATURE-----"),), type=RevisionType.GIT, synthetic=False)
    // >>> rev.swhid()
    // CoreSWHID.from_string('c488a708317e88a4059d6e990f5d9004c4a4c205')

    let swhid = revision_swhid(&repo, &commit_oid, &mut HashMap::new()).unwrap();
    assert_eq!(
        swhid.to_string(),
        "swh:1:rev:c488a708317e88a4059d6e990f5d9004c4a4c205"
    );
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

    let swhid = release_swhid(&repo, &tag_oid).unwrap();
    assert_eq!(
        swhid.to_string(),
        "swh:1:rel:46d326edb8bfc49b757ccd09930365595806bfc0"
    );
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

    repo.set_head("refs/heads/main").unwrap();

    let snp = snapshot_from_git(&repo).unwrap();
    assert_eq!(
        snp,
        Snapshot::new(vec![
            Branch {
                name: bs("HEAD"),
                target: BranchTarget::Alias(Some(bs("refs/heads/main"))),
            },
            Branch {
                name: bs("refs/heads/main"),
                target: BranchTarget::Revision(Some(commit_hash)),
            },
            Branch {
                name: bs("refs/tags/v1.0"),
                target: BranchTarget::Release(Some(tag_hash)),
            },
            Branch {
                name: bs("refs/heads/tree-branch"),
                target: BranchTarget::Directory(Some(tree_hash)),
            },
        ])
        .unwrap(),
    );

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    // using:
    // Snapshot({
    //     b"refs/heads/tree-branch": SnapshotBranch(target=bytes.fromhex("0efb37b28c53c7e4fbd253bb04a4df14008f63fe"), target_type=TargetType.DIRECTORY),
    //     b"refs/heads/main": SnapshotBranch(target=bytes.fromhex("07cde6575fb633ef9b5ecbe730e6eb97475a2fd9"), target_type=TargetType.REVISION),
    //     b"refs/tags/v1.0": SnapshotBranch(target=bytes.fromhex("46d326edb8bfc49b757ccd09930365595806bfc0"), target_type=TargetType.RELEASE),
    //     b"HEAD": SnapshotBranch(target=b"refs/heads/main", target_type=TargetType.ALIAS),
    // }).swhid()
    assert_eq!(
        snapshot_swhid(&repo).unwrap().to_string(),
        "swh:1:snp:a0bfd8450daaf74c55c2375f21e40745bc5f95b7"
    );
}

/// Snapshot with a dangling branch (ref pointing to a missing object) is computed successfully;
/// the dangling branch is included with empty target per SWHID v1.2 Clause 5.6.
#[test]
fn test_snapshot_swhid_with_dangling_branch() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();

    // One valid commit so the repo has normal refs and HEAD.
    let mut index = repo.index().unwrap();
    let file_path = tmp.child("f.txt");
    file_path.write_str("x").unwrap();
    index
        .add_path(file_path.path().strip_prefix(tmp.path()).unwrap())
        .unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let sig = Signature::new("U", "u@x", &Time::new(1763027354, 60)).unwrap();
    repo.commit(Some("refs/heads/main"), &sig, &sig, "commit", &tree, &[])
        .unwrap();
    repo.set_head("refs/heads/main").unwrap();

    // Dangling ref: write ref file pointing to an OID that does not exist in the object store.
    // (git2 refuses to create such a ref via API, so we write the ref file directly.)
    let git_dir = tmp.path().join(".git");
    let ref_path = git_dir.join("refs/heads/dangling");
    std::fs::create_dir_all(ref_path.parent().unwrap()).unwrap();
    std::fs::write(&ref_path, "0000000000000000000000000000000000000000\n").unwrap();

    let snp = snapshot_from_git(&repo).unwrap();
    let branches: Vec<_> = snp.branches().to_vec();
    assert!(
        branches
            .iter()
            .any(|b| b.name.as_ref() == b"refs/heads/dangling"
                && matches!(b.target, BranchTarget::Revision(None))),
        "snapshot should contain dangling branch with empty target; got {:?}",
        branches
    );
    // Snapshot computation must not fail; SWHID is stable for this repo.
    let _ = snapshot_swhid(&repo).unwrap();
}
