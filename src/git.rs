//! SWHID v1.2 VCS integration for Git repositories
//!
//! This module provides SWHID v1.2 compliant functionality to compute SWHIDs
//! from Git repository objects when the `git` feature is enabled:
//! - Revision SWHIDs (commits) - `swh:1:rev:<digest>`
//! - Release SWHIDs (tags) - `swh:1:rel:<digest>`
//! - Snapshot SWHIDs (repository state) - `swh:1:snp:<digest>`
//!
//! Computation is recursive per the spec: revision manifests use directory
//! SWHID and parent revision SWHIDs; release uses target SWHID; snapshot
//! branches use revision/release/directory/content SWHIDs; directory from
//! tree uses content/directory SWHIDs for entries.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::Path;

use git2::{ObjectType as GitObjectType, Repository, Signature};

use crate::directory::{dir_manifest, Entry as DirEntry};
use crate::error::SwhidError;
use crate::hash::{hash_content, hash_swhid_object};
use crate::release::Release;
use crate::revision::Revision;
use crate::snapshot::{Branch, BranchTarget, Snapshot};
use crate::Bytestring;
use crate::Swhid;

fn io_error(msg: String) -> SwhidError {
    SwhidError::Io(std::io::Error::other(msg))
}

/// Content SWHID digest (20 bytes) from a Git blob OID.
/// Per spec 5.2: intrinsic identifier is hash of blob object format.
fn content_swhid_from_blob(repo: &Repository, blob_oid: git2::Oid) -> Result<[u8; 20], SwhidError> {
    let blob = repo
        .find_blob(blob_oid)
        .map_err(|e| io_error(format!("Failed to find blob {blob_oid}: {e}")))?;
    let bytes = blob.content();
    Ok(hash_content(bytes))
}

/// Directory SWHID digest (20 bytes) from a Git tree OID.
/// Per spec 5.3: compute SWHID of each entry (content or directory), then manifest.
fn directory_swhid_from_tree(
    repo: &Repository,
    tree_oid: git2::Oid,
    cache: &mut HashMap<git2::Oid, [u8; 20]>,
) -> Result<[u8; 20], SwhidError> {
    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| io_error(format!("Failed to find tree {tree_oid}: {e}")))?;
    let mut entries: Vec<DirEntry> = Vec::new();
    for entry in tree.iter() {
        let name = entry.name_bytes().to_owned().into_boxed_slice();
        let mode = entry.filemode() as u32;
        let id = match entry.kind() {
            Some(GitObjectType::Blob) => match cache.entry(entry.id()) {
                Entry::Occupied(e) => *e.get(),
                Entry::Vacant(e) => *e.insert(content_swhid_from_blob(repo, entry.id())?),
            },
            Some(GitObjectType::Tree) => match cache.entry(entry.id()) {
                Entry::Occupied(e) => *e.get(),
                Entry::Vacant(_) => {
                    let swhid = directory_swhid_from_tree(repo, entry.id(), cache)?;
                    cache.insert(entry.id(), swhid);
                    swhid
                }
            },
            _ => {
                return Err(io_error(format!(
                    "Tree entry {:?} has unsupported type",
                    entry.name()
                )));
            }
        };
        entries.push(DirEntry::new(name, mode, id));
    }
    let manifest =
        dir_manifest(entries).map_err(|e| io_error(format!("Directory manifest: {e}")))?;
    Ok(hash_swhid_object("tree", &manifest))
}

