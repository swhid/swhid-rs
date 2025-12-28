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
//!
//! Also supports SWHID v2 with SHA256 Git repositories.

use crate::error::SwhidError;
use crate::Swhid;
use crate::config::HashConfig;
use std::path::Path;

use git2::{ObjectType as GitObjectType, Repository, Signature};

use crate::release::Release;
use crate::revision::Revision;
use crate::snapshot::{Branch, BranchTarget, Snapshot};
use crate::Bytestring;

fn io_error(msg: String) -> SwhidError {
    SwhidError::Io(std::io::Error::other(msg))
}

/// Detect the hash algorithm used by a Git repository.
///
/// Returns "sha1" for SHA1 repositories (20-byte OIDs) or "sha256" for SHA256 repositories (32-byte OIDs).
pub fn detect_repo_hash_algorithm(repo: &Repository) -> Result<&'static str, SwhidError> {
    // Try to get any object to determine OID size
    // We'll check HEAD commit if available, or use a different method
    if let Ok(head) = repo.head() {
        if let Some(oid) = head.target() {
            let oid_bytes = oid.as_bytes();
            match oid_bytes.len() {
                20 => Ok("sha1"),
                32 => Ok("sha256"),
                _ => Err(io_error(format!(
                    "Unexpected OID length: {} (expected 20 or 32 bytes)",
                    oid_bytes.len()
                ))),
            }
        } else {
            // Fallback: assume SHA1 for compatibility
            Ok("sha1")
        }
    } else {
        // Fallback: assume SHA1 for compatibility
        Ok("sha1")
    }
}

/// Convert a Git OID to a byte array (supports both SHA1 and SHA256).
///
/// Returns a Vec<u8> with the OID bytes (20 bytes for SHA1, 32 bytes for SHA256).
pub fn oid_to_vec(oid: git2::Oid) -> Vec<u8> {
    oid.as_bytes().to_vec()
}

/// Convert a Git OID to a 20-byte array (SHA1, backward compatibility).
///
/// This function is for backward compatibility with existing code that expects [u8; 20].
/// For SHA256 repositories, this will return an error.
pub fn oid_to_array(oid: git2::Oid) -> Result<[u8; 20], SwhidError> {
    let bytes = oid.as_bytes();
    if bytes.len() != 20 {
        return Err(io_error(format!(
            "Expected 20-byte OID (SHA1), got {} bytes (possibly SHA256 repository)",
            bytes.len()
        )));
    }
    bytes
        .try_into()
        .map_err(|e| io_error(format!("Unexpected OID conversion error: {e}")))
}

fn parse_signature(sig: Signature) -> (Bytestring, i64, Bytestring) {
    let name = sig.name_bytes();
    let email = sig.email_bytes();

    let mut full_name = Vec::with_capacity(name.len() + email.len() + 3);
    full_name.extend_from_slice(name);
    full_name.extend_from_slice(b" <");
    full_name.extend_from_slice(email);
    full_name.push(b'>');

    let when = sig.when();
    let offset_minutes = when.offset_minutes();
    let offset_hours = offset_minutes / 60;
    let offset_minutes = offset_minutes % 60;
    let sign = when.sign();
    let offset = format!("{sign}{offset_hours:02}{offset_minutes:02}");

    (full_name.into(), when.seconds(), offset.into_bytes().into())
}

