//! SWHID v1.2 VCS integration for Git repositories (gitoxide backend)
//!
//! This module provides the same functionality as the `git` module but uses
//! gitoxide (gix) instead of libgit2. The gitoxide crate is licensed MIT/Apache-2.0,
//! making it compatible with all permissive license requirements.
//!
//! Enable with `--features gitoxide`. Provides:
//! - Revision SWHIDs (commits) - `swh:1:rev:<digest>`
//! - Release SWHIDs (tags) - `swh:1:rel:<digest>`
//! - Snapshot SWHIDs (repository state) - `swh:1:snp:<digest>`

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::Path;

use gix::object::Kind;
use gix::ObjectId;

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

/// Convert a BStr/BString to Box<[u8]> (our Bytestring type).
fn bstr_to_box(b: &[u8]) -> Box<[u8]> {
    b.to_vec().into_boxed_slice()
}

/// Content SWHID digest (20 bytes) from a Git blob OID.
fn content_swhid_from_blob(
    repo: &gix::Repository,
    blob_oid: ObjectId,
) -> Result<[u8; 20], SwhidError> {
    let blob = repo
        .find_blob(blob_oid)
        .map_err(|e| io_error(format!("Failed to find blob {blob_oid}: {e}")))?;
    Ok(hash_content(blob.data.as_ref()))
}

/// Directory SWHID digest (20 bytes) from a Git tree OID.
fn directory_swhid_from_tree(
    repo: &gix::Repository,
    tree_oid: ObjectId,
    cache: &mut HashMap<ObjectId, [u8; 20]>,
) -> Result<[u8; 20], SwhidError> {
    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| io_error(format!("Failed to find tree {tree_oid}: {e}")))?;
    let mut entries: Vec<DirEntry> = Vec::new();
    for entry_ref in tree.iter() {
        let entry = entry_ref.map_err(|e| io_error(format!("Failed to read tree entry: {e}")))?;
        let name = bstr_to_box(entry.filename());
        // Convert mode to u32 via the octal string representation
        let mode_str = entry.mode().as_str();
        let mode = u32::from_str_radix(mode_str, 8).unwrap_or(0o100644);
        let oid = entry.oid();
        let entry_kind = entry.mode().kind();

        let id = match entry_kind {
            gix::object::tree::EntryKind::Blob | gix::object::tree::EntryKind::BlobExecutable => {
                match cache.entry(oid.into()) {
                    Entry::Occupied(e) => *e.get(),
                    Entry::Vacant(e) => *e.insert(content_swhid_from_blob(repo, oid.into())?),
                }
            }
            gix::object::tree::EntryKind::Tree => match cache.entry(oid.into()) {
                Entry::Occupied(e) => *e.get(),
                Entry::Vacant(_) => {
                    let swhid = directory_swhid_from_tree(repo, oid.into(), cache)?;
                    cache.insert(oid.into(), swhid);
                    swhid
                }
            },
            gix::object::tree::EntryKind::Link => {
                // Symlinks are treated as blobs for SWHID purposes
                match cache.entry(oid.into()) {
                    Entry::Occupied(e) => *e.get(),
                    Entry::Vacant(e) => *e.insert(content_swhid_from_blob(repo, oid.into())?),
                }
            }
            _ => {
                return Err(io_error(format!(
                    "Tree entry {:?} has unsupported type",
                    String::from_utf8_lossy(entry.filename())
                )));
            }
        };
        entries.push(DirEntry::new(name, mode, id));
    }
    let manifest =
        dir_manifest(entries).map_err(|e| io_error(format!("Directory manifest: {e}")))?;
    Ok(hash_swhid_object("tree", &manifest))
}

