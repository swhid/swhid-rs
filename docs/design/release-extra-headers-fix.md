# Fix for Release (Tag) Extra Headers Extraction

## Problem Summary

The `swhid-rs` implementation correctly extracts extra headers (including GPG signatures) from Git commit objects for revisions, but fails to do the same for Git tag objects used in releases. This causes incorrect SWHID computation for:

- `lightweight_release` - Lightweight tags (not annotated tags, but should still be handled)
- `signed_release_v1` - Signed tag v1.0.0
- `signed_release_v2` - Signed tag v2.0.0  
- `signed_release_v2_1` - Signed tag v2.1.0

## Root Cause

In `src/git.rs`, the `release_from_git` function sets `extra_headers: Vec::new()` with a FIXME comment indicating that git2 doesn't expose extra headers. However, similar to commits, we need to extract extra headers directly from the raw Git tag object to ensure correct SWHID computation.

### Current Code (Line 270)

```rust
extra_headers: Vec::new(), // FIXME: does not seem to be exposed by git2
```

### Git Tag Object Structure

Git tag objects have the following structure:
```
object <hash>
type <type>
tag <name>
tagger <signature>
<extra headers> (gpgsig, encoding, etc.)  ← These are missing!
<empty line>
<message>
```

Unlike commits where GPG signatures are in a `gpgsig` header, tag objects can have:
1. Extra headers (like `gpgsig`) - similar to commits
2. GPG signatures embedded in the message (in some cases)

The current implementation only uses `tag.message_bytes()` from git2, which may not preserve the exact format needed for correct SWHID computation.

## Solution

Create a function `extract_tag_extra_headers` similar to `extract_commit_extra_headers` that:
1. Reads the raw tag object from the object database
2. Parses the tag object structure
3. Extracts extra headers (like `gpgsig`, `encoding`, etc.)
4. Also extracts the message directly from the raw object to ensure exact byte-for-byte match

Additionally, handle lightweight tags correctly (they point directly to commits, not tag objects).

## Implementation

### Step 1: Add `extract_tag_extra_headers` function

Add this function after `extract_commit_extra_headers` in `src/git.rs`:

```rust
/// Extract extra headers and message from a raw Git tag object
/// 
/// This parses the raw Git tag object to extract all extra headers,
/// including GPG signatures which may be stored in a `gpgsig` header
/// or embedded in the message.
/// 
/// Per SWHID spec 5.5, extra headers must be included in the serialization.
/// Returns (extra_headers, message)
fn extract_tag_extra_headers_and_message(
    repo: &Repository, 
    tag_oid: git2::Oid
) -> Result<(Vec<(Bytestring, Bytestring)>, Option<Bytestring>), SwhidError> {
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
    
    let mut extra_headers = Vec::new();
    let lines: Vec<&[u8]> = content.split(|&b| b == b'\n').collect();
    
    let mut i = 0;
    
    // Skip standard headers: object, type, tag, tagger
    while i < lines.len() {
        let line = lines[i];
        if line.is_empty() {
            // Empty line separates headers from message
            break;
        }
        
        // Check if this is a standard header we should skip
        if line.starts_with(b"object ") 
            || line.starts_with(b"type ")
            || line.starts_with(b"tag ")
            || line.starts_with(b"tagger ") {
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
    
    Ok((extra_headers, message))
}
```

### Step 2: Update `release_from_git` to use the new function

Replace the `release_from_git` function (lines 232-273) with:

```rust
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

    // Extract extra headers and message from raw Git tag object
    // Per SWHID spec 5.5, extra headers must be included in serialization
    // Also extract message directly from raw object to ensure exact byte match
    let (extra_headers, message) = extract_tag_extra_headers_and_message(repo, *tag_oid)?;

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
        extra_headers,
        message,
    })
}
```

### Step 3: Handle Lightweight Tags

For lightweight tags (which point directly to commits, not tag objects), the current code will fail because `repo.find_tag()` will not find a tag object. We need to handle this case:

```rust
pub fn release_swhid(repo: &Repository, tag_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    // Check if this is actually a tag object or a lightweight tag (points to commit)
    let obj = repo
        .find_object(*tag_oid, None)
        .map_err(|e| io_error(format!("Failed to find object: {e}")))?;
    
    match obj.kind() {
        Some(git2::ObjectType::Tag) => {
            // Annotated tag - reconstruct from components including extra headers
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
```

However, looking at the test case `lightweight_release`, it seems the test expects lightweight tags to be handled. Let's check what the expected behavior should be by looking at other implementations.

Actually, based on the other implementations (git, pygit2, git-cmd), they all raise an error for lightweight tags. So the `lightweight_release` test might be expected to fail, or it might need special handling.

For now, the fix should focus on annotated tags with extra headers. Lightweight tag handling can be addressed separately if needed.

## Testing

After implementing the fix, verify:

1. **Signed tags**: All three signed release tests should pass
   - `signed_release_v1` → `swh:1:rel:d6bc712db2ffad219e410155850770f2a6f80566`
   - `signed_release_v2` → `swh:1:rel:90b798f42ee8c20dc94b119fc4139b79a03c3b7e`
   - `signed_release_v2_1` → `swh:1:rel:dc4a4d4c9110311ff03e0a6f218ecfcb3247ac0b`

2. **Lightweight tags**: Determine expected behavior (likely should raise an error)

3. **Regular annotated tags**: Should continue to work correctly

## Key Differences from Commit Extraction

1. **Standard headers differ**: Tags have `object`, `type`, `tag`, `tagger` instead of `tree`, `parent`, `author`, `committer`
2. **Message extraction**: Need to extract message directly from raw object to ensure exact byte match
3. **Lightweight tag handling**: Tags can be lightweight (point to commits) or annotated (tag objects)

## Notes

- The message extraction from the raw object ensures that any GPG signatures embedded in the message (rather than in a `gpgsig` header) are preserved exactly as Git stores them
- This approach matches the pattern used for commits, ensuring consistency
- The fix maintains the "reconstruct from components" approach while correctly handling all parts of the tag object


