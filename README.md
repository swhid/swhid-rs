# swhid-rs: SWHID v1.2 reference and v2 exploration implementation

This crate provides a minimal implementation of the SWHID (SoftWare Hash IDentifier) format as defined in **ISO/IEC 18670:2025** and detailed in the SWHID v1.2 specification.

## Exploration status

> **Note on versioning:** This branch (`v2-typespecialisation`) is an experimental refactor of the swhid-rs library. It explores the transition from the SHA1-only model of SWHID v1 to the modular architecture required for SWHID v2 (ISO 18670). It is not the current stable reference for v1; for that, please refer to the `main` branch. This branch is suitable for v2-alpha testing.

This implementation is **fully compliant** with SWHID v1.2 and provides:

- Core identifier representation and parsing/printing (`swh:1:<tag>:<id>`)
- All SWHID v1.2 object types: contents (`cnt`), directories (`dir`), revisions (`rev`),
  releases (`rel`), snapshots (`snp`)
- Qualified identifiers (origin, visit, anchor, path, lines, bytes)
- SWHID v1.2 compliant hash computation for **content** and **directory** objects

VCS Integration (optional):
- Computing `rev`, `rel`, `snp` SWHIDs from VCS metadata (requires `git` feature)
- Git repository support for revision, release, and snapshot SWHID computation

## Technical highlights

- **Modular hashing:** Multiple hash algorithms (SHA-1, SHA-256, SHA-512) via `HashFunction` trait and `HashConfig<H, E>`.
- **Type-level specialization:** Hash and encoder are fixed at compile time; no runtime dispatch; zero-cost abstraction.
- **Config-based pipeline:** `swhid_with_config(&config)` on Content, Directory, Revision, Release, Snapshot; `HashConfig::v1()`, `HashConfig::v2()`, etc.
- **Feature-gated builds:** Cargo features select which hashes and encodings are compiled (e.g. `sha1`, `sha256`, `encoding-hex`, `encoding-base64url`).

## Features

| Feature | Description |
|---------|-------------|
| `sha1` | SHA-1 hash (default) |
| `sha256` | SHA-256 hash |
| `sha512` | SHA-512 hash |
| `encoding-hex` | Hex encoding (default) |
| `encoding-base64` | Base64 encoding |
| `encoding-base64url` | Base64url encoding |
| `encoding-base32` | Base32 encoding (RFC 4648) |
| `encoding-base32hex` | Base32hex encoding |
| `encoding-z85` | Z85 encoding (ZeroMQ Base85) |
| `git` | VCS integration for SWHID v1.2 revision/release/snapshot computation |
| `serde` | Enable `Serialize`/`Deserialize` for all public types |

## Installing the CLI
- **Install (Rust):** `cargo install swhid` (add `--features git` for VCS commands).
- **Binaries:** [Releases](https://github.com/swhid/swhid-rs/releases) (tagged versions) or [Actions](https://github.com/swhid/swhid-rs/actions/workflows/release-binaries.yml) (latest build). Download for your OS/arch, extract, and run (e.g. `./swhid --help`).
- **More:** [User guide](docs/user-guide.md) for all install options and CLI usage.

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

### Creating a SWHID with config (v2)

```rust,no_run
use swhid::{Content, HashConfig};

// V2 (SHA-256 + base64url) - requires sha256 and encoding-base64url features
#[cfg(all(feature = "sha256", feature = "encoding-base64url"))]
{
    let config = HashConfig::v2();
    let content = Content::from_bytes(b"Hello, World!");
    let swhid = content.swhid_with_config(&config);
    println!("V2 SWHID: {}", swhid.to_string_encoded(&config.encoder));
}
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
    let revision_swhid = git::revision_swhid(&repo, &head_commit, &mut Default::default())?;

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
# Content SWHIDs
swhid content --file README.md
echo "Hello, World!" | swhid content

# Hash and format options (v1 and v2)
swhid content --hash sha1 --format hex --file README.md      # v1
swhid content --hash sha256 --format base64url --file README.md  # v2

# Directory SWHIDs
swhid dir .
swhid dir --exclude .tmp --exclude .log /path/to/project

# VCS SWHIDs (requires --features git)
swhid git revision /path/to/git/repo [COMMIT]
swhid git release /path/to/git/repo v1.0.0
swhid git snapshot /path/to/git/repo
swhid git tags /path/to/git/repo

# Parse and validate SWHIDs
swhid parse 'swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391'
swhid parse 'swh:1:dir:...;origin=https://github.com/user/repo;path=/src/main.rs;lines=10-20'

# Verify SWHIDs
swhid verify README.md 'swh:1:cnt:...'
```

## License

Licensed under **MIT**.

## References

- [Software Hash Identifier specification](https://swhid.org/swhid-specification/v1.2/)
- **ISO/IEC 18670:2025** — International standard for Software Heritage Identifiers
- **Software Heritage** — [softwareheritage.org](https://www.softwareheritage.org/)