/// Parse a gitoxide actor signature into (fullname, timestamp, offset).
fn parse_gix_signature(sig: gix::actor::SignatureRef<'_>) -> (Bytestring, i64, Bytestring) {
    // sig.time is a raw string like "1763027354 +0100"
    let time_str = sig.time.trim();
    let (timestamp, offset) = if let Some(space_pos) = time_str.rfind(' ') {
        let ts_str = &time_str[..space_pos];
        let tz_str = &time_str[space_pos + 1..];
        let ts = ts_str.parse::<i64>().unwrap_or(0);
        (ts, tz_str.to_string())
    } else {
        (time_str.parse::<i64>().unwrap_or(0), "+0000".to_string())
    };

    crate::utils::build_signature(sig.name.as_ref(), sig.email.as_ref(), timestamp, &offset)
}

/// Returns key-value pairs from a raw commit/tag header
fn parse_header(mut manifest: &[u8]) -> Result<Vec<(&[u8], Bytestring)>, SwhidError> {
    let mut headers = Vec::new();
    while !manifest.is_empty() {
        let Some(newline_position) = manifest.iter().position(|&byte| byte == b'\n') else {
            return Err(io_error("Header line is missing a line end".to_owned()));
        };
        let first_line = &manifest[..newline_position];
        manifest = &manifest[newline_position + 1..];

        let Some(delimiter_position) = first_line.iter().position(|&byte| byte == b' ') else {
            return Err(io_error("Header line is missing a value".to_owned()));
        };
        let key = &first_line[..delimiter_position];
        if key.is_empty() {
            return Err(io_error("Empty key".to_owned()));
        };
        let mut value = first_line[delimiter_position + 1..].to_vec();

        while let Some(newline_position) = manifest.iter().position(|&byte| byte == b'\n') {
            let line = &manifest[..newline_position];
            match line.split_first() {
                None => {
                    return Err(io_error("Empty line".to_owned()));
                }
                Some((b' ', value_line)) => {
                    value.push(b'\n');
                    value.extend_from_slice(value_line);
                }
                Some(_) => {
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
pub fn revision_swhid(
    repo: &gix::Repository,
    commit_oid: &ObjectId,
    cache: &mut HashMap<ObjectId, [u8; 20]>,
) -> Result<Swhid, SwhidError> {
    revision_from_git(repo, commit_oid, cache).map(|rev| rev.swhid())
}

#[doc(hidden)]
pub fn revision_from_git(
    repo: &gix::Repository,
    commit_oid: &ObjectId,
    cache: &mut HashMap<ObjectId, [u8; 20]>,
) -> Result<Revision, SwhidError> {
    let commit_obj = repo
        .find_object(*commit_oid)
        .map_err(|e| io_error(format!("Failed to find commit: {e}")))?;
    let commit = commit_obj
        .try_into_commit()
        .map_err(|e| io_error(format!("Object is not a commit: {e}")))?;
    let commit_ref = commit
        .decode()
        .map_err(|e| io_error(format!("Failed to decode commit: {e}")))?;

    let tree_oid = commit_ref.tree();
    let directory = directory_swhid_from_tree(repo, tree_oid.into(), cache)?;

    let parents: Vec<[u8; 20]> = commit_ref
        .parents()
        .map(|parent_oid| {
            let parent_id: ObjectId = parent_oid.into();
            match cache.entry(parent_id) {
                Entry::Occupied(e) => Ok(*e.get()),
                Entry::Vacant(_) => {
                    let swhid = *revision_swhid(repo, &parent_id, cache)?.digest_bytes();
                    cache.insert(parent_id, swhid);
                    Ok(swhid)
                }
            }
        })
        .collect::<Result<Vec<_>, SwhidError>>()?;

    let (author, author_timestamp, author_timestamp_offset) =
        parse_gix_signature(commit_ref.author);
    let (committer, committer_timestamp, committer_timestamp_offset) =
        parse_gix_signature(commit_ref.committer);

    // Parse extra headers from raw commit data
    let raw_data: &[u8] = commit.data.as_ref();
    // Find the header section (everything before the first \n\n)
    let header_end = raw_data
        .windows(2)
        .position(|w| w == b"\n\n")
        .unwrap_or(raw_data.len());
    let raw_header = &raw_data[..header_end + 1]; // include trailing \n

    let headers = parse_header(raw_header)?;
    let extra_headers = headers
        .into_iter()
        .filter(|(key, _value)| !matches!(*key, b"tree" | b"parent" | b"author" | b"committer"))
        .map(|(key, value)| (key.into(), value))
        .collect();

    let message_bytes: &[u8] = commit_ref.message.as_ref();
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
        message: Some(message_bytes.to_vec().into_boxed_slice()),
    })
}

/// Compute a SWHID v1.2 release identifier from a Git tag
pub fn release_swhid(repo: &gix::Repository, tag_oid: &ObjectId) -> Result<Swhid, SwhidError> {
    release_from_git(repo, tag_oid).map(|rel| rel.swhid())
}

#[doc(hidden)]
pub fn release_from_git(repo: &gix::Repository, tag_oid: &ObjectId) -> Result<Release, SwhidError> {
    let tag_obj = repo
        .find_object(*tag_oid)
        .map_err(|e| io_error(format!("Failed to find tag: {e}")))?;
    let tag = tag_obj
        .try_into_tag()
        .map_err(|e| io_error(format!("Object is not a tag: {e}")))?;
    let tag_ref = tag
        .decode()
        .map_err(|e| io_error(format!("Failed to decode tag: {e}")))?;

    let target_oid: ObjectId = tag_ref.target().into();
    let target_kind = tag_ref.target_kind;

    use crate::release::ReleaseTargetType;

    let object = match target_kind {
        Kind::Commit => *revision_swhid(repo, &target_oid, &mut HashMap::new())?.digest_bytes(),
        Kind::Tree => directory_swhid_from_tree(repo, target_oid, &mut HashMap::new())?,
        Kind::Blob => content_swhid_from_blob(repo, target_oid)?,
        Kind::Tag => *release_swhid(repo, &target_oid)?.digest_bytes(),
    };

    let object_type = match target_kind {
        Kind::Commit => ReleaseTargetType::Revision,
        Kind::Tree => ReleaseTargetType::Directory,
        Kind::Blob => ReleaseTargetType::Content,
        Kind::Tag => ReleaseTargetType::Release,
    };

    let (author, author_timestamp, author_timestamp_offset) = match tag_ref.tagger {
        Some(tagger) => {
            let (a, t, o) = parse_gix_signature(tagger);
            (Some(a), Some(t), Some(o))
        }
        None => (None, None, None),
    };

    let tag_name: &[u8] = tag_ref.name.as_ref();
    let tag_message: Option<Box<[u8]>> = if tag_ref.message.is_empty() {
        None
    } else {
        let msg_bytes: &[u8] = tag_ref.message.as_ref();
        Some(msg_bytes.to_vec().into_boxed_slice())
    };

    Ok(Release {
        object,
        object_type,
        name: tag_name.to_vec().into_boxed_slice(),
        author,
        author_timestamp,
        author_timestamp_offset,
        extra_headers: Vec::new(),
        message: tag_message,
    })
}

/// Compute a SWHID v1.2 snapshot identifier from a Git repository
pub fn snapshot_swhid(repo: &gix::Repository) -> Result<Swhid, SwhidError> {
    snapshot_from_git(repo).map(|snp| snp.swhid())
}

#[doc(hidden)]
pub fn snapshot_from_git(repo: &gix::Repository) -> Result<Snapshot, SwhidError> {
    let references = repo
        .references()
        .map_err(|e| io_error(format!("Failed to list references: {e}")))?;

    let all_refs = references
        .all()
        .map_err(|e| io_error(format!("Failed to iterate references: {e}")))?;

    let mut cache = HashMap::new();
    let mut branches: Vec<Branch> = Vec::new();

    for reference in all_refs {
        let reference =
            reference.map_err(|e| io_error(format!("Failed to read reference: {e}")))?;
        if let Some(branch) = reference_to_branch(repo, &reference, &mut cache)? {
            branches.push(branch);
        }
    }

    // Add HEAD as alias
    let head = repo
        .head_ref()
        .map_err(|e| io_error(format!("Failed to get HEAD: {e}")))?;
    if let Some(head_ref) = head {
        let target_name: &[u8] = head_ref.name().as_bstr().as_ref();
        branches.push(Branch {
            name: (*b"HEAD").into(),
            target: BranchTarget::Alias(Some(target_name.to_vec().into_boxed_slice())),
        });
    }

    Snapshot::new(branches).map_err(|e| io_error(format!("Invalid snapshot: {e}")))
}

fn reference_to_branch(
    repo: &gix::Repository,
    reference: &gix::Reference<'_>,
    cache: &mut HashMap<ObjectId, [u8; 20]>,
) -> Result<Option<Branch>, SwhidError> {
    let name_bytes: &[u8] = reference.name().as_bstr().as_ref();
    // Only process branches and tags
    if !name_bytes.starts_with(b"refs/heads/") && !name_bytes.starts_with(b"refs/tags/") {
        return Ok(None);
    }

    let name: Box<[u8]> = name_bytes.to_vec().into_boxed_slice();

    // Check if this is a direct (peeled) or symbolic reference
    match reference.try_id() {
        Some(oid) => {
            let target_id: ObjectId = oid.into();
            let obj = match repo.find_object(target_id) {
                Ok(obj) => obj,
                Err(_) => {
                    // Dangling branch
                    return Ok(Some(Branch {
                        name,
                        target: BranchTarget::Revision(None),
                    }));
                }
            };
            let target = match obj.kind {
                Kind::Commit => {
                    let digest = *revision_swhid(repo, &target_id, cache)?.digest_bytes();
                    BranchTarget::Revision(Some(digest))
                }
                Kind::Tree => {
                    let digest = directory_swhid_from_tree(repo, target_id, cache)?;
                    BranchTarget::Directory(Some(digest))
                }
                Kind::Blob => {
                    let digest = match cache.entry(target_id) {
                        Entry::Occupied(e) => *e.get(),
                        Entry::Vacant(e) => *e.insert(content_swhid_from_blob(repo, target_id)?),
                    };
                    BranchTarget::Content(Some(digest))
                }
                Kind::Tag => {
                    let digest = *release_swhid(repo, &target_id)?.digest_bytes();
                    BranchTarget::Release(Some(digest))
                }
            };
            Ok(Some(Branch { name, target }))
        }
        None => {
            // Symbolic reference
            if let Some(target_name) = reference.name().as_bstr().strip_prefix(b"ref: ") {
                Ok(Some(Branch {
                    name,
                    target: BranchTarget::Alias(Some(target_name.to_vec().into_boxed_slice())),
                }))
            } else {
                // Try symbolic_target approach
                Ok(Some(Branch {
                    name,
                    target: BranchTarget::Revision(None),
                }))
            }
        }
    }
}

/// Open a Git repository for SWHID v1.2 computation
pub fn open_repo(path: &Path) -> Result<gix::Repository, SwhidError> {
    gix::open(path).map_err(|e| io_error(format!("Failed to open repository: {e}")))
}

/// Get the HEAD commit OID of a Git repository
pub fn get_head_commit(repo: &gix::Repository) -> Result<ObjectId, SwhidError> {
    let head = repo
        .head_commit()
        .map_err(|e| io_error(format!("Failed to get HEAD commit: {e}")))?;
    Ok(head.id().into())
}

/// Get all annotated tag OIDs in a Git repository
pub fn get_tags(repo: &gix::Repository) -> Result<Vec<ObjectId>, SwhidError> {
    let references = repo
        .references()
        .map_err(|e| io_error(format!("Failed to list references: {e}")))?;

    let tag_refs = references
        .prefixed("refs/tags/")
        .map_err(|e| io_error(format!("Failed to filter tags: {e}")))?;

    let mut tags = Vec::new();
    for reference in tag_refs {
        let reference =
            reference.map_err(|e| io_error(format!("Failed to read tag reference: {e}")))?;
        if let Some(oid) = reference.try_id() {
            tags.push(oid.into());
        }
    }
    Ok(tags)
}
