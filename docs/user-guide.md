# SWHID user guide

This guide describes how to use the `swhid` library and CLI for computing and parsing SWHIDs (ISO/IEC 18670) on the **v1.2 reference implementation** (SHA-1, hex encoding).

## Library usage

### SWHID v1 identifiers

The library produces **SWHID v1** identifiers: SHA-1 digest, lowercase hex encoding, version `1` in the URI.

- **Content:** `Content::from_bytes(bytes).swhid()` -> `swh:1:cnt:<40 hex chars>`
- **Directory:** `Directory::new(entries)?.swhid()?` or `DiskDirectoryBuilder::new(path).build()?.swhid()?`
- **Revision / Release / Snapshot:** construct the type from manifest data, then `.swhid()`

Parsing: `Swhid::from_str` or `"swh:1:cnt:...".parse::<Swhid>()`. Display uses lowercase hex.

### Git integration

Two backend options are available for computing revision, release, and snapshot SWHIDs from Git repositories:

- **`git` feature** — uses libgit2 (via the `git2` crate).
- **`gitoxide` feature** — uses gitoxide (via the `gix` crate).

Both produce identical SWHIDs.

With the `git` feature, use `swhid::git`. With the `gitoxide` feature, use `swhid::git_gix`. Both modules expose the same functions: `revision_swhid`, `release_swhid`, `snapshot_swhid`, `open_repo`, `get_head_commit`, `get_tags`.

## CLI

### Installing and testing the CLI

1. **From crates.io**  
   `cargo install swhid`  
   With Git support (libgit2): `cargo install swhid --features git`  
   With Git support (gitoxide): `cargo install swhid --features gitoxide`

2. **From source**  
   `cargo run --bin swhid -- [args...]` or `cargo build --release && ./target/release/swhid [args...]`

3. **Pre-built binaries**  
   CI builds binaries for Linux (x86_64), macOS (aarch64), and Windows (x86_64). Download from the latest [Release binaries](https://github.com/swhid/swhid-rs/actions/workflows/release-binaries.yml) run (Artifacts), or from [Releases](https://github.com/swhid/swhid-rs/releases) for tagged versions. Extract and run (e.g. `chmod +x swhid && ./swhid --help`).

### Commands

- **Content:** `swhid content [--file PATH]` — read from file or stdin, print SWHID.
- **Directory:** `swhid dir PATH [options]` — compute directory SWHID (see `--help` for walk and permission options).
- **Parse:** `swhid parse "swh:1:cnt:..."`
- **Verify:** `swhid verify PATH SWHID` — compute SWHID for path and compare to given SWHID.
- **Git** (with `git` feature): `swhid git revision REPO [COMMIT]`, `swhid git release REPO TAG`, `swhid git snapshot REPO`, `swhid git tags REPO`.

## Examples

### Parsing a SWHID

```rust
use std::path::Path;
use swhid::*;

let swhid: Swhid = "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391".parse()?;
println!("Object type: {:?}", swhid.object_type());
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

# Ok::<_, Box<dyn std::error::Error>>(())
```

### VCS integration (git feature — libgit2)

```rust,no_run
use std::path::PathBuf;

#[cfg(feature = "git")]
{
    use swhid::git;

    let repo = git::open_repo(&PathBuf::from("/path/to/git/repo"))?;
    let head_commit = git::get_head_commit(&repo)?;
    let revision_swhid = git::revision_swhid(&repo, &head_commit)?;
    let tag_oid = repo.refname_to_id("refs/tags/v1.0.0")?;
    let release_swhid = git::release_swhid(&repo, &tag_oid)?;
    let snapshot_swhid = git::snapshot_swhid(&repo)?;
}

# Ok::<_, Box<dyn std::error::Error>>(())
```

### VCS integration (gitoxide feature)

```rust,no_run
use std::path::PathBuf;

#[cfg(feature = "gitoxide")]
{
    use swhid::git_gix;

    let repo = git_gix::open_repo(&PathBuf::from("/path/to/git/repo"))?;
    let head_commit = git_gix::get_head_commit(&repo)?;
    let revision_swhid = git_gix::revision_swhid(&repo, &head_commit, &mut std::collections::HashMap::new())?;
    let snapshot_swhid = git_gix::snapshot_swhid(&repo)?;
}

# Ok::<_, Box<dyn std::error::Error>>(())
```

## CLI examples

```bash
# Content
swhid content --file README.md
echo "Hello, World!" | swhid content

# Directory
swhid dir .
swhid dir --exclude .tmp --exclude .log /path/to/project

# Parse and verify
swhid parse 'swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391'
swhid verify README.md 'swh:1:cnt:...'

# Git (requires --features git or --features gitoxide)
swhid git revision /path/to/git/repo [COMMIT]
swhid git release /path/to/git/repo v1.0.0
swhid git snapshot /path/to/git/repo
swhid git tags /path/to/git/repo
```

## Cargo features

| Feature | Backend | Description |
|---------|---------|-------------|
| `git` | libgit2 | VCS integration for revision, release, and snapshot SWHIDs |
| `gitoxide` | gix | Same VCS integration, pure-Rust backend |
| `serde` | — | `Serialize`/`Deserialize` for public types |

Both `git` and `gitoxide` provide the same SWHID computation for revision, release, and snapshot objects. They differ only in the underlying Git library. When both features are enabled, `git` (libgit2) takes priority in the CLI.
