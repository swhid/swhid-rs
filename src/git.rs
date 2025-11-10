//! SWHID v1.2 VCS integration for Git repositories
//!
//! This module provides SWHID v1.2 compliant functionality to compute SWHIDs
//! from Git repository objects when the `git` feature is enabled:
//! - Revision SWHIDs (commits) - `swh:1:rev:<digest>`
//! - Release SWHIDs (tags) - `swh:1:rel:<digest>`
//! - Snapshot SWHIDs (repository state) - `swh:1:snp:<digest>`
//!
//! This module implements the SWHID v1.2 specification for VCS objects,
//! using Git as the reference VCS implementation.

use crate::error::SwhidError;
use crate::Swhid;
use std::path::Path;

#[cfg(feature = "git")]
use git2::{ObjectType as GitObjectType, Repository};

/// Compute a SWHID v1.2 revision identifier from a Git commit
///
/// This implements the SWHID v1.2 revision hashing algorithm for Git commits,
/// creating a `swh:1:rev:<digest>` identifier according to the specification.
#[cfg(feature = "git")]
pub fn revision_swhid(repo: &Repository, commit_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    let commit = repo
        .find_commit(*commit_oid)
        .map_err(|e| SwhidError::Io(format!("Failed to find commit: {e}")))?;

    let tree = commit
        .tree()
        .map_err(|e| SwhidError::Io(format!("Failed to get commit tree: {e}")))?;

    let tree_oid = tree.id();

    // Create commit object content
    let mut commit_content = Vec::new();
    commit_content.extend_from_slice(b"tree ");
    commit_content.extend_from_slice(tree_oid.as_bytes());
    commit_content.push(b'\n');

    if commit.parents().next().is_some() {
        for parent in commit.parents() {
            commit_content.extend_from_slice(b"parent ");
            commit_content.extend_from_slice(parent.id().as_bytes());
            commit_content.push(b'\n');
        }
    }

    let author = commit.author();
    commit_content.extend_from_slice(b"author ");
    commit_content.extend_from_slice(author.to_string().as_bytes());
    commit_content.push(b'\n');

    let committer = commit.committer();
    commit_content.extend_from_slice(b"committer ");
    commit_content.extend_from_slice(committer.to_string().as_bytes());
    commit_content.push(b'\n');

    commit_content.push(b'\n');
    if let Some(message) = commit.message() {
        commit_content.extend_from_slice(message.as_bytes());
    }

    let digest = crate::hash::hash_swhid_object("commit", &commit_content);
    Ok(Swhid::new(crate::ObjectType::Revision, digest))
}

/// Compute a SWHID v1.2 release identifier from a Git tag
///
/// This implements the SWHID v1.2 release hashing algorithm for Git tags,
/// creating a `swh:1:rel:<digest>` identifier according to the specification.
#[cfg(feature = "git")]
pub fn release_swhid(repo: &Repository, tag_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    let tag = repo
        .find_tag(*tag_oid)
        .map_err(|e| SwhidError::Io(format!("Failed to find tag: {e}")))?;

    let target = tag
        .target()
        .map_err(|e| SwhidError::Io(format!("Failed to get tag target: {e}")))?;
    let target_oid = target.id();

    // Create tag object content
    let mut tag_content = Vec::new();
    tag_content.extend_from_slice(b"object ");
    tag_content.extend_from_slice(target_oid.as_bytes());
    tag_content.push(b'\n');

    tag_content.extend_from_slice(b"type ");
    match target.kind() {
        Some(GitObjectType::Commit) => tag_content.extend_from_slice(b"commit"),
        Some(GitObjectType::Tree) => tag_content.extend_from_slice(b"tree"),
        Some(GitObjectType::Blob) => tag_content.extend_from_slice(b"blob"),
        Some(GitObjectType::Tag) => tag_content.extend_from_slice(b"tag"),
        _ => return Err(SwhidError::Io("Unknown target type".to_string())),
    }
    tag_content.push(b'\n');

    if let Some(tagger) = tag.tagger() {
        tag_content.extend_from_slice(b"tagger ");
        tag_content.extend_from_slice(tagger.to_string().as_bytes());
        tag_content.push(b'\n');
    }

    tag_content.push(b'\n');
    if let Some(message) = tag.message() {
        tag_content.extend_from_slice(message.as_bytes());
    }

    let digest = crate::hash::hash_swhid_object("tag", &tag_content);
    Ok(Swhid::new(crate::ObjectType::Release, digest))
}

/// Compute a SWHID v1.2 snapshot identifier from a Git repository
///
/// This implements the SWHID v1.2 snapshot hashing algorithm for Git repositories,
/// creating a `swh:1:snp:<digest>` identifier according to the specification.
#[cfg(feature = "git")]
pub fn snapshot_swhid(repo: &Repository, commit_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    let commit = repo
        .find_commit(*commit_oid)
        .map_err(|e| SwhidError::Io(format!("Failed to find commit: {e}")))?;

    let _tree = commit
        .tree()
        .map_err(|e| SwhidError::Io(format!("Failed to get commit tree: {e}")))?;

    // Create snapshot content
    let mut snapshot_content = Vec::new();

    // Add revision SWHID
    let revision = revision_swhid(repo, commit_oid)?;
    snapshot_content.extend_from_slice(b"revision ");
    snapshot_content.extend_from_slice(revision.to_string().as_bytes());
    snapshot_content.push(b'\n');

    // Add directory SWHID
    let dir_swhid =
        crate::directory::DiskDirectoryBuilder::new(repo.path().parent().unwrap_or(Path::new(".")))
            .swhid()?;
    snapshot_content.extend_from_slice(b"directory ");
    snapshot_content.extend_from_slice(dir_swhid.to_string().as_bytes());
    snapshot_content.push(b'\n');

    let digest = crate::hash::hash_swhid_object("snapshot", &snapshot_content);
    Ok(Swhid::new(crate::ObjectType::Snapshot, digest))
}

