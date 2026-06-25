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
- **`src/directory.rs`** — `Entry`, `Directory`, `DiskDirectoryBuilder`, `dir_manifest`, `swhid()` / `swhid_with_config()`, recursive display (`PathEntry`, `SwhidCollector`, `DiskDirectoryBuilder::recursive_swhids()`).
- **`src/revision.rs`**, **`src/release.rs`**, **`src/snapshot.rs`** — manifest types and `swhid()` / `swhid_with_config()`.
- **`src/git.rs`** — (optional) revision/release/snapshot from Git; `*_swhid_with_config` and helpers.
- **`src/main.rs`** — CLI: content, dir, parse, verify, git; `--hash` / `--format` dispatch to config-based APIs.

## Recursive directory traversal ordering (`swhid dir -R`)

`DiskDirectoryBuilder::recursive_swhids()` returns `Vec<PathEntry>` (relative path + SWHID),
which `swhid dir -R` prints as `SWHID<TAB>PATH`. The output order is **not** the traversal
order — it is produced by a single sort at the end of `recursive_swhids()`:

```rust
recursive_swhids.sort_unstable_by(|a, b| a.path.cmp(&b.path));
```

Implications when modifying this code:

- **The traversal order is irrelevant.** `read_dir_with_permission_source` recurses into a
  directory before pushing the directory itself (post-order) and visits a directory's entries
  in raw `fs::read_dir` order (OS-dependent, unsorted). The final sort overrides both, so the
  result is deterministic regardless of filesystem enumeration order (paths are distinct, so
  `sort_unstable` ties never occur).
- **Top-to-bottom (pre-order) is a property of the sort key, not the walk.** The key is a
  `PathBuf`, and `Path::cmp` compares **component-by-component**, not as a flat byte string.
  Because a directory's path is a prefix of its children's, each directory sorts immediately
  above its whole subtree, and siblings group together — equivalent to a pre-order DFS with
  siblings visited in sorted order. The root is appended as `"."` and sorts first.
- **Why component-wise matters:** a sibling like `src-extra` must sort *after* the entire
  `src/` subtree, not between `src` and `src/a.rs`. A naive byte-string sort would place it
  in the middle (since `-` 0x2D < `/` 0x2F); `Path::cmp` avoids this. If you ever switch the
  comparator (e.g. to sort on `path.to_string_lossy()`), this invariant breaks.

To change the ordering (e.g. files-before-directories, or directory-grouped with indentation
derived from `path.components().count()`), edit only this comparator.

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

This is the single canonical v2 trunk. Two earlier v2 explorations — `v2-exploration` (runtime `Box<dyn>` dispatch) and `v2-plugins` (a pluggable-trait pipeline) — were alternative redesigns of the same hash/serialization/config subsystem; their worthwhile pieces (non-hex `Swhid::parse_with`, raw-tag `gpgsig` extra-header extraction, the `docs/design/` spec notes) have been folded in here, and the branches are retained only as the `archive/v2-exploration`, `archive/v2-exploration-backup`, and `archive/v2-plugins` tags.

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
