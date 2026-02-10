# Experimental SWHID v2 options

This document describes the **experimental** CLI and library options added to try out potential candidates for a future SWHID v2. They do not change the v1.2 specification; they only affect how identifiers are computed and displayed when explicitly requested.

## Concepts

- **Version**: SWHID version in the URI (`swh:1:...` vs `swh:2:...`). Version 1 = SHA-1, 20-byte digest. Version 2 = SHA-256, 32-byte digest (experimental).
- **Hash**: Algorithm used to compute the digest (sha1 or sha256).
- **Format**: How the digest is encoded in the SWHID string (hex, base64, base64url, base32, base32hex, z85).

**Rule**: Version 1 is used **only** for sha1 + hex. Any other combination (sha256, or non-hex format) uses version 2.

## CLI options

Available on `content`, `dir`, and `git` (revision, release, snapshot):

| Option     | Description |
|-----------|-------------|
| `--version <VERSION>` | `1` (SHA-1, default) or `2` (SHA-256). V1 only applies to sha1+hex. |
| `--hash <HASH>`       | `sha1` or `sha256`. Overrides the hash implied by `--version`. |
| `--format <FORMAT>`   | Digest encoding: `hex`, `base64`, `base64url`, `base32`, `base32hex`, `z85`. Default: `hex`. |

### Examples

```bash
# Default: v1 (SHA-1 + hex)
swhid content --file README.md

# v2 with hex (SHA-256, 64-char digest)
swhid content --version 2 --file README.md
swhid content --hash sha256 --format hex --file README.md

# v2 with other encodings (same digest, different string)
swhid content --hash sha256 --format base64   --file README.md
swhid content --hash sha256 --format base64url --file README.md
swhid content --hash sha256 --format base32   --file README.md
swhid content --hash sha256 --format z85      --file README.md

# Git commands
swhid git --version 2 revision /path/to/repo
swhid git --hash sha256 --format z85 snapshot /path/to/repo
```

### Format compactness (SHA-256, 32-byte digest)

| Format     | Approx. length | Notes                    |
|-----------|----------------|--------------------------|
| hex       | 64 chars       | Default, SWHID v1 style   |
| base64    | 44 chars       | Standard, with padding   |
| base64url | 43 chars       | URL-safe, no padding     |
| base32    | 52–56 chars    | RFC 4648, with padding   |
| base32hex | 52–56 chars    | 0-9, A-V variant         |
| z85       | 40 chars       | ZeroMQ Base85, compact   |

## Library usage

Use `HashConfig` and `swhid_with_config` to get the same behavior from code:

- `HashConfig::v1()` — SHA-1 + hex (version 1).
- `HashConfig::v2_sha256_hex()` — SHA-256 + hex (version 2).
- `HashConfig::v2_sha256_base64()`, `v2_sha256_base64url()`, `v2_sha256_base32()`, `v2_sha256_base32hex()`, `v2_sha256_z85()` — SHA-256 with other encodings.

Example:

```rust
use swhid::{Content, HashConfig};

let content = Content::from_bytes(b"Hello");
let swhid_v1 = content.swhid();                                    // v1, hex
let swhid_v2_hex = content.swhid_with_config(&HashConfig::v2_sha256_hex());
let swhid_v2_z85 = content.swhid_with_config(&HashConfig::v2_sha256_z85());
```

For Git: `revision_swhid_with_config`, `release_swhid_with_config`, `snapshot_swhid_with_config` take a `&HashConfig`.

## Status

These options are **experimental** and for evaluating v2 candidates. The SWHID v1.2 specification and default behavior are unchanged.
