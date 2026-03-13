# SWHID user guide

This guide describes how to use the `swhid` library and CLI on the **v2-typespecialisation** branch: v1.2 compliant SWHIDs plus a config-based pipeline for v1 and v2 (multiple hashes and encodings).

## SWHID v2 exploration

This branch is part of the **SWHID v2 exploration** effort. Open questions include which hash algorithm (SHA-256 is a strong candidate, given Git’s roadmap) and which user-facing encoding to use. Internally, the Merkle graph is built on hex; for user convenience, we may prefer a more compact encoding (hex, base64url, base64, base32, base32hex, or z85). 
## Library usage

### Default (v1) SWHIDs

By default, the library produces **SWHID v1** identifiers: SHA-1 digest, lowercase hex encoding, version `1` in the URI:

- **Content:** `Content::from_bytes(bytes).swhid()` -> `swh:1:cnt:<40 hex chars>`
- **Directory:** `Directory::new(entries)?.swhid()?` or `DiskDirectoryBuilder::new(path).build()?.swhid()?`
- **Revision / Release / Snapshot:** construct the type from manifest data, then `.swhid()`

Parsing: `Swhid::from_str` or `"swh:1:cnt:...".parse::<Swhid>()`. Display uses lowercase hex.

### Config-based pipeline (v1 and v2)

To choose hash and encoding explicitly, use `HashConfig` and `swhid_with_config`:

```rust
use swhid::{Content, HashConfig, Swhid};

// V1 (SHA-1 + hex) — requires sha1 + encoding-hex features
let config = HashConfig::v1();
let content = Content::from_bytes(b"data");
let swhid = content.swhid_with_config(&config);
let s = swhid.to_string_encoded(&config.encoder); // same as swhid.to_string() for v1

// V2 (SHA-256 + base64url) — requires sha256 + encoding-base64url features
#[cfg(all(feature = "sha256", feature = "encoding-base64url"))]
let config_v2 = HashConfig::v2();
```

- **Content:** `content.swhid_with_config(&config)`
- **Directory:** `dir.swhid_with_config(&config)?`
- **Revision / Release / Snapshot:** `rev.swhid_with_config(&config)`, etc.
- **Display:** `swhid.to_string_encoded(&config.encoder)` so the string uses the config’s encoding (hex, base64url, etc.).

Config constructors are feature-gated: `HashConfig::v1()`, `HashConfig::v2()`, `HashConfig::v2_hex()`, `HashConfig::v2_base64()`, `HashConfig::v2_base32()`, `HashConfig::v2_base32hex()`, `HashConfig::v2_z85()`, `HashConfig::sha512_hex()`, `HashConfig::sha512_base64url()`.

### Git integration

With the `git` feature, use `swhid::git` with config: `revision_swhid_with_config`, `release_swhid_with_config`, `snapshot_swhid_with_config`.

## CLI

### Installing and testing the CLI

1. **From crates.io**
 `cargo install swhid` (default features include sha1, sha256, encoding-hex, encoding-base64url).
 With Git support: `cargo install swhid --features git`

2. **From source**
 `cargo run --bin swhid -- [args...]` or `cargo build --release && ./target/release/swhid [args...]`

