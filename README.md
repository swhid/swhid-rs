# swhid-rs: SWHID v1.2 reference implementation

This crate provides a minimal implementation of the SWHID (SoftWare Hash IDentifier) format as defined in **ISO/IEC 18670:2025** and detailed in the SWHID v1.2 specification.

This implementation is **fully compliant** with SWHID v1.2 and provides:
- Core identifier representation and parsing/printing (`swh:1:<tag>:<id>`)
- All SWHID v1.2 object types: contents (`cnt`), directories (`dir`), revisions (`rev`),
  releases (`rel`), snapshots (`snp`)
- Qualified identifiers (origin, visit, anchor, path, lines, bytes)
- SWHID v1.2 compliant hash computation for **content** and **directory** objects

**SWHID v2 Support (Experimental):**
- Support for SHA256 hash function (in addition to SHA1)
- Multiple serialization formats: hex, base64, base64url, base32, base32hex, z85
- Configurable hash and serialization via `HashConfig`
- Automatic detection of Git repository hash algorithm (SHA1 vs SHA256)

VCS Integration (optional):
- Computing `rev`, `rel`, `snp` SWHIDs from VCS metadata (requires `git` feature)
- Git repository support for revision, release, and snapshot SWHID computation
- **Current limitation**: Only SHA1 Git repositories are fully supported
- SHA256 Git repository support is planned but requires architectural changes

## Features

| Feature | Description |
|---------|-------------|
| `serde` | Enable `Serialize`/`Deserialize` for all public types |
| `git` | Enable VCS integration for SWHID v1.2 revision/release/snapshot computation |

## Serialization Formats

SWHID v2 supports multiple serialization formats for hash digests, each with different characteristics:

| Format     | SHA1 (20 bytes) | SHA256 (32 bytes) | Use Case                          |
|------------|-----------------|-------------------|-----------------------------------|
| **hex**    | 40 chars        | 64 chars          | Default, Git-compatible           |
| **base64** | 28 chars        | 44 chars          | Standard Base64, compact          |
| **base64url** | 27 chars      | 43 chars          | URL-safe, no padding             |
| **base32** | 32 chars        | 52 chars          | RFC 4648 standard                 |
| **base32hex** | 32 chars      | 52 chars          | Base32hex variant                 |
| **z85**    | 25 chars        | 40 chars          | Most compact, ZeroMQ variant      |

**Note:** SWHID v1 uses SHA1 + hex (40 characters). SWHID v2 uses SHA256 with configurable serialization format.


## Examples

### Parsing a SWHID

```rust
use std::path::Path;
use swhid::*;

let swhid: Swhid = "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391".parse()?;
println!("Object type: {:?}", swhid.object_type()); // Content
println!("Digest: {}", swhid.digest_hex());

# Ok::<_, Box<dyn std::error::Error>>(())
```

### Creating a SWHID

```rust,no_run
use std::path::Path;
use swhid::*;

let content = Content::from_bytes(b"Hello, World!");
let swhid = content.swhid();
println!("Content SWHID: {}", swhid);

let dir = DiskDirectoryBuilder::new(Path::new("/path/to/directory"));
let swhid = dir.swhid()?;
println!("Directory SWHID: {}", swhid);

# Ok::<_, Box<dyn std::error::Error>>(())
```

### Creating a SWHID v2 with Different Serialization Formats

```rust,no_run
use swhid::{Content, config::HashConfig};

let content = Content::from_bytes(b"Hello, World!");

// V1 (default): SHA1 + hex
let v1_swhid = content.swhid();
println!("V1 SWHID: {}", v1_swhid); // swh:1:cnt:...

// V2 with different serialization formats
let hex_config = HashConfig::v2_sha256_hex();
let base64_config = HashConfig::v2_sha256_base64();
let z85_config = HashConfig::v2_sha256_z85();

let hex_swhid = content.swhid_with_config(&hex_config);
let base64_swhid = content.swhid_with_config(&base64_config);
let z85_swhid = content.swhid_with_config(&z85_config);

// All produce the same digest bytes (same hash function)
assert_eq!(hex_swhid.digest_bytes(), base64_swhid.digest_bytes());
assert_eq!(hex_swhid.digest_bytes(), z85_swhid.digest_bytes());

// But different string representations (Display uses hex for all)
println!("V2 hex: {}", hex_swhid);
println!("V2 base64: {}", base64_swhid);
println!("V2 z85: {}", z85_swhid);
```

