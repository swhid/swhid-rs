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

use git2::{Repository, Signature};

use crate::release::Release;
use crate::revision::Revision;
use crate::snapshot::{Branch, BranchTarget, Snapshot};
use crate::Bytestring;

fn io_error(msg: String) -> SwhidError {
    SwhidError::Io(std::io::Error::other(msg))
}

fn oid_to_array(oid: git2::Oid) -> Result<[u8; 20], SwhidError> {
    oid.as_bytes()
        .try_into()
        .map_err(|e| io_error(format!("Unexpected tree_oid length: {e}")))
}

/// Extract extra headers (including gpgsig) from a raw Git commit object
/// 
/// This parses the raw Git commit object to extract all extra headers,
/// including GPG signatures which are stored in the `gpgsig` header.
/// 
/// Per SWHID spec 5.4, extra headers must be included in the serialization.
fn extract_commit_extra_headers(repo: &Repository, commit_oid: git2::Oid) -> Result<Vec<(Bytestring, Bytestring)>, SwhidError> {
    // Read the raw commit object from the object database
    let odb = repo.odb()
        .map_err(|e| io_error(format!("Failed to get object database: {e}")))?;
    
    let odb_obj = odb.read(commit_oid)
        .map_err(|e| io_error(format!("Failed to read commit object: {e}")))?;
    
    // OdbObject::data() returns the decompressed object data
    // Format: "commit <length>\0<content>" for loose objects
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
    
    // Parse the commit object to extract headers
    // Git commit format:
    // tree <hash>\n
    // parent <hash>\n (optional, multiple)
    // author <info>\n
    // committer <info>\n
    // <extra headers>\n (gpgsig, encoding, etc.)
    // \n
    // <message>
    
    let mut extra_headers = Vec::new();
    let lines: Vec<&[u8]> = content.split(|&b| b == b'\n').collect();
    
    let mut i = 0;
    
    // Skip standard headers: tree, parent(s), author, committer
    while i < lines.len() {
        let line = lines[i];
        if line.is_empty() {
            // Empty line separates headers from message
            break;
        }
        
        // Check if this is a standard header we should skip
        if line.starts_with(b"tree ") 
            || line.starts_with(b"parent ")
            || line.starts_with(b"author ")
            || line.starts_with(b"committer ") {
            i += 1;
            continue;
        }
        
        // This is an extra header - parse key:value
        // Headers can span multiple lines (like gpgsig)
        if let Some(space_idx) = line.iter().position(|&b| b == b' ') {
            let key = &line[..space_idx];
            let value_start = space_idx + 1;
            
            // Check if this is a multi-line header (like gpgsig)
            if key == b"gpgsig" || key == b"encoding" {
                // Multi-line header: value continues on subsequent lines
                // Each continuation line starts with a space
                let mut value = Vec::from(&line[value_start..]);
                i += 1;
                
                // Collect continuation lines (they start with space)
                while i < lines.len() && !lines[i].is_empty() {
                    let cont_line = lines[i];
                    if cont_line.starts_with(b" ") {
                        // Continuation line - remove leading space and add
                        value.push(b'\n');
                        value.extend_from_slice(&cont_line[1..]);
                        i += 1;
                    } else {
                        // Not a continuation - this is the next header or message
                        break;
                    }
                }
                
                extra_headers.push((
                    Bytestring::from(key),
                    Bytestring::from(value)
                ));
            } else {
                // Single-line header
                let value = &line[value_start..];
                extra_headers.push((
                    Bytestring::from(key),
                    Bytestring::from(value)
                ));
                i += 1;
            }
        } else {
            // No space found - malformed header, skip
            i += 1;
        }
    }
    
    Ok(extra_headers)
}