/// Returns key-value pairs and the message
fn parse_header(mut manifest: &[u8]) -> Result<Vec<(&[u8], Bytestring)>, SwhidError> {
    let mut headers = Vec::new();
    while !manifest.is_empty() {
        // Pop first line
        let Some(newline_position) = manifest.iter().position(|&byte| byte == b'\n') else {
            return Err(io_error("Header line is missing a line end".to_owned()));
        };
        let first_line = &manifest[..newline_position];
        manifest = &manifest[newline_position + 1..];

        // The first line is a key and a value. Extract the key and the first line of the value
        let Some(delimiter_position) = first_line.iter().position(|&byte| byte == b' ') else {
            return Err(io_error("Header line is missing a value".to_owned()));
        };
        let key = &first_line[..delimiter_position];
        if key.is_empty() {
            return Err(io_error("Empty key".to_owned()));
        };
        let mut value = first_line[delimiter_position + 1..].to_vec();

        // Read line by line until we find one that does not start
        // with a space, which is the next key-value.
        while let Some(newline_position) = manifest.iter().position(|&byte| byte == b'\n') {
            let line = &manifest[..newline_position];
            match line.split_first() {
                None => {
                    return Err(io_error("Empty line".to_owned()));
                }
                Some((b' ', value_line)) => {
                    // continuation line
                    value.push(b'\n');
                    value.extend_from_slice(value_line);
                }
                Some(_) => {
                    // new key-value pair
                    break;
                }
            }
            manifest = &manifest[newline_position + 1..];
        }
        headers.push((key, value.into_boxed_slice()));
    }

    Ok(headers)
}

/// Compute a SWHID v1.2 revision identifier from a Git commit
///
/// This implements the SWHID v1.2 revision hashing algorithm for Git commits,
/// creating a `swh:1:rev:<digest>` identifier according to the specification.
///
/// Automatically detects the repository's hash algorithm (SHA1 or SHA256) and
/// uses the appropriate configuration.
pub fn revision_swhid(repo: &Repository, commit_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    let hash_algo = detect_repo_hash_algorithm(repo)?;
    let config = match hash_algo {
        "sha1" => HashConfig::v1(),
        "sha256" => HashConfig::v2_sha256_hex(), // Use hex to match Git OID format
        _ => return Err(io_error(format!("Unsupported hash algorithm: {}", hash_algo))),
    };
    revision_swhid_with_config(repo, commit_oid, &config)
}

/// Compute a SWHID revision identifier from a Git commit using the specified hash configuration.
///
/// This allows computing SWHIDs with different hash functions and serialization formats
/// for v2 experimentation.
pub fn revision_swhid_with_config(
    repo: &Repository,
    commit_oid: &git2::Oid,
    config: &HashConfig,
) -> Result<Swhid, SwhidError> {
    revision_from_git(repo, commit_oid).map(|rev| rev.swhid_with_config(config))
}

#[doc(hidden)]
pub fn revision_from_git(
    repo: &Repository,
    commit_oid: &git2::Oid,
) -> Result<Revision, SwhidError> {
    let commit = repo
        .find_commit(*commit_oid)
        .map_err(|e| io_error(format!("Failed to find commit: {e}")))?;

    let tree = commit
        .tree()
        .map_err(|e| io_error(format!("Failed to get commit tree: {e}")))?;

    let tree_oid = tree.id();

    let (author, author_timestamp, author_timestamp_offset) = parse_signature(commit.author());
    let (committer, committer_timestamp, committer_timestamp_offset) =
        parse_signature(commit.committer());

    let headers = parse_header(commit.raw_header_bytes())?;

    let extra_headers = headers
        .into_iter()
        .filter(|(key, _value)| !matches!(*key, b"tree" | b"parent" | b"author" | b"committer"))
        .map(|(key, value)| (key.into(), value))
        .collect();

    Ok(Revision {
        directory: oid_to_array(tree_oid)?,
        parents: commit
            .parents()
            .map(|parent| oid_to_array(parent.id()))
            .collect::<Result<_, _>>()?,
        author,
        author_timestamp,
        author_timestamp_offset,
        committer,
        committer_timestamp,
        committer_timestamp_offset,
        extra_headers,
        message: Some(commit.message_bytes().into()),
    })
}

