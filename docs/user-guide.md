# SWHID user guide

This guide describes how to use the `swhid` library and CLI for computing and parsing SWHIDs (ISO/IEC 18670).

> **Exploration status:** This branch (`v2-typespecialisation`) is an experimental refactor. For the stable v1 reference, see the `main` branch. See [README.md](../README.md) for details.

## Library usage

### Default (v1) SWHIDs

By default, the library produces **SWHID v1** identifiers: SHA-1 digest, lowercase hex encoding, version `1` in the URI:

- **Content**: `Content::from_bytes(bytes).swhid()` → `swh:1:cnt:<40 hex chars>`
- **Directory**: `Directory::new(entries)?.swhid()?` or `DiskDirectoryBuilder::new(path).build()?.swhid()?`
- **Revision / Release / Snapshot**: construct the type from manifest data, then `.swhid()`

Parsing uses `Swhid::from_str` or `"swh:1:cnt:...".parse::<Swhid>()`. Display uses lowercase hex.

### Config-based pipeline (v1 and v2)

To choose hash and encoding explicitly, use `HashConfig` and `swhid_with_config`:

```rust
use swhid::{Content, HashConfig, Swhid};

// V1 (SHA-1 + hex) when default or sha1 + encoding-hex enabled
let config = HashConfig::v1();
let content = Content::from_bytes(b"data");
let swhid = content.swhid_with_config(&config);
let s = swhid.to_string_encoded(&config.encoder); // same as swhid.to_string() for v1

// V2 (SHA-256 + base64url) when sha256 + encoding-base64url enabled
#[cfg(all(feature = "sha256", feature = "encoding-base64url"))]
let config_v2 = HashConfig::v2();
```

- **Content**: `content.swhid_with_config(&config)`
- **Directory**: `dir.swhid_with_config(&config)?`
- **Revision / Release / Snapshot**: `rev.swhid_with_config(&config)`, etc.
- **Display**: `swhid.to_string_encoded(&config.encoder)` so the string uses the config’s encoding (hex or base64url).

### Git integration

With the `git` feature, use `swhid::git` to compute revision, release, and snapshot SWHIDs from a Git repository. The same config-based API exists: `revision_swhid_with_config`, `release_swhid_with_config`, `snapshot_swhid_with_config`.

## CLI

### Installing and testing the CLI

You can try the CLI in three ways:

1. **From crates.io (recommended for regular use)**  
   If you have Rust installed:
   ```bash
   cargo install swhid
   ```
   With Git support (revision/release/snapshot commands):
   ```bash
   cargo install swhid --features git
   ```

2. **From source**  
   In a checkout of the repo:
   ```bash
   cargo run --bin swhid -- [args...]
   # or build once, then run:
   cargo build --release && ./target/release/swhid [args...]
   ```

3. **Pre-built binaries**  
   CI builds release binaries for:
   - `x86_64-unknown-linux-gnu`
   - `aarch64-apple-darwin`
   - `x86_64-pc-windows-msvc`  
   Download the artifact for your platform from the latest [Release binaries](https://github.com/swhid/swhid-rs/actions/workflows/release-binaries.yml) workflow run (Actions → Release binaries → select run → Artifacts). Extract the binary and run it (e.g. `chmod +x swhid && ./swhid --help`). For tagged versions (e.g. `v0.2.3`), a [GitHub Release](https://github.com/swhid/swhid-rs/releases) is created automatically with these binaries attached.

### Commands

- **Content**: `swhid content [--file PATH]` — read from file or stdin, print SWHID.
- **Directory**: `swhid dir PATH [options]` — compute directory SWHID (see `--help` for walk and permission options).
- **Parse**: `swhid parse "swh:1:cnt:..."`
- **Verify**: `swhid verify PATH SWHID` — compute SWHID for path and compare to given SWHID.
- **Git** (with `git` feature): `swhid git revision REPO [COMMIT]`, `swhid git release REPO TAG`, `swhid git snapshot REPO`, `swhid git tags REPO`.

### Hash and format options

Global options (apply to content, dir, verify, and git commands when both are set):

- `--hash HASH`: `sha1`, `sha256`, or `sha512` (requires corresponding feature).
- `--format FORMAT`: `hex`, `base64`, `base64url`, `base32`, `base32hex`, or `z85` (requires corresponding feature).

Examples:

- Default (v1): `swhid content` → `swh:1:cnt:<hex>`
- Explicit v1: `swhid --hash sha1 --format hex content`
- V2 (if built with sha256 + encoding-base64url): `swhid --hash sha256 --format base64url content`
- V2 with hex: `swhid --hash sha256 --format hex content`
- V2 with z85 (most compact): `swhid --hash sha256 --format z85 content` (requires encoding-z85)
- SHA-512: `swhid --hash sha512 --format hex content` (requires sha512)

When using `--hash` and `--format`, **verify** compares the computed and expected strings directly (both in that encoding).

## Cargo features

| Feature           | Description                          |
|------------------|--------------------------------------|
| `sha1`           | SHA-1 (default)                      |
| `sha256`         | SHA-256                              |
| `sha512`         | SHA-512                              |
| `encoding-hex`   | Hex encoding (default)               |
| `encoding-base64`| Base64 encoding                      |
| `encoding-base64url` | Base64url encoding               |
| `encoding-base32`| Base32 encoding (RFC 4648)            |
| `encoding-base32hex` | Base32hex encoding                |
| `encoding-z85`   | Z85 encoding (ZeroMQ Base85)          |
| `git`            | Git repo integration                 |
| `serde`          | Serialize/Deserialize for public types |

At least one hash and one encoding feature must be enabled. Defaults: `sha1`, `sha256`, `encoding-hex`, `encoding-base64url`.
