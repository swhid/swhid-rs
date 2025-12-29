# swhid-rs: SWHID v1.2 reference implementation

This crate provides a minimal implementation of the SWHID (SoftWare Hash IDentifier) format as defined in **ISO/IEC 18670:2025** and detailed in the SWHID v1.2 specification;

This implementation is **fully compliant** with SWHID v1.2 and provides:
- Core identifier representation and parsing/printing (`swh:1:<tag>:<id>`)
- All SWHID v1.2 object types: contents (`cnt`), directories (`dir`), revisions (`rev`),
  releases (`rel`), snapshots (`snp`)
- Qualified identifiers (origin, visit, anchor, path, lines, bytes)
- SWHID v1.2 compliant hash computation for **content** and **directory** objects

VCS Integration (optional):
- Computing `rev`, `rel`, `snp` SWHIDs from VCS metadata (requires `git` feature)
- Git repository support for revision, release, and snapshot SWHID computation

## Features

| Feature | Description |
|---------|-------------|
| `serde` | Enable `Serialize`/`Deserialize` for all public types |
| `git` | Enable VCS integration for SWHID v1.2 revision/release/snapshot computation |


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
# Content SWHIDs
swhid content --file README.md
echo "Hello, World!" | swhid content

# Directory SWHIDs
swhid dir .
swhid dir --exclude-suffix .tmp --exclude-suffix .log /path/to/project

# VCS SWHIDs (requires --features git)
swhid git revision --repo /path/to/git/repo
swhid git release --repo /path/to/git/repo --tag v1.0.0
swhid git snapshot --repo /path/to/git/repo
swhid git tags --repo /path/to/git/repo

# Parse and validate SWHIDs
swhid parse 'swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391'
swhid parse 'swh:1:dir:...;origin=https://github.com/user/repo;path=/src/main.rs;lines=10-20'

# Verify SWHIDs
swhid verify --file README.md --expected 'swh:1:cnt:...'
```

## License

Dual-licensed under **MIT** or **Apache-2.0** at your option.

## References

- [Software Heritage Identifier specification v1.6](https://docs.softwareheritage.org/devel/swh-model/persistent-identifiers.html)
- **ISO/IEC 18670:2025** — International standard for Software Heritage Identifiers
- **Software Heritage** — [softwareheritage.org](https://www.softwareheritage.org/)