/// Extract all components from a raw Git tag object
/// 
/// This parses the raw Git tag object to extract all components needed for
/// reconstruction, ensuring exact byte-for-byte match with the original.
/// 
/// Per SWHID spec 5.5, extra headers must be included in the serialization.
/// Returns (object_hash, object_type, tag_name, tagger_line, extra_headers, message)
fn extract_tag_components(
    repo: &Repository, 
    tag_oid: git2::Oid
) -> Result<([u8; 20], Bytestring, Bytestring, Option<Bytestring>, Vec<(Bytestring, Bytestring)>, Option<Bytestring>), SwhidError> {
    // Read the raw tag object from the object database
    let odb = repo.odb()
        .map_err(|e| io_error(format!("Failed to get object database: {e}")))?;
    
    let odb_obj = odb.read(tag_oid)
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
    
    let mut object_hash_bytes = None;
    let mut object_type_bytes = None;
    let mut tag_name_bytes = None;
    let mut tagger_line_bytes = None;
    let mut extra_headers = Vec::new();
    let lines: Vec<&[u8]> = content.split(|&b| b == b'\n').collect();
    
    let mut i = 0;
    
    // Extract standard headers: object, type, tag, tagger
    while i < lines.len() {
        let line = lines[i];
        if line.is_empty() {
            // Empty line separates headers from message
            break;
        }
        
        // Extract standard headers
        if line.starts_with(b"object ") {
            let hash_str = std::str::from_utf8(&line[7..])
                .map_err(|e| io_error(format!("Invalid object hash: {e}")))?;
            let hash_array = hex::decode(hash_str)
                .map_err(|e| io_error(format!("Failed to decode object hash: {e}")))?;
            object_hash_bytes = Some(hash_array.try_into()
                .map_err(|_| io_error("Invalid object hash length".to_string()))?);
            i += 1;
            continue;
        } else if line.starts_with(b"type ") {
            object_type_bytes = Some(Bytestring::from(&line[5..]));
            i += 1;
            continue;
        } else if line.starts_with(b"tag ") {
            tag_name_bytes = Some(Bytestring::from(&line[4..]));
            i += 1;
            continue;
        } else if line.starts_with(b"tagger ") {
            tagger_line_bytes = Some(Bytestring::from(&line[7..]));
            i += 1;
            continue;
        }
        
        // This is an extra header - parse key:value
        // Headers can span multiple lines (like gpgsig)
        if let Some(space_idx) = line.iter().position(|&b| b == b' ') {
            let key = &line[..space_idx];
            let value_start = space_idx + 1;
            
            // Check if this is a multi-line header (like gpgsig)
            if key == b"gpgsig" || key == b"encoding" {
                // Multi-line header: value continues on subsequent lines
                // Each continuation line starts with a space
                let mut value = Vec::from(&line[value_start..]);
                i += 1;
                
                // Collect continuation lines (they start with space)
                while i < lines.len() && !lines[i].is_empty() {
                    let cont_line = lines[i];
                    if cont_line.starts_with(b" ") {
                        // Continuation line - remove leading space and add
                        value.push(b'\n');
                        value.extend_from_slice(&cont_line[1..]);
                        i += 1;
                    } else {
                        // Not a continuation - this is the next header or message
                        break;
                    }
                }
                
                extra_headers.push((
                    Bytestring::from(key),
                    Bytestring::from(value)
                ));
            } else {
                // Single-line header
                let value = &line[value_start..];
                extra_headers.push((
                    Bytestring::from(key),
                    Bytestring::from(value)
                ));
                i += 1;
            }
        } else {
            // No space found - malformed header, skip
            i += 1;
        }
    }
    
    // Extract message (everything after the empty line)
    // Skip the empty line
    if i < lines.len() && lines[i].is_empty() {
        i += 1;
    }
    
    // Collect remaining lines as message
    let message = if i < lines.len() {
        let message_lines = &lines[i..];
        if !message_lines.is_empty() {
            // Join lines with newlines, but preserve the exact format
            // Note: The last line might not have a trailing newline in the original
            let mut message_bytes = Vec::new();
            for (idx, line) in message_lines.iter().enumerate() {
                if idx > 0 {
                    message_bytes.push(b'\n');
                }
                message_bytes.extend_from_slice(line);
            }
            Some(Bytestring::from(message_bytes))
        } else {
            None
        }
    } else {
        None
    };
    
    let object_hash = object_hash_bytes.ok_or_else(|| io_error("Missing object header".to_string()))?;
    let object_type = object_type_bytes.ok_or_else(|| io_error("Missing type header".to_string()))?;
    let tag_name = tag_name_bytes.ok_or_else(|| io_error("Missing tag header".to_string()))?;
    
    Ok((object_hash, object_type, tag_name, tagger_line_bytes, extra_headers, message))
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

/// Compute a SWHID v1.2 revision identifier from a Git commit
///
/// This implements the SWHID v1.2 revision hashing algorithm for Git commits,
/// creating a `swh:1:rev:<digest>` identifier according to the specification.
///
/// This follows the spec-compliant approach: extracts all components including
/// GPG signatures from the raw Git object and reconstructs the manifest per
/// SWHID spec 5.4. This ensures full compliance with the specification.
pub fn revision_swhid(repo: &Repository, commit_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    // Spec-compliant approach: reconstruct from components including extra headers
    // This extracts gpgsig headers from the raw Git object per SWHID spec 5.4
    revision_from_git(repo, commit_oid).map(|rev| rev.swhid())
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

    // Extract extra headers (including gpgsig) from raw Git commit object
    // Per SWHID spec 5.4, extra headers must be included in serialization
    let extra_headers = extract_commit_extra_headers(repo, *commit_oid)?;

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
/// This follows the spec-compliant approach: extracts all components including
/// GPG signatures (embedded in the message) from the raw Git object and
/// reconstructs the manifest per SWHID spec 5.5. This ensures full compliance.
pub fn release_swhid(repo: &Repository, tag_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    // Check if this is actually a tag object or a lightweight tag (points to commit)
    let obj = repo
        .find_object(*tag_oid, None)
        .map_err(|e| io_error(format!("Failed to find object: {e}")))?;
    
    match obj.kind() {
        Some(git2::ObjectType::Tag) => {
            // Annotated tag - reconstruct from components including extra headers
            // Spec-compliant approach: reconstruct from components including message with signatures
            // GPG signatures in tags are embedded in the message per SWHID spec 5.5
            release_from_git(repo, tag_oid).map(|rel| rel.swhid())
        }
        Some(git2::ObjectType::Commit) => {
            // Lightweight tag - for SWHID, lightweight tags pointing to commits
            // should be treated as releases, but we need to create a minimal tag object
            // However, per SWHID spec, releases should be annotated tags.
            // For now, return an error indicating lightweight tags are not supported
            Err(io_error("Lightweight tags are not supported for releases. Use annotated tags instead.".to_string()))
        }
        _ => {
            Err(io_error(format!("Object {} is not a tag object", tag_oid)))
        }
    }
}

#[doc(hidden)]
pub fn release_from_git(repo: &Repository, tag_oid: &git2::Oid) -> Result<Release, SwhidError> {
    use crate::release::ReleaseTargetType;

    let tag = repo
        .find_tag(*tag_oid)
        .map_err(|e| io_error(format!("Failed to find tag: {e}")))?;

    // Extract all components from raw Git tag object to ensure exact byte-for-byte match
    // Per SWHID spec 5.5, we need to reconstruct exactly as Git stores it
    let (object_hash, object_type_str, tag_name, tagger_line, extra_headers, message) = 
        extract_tag_components(repo, *tag_oid)?;

    // Parse tagger line if present to get author info
    // Clone tagger_line since we need to use it later in the Release struct
    let (author, author_timestamp, author_timestamp_offset) = if let Some(ref tagger_bytes) = tagger_line {
        // Parse the tagger line: "Name <email> timestamp offset"
        let tagger_str = std::str::from_utf8(tagger_bytes)
            .map_err(|e| io_error(format!("Invalid tagger line: {e}")))?;
        
        // Find the last two space-separated parts (timestamp and offset)
        let parts: Vec<&str> = tagger_str.rsplitn(3, ' ').collect();
        if parts.len() >= 3 {
            let offset_str = parts[0];
            let timestamp_str = parts[1];
            let name_email = parts[2..].join(" ");
            
            let timestamp = timestamp_str.parse::<i64>()
                .map_err(|e| io_error(format!("Invalid timestamp: {e}")))?;
            
            (
                Some(Bytestring::from(name_email.as_bytes())),
                Some(timestamp),
                Some(Bytestring::from(offset_str.as_bytes())),
            )
        } else {
            // Fallback to git2 parsing if format is unexpected
            match tag.tagger() {
                Some(tagger) => {
                    let (author, author_timestamp, author_timestamp_offset) = parse_signature(tagger);
                    (Some(author), Some(author_timestamp), Some(author_timestamp_offset))
                }
                None => (None, None, None),
            }
        }
    } else {
        (None, None, None)
    };

    // Determine object type from the extracted type string
    let object_type = match object_type_str.as_ref() {
        b"commit" => ReleaseTargetType::Revision,
        b"tree" => ReleaseTargetType::Directory,
        b"blob" => ReleaseTargetType::Content,
        b"tag" | b"release" => ReleaseTargetType::Release,
        _ => return Err(io_error(format!("Unknown object type: {:?}", object_type_str))),
    };

    Ok(Release {
        object: object_hash,
        object_type,
        name: tag_name,
        author,
        author_timestamp,
        author_timestamp_offset,
        extra_headers,
        message,
        raw_tagger_line: tagger_line.clone(),
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