/// Compute a SWHID v1.2 release identifier from a Git tag
///
/// This implements the SWHID v1.2 release hashing algorithm for Git tags,
/// creating a `swh:1:rel:<digest>` identifier according to the specification.
///
/// Automatically detects the repository's hash algorithm (SHA1 or SHA256) and
/// uses the appropriate configuration.
pub fn release_swhid(repo: &Repository, tag_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    let hash_algo = detect_repo_hash_algorithm(repo)?;
    let config = match hash_algo {
        "sha1" => HashConfig::v1(),
        "sha256" => HashConfig::v2_sha256_hex(), // Use hex to match Git OID format
        _ => return Err(io_error(format!("Unsupported hash algorithm: {}", hash_algo))),
    };
    release_swhid_with_config(repo, tag_oid, &config)
}

/// Compute a SWHID release identifier from a Git tag using the specified hash configuration.
///
/// This allows computing SWHIDs with different hash functions and serialization formats
/// for v2 experimentation.
pub fn release_swhid_with_config(
    repo: &Repository,
    tag_oid: &git2::Oid,
    config: &HashConfig,
) -> Result<Swhid, SwhidError> {
    release_from_git(repo, tag_oid).map(|rel| rel.swhid_with_config(config))
}

#[doc(hidden)]
pub fn release_from_git(repo: &Repository, tag_oid: &git2::Oid) -> Result<Release, SwhidError> {
    use crate::release::ReleaseTargetType;

    let tag = repo
        .find_tag(*tag_oid)
        .map_err(|e| io_error(format!("Failed to find tag: {e}")))?;

    let target = tag
        .target()
        .map_err(|e| io_error(format!("Failed to get tag target: {e}")))?;
    let target_oid = target.id();

    let (author, author_timestamp, author_timestamp_offset) = match tag.tagger() {
        Some(tagger) => {
            let (author, author_timestamp, author_timestamp_offset) = parse_signature(tagger);
            (
                Some(author),
                Some(author_timestamp),
                Some(author_timestamp_offset),
            )
        }
        None => (None, None, None),
    };

    Ok(Release {
        object: oid_to_array(target_oid)?,
        object_type: match target.kind() {
            Some(GitObjectType::Commit) => ReleaseTargetType::Revision,
            Some(GitObjectType::Tree) => ReleaseTargetType::Directory,
            Some(GitObjectType::Blob) => ReleaseTargetType::Content,
            Some(GitObjectType::Tag) => ReleaseTargetType::Release,
            _ => return Err(io_error("Unknown target type".to_string())),
        },
        name: tag.name_bytes().into(),
        author,
        author_timestamp,
        author_timestamp_offset,
        extra_headers: Vec::new(), // FIXME: does not seem to be exposed by git2
        message: tag.message_bytes().map(Into::into),
    })
}

/// Compute a SWHID v1.2 snapshot identifier from a Git repository
///
/// This implements the SWHID v1.2 snapshot hashing algorithm for Git repositories,
/// creating a `swh:1:snp:<digest>` identifier according to the specification.
///
/// Automatically detects the repository's hash algorithm (SHA1 or SHA256) and
/// uses the appropriate configuration.
pub fn snapshot_swhid(repo: &Repository) -> Result<Swhid, SwhidError> {
    let hash_algo = detect_repo_hash_algorithm(repo)?;
    let config = match hash_algo {
        "sha1" => HashConfig::v1(),
        "sha256" => HashConfig::v2_sha256_hex(), // Use hex to match Git OID format
        _ => return Err(io_error(format!("Unsupported hash algorithm: {}", hash_algo))),
    };
    snapshot_swhid_with_config(repo, &config)
}

/// Compute a SWHID snapshot identifier from a Git repository using the specified hash configuration.
///
/// This allows computing SWHIDs with different hash functions and serialization formats
/// for v2 experimentation.
pub fn snapshot_swhid_with_config(
    repo: &Repository,
    config: &HashConfig,
) -> Result<Swhid, SwhidError> {
    snapshot_from_git(repo).map(|snp| snp.swhid_with_config(config))
}