/// Open a Git repository for SWHID v1.2 computation
///
/// This function opens a Git repository to enable SWHID v1.2 computation
/// for revision, release, and snapshot objects.
#[cfg(feature = "git")]
pub fn open_repo(path: &Path) -> Result<Repository, SwhidError> {
    Repository::open(path).map_err(|e| SwhidError::Io(format!("Failed to open repository: {e}")))
}

/// Get the HEAD commit of a Git repository for SWHID v1.2 computation
#[cfg(feature = "git")]
pub fn get_head_commit(repo: &Repository) -> Result<git2::Oid, SwhidError> {
    let head = repo
        .head()
        .map_err(|e| SwhidError::Io(format!("Failed to get HEAD: {e}")))?;

    head.target()
        .ok_or_else(|| SwhidError::Io("HEAD is not a direct reference".to_string()))
}

/// Get all tags in a Git repository for SWHID v1.2 release computation
#[cfg(feature = "git")]
pub fn get_tags(repo: &Repository) -> Result<Vec<git2::Oid>, SwhidError> {
    let mut tags = Vec::new();
    let tag_names = repo
        .tag_names(None)
        .map_err(|e| SwhidError::Io(format!("Failed to get tag names: {e}")))?;

    for tag_name in tag_names.iter().flatten() {
        if let Ok(tag_oid) = repo.refname_to_id(&format!("refs/tags/{tag_name}")) {
            tags.push(tag_oid);
        }
    }

    Ok(tags)
}

/// Check if a path is a Git repository for SWHID v1.2 computation
#[cfg(feature = "git")]
pub fn is_git_repo(path: &Path) -> bool {
    Repository::open(path).is_ok()
}

// Stub implementations when git feature is disabled
#[cfg(not(feature = "git"))]
pub fn revision_swhid(_repo: &(), _commit_oid: &()) -> Result<Swhid, SwhidError> {
    Err(SwhidError::Io(
        "Git feature not enabled - SWHID v1.2 VCS support requires 'git' feature".to_string(),
    ))
}

#[cfg(not(feature = "git"))]
pub fn release_swhid(_repo: &(), _tag_oid: &()) -> Result<Swhid, SwhidError> {
    Err(SwhidError::Io(
        "Git feature not enabled - SWHID v1.2 VCS support requires 'git' feature".to_string(),
    ))
}

#[cfg(not(feature = "git"))]
pub fn snapshot_swhid(_repo: &(), _commit_oid: &()) -> Result<Swhid, SwhidError> {
    Err(SwhidError::Io(
        "Git feature not enabled - SWHID v1.2 VCS support requires 'git' feature".to_string(),
    ))
}

#[cfg(not(feature = "git"))]
pub fn open_repo(_path: &Path) -> Result<(), SwhidError> {
    Err(SwhidError::Io(
        "Git feature not enabled - SWHID v1.2 VCS support requires 'git' feature".to_string(),
    ))
}

#[cfg(not(feature = "git"))]
pub fn get_head_commit(_repo: &()) -> Result<(), SwhidError> {
    Err(SwhidError::Io(
        "Git feature not enabled - SWHID v1.2 VCS support requires 'git' feature".to_string(),
    ))
}

#[cfg(not(feature = "git"))]
pub fn get_tags(_repo: &()) -> Result<Vec<()>, SwhidError> {
    Err(SwhidError::Io(
        "Git feature not enabled - SWHID v1.2 VCS support requires 'git' feature".to_string(),
    ))
}

#[cfg(not(feature = "git"))]
pub fn is_git_repo(_path: &Path) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ObjectType;
    use assert_fs::prelude::*;

    #[cfg(feature = "git")]
    #[test]
    fn test_git_repo_detection() {
        let tmp = assert_fs::TempDir::new().unwrap();
        assert!(!is_git_repo(tmp.path()));

        // Create a simple git repo
        let _repo = git2::Repository::init(tmp.path()).unwrap();
        assert!(is_git_repo(tmp.path()));
    }

    #[cfg(feature = "git")]
    #[test]
    fn test_revision_swhid() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let repo = git2::Repository::init(tmp.path()).unwrap();

        // Create a simple commit
        let mut index = repo.index().unwrap();
        let file_path = tmp.child("test.txt");
        file_path.write_str("test content").unwrap();

        index
            .add_path(file_path.path().strip_prefix(tmp.path()).unwrap())
            .unwrap();
        let tree_oid = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        let sig = git2::Signature::now("Test User", "test@example.com").unwrap();
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
        assert_eq!(swhid.object_type(), ObjectType::Revision);
    }

    #[cfg(not(feature = "git"))]
    #[test]
    fn test_git_disabled() {
        assert!(!is_git_repo(Path::new("/tmp")));
    }
}
