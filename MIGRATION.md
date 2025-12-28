# Migration Guide: SWHID v1 to v2

This guide helps you migrate from SWHID v1 to v2 in the `swhid-rs` library.

## Overview

SWHID v2 introduces:
- **SHA256 hash function** (instead of SHA1)
- **Multiple serialization formats** (hex, base64, base32, z85, etc.)
- **Variable-length digests** (32 bytes for SHA256 vs 20 bytes for SHA1)

The library maintains **full backward compatibility** with v1. All existing v1 code continues to work without changes.

## When to Migrate

**You don't need to migrate** if:
- Your code only uses v1 SWHIDs
- You're satisfied with SHA1 + hex encoding
- You need compatibility with existing v1 SWHID databases

**Consider migrating to v2** if:
- You want enhanced security (SHA256 instead of SHA1)
- You need more compact identifiers (base64, z85, etc.)
- You're building new systems without v1 compatibility requirements

## Migration Strategies

### Strategy 1: Gradual Migration (Recommended)

Keep v1 as default, use v2 selectively:

```rust
use swhid::{Content, config::HashConfig};

let content = Content::from_bytes(b"Hello, World!");

// Continue using v1 (default, no changes needed)
let v1_swhid = content.swhid();
println!("V1: {}", v1_swhid); // swh:1:cnt:...

// Use v2 for new features
let v2_config = HashConfig::v2_sha256_hex();
let v2_swhid = content.swhid_with_config(&v2_config);
println!("V2: {}", v2_swhid); // swh:2:cnt:...
```

**Benefits:**
- No breaking changes
- Can adopt v2 incrementally
- Maintains compatibility with existing systems

### Strategy 2: Full Migration

Switch entirely to v2:

```rust
use swhid::{Content, config::HashConfig};

// Replace all swhid() calls with swhid_with_config()
let content = Content::from_bytes(b"Hello, World!");
let config = HashConfig::v2_sha256_hex(); // Or your preferred format
let swhid = content.swhid_with_config(&config);
```

**Benefits:**
- Consistent v2 usage throughout
- Enhanced security with SHA256
- More compact identifiers (if using base64/z85)

**Considerations:**
- All SWHIDs will be v2 format
- May need to update databases/storage
- May break compatibility with v1-only systems

## Code Changes

### Content Objects

**Before (v1):**
```rust
use swhid::Content;

let content = Content::from_bytes(b"data");
let swhid = content.swhid(); // Always v1
```

**After (v2):**
```rust
use swhid::{Content, config::HashConfig};

let content = Content::from_bytes(b"data");
let config = HashConfig::v2_sha256_hex();
let swhid = content.swhid_with_config(&config); // v2
```

### Directory Objects

**Before (v1):**
```rust
use swhid::DiskDirectoryBuilder;

let dir = DiskDirectoryBuilder::new(path);
let swhid = dir.swhid()?; // Always v1
```

**After (v2):**
```rust
use swhid::{DiskDirectoryBuilder, config::HashConfig};

let dir = DiskDirectoryBuilder::new(path);
let config = HashConfig::v2_sha256_base64(); // More compact
let swhid = dir.swhid_with_config(&config)?; // v2
```

### Git Integration

**Before (v1):**
```rust
use swhid::git;

let repo = git::open_repo(path)?;
let commit_oid = git::get_head_commit(&repo)?;
let swhid = git::revision_swhid(&repo, &commit_oid)?; // Always v1
```

**After (v2):**
```rust
use swhid::{git, config::HashConfig};

let repo = git::open_repo(path)?;
let commit_oid = git::get_head_commit(&repo)?;
let config = HashConfig::v2_sha256_hex();
let swhid = git::revision_swhid_with_config(&repo, &commit_oid, &config)?; // v2
```

### Parsing SWHIDs

**Before (v1):**
```rust
let swhid: Swhid = "swh:1:cnt:...".parse()?; // Only hex
```

