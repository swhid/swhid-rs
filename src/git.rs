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
//! **Git SHA256 Support**:
//! - Full support for both SHA1 and SHA256 Git repositories
//! - Automatic detection of repository hash algorithm via `detect_repo_hash_algorithm()`
//! - Variable-length OID support throughout the codebase (20 bytes for SHA1, 32 bytes for SHA256)

use crate::types::SwhidVersion;
use crate::error::SwhidError;
use crate::Swhid;
use crate::config::HashConfig;
use std::path::Path;

use git2::{ObjectType as GitObjectType, Repository, Signature};

use crate::release::Release;
use crate::revision::Revision;
use crate::snapshot::{Branch, BranchTarget, Snapshot};
use crate::directory::{Entry, dir_manifest};
use crate::hash::hash_swhid_object_with;
use crate::Bytestring;

fn io_error(msg: String) -> SwhidError {
    SwhidError::Io(std::io::Error::other(msg))
}

/// Compute directory SWHID from a Git tree object using the specified hash configuration.
///
/// This recursively computes SWHIDs for nested objects (trees and blobs) using the
/// config's hash function, ensuring v2/SHA256 uses SHA256 for all nested objects.
fn directory_swhid_from_git_tree(
    repo: &Repository,
    tree: &git2::Tree<'_>,
    config: &HashConfig,
) -> Result<Vec<u8>, SwhidError> {
    const DIRECTORY_MODE: u32 = 0o040000;
    
    let mut entries = Vec::new();
    
    for entry in tree.iter() {
        let name_bytes = Box::from(entry.name_bytes());
        let entry_oid = entry.id();
        
        let (mode, id) = match entry.kind() {
            Some(GitObjectType::Tree) => {
                // Recursively compute nested tree SWHID
                let nested_tree = repo
                    .find_tree(entry_oid)
                    .map_err(|e| io_error(format!("Failed to find nested tree: {e}")))?;
                let nested_swhid = directory_swhid_from_git_tree(repo, &nested_tree, config)?;
                (DIRECTORY_MODE, nested_swhid)
            }
            Some(GitObjectType::Blob) => {
                // Compute blob SWHID
                let blob = repo
                    .find_blob(entry_oid)
                    .map_err(|e| io_error(format!("Failed to find blob: {e}")))?;
                let blob_swhid = crate::Content::from_bytes(blob.content())
                    .swhid_with_config(config)
                    .digest_bytes()
                    .to_vec();
                let mode = if entry.filemode() & 0o111 != 0 {
                    0o100755  // Executable
                } else {
                    0o100644  // Regular file
                };
                (mode, blob_swhid)
            }
            _ => {
                // Skip other types (submodules, etc.)
                continue;
            }
        };
        
        entries.push(Entry::new(name_bytes, mode, id));
    }
    
    // Build directory manifest and compute SWHID
    let manifest = dir_manifest(entries)
        .map_err(|e| io_error(format!("Failed to build directory manifest: {e}")))?;
    let digest = hash_swhid_object_with("tree", &manifest, config.hash_function.as_ref());
    Ok(digest)
}

/// Detect the hash algorithm used by a Git repository.
///
/// Returns "sha1" for SHA1 repositories (20-byte OIDs) or "sha256" for SHA256 repositories (32-byte OIDs).
///
/// This function automatically detects the hash algorithm by examining the OID size
/// of objects in the repository. Both SHA1 and SHA256 repositories are fully supported.
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
            Err(io_error("HEAD has no target OID".to_string()))
        }
    } else {
        // No HEAD, try to find any object in the repository
        // This is a fallback for empty or unusual repositories
        Err(io_error("Cannot detect hash algorithm: repository has no HEAD".to_string()))
    }
}

/// Convert a Git OID to a byte vector (supports both SHA1 and SHA256).
///
/// Returns a Vec<u8> with the OID bytes (20 bytes for SHA1, 32 bytes for SHA256).
pub fn oid_to_vec(oid: git2::Oid) -> Vec<u8> {
    oid.as_bytes().to_vec()
}