#[doc(hidden)]
pub fn snapshot_from_git(repo: &Repository) -> Result<Snapshot, SwhidError> {
    let references = repo
        .references()
        .map_err(|e| io_error(format!("Failed to list references: {e}")))?;

    let mut branches: Vec<_> = references
        .flat_map(|reference| match reference {
            Ok(reference) => reference_to_branch(repo, reference).transpose(),
            Err(e) => Some(Err(io_error(format!("Failed to read reference: {e}")))),
        })
        .collect::<Result<_, _>>()?;

    let head = repo
        .head()
        .map_err(|e| io_error(format!("Failed to get HEAD: {e}")))?;
    if let Some(head_branch) = reference_to_branch(repo, head)? {
        let Branch { name, target: _ } = head_branch;
        branches.push(Branch {
            name: (*b"HEAD").into(),
            target: BranchTarget::Alias(Some(name)),
        });
    }

    Snapshot::new(branches).map_err(|e| io_error(format!("Invalid snapshot: {e}")))
}

fn reference_to_branch(
    repo: &Repository,
    reference: git2::Reference<'_>,
) -> Result<Option<Branch>, SwhidError> {
    if !reference.is_branch() && !reference.is_tag() {
        return Ok(None);
    }

    let name = reference.name_bytes().to_owned().into_boxed_slice();
    let target = match reference.kind() {
        None => {
            // Dangling reference.
            //
            // FIXME: We need to define a type (because of
            // https://github.com/swhid/specification/issues/64), so let's assume it's
            // a commit.
            if reference.target().is_some() {
                return Err(io_error(format!(
                    "Reference {} has None kind, but has a target",
                    String::from_utf8_lossy(&name)
                )));
            }
            if reference.symbolic_target_bytes().is_some() {
                return Err(io_error(format!(
                    "Reference {} has None kind, but has a symbolic target",
                    String::from_utf8_lossy(&name)
                )));
            }
            BranchTarget::Revision(None)
        }
        Some(git2::ReferenceType::Direct) => {
            let Some(target_id) = reference.target() else {
                return Err(io_error(format!(
                    "Reference {} has Direct kind, but has no target",
                    String::from_utf8_lossy(&name)
                )));
            };
            let target = repo
                .find_object(target_id, None)
                .map_err(|e| io_error(format!("Could not find object {target_id}: {e}")))?;
            let target_id = oid_to_array(target_id)?;
            match target.kind() {
                None => {
                    // Dangling reference.
                    //
                    // FIXME: We need to define a type (because of
                    // https://github.com/swhid/specification/issues/64), so let's assume it's
                    // a commit.
                    BranchTarget::Revision(Some(target_id))
                }
                Some(git2::ObjectType::Any) => panic!("git2 returned an object with type 'Any'"),
                Some(git2::ObjectType::Commit) => BranchTarget::Revision(Some(target_id)),
                Some(git2::ObjectType::Tree) => BranchTarget::Directory(Some(target_id)),
                Some(git2::ObjectType::Blob) => BranchTarget::Content(Some(target_id)),
                Some(git2::ObjectType::Tag) => BranchTarget::Release(Some(target_id)),
            }
        }
        Some(git2::ReferenceType::Symbolic) => {
            let Some(target) = reference.symbolic_target_bytes() else {
                return Err(io_error(format!(
                    "Reference {} has Symbolic kind, but has no symbolic target",
                    String::from_utf8_lossy(&name)
                )));
            };
            BranchTarget::Alias(Some(target.into()))
        }
    };
    Ok(Some(Branch { name, target }))
}

/// Open a Git repository for SWHID v1.2 computation
///
/// This function opens a Git repository to enable SWHID v1.2 computation
/// for revision, release, and snapshot objects.
pub fn open_repo(path: &Path) -> Result<Repository, SwhidError> {
    Repository::open(path).map_err(|e| io_error(format!("Failed to open repository: {e}")))
}