### Creating a qualified SWHID

```rust,no_run
use swhid::{ByteRange, LineRange, Swhid, QualifiedSwhid};

let core: Swhid = "swh:1:cnt:...".parse()?;
let qualified = QualifiedSwhid::new(core)
    .with_origin("https://github.com/user/repo")
    .with_path("/src/main.rs")
    .with_lines(LineRange { start: 10, end: Some(20) })
    .with_bytes(ByteRange { start: 100, end: Some(200) });

println!("Qualified SWHID: {}", qualified);
// Output: swh:1:cnt:...;origin=https://github.com/user/repo;path=/src/main.rs;lines=10-20;bytes=100-200

# Ok::<_, Box<dyn std::error::Error>>(())
```

### VCS Integration (Git Feature)

```rust,no_run
use std::path::PathBuf;

#[cfg(feature = "git")]
{
    use swhid::git;

    let repo = git::open_repo(&PathBuf::from("/path/to/git/repo"))?;
    
    // Get HEAD commit SWHID v1.2
    let head_commit = git::get_head_commit(&repo)?;
    let revision_swhid = git::revision_swhid(&repo, &head_commit)?;
    
    // Get tag SWHID v1.2
    let tag_oid = repo.refname_to_id("refs/tags/v1.0.0")?;
    let release_swhid = git::release_swhid(&repo, &tag_oid)?;
    
    // Get snapshot SWHID v1.2
    let snapshot_swhid = git::snapshot_swhid(&repo)?;
}

# Ok::<_, Box<dyn std::error::Error>>(())
```

## CLI Tool

```bash
# Content SWHIDs (v1, default)
swhid content --file README.md
echo "Hello, World!" | swhid content

# Content SWHIDs (v2 with different serialization formats)
echo "Hello, World!" | swhid --version 2 --hash sha256 --serialization hex content
echo "Hello, World!" | swhid --version 2 --hash sha256 --serialization base64 content
echo "Hello, World!" | swhid --version 2 --hash sha256 --serialization base64url content
echo "Hello, World!" | swhid --version 2 --hash sha256 --serialization base32 content
echo "Hello, World!" | swhid --version 2 --hash sha256 --serialization base32hex content
echo "Hello, World!" | swhid --version 2 --hash sha256 --serialization z85 content

# Directory SWHIDs
swhid dir .
swhid dir --exclude-suffix .tmp --exclude-suffix .log /path/to/project
swhid --version 2 --hash sha256 --serialization z85 dir /path/to/project

# VCS SWHIDs (requires --features git)
# Currently supports SHA1 Git repositories only
swhid git revision --repo /path/to/git/repo
swhid git release --repo /path/to/git/repo --tag v1.0.0
swhid git snapshot --repo /path/to/git/repo
swhid git tags --repo /path/to/git/repo

# Parse and validate SWHIDs
swhid parse 'swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391'
swhid parse 'swh:2:cnt:a0a477f1ecf419c7eaa7fe256c5c12fb03bee86df9a22aad25f85930de203e14'
swhid parse 'swh:1:dir:...;origin=https://github.com/user/repo;path=/src/main.rs;lines=10-20'

# Verify SWHIDs
swhid verify --file README.md --expected 'swh:1:cnt:...'
swhid --version 2 --hash sha256 --serialization z85 verify --file README.md --expected 'swh:2:cnt:...'
```

### CLI Options

- `--version <VERSION>`: SWHID version (1 or 2, default: 1)
- `--hash <HASH>`: Hash function (sha1 or sha256, default: sha1)
- `--serialization <FORMAT>`: Serialization format (hex, base64, base64url, base32, base32hex, or z85, default: hex)

**Valid combinations:**
- v1: sha1 + hex (only)
- v2: sha256 + hex/base64/base64url/base32/base32hex/z85

## License

Licensed under **MIT**.

## References

- [Software Heritage Identifier specification v1.6](https://docs.softwareheritage.org/devel/swh-model/persistent-identifiers.html)
- **ISO/IEC 18670:2025** — International standard for Software Heritage Identifiers
- **Software Heritage** — [softwareheritage.org](https://www.softwareheritage.org/)
