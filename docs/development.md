# Development guide

This document describes how to work on the `swhid-rs` codebase: layout, testing, and extension points.

## Layout

- **`src/core.rs`** — `ObjectType`, `Swhid` (core identifier), `FromStr`/`Display`, `to_string_encoded`.
- **`src/digest.rs`** — `Digest` enum (Sha1/Sha256/Sha512, feature-gated), `as_bytes`, `from_bytes`, `From<[u8; N]>`.
- **`src/types.rs`** — `SwhidVersion` (V1, V2).
- **`src/config.rs`** — `HashConfig<H, E>`, `v1()`, `v2()`, `v2_hex()`, `v2_base64()`, `v2_base32()`, `v2_base32hex()`, `v2_z85()`, `sha512_hex()`, `sha512_base64url()` (feature-gated).
- **`src/serialization/`** — `DigestSerializer`, `HexSerializer`, `Base64Serializer`, `Base64UrlSerializer`, `Base32Serializer`, `Base32HexSerializer`, `Z85Serializer`.
- **`src/hash/`** — `HashFunction` trait, `swhid_object_header`, per-hash modules (`sha1`, `sha256`, `sha512`).
- **`src/content.rs`** — `Content`, `swhid()` / `swhid_with_config()`.
- **`src/directory.rs`** — `Entry`, `Directory`, `DiskDirectoryBuilder`, `dir_manifest`, `swhid()` / `swhid_with_config()`.
- **`src/revision.rs`**, **`src/release.rs`**, **`src/snapshot.rs`** — manifest types and `swhid()` / `swhid_with_config()`.
- **`src/git.rs`** — (optional) revision/release/snapshot from Git; `*_swhid_with_config` and helpers.
- **`src/main.rs`** — CLI: content, dir, parse, verify, git; `--hash` / `--format` dispatch to config-based APIs.

## Testing

- **Unit tests**: in `src/*.rs` under `#[cfg(test)]` modules.
- **Integration tests**: `tests/*.rs` (content, directory, revision, release, snapshot, git).
- **Doc tests**: in `src/lib.rs` and public API.

Commands:

```bash
cargo test
cargo test --features git
cargo test --no-default-features --features sha1,encoding-hex
```

CI (`.github/workflows/rust.yml`) runs format check, clippy, and tests with default features and with `git`.

## Adding a new hash or encoding

1. **Hash**
   - Add a feature in `Cargo.toml` and an optional dependency.
   - In `src/hash/`, add a module implementing `HashFunction` (fixed output type that implements `Into<Digest>`).
   - In `src/digest.rs`, add a `Digest` variant and `From<[u8; N]>` for that size; extend `from_bytes` for length `N`.
   - Optionally add a `HashConfig` constructor (e.g. `v2()`) in `src/config.rs` for a specific (hash, encoding) pair.

2. **Encoding**
   - Add a feature and optional dependency if needed.
   - In `src/serialization/`, implement `DigestSerializer` (encode/decode digest bytes to/from string).
   - Export the serializer and use it in config constructors and CLI as needed.

## Branch `v2-typespecialisation`

The branch introduces type-level hash and encoding (no runtime dispatch): `Digest` enum, `HashConfig<H, E>`, and `swhid_with_config` across content, directory, revision, release, snapshot, and git. Default behaviour remains v1 (SHA-1 + hex).

### Architecture summary

- **Type-level H and E:** Hash and encoder are fixed at the call site via `HashConfig<H, E>`. No `dyn`; no runtime branch on hash or encoding in the hot path.
- **Monomorphization:** One monomorphization per `(H, E)` pair actually used. The pipeline (content, directory, revision, release, snapshot, git) is generic over `H` and `E`.
- **Zero-cost abstraction:** The compiler specializes each config combination at compile time; there is no runtime overhead compared to hand-written per-hash code.

### Technical highlights

- **Modular hashing:** Multiple hash algorithms (SHA-1, SHA-256, SHA-512) via `HashFunction` trait and `HashConfig<H, E>`.
- **Config-based pipeline:** `swhid_with_config(&config)` on Content, Directory, Revision, Release, Snapshot; `HashConfig::v1()`, `HashConfig::v2()`, etc.
- **Feature-gated builds:** Cargo features select which hashes and encodings are compiled.

### Contributor note

The trait-heavy design may require more upfront understanding. Inline documentation in `config.rs`, `hash/mod.rs`, and `content.rs` helps.