/// Get the HEAD commit of a Git repository for SWHID v1.2 computation
pub fn get_head_commit(repo: &Repository) -> Result<git2::Oid, SwhidError> {
    let head = repo
        .head()
        .map_err(|e| io_error(format!("Failed to get HEAD: {e}")))?;

    head.target()
        .ok_or_else(|| io_error("HEAD is not a direct reference".to_string()))
}

/// Get all tags in a Git repository for SWHID v1.2 release computation
pub fn get_tags(repo: &Repository) -> Result<Vec<git2::Oid>, SwhidError> {
    let mut tags = Vec::new();
    let tag_names = repo
        .tag_names(None)
        .map_err(|e| io_error(format!("Failed to get tag names: {e}")))?;

    for tag_name in tag_names.iter().flatten() {
        if let Ok(tag_oid) = repo.refname_to_id(&format!("refs/tags/{tag_name}")) {
            tags.push(tag_oid);
        }
    }

    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::HashConfig;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_detect_repo_hash_algorithm_sha1() {
        // Create a temporary Git repository
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        
        // Initialize a Git repository (defaults to SHA1)
        let repo = Repository::init(repo_path).unwrap();
        
        // Create a test file and commit
        let sig = Signature::now("Test", "test@example.com").unwrap();
        let mut index = repo.index().unwrap();
        let oid = index.write_tree().unwrap();
        let tree = repo.find_tree(oid).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Test commit", &tree, &[]).unwrap();
        
        // Detect hash algorithm
        let algo = detect_repo_hash_algorithm(&repo).unwrap();
        assert_eq!(algo, "sha1");
    }

    #[test]
    fn test_oid_to_vec() {
        // Create a test OID (SHA1, 20 bytes)
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let repo = Repository::init(repo_path).unwrap();
        
        // Create a commit so HEAD exists
        let sig = Signature::now("Test", "test@example.com").unwrap();
        let mut index = repo.index().unwrap();
        let oid = index.write_tree().unwrap();
        let tree = repo.find_tree(oid).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Test commit", &tree, &[]).unwrap();
        
        let head = repo.head().unwrap();
        let oid = head.target().unwrap();
        
        let vec = oid_to_vec(oid);
        assert_eq!(vec.len(), 20); // SHA1 OID length
    }

    #[test]
    fn test_oid_to_array_sha1() {
        // Create a test OID (SHA1, 20 bytes)
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let repo = Repository::init(repo_path).unwrap();
        
        // Create a commit so HEAD exists
        let sig = Signature::now("Test", "test@example.com").unwrap();
        let mut index = repo.index().unwrap();
        let oid = index.write_tree().unwrap();
        let tree = repo.find_tree(oid).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Test commit", &tree, &[]).unwrap();
        
        let head = repo.head().unwrap();
        let oid = head.target().unwrap();
        
        let array = oid_to_array(oid).unwrap();
        assert_eq!(array.len(), 20);
    }

    #[test]
    fn test_revision_swhid_with_config() {
        // Create a temporary Git repository
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        
        let repo = Repository::init(repo_path).unwrap();
        
        // Create a test file and commit
        let sig = Signature::now("Test", "test@example.com").unwrap();
        let mut index = repo.index().unwrap();
        let oid = index.write_tree().unwrap();
        let tree = repo.find_tree(oid).unwrap();
        let commit_oid = repo.commit(Some("HEAD"), &sig, &sig, "Test commit", &tree, &[]).unwrap();
        
        // Test v1 config
        let v1_config = HashConfig::v1();
        let v1_swhid = revision_swhid_with_config(&repo, &commit_oid, &v1_config).unwrap();
        assert_eq!(v1_swhid.version(), "1");
        assert_eq!(v1_swhid.digest_bytes().len(), 20);
        
        // Test v2 config (will use SHA256 hash function)
        let v2_config = HashConfig::v2_sha256_hex();
        let v2_swhid = revision_swhid_with_config(&repo, &commit_oid, &v2_config).unwrap();
        assert_eq!(v2_swhid.version(), "2");
        assert_eq!(v2_swhid.digest_bytes().len(), 32);
    }
}
