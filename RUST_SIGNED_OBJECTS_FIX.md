# Rust Implementation Fix for Signed Git Objects

## Problem Identification

The Rust implementation (`swhid-rs`) incorrectly computes SWHIDs for signed Git objects (revisions and releases) by reconstructing the object from parsed fields, which loses the GPG signature information.

### Root Cause

1. **For Revisions** (`src/git.rs:revision_from_git`):
   - The code reconstructs a `Revision` struct from parsed commit fields
   - The `extra_headers` field is set to `Vec::new()` with a FIXME comment: `// FIXME: does not seem to be exposed by git2`
   - GPG signatures are stored in Git commit objects as `gpgsig` headers, but `git2`'s `Commit` object doesn't expose them
   - The reconstructed manifest is then hashed, producing a different hash than the original Git object

2. **For Releases** (`src/git.rs:release_from_git`):
   - Similar issue: reconstructs a `Release` struct from parsed tag fields
   - `extra_headers` is also empty
   - GPG signatures in tag objects are embedded in the tag message, but `git2`'s `Tag` object doesn't expose the raw signature

3. **The Core Issue**:
   - According to the SWHID specification, for Git objects, the SWHID should be the **Git object hash** (OID), which includes GPG signatures
   - The Rust implementation is reconstructing the object and hashing that, which loses the signature
   - Other implementations (git-cmd, git, pygit2) correctly use the Git object hash directly

### Evidence

- **Expected behavior**: SWHID = Git object hash (includes GPG signature)
  - Revision: `swh:1:rev:<commit_oid>` where `commit_oid = git rev-parse <commit>`
  - Release: `swh:1:rel:<tag_oid>` where `tag_oid = git rev-parse <tag>`

- **Current Rust behavior**: Reconstructs object from parsed fields, hashes the manifest
  - This produces different hashes because GPG signatures are missing

## Solution

The fix is simple: **use the Git object OID directly** instead of reconstructing the object.

### Minimal Fix

For both `revision_swhid` and `release_swhid` functions, instead of:
1. Parsing the object into a struct
2. Reconstructing the manifest
3. Hashing the manifest

We should:
1. Use the Git object OID directly
2. Return `swh:1:rev:<oid>` or `swh:1:rel:<oid>`

### Code Changes

#### File: `src/git.rs`

**Change 1: `revision_swhid` function (line 57-59)**

```rust
// BEFORE:
pub fn revision_swhid(repo: &Repository, commit_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    revision_from_git(repo, commit_oid).map(|rev| rev.swhid())
}

// AFTER:
pub fn revision_swhid(repo: &Repository, commit_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    // For Git objects, SWHID is the Git object hash directly (includes GPG signatures)
    let oid_array = oid_to_array(*commit_oid)?;
    Ok(Swhid::new(crate::ObjectType::Revision, oid_array))
}
```

**Change 2: `release_swhid` function (line 101-103)**

```rust
// BEFORE:
pub fn release_swhid(repo: &Repository, tag_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    release_from_git(repo, tag_oid).map(|rel| rel.swhid())
}

// AFTER:
pub fn release_swhid(repo: &Repository, tag_oid: &git2::Oid) -> Result<Swhid, SwhidError> {
    // For Git objects, SWHID is the Git object hash directly (includes GPG signatures)
    let oid_array = oid_to_array(*tag_oid)?;
    Ok(Swhid::new(crate::ObjectType::Release, oid_array))
}
```

### Why This Works

1. **Git object hashes include GPG signatures**: When Git computes the hash of a commit or tag object, it includes the entire object content, including GPG signatures.

2. **SWHID specification alignment**: The SWHID specification states that for Git objects, the SWHID should be the Git object hash. This is what all other implementations do.

3. **Consistency**: This makes the Rust implementation consistent with:
   - `git-cmd`: Uses `git rev-parse` to get object hash
   - `git` (dulwich): Uses the object's `id` property
   - `pygit2`: Uses the object's `id.hex` property

### Impact

- **Breaking change**: This changes the SWHID computation for all Git revisions and releases
- **Correctness**: This fixes the bug and makes the implementation compliant with the specification
- **Test updates**: All tests that compute SWHIDs from Git objects will need to be updated

### Testing

After applying the fix, the following should pass:
- Signed revision tests: `swh:1:rev:8a1241cc9d81178d7c1c29201354b2cb309601fe`
- Signed release tests: `swh:1:rel:d6bc712db2ffad219e410155850770f2a6f80566`

### Note on `revision_from_git` and `release_from_git`

These functions are marked `#[doc(hidden)]` and may be used internally. However, they should not be used for SWHID computation of Git objects. They can remain for other purposes (e.g., if someone needs to parse Git objects for other reasons), but the public API functions should use the OID directly.

## Full Explanation

### How Git Object Hashing Works

Git objects are stored with their content and a header. The hash is computed as:
```
SHA1("type length\0content")
```

For signed commits, the `gpgsig` header is part of the content:
```
tree <hash>
parent <hash>
author <info>
committer <info>
gpgsig -----BEGIN PGP SIGNATURE-----
...
-----END PGP SIGNATURE-----

<message>
```

For signed tags, the GPG signature is embedded in the message:
```
object <hash>
type <type>
tag <name>
tagger <info>

<message>
-----BEGIN PGP SIGNATURE-----
...
-----END PGP SIGNATURE-----
```

When Git computes the object hash, it includes **everything** in the object, including GPG signatures.

### Why `git2` Doesn't Expose GPG Signatures

The `git2` library (libgit2) is a high-level wrapper around Git's object model. It parses objects into structured types (`Commit`, `Tag`, etc.) but doesn't expose raw headers like `gpgsig` because:
1. It focuses on the semantic content of objects
2. GPG signatures are verification metadata, not part of the logical object structure
3. The library provides signature verification separately

However, for SWHID computation, we need the **raw object hash**, not a reconstructed object.

### The Correct Approach

The SWHID specification for Git objects is clear: use the Git object hash directly. This is because:
1. Git object hashes are deterministic and include all content (including signatures)
2. They are the canonical identifier for Git objects
3. All other implementations follow this approach

The fix is to use the OID (Object ID) that Git2 provides, which is the hash of the full object including GPG signatures.