fn parse_signature(sig: Signature) -> (Bytestring, i64, Bytestring) {
    let when = sig.when();
    let sign = when.sign();
    let offset_minutes = when.offset_minutes().abs();
    let offset_hours = offset_minutes / 60;
    let offset_minutes = offset_minutes % 60;
    let offset = format!("{sign}{offset_hours:02}{offset_minutes:02}");

    crate::utils::build_signature(sig.name_bytes(), sig.email_bytes(), when.seconds(), &offset)
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
pub fn revision_swhid(
    repo: &Repository,
    commit_oid: &git2::Oid,
    cache: &mut HashMap<git2::Oid, [u8; 20]>,
) -> Result<Swhid, SwhidError> {
    revision_from_git(repo, commit_oid, cache).map(|rev| rev.swhid())
}

#[doc(hidden)]
pub fn revision_from_git(
    repo: &Repository,
    commit_oid: &git2::Oid,
    cache: &mut HashMap<git2::Oid, [u8; 20]>,
) -> Result<Revision, SwhidError> {
    let commit = repo
        .find_commit(*commit_oid)
        .map_err(|e| io_error(format!("Failed to find commit: {e}")))?;

    let tree = commit
        .tree()
        .map_err(|e| io_error(format!("Failed to get commit tree: {e}")))?;

    let tree_oid = tree.id();
    let directory = directory_swhid_from_tree(repo, tree_oid, cache)?;
    let parents: Vec<[u8; 20]> = commit
        .parents()
        .map(|p| match cache.entry(p.id()) {
            Entry::Occupied(e) => Ok(*e.get()),
            Entry::Vacant(_) => {
                let swhid = *revision_swhid(repo, &p.id(), cache)?.digest_bytes();
                cache.insert(p.id(), swhid);
                Ok(swhid)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

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
        message: Some(commit.message_bytes().into()),
    })
}

/// Compute a SWHID v1.2 release identifier from a Git tag
///
/// This implements the SWHID v1.2 release hashing algorithm for Git tags,
/// creating a `swh:1:rel:<digest>` identifier according to the specification.
pub fn release_swhid(repo: &Repository, tag_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    release_from_git(repo, tag_oid).map(|rel| rel.swhid())
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
    let object = match target.kind() {
        Some(GitObjectType::Commit) => {
            *revision_swhid(repo, &target_oid, &mut HashMap::new())?.digest_bytes()
        }
        Some(GitObjectType::Tree) => {
            directory_swhid_from_tree(repo, target_oid, &mut HashMap::new())?
        }
        Some(GitObjectType::Blob) => content_swhid_from_blob(repo, target_oid)?,
        Some(GitObjectType::Tag) => *release_swhid(repo, &target_oid)?.digest_bytes(),
        _ => return Err(io_error("Unknown target type".to_string())),
    };
    let object_type = match target.kind() {
        Some(GitObjectType::Commit) => ReleaseTargetType::Revision,
        Some(GitObjectType::Tree) => ReleaseTargetType::Directory,
        Some(GitObjectType::Blob) => ReleaseTargetType::Content,
        Some(GitObjectType::Tag) => ReleaseTargetType::Release,
        _ => return Err(io_error("Unknown target type".to_string())),
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

    Ok(Release {
        object,
        object_type,
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
pub fn snapshot_swhid(repo: &Repository) -> Result<Swhid, SwhidError> {
    snapshot_from_git(repo).map(|snp| snp.swhid())
}

#[doc(hidden)]
pub fn snapshot_from_git(repo: &Repository) -> Result<Snapshot, SwhidError> {
    let references = repo
        .references()
        .map_err(|e| io_error(format!("Failed to list references: {e}")))?;

    let mut cache = HashMap::new();
    let mut branches: Vec<_> = references
        .flat_map(|reference| match reference {
            Ok(reference) => reference_to_branch(repo, reference, &mut cache).transpose(),
            Err(e) => Some(Err(io_error(format!("Failed to read reference: {e}")))),
        })
        .collect::<Result<_, _>>()?;

    let head = repo
        .head()
        .map_err(|e| io_error(format!("Failed to get HEAD: {e}")))?;
    if let Some(head_branch) = reference_to_branch(repo, head, &mut cache)? {
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
    cache: &mut HashMap<git2::Oid, [u8; 20]>,
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
            let target = match repo.find_object(target_id, None) {
                Ok(obj) => obj,
                Err(e) if e.code() == git2::ErrorCode::NotFound => {
                    // Dangling branch (ref points to missing object). SWHID v1.2 Clause 5.6:
                    // "for dangling branches, the empty string".
                    return Ok(Some(Branch {
                        name,
                        target: BranchTarget::Revision(None),
                    }));
                }
                Err(e) => {
                    return Err(io_error(format!("Could not find object {target_id}: {e}")));
                }
            };
            let target = match target.kind() {
                None => {
                    // Dangling reference (object has no kind).
                    // FIXME: https://github.com/swhid/specification/issues/64
                    BranchTarget::Revision(None)
                }
                Some(git2::ObjectType::Any) => panic!("git2 returned an object with type 'Any'"),
                Some(git2::ObjectType::Commit) => {
                    let digest = *revision_swhid(repo, &target_id, cache)?.digest_bytes();
                    BranchTarget::Revision(Some(digest))
                }
                Some(git2::ObjectType::Tree) => {
                    let digest = directory_swhid_from_tree(repo, target_id, cache)?;
                    BranchTarget::Directory(Some(digest))
                }
                Some(git2::ObjectType::Blob) => {
                    let digest = match cache.entry(target_id) {
                        Entry::Occupied(e) => *e.get(),
                        Entry::Vacant(e) => *e.insert(content_swhid_from_blob(repo, target_id)?),
                    };
                    BranchTarget::Content(Some(digest))
                }
                Some(git2::ObjectType::Tag) => {
                    let digest = *release_swhid(repo, &target_id)?.digest_bytes();
                    BranchTarget::Release(Some(digest))
                }
            };
            target
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