**After (v2 with different formats):**
```rust
use swhid::{Swhid, serialization::Base64Serializer, types::SwhidVersion};

// Parse hex (canonical format, works for both v1 and v2)
let swhid: Swhid = "swh:2:cnt:...".parse()?;

// Parse base64 v2 SWHID
let base64_str = "swh:2:cnt:..."; // base64 encoded
let swhid = Swhid::parse_with(&base64_str, &Base64Serializer::new(), SwhidVersion::V2)?;
```

## Serialization Format Selection

Choose a format based on your needs:

| Format     | Length (SHA256) | Use Case                          |
|------------|----------------|-----------------------------------|
| **hex**    | 64 chars       | Default, Git-compatible           |
| **base64** | 44 chars       | Standard, compact                  |
| **base64url** | 43 chars    | URL-safe, no padding              |
| **base32** | 52 chars       | RFC 4648 standard                 |
| **base32hex** | 52 chars    | Base32hex variant                  |
| **z85**    | 40 chars       | Most compact, ZeroMQ variant      |

**Recommendation:**
- Use **hex** for Git compatibility and maximum compatibility
- Use **base64** or **base64url** for compactness with wide support
- Use **z85** for maximum compactness (if you control both encoding and decoding)

## CLI Migration

**Before (v1):**
```bash
swhid content --file README.md
```

**After (v2):**
```bash
# Explicit v2 with hex (default for v2)
swhid --version 2 --hash sha256 --serialization hex content --file README.md

# v2 with more compact format
swhid --version 2 --hash sha256 --serialization base64 content --file README.md
```

## Backward Compatibility

The library maintains full backward compatibility:

- All v1 APIs continue to work
- v1 SWHIDs parse correctly
- Default behavior remains v1
- No breaking changes to public API

## Testing Migration

After migrating, verify:

1. **SWHID computation produces expected results:**
```rust
let content = Content::from_bytes(b"test");
let v1_swhid = content.swhid();
let v2_swhid = content.swhid_with_config(&HashConfig::v2_sha256_hex());

// They should be different (different hash functions)
assert_ne!(v1_swhid.digest_bytes(), v2_swhid.digest_bytes());
assert_eq!(v1_swhid.version(), SwhidVersion::V1);
assert_eq!(v2_swhid.version(), SwhidVersion::V2);
```

2. **Parsing works correctly:**
```rust
// v1 SWHIDs still parse
let v1: Swhid = "swh:1:cnt:...".parse()?;

// v2 SWHIDs parse
let v2: Swhid = "swh:2:cnt:...".parse()?;
```

3. **Roundtrip works:**
```rust
let swhid = content.swhid_with_config(&config);
let encoded = swhid.to_string_with(&Base64Serializer::new())?;
let parsed = Swhid::parse_with(&encoded, &Base64Serializer::new(), SwhidVersion::V2)?;
assert_eq!(parsed.digest_bytes(), swhid.digest_bytes());
```

## Common Issues

### Issue: "Invalid combination" error in CLI

**Cause:** Using incompatible version/hash/serialization combination.

**Solution:** Use valid combinations:
- v1: sha1 + hex (only)
- v2: sha256 + hex/base64/base64url/base32/base32hex/z85

### Issue: SWHID length mismatch

**Cause:** v1 uses 40 hex chars, v2 uses 64 hex chars (or other lengths with different formats).

**Solution:** Update code that assumes fixed SWHID length. Use `digest_bytes().len()` to check digest size.

### Issue: Database schema assumes 40-char hex

**Cause:** Existing schema designed for v1 (20-byte SHA1 = 40 hex chars).

**Solution:** 
- Option 1: Store both v1 and v2 SWHIDs
- Option 2: Migrate schema to support variable-length digests
- Option 3: Use v1 for compatibility, v2 for new features

## Further Reading

- [README.md](README.md) - Overview and quick start
- [TUTORIAL.md](TUTORIAL.md) - Comprehensive user tutorial
- [DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md) - Developer documentation
- [REFERENCE.md](REFERENCE.md) - Detailed architectural reference