3. **Pre-built binaries**  
   CI builds binaries for Linux (x86_64), macOS (aarch64), and Windows (x86_64). Download from the latest [Release binaries](https://github.com/swhid/swhid-rs/actions/workflows/release-binaries.yml) run (Artifacts), or from [Releases](https://github.com/swhid/swhid-rs/releases).

   **Experimental binaries (this branch):** Pre-releases tagged `v2-exp-YYYYMMDD` (e.g. `v2-exp-20260301`) provide binaries with full v2 support (all hash/format combinations). They are named `swhid-v2-exp-<platform>` to distinguish them from stable v1 binaries. Stable releases use tags like `v0.2.3` and produce `swhid-<platform>`.

   **Generating binaries for this branch:** Use tag `v2-exp-YYYYMMDD` (date-based). Run `git tag v2-exp-20260301` then `git push origin v2-exp-20260301`. The workflow creates a pre-release with `swhid-v2-exp-*` binaries.

### Commands

- **Content:** `swhid content [--file PATH]` — read from file or stdin, print SWHID.
- **Directory:** `swhid dir PATH [options]` — compute directory SWHID (see `--help` for walk and permission options).
- **Parse:** `swhid parse "swh:1:cnt:..."` or `swh:2:cnt:...`
- **Verify:** `swhid verify PATH SWHID` — compute SWHID for path and compare to given SWHID.
- **Git** (with `git` feature): `swhid git revision REPO [COMMIT]`, `swhid git release REPO TAG`, `swhid git snapshot REPO`, `swhid git tags REPO`.

### Hash and format options

Global options (apply to content, dir, verify, and git commands when set):

- `--hash HASH`: `sha1`, `sha256`, or `sha512` (requires corresponding feature).
- `--format FORMAT`: `hex`, `base64`, `base64url`, `base32`, `base32hex`, or `z85` (requires corresponding feature).

Examples:

- Default (v1): `swhid content` -> `swh:1:cnt:<hex>`
- Explicit v1: `swhid --hash sha1 --format hex content`
- V2 (sha256 + base64url): `swhid --hash sha256 --format base64url content`
- V2 with hex: `swhid --hash sha256 --format hex content`
- V2 with z85: `swhid --hash sha256 --format z85 content` (requires encoding-z85)
- SHA-512: `swhid --hash sha512 --format hex content` (requires sha512)

When using `--hash` and `--format`, **verify** compares the computed and expected strings in that encoding.

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

### Creating a SWHID (v1)

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
use swhid::{Content, HashConfig, Swhid};

#[cfg(all(feature = "sha256", feature = "encoding-base64url"))]
{
let config = HashConfig::v2();
let content = Content::from_bytes(b"Hello, World!");
let swhid = content.swhid_with_config(&config);
println!("V2 SWHID: {}", swhid.to_string_encoded(&config.encoder));
}

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

### VCS integration (Git feature)

```rust,no_run
use std::path::PathBuf;

#[cfg(feature = "git")]
{
use swhid::git;
use swhid::HashConfig;

let repo = git::open_repo(&PathBuf::from("/path/to/git/repo"))?;
let head_commit = git::get_head_commit(&repo)?;
let config = HashConfig::v1(); // or HashConfig::v2() when features enabled
let revision_swhid = git::revision_swhid_with_config(&repo, &head_commit, &config)?;
let tag_oid = repo.refname_to_id("refs/tags/v1.0.0")?;
let release_swhid = git::release_swhid_with_config(&repo, &tag_oid, &config)?;
let snapshot_swhid = git::snapshot_swhid_with_config(&repo, &config)?;
}

# Ok::<_, Box<dyn std::error::Error>>(())
```

## CLI examples

```bash
# Content (v1 default)
swhid content --file README.md
echo "Hello, World!" | swhid content

# Content (v2)
swhid --hash sha256 --format base64url content --file README.md
swhid --hash sha256 --format hex content --file README.md

# Directory
swhid dir .
swhid dir --exclude .tmp --exclude .log /path/to/project

# Parse and verify
swhid parse 'swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391'
swhid verify README.md 'swh:1:cnt:...'

# Git (requires --features git)
swhid git revision /path/to/git/repo [COMMIT]
swhid git release /path/to/git/repo v1.0.0
swhid git snapshot /path/to/git/repo
swhid git tags /path/to/git/repo
```

## Test results dashboard

The [SWHID Test Results Dashboard](https://www.swhid.org/swhid-exploration-deploy/) shows how different implementations (including this one) perform against the SWHID test suite.

- **Implementations:** rust (swhid-rs), go, python, ruby, git, etc. When both v1 and v2 tests run, results appear in `_v1` and `_v2` columns (e.g. rust_v1, rust_v2). The v1 dashboard runs v1 only; the v2 dashboard runs v2 only.
- **Platforms:** Ubuntu, Windows, macOS
- **Pass / fail / skip:** Each run reports how many tests passed, failed, or were skipped (e.g. when a hash/format combo is not enabled)
- **Detailed results:** Expand a run to see per-test outcomes

The dashboard is generated by the SWHID Testing Harness and is updated as new runs are submitted.

## Cargo features

| Feature | Description|
|---------------------|--------------------------------------|
| `sha1`| SHA-1 hash (default) |
| `sha256`| SHA-256 hash |
| `sha512`| SHA-512 hash |
| `encoding-hex` | Hex encoding (default) |
| `encoding-base64`| Base64 encoding|
| `encoding-base64url` | Base64url encoding |
| `encoding-base32` | Base32 encoding (RFC 4648) |
| `encoding-base32hex`| Base32hex encoding |
| `encoding-z85`| Z85 encoding (ZeroMQ Base85) |
| `git` | VCS integration for rev/rel/snp|
| `serde` | Serialize/Deserialize for public types|

Default features: `sha1`, `sha256`, `encoding-hex`, `encoding-base64url`. At least one hash and one encoding must be enabled.