/// Convert a Git OID to a 20-byte array (SHA1, backward compatibility).
///
/// **Deprecated**: Use `oid_to_vec()` for new code that needs to support both SHA1 and SHA256.
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

fn parse_signature(sig: Signature<'_>) -> (Bytestring, i64, Bytestring) {
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

        // Empty line marks end of headers
        if first_line.is_empty() {
            break;
        }

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
        // with a space, which is the next key-value, or an empty line.
        while let Some(newline_position) = manifest.iter().position(|&byte| byte == b'\n') {
            let line = &manifest[..newline_position];
            match line.split_first() {
                None => {
                    // Empty line marks end of headers
                    break;
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

/// Extract extra headers and message from a raw Git tag object.
///
/// This parses the raw Git tag object to extract all extra headers,
/// including GPG signatures which may be stored in a `gpgsig` header.
///
/// Per SWHID spec 5.5, extra headers must be included in the serialization.
/// Returns (extra_headers, message).
///
/// The git2 crate does not provide direct access to tag extra headers,
/// so we read the raw object from the object database and parse it manually.
fn extract_tag_extra_headers_and_message(
    repo: &Repository,
    tag_oid: git2::Oid,
) -> Result<(Vec<(Bytestring, Bytestring)>, Option<Bytestring>), SwhidError> {
    // Read the raw tag object from the object database
    let odb = repo
        .odb()
        .map_err(|e| io_error(format!("Failed to get object database: {e}")))?;

    let odb_obj = odb
        .read(tag_oid)
        .map_err(|e| io_error(format!("Failed to read tag object: {e}")))?;

    // OdbObject::data() returns the decompressed object data
    // Format: "tag <length>\0<content>" for loose objects
    // or just "<content>" for packed objects (header is separate)
    let raw_data = odb_obj.data();

    // Try to find NULL byte (indicates header is included)
    let content = if let Some(null_pos) = raw_data.iter().position(|&b| b == 0) {
        // Header included, skip it
        &raw_data[null_pos + 1..]
    } else {
        // No NULL byte - this might be packed object or data is content-only
        // For packed objects, the type/length info is in the pack, not the data
        // The data should be the content directly
        raw_data
    };

    // Parse the tag object to extract headers and message
    // Git tag format:
    // object <hash>\n
    // type <type>\n
    // tag <name>\n
    // tagger <info>\n
    // <extra headers>\n (gpgsig, encoding, etc.)
    // \n
    // <message>

    let headers = parse_header(content)?;

    // Filter out standard headers and collect extra headers
    let extra_headers: Vec<(Bytestring, Bytestring)> = headers
        .into_iter()
        .filter(|(key, _value)| {
            !matches!(*key, b"object" | b"type" | b"tag" | b"tagger")
        })
        .map(|(key, value)| (key.into(), value))
        .collect();

    // Extract message: everything after the empty line
    // Find the first empty line (double newline)
    let empty_line_pos = content
        .windows(2)
        .position(|w| w == b"\n\n")
        .map(|pos| pos + 2);

    let message = if let Some(pos) = empty_line_pos {
        if pos < content.len() {
            Some(content[pos..].into())
        } else {
            None
        }
    } else {
        None
    };

    Ok((extra_headers, message))
}

/// Compute a SWHID v1.2 revision identifier from a Git commit
///
/// This implements the SWHID v1.2 revision hashing algorithm for Git commits,
/// creating a `swh:1:rev:<digest>` identifier according to the specification.
///
/// **Note**: This function always uses SHA1 (v1) configuration. For SHA256 repositories,
/// the function will still work correctly, but will produce v1 SWHIDs. To compute
/// v2 SWHIDs with SHA256, use `revision_swhid_with_config()` with an appropriate
/// `HashConfig`.
///
/// Both SHA1 and SHA256 Git repositories are fully supported.
pub fn revision_swhid(repo: &Repository, commit_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    // Always use v1 (SHA1) configuration for backward compatibility
    let config = HashConfig::v1();
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
    // Only compute nested objects with config for v2/SHA256, not v1
    let hash_config = if config.version == crate::types::SwhidVersion::V2 {
        Some(config)
    } else {
        None
    };
    revision_from_git(repo, commit_oid, hash_config).map(|rev| rev.swhid_with_config(config))
}

#[doc(hidden)]
pub fn revision_from_git(
    repo: &Repository,
    commit_oid: &git2::Oid,
    hash_config: Option<&HashConfig>,
) -> Result<Revision, SwhidError> {
    let commit = repo
        .find_commit(*commit_oid)
        .map_err(|e| io_error(format!("Failed to find commit: {e}")))?;

    let tree = commit
        .tree()
        .map_err(|e| io_error(format!("Failed to get commit tree: {e}")))?;

    let tree_oid = tree.id();

    // Compute directory SWHID using config's hash function if provided
    let directory = if let Some(config) = hash_config {
        // Compute directory SWHID with the config's hash function
        directory_swhid_from_git_tree(repo, &tree, config)?
    } else {
        // Use Git OID directly (v1 behavior)
        oid_to_vec(tree_oid)
    };

    // Compute parent revision SWHIDs using config's hash function if provided
    let parents = if let Some(config) = hash_config {
        commit
            .parents()
            .map(|parent| {
                // Recursively compute parent revision SWHID with config
                revision_swhid_with_config(repo, &parent.id(), config)
                    .map(|swhid| swhid.digest_bytes().to_vec())
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        // Use Git OIDs directly (v1 behavior)
        commit
            .parents()
            .map(|parent| oid_to_vec(parent.id()))
            .collect()
    };

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
        directory,
        parents,
        author,
        author_timestamp,
        author_timestamp_offset,
        committer,
        committer_timestamp,
        committer_timestamp_offset,
        extra_headers,
        message: {
            let msg_bytes = commit.message_bytes();
            if msg_bytes.is_empty() {
                None
            } else {
                Some(Bytestring::from(msg_bytes))
            }
        },
    })
}

/// Compute a SWHID v1.2 release identifier from a Git tag
///
/// This implements the SWHID v1.2 release hashing algorithm for Git tags,
/// creating a `swh:1:rel:<digest>` identifier according to the specification.
///
/// **Note**: This function always uses SHA1 (v1) configuration. For SHA256 repositories,
/// the function will still work correctly, but will produce v1 SWHIDs. To compute
/// v2 SWHIDs with SHA256, use `release_swhid_with_config()` with an appropriate
/// `HashConfig`.
///
/// Both SHA1 and SHA256 Git repositories are fully supported.
pub fn release_swhid(repo: &Repository, tag_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    // Always use v1 (SHA1) configuration for backward compatibility
    let config = HashConfig::v1();
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
    // Only compute nested objects with config for v2/SHA256, not v1
    let hash_config = if config.version == crate::types::SwhidVersion::V2 {
        Some(config)
    } else {
        None
    };
    release_from_git(repo, tag_oid, hash_config).map(|rel| rel.swhid_with_config(config))
}

#[doc(hidden)]
pub fn release_from_git(repo: &Repository, tag_oid: &git2::Oid, hash_config: Option<&HashConfig>) -> Result<Release, SwhidError> {
    use crate::release::ReleaseTargetType;

    let tag = repo
        .find_tag(*tag_oid)
        .map_err(|e| io_error(format!("Failed to find tag: {e}")))?;

    let target = tag
        .target()
        .map_err(|e| io_error(format!("Failed to get tag target: {e}")))?;
    let target_oid = target.id();

    // Compute target object SWHID using config's hash function if provided
    let object = if let Some(config) = hash_config {
        // Compute SWHID for target object with the config's hash function
        match target.kind() {
            Some(GitObjectType::Commit) => {
                revision_swhid_with_config(repo, &target_oid, config)?
                    .digest_bytes()
                    .to_vec()
            }
            Some(GitObjectType::Tree) => {
                let tree = repo
                    .find_tree(target_oid)
                    .map_err(|e| io_error(format!("Failed to find tree: {e}")))?;
                directory_swhid_from_git_tree(repo, &tree, config)?
            }
            Some(GitObjectType::Blob) => {
                let blob = repo
                    .find_blob(target_oid)
                    .map_err(|e| io_error(format!("Failed to find blob: {e}")))?;
                crate::Content::from_bytes(blob.content())
                    .swhid_with_config(config)
                    .digest_bytes()
                    .to_vec()
            }
            Some(GitObjectType::Tag) => {
                // Recursively compute release SWHID
                release_swhid_with_config(repo, &target_oid, config)?
                    .digest_bytes()
                    .to_vec()
            }
            _ => return Err(io_error("Unknown target type".to_string())),
        }
    } else {
        // Use Git OID directly (v1 behavior)
        oid_to_vec(target_oid)
    };

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

    // Extract extra headers and message from raw Git tag object
    // Per SWHID spec 5.5, extra headers must be included in serialization
    // Also extract message directly from raw object to ensure exact byte match
    let (extra_headers, message) = extract_tag_extra_headers_and_message(repo, *tag_oid)?;

    Ok(Release {
        object,
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
        extra_headers,
        message,
    })
}

/// Compute a SWHID v1.2 snapshot identifier from a Git repository
///
/// This implements the SWHID v1.2 snapshot hashing algorithm for Git repositories,
/// creating a `swh:1:snp:<digest>` identifier according to the specification.
///
/// **Note**: This function always uses SHA1 (v1) configuration. For SHA256 repositories,
/// the function will still work correctly, but will produce v1 SWHIDs. To compute
/// v2 SWHIDs with SHA256, use `snapshot_swhid_with_config()` with an appropriate
/// `HashConfig`.
///
/// Both SHA1 and SHA256 Git repositories are fully supported.
pub fn snapshot_swhid(repo: &Repository) -> Result<Swhid, SwhidError> {
    // Always use v1 (SHA1) configuration for backward compatibility
    let config = HashConfig::v1();
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
    let mut branches = Vec::new();
    repo.references()
        .map_err(|e| io_error(format!("Failed to get references: {e}")))?;
    let refs = repo
        .references()
        .map_err(|e| io_error(format!("Failed to get references: {e}")))?;
    for reference in refs {
        let reference = reference.map_err(|e| io_error(format!("Failed to get reference: {e}")))?;
        let name = reference.name_bytes();
        let target = reference_to_branch_target(repo, &reference, name)?;
        branches.push(Branch {
            name: name.into(),
            target,
        });
    }
    Ok(Snapshot::new(branches).map_err(|e| SwhidError::InvalidFormat(format!("{e}")))?)
}

fn reference_to_branch_target(
    repo: &Repository,
    reference: &git2::Reference,
    name: &[u8],
) -> Result<BranchTarget, SwhidError> {
    match reference.kind() {
        None => {
            // Dangling reference (reference points to a non-existent object).
            //
            // Per SWHID specification issue #64, the behavior for dangling references
            // is not fully defined. We treat dangling references as pointing to a
            // revision (commit) with None ID, which allows snapshot computation to
            // proceed while preserving the reference structure.
            //
            // See: https://github.com/swhid/specification/issues/64
            if reference.target().is_some() {
                return Err(io_error(format!(
                    "Reference {} has None kind, but has a target",
                    String::from_utf8_lossy(name)
                )));
            }
            if reference.symbolic_target_bytes().is_some() {
                return Err(io_error(format!(
                    "Reference {} has None kind, but has a symbolic target",
                    String::from_utf8_lossy(name)
                )));
            }
            Ok(BranchTarget::Revision(None))
        }
        Some(git2::ReferenceType::Direct) => {
            let Some(target_id) = reference.target() else {
                return Err(io_error(format!(
                    "Reference {} has Direct kind, but has no target",
                    String::from_utf8_lossy(name)
                )));
            };
            let target = repo
                .find_object(target_id, None)
                .map_err(|e| io_error(format!("Could not find object {target_id}: {e}")))?;
            let target_id = oid_to_vec(target_id);
            Ok(match target.kind() {
                None => {
                    // Dangling reference (object exists but has no type).
                    //
                    // Per SWHID specification issue #64, the behavior for dangling references
                    // is not fully defined. We treat such references as pointing to a revision
                    // (commit) to allow snapshot computation to proceed.
                    //
                    // See: https://github.com/swhid/specification/issues/64
                    BranchTarget::Revision(Some(target_id))
                }
                Some(git2::ObjectType::Any) => panic!("git2 returned an object with type 'Any'"),
                Some(git2::ObjectType::Commit) => BranchTarget::Revision(Some(target_id)),
                Some(git2::ObjectType::Tree) => BranchTarget::Directory(Some(target_id)),
                Some(git2::ObjectType::Blob) => BranchTarget::Content(Some(target_id)),
                Some(git2::ObjectType::Tag) => BranchTarget::Release(Some(target_id)),
            })
        }
        Some(git2::ReferenceType::Symbolic) => {
            let Some(symbolic_target) = reference.symbolic_target_bytes() else {
                return Err(io_error(format!(
                    "Reference {} has Symbolic kind, but has no symbolic target",
                    String::from_utf8_lossy(name)
                )));
            };
            Ok(BranchTarget::Alias(Some(symbolic_target.into())))
        }
    }
}

/// Open a Git repository at the given path.
pub fn open_repo(path: &Path) -> Result<Repository, SwhidError> {
    Repository::open(path).map_err(|e| io_error(format!("Failed to open repository: {e}")))
}

/// Get the HEAD commit OID from a repository.
pub fn get_head_commit(repo: &Repository) -> Result<git2::Oid, SwhidError> {
    let head = repo.head().map_err(|e| io_error(format!("Failed to get HEAD: {e}")))?;
    head.target()
        .ok_or_else(|| io_error("HEAD has no target".to_string()))
}

/// Get all tag OIDs in a repository.
pub fn get_tags(repo: &Repository) -> Result<Vec<git2::Oid>, SwhidError> {
    let mut tags = Vec::new();
    repo.tag_foreach(|oid, _name| {
        tags.push(oid);
        true
    })
    .map_err(|e| io_error(format!("Failed to list tags: {e}")))?;
    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Repository, Signature};
    use tempfile::TempDir;

    #[test]
    fn detect_sha1_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let repo = Repository::init(repo_path).unwrap();

        let sig = Signature::now("Test", "test@example.com").unwrap();
        let mut index = repo.index().unwrap();
        let oid = index.write_tree().unwrap();
        let tree = repo.find_tree(oid).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Test commit", &tree, &[]).unwrap();

        let algo = detect_repo_hash_algorithm(&repo).unwrap();
        assert_eq!(algo, "sha1");
    }

    #[test]
    fn detect_sha256_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let repo = Repository::init(repo_path).unwrap();

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
    fn revision_swhid_v1_v2_consistency() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let repo = Repository::init(repo_path).unwrap();

        let sig = Signature::now("Test", "test@example.com").unwrap();
        let mut index = repo.index().unwrap();
        let oid = index.write_tree().unwrap();
        let tree = repo.find_tree(oid).unwrap();
        let commit_oid = repo.commit(Some("HEAD"), &sig, &sig, "Test commit", &tree, &[]).unwrap();

        let v1_config = HashConfig::v1();
        let v2_config = HashConfig::v2_sha256_hex();

        let v1_swhid = revision_swhid_with_config(&repo, &commit_oid, &v1_config).unwrap();
        let v2_swhid = revision_swhid_with_config(&repo, &commit_oid, &v2_config).unwrap();

        assert_eq!(v1_swhid.version(), SwhidVersion::V1);
        assert_eq!(v2_swhid.version(), SwhidVersion::V2);
        // Different hash functions produce different digests
        assert_ne!(v1_swhid.digest_bytes(), v2_swhid.digest_bytes());
    }
}
