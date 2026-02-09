# Evolve to modular hash and serialization (v2-plugins)

This document tracks the refactor to make hash and serialization **pluggable** inside the core SWHID computation, with a single code path and v1 as the default config.

## Stage 1 — SHA1 + Hex only (replicate current behavior)

Goal: introduce plugin types and implementations (HashFunction, DigestSerializer) with only SHA1 and Hex; all existing behavior unchanged; validate with `cargo test` and swhid-rs-tools.

### Batch 1: Types and serialization module

- **Branch**: `v2-plugins` from `main`.
- **Added** `src/types.rs`: `SwhidVersion::V1`, `HashAlgorithm::Sha1`, `Encoding::Hex` (minimal enums for Stage 1).
- **Added** `src/serialization/mod.rs`: trait `DigestSerializer` (`encode`, `decode`), `HexSerializer` using the `hex` crate (lowercase; decode expects 40 hex chars).
- **Updated** `src/lib.rs`: `pub mod types`, `pub mod serialization`; re-exports for `DigestSerializer`, `HexSerializer`, `Encoding`, `HashAlgorithm`, `SwhidVersion`.

### Batch 2: Hash plugin and HashConfig

- **Updated** `src/hash.rs`: Added `HashFunction` trait (`hash`, `digest_size`, `name`), `Sha1Hash` impl, and `hash_swhid_object_generic(typ, payload, hasher)`. `hash_content` and `hash_swhid_object` now call the generic path with `Sha1Hash`; public API and signatures unchanged.
- **Added** `src/config.rs`: `HashConfig` with `hash_function`, `serializer`, `version`; `HashConfig::v1()` (SHA1 + Hex + V1).
- **Updated** `src/lib.rs`: `pub mod config`, re-exports for `HashConfig`, `HashFunction`, `Sha1Hash`.

### Batch 3: Wire core and manifests to Hex serializer

- **Updated** `src/core.rs`: `digest_hex()`, `Display`, and `FromStr` use `HexSerializer` (trait `DigestSerializer` in scope). No direct `hex::encode` / `hex::decode`; digest encoding goes through the plugin.
- **Updated** `src/revision.rs`: `rev_manifest` uses `HexSerializer.encode` for tree and parent digests.
- **Updated** `src/release.rs`: `rel_manifest` uses `HexSerializer.encode` for object digest.
- Behavior unchanged; all tests pass.

### Validation gate (Stage 1 complete)

- **cargo test**: All 106 lib tests + integration tests pass.
- **swhid-rs-tools**: With `SWHID_RS_PATH=/path/to/swhid-rs`, `./tools/test_rust.sh`:
  - content: 6/6 passed
  - directory: 5/5 passed
  - revision: 13/13 passed
  - release: 11/11 passed
  - snapshot: category not in harness config (no payloads); rust binary runs.
- **Checklist**: No new public API; `Swhid`, `hash_content`, `hash_swhid_object` unchanged; only SHA1 and Hex in use. Stage 1 done.

---

## Stage 2 — Extra plugins and single config-based pipeline

### Batch 2.1: SHA256 plugin and v2 config

- **Cargo.toml**: added `sha2 = "0.10"`.
- **types.rs**: `SwhidVersion::V2`, `HashAlgorithm::Sha256`.
- **hash.rs**: `Sha256Hash` implementing `HashFunction` (32-byte digest).
- **serialization**: `HexSerializer::decode` accepts any even-length hex (40 for v1, 64 for v2).
- **config.rs**: `HashConfig::v2_sha256_hex()` (SHA256 + Hex + V2).
- **lib.rs**: re-export `Sha256Hash`.

### Batch 2.2: Swhid evolution (variable digest + version)

- **core.rs**: `Swhid` now has `digest: Vec<u8>` and `version: SwhidVersion`. Added `new(object_type, digest, version)`, `new_v1(object_type, [u8; 20])`, `digest_bytes() -> &[u8]`, `version()`. Display uses `version_str()` (1 or 2). FromStr accepts version "1" or "2" and variable-length hex digest.
- **content, directory, revision, release, snapshot**: all use `Swhid::new_v1(...)` when producing v1 SWHIDs.
- **git.rs**: v1 path copies `digest_bytes()` to `[u8; 20]` where needed (parents, release target, snapshot branch targets).
- **core test**: `swhid_parse_invalid_version` now only rejects "0" and "3"; "2" is valid.

### Batch 2.3: Content — config-based SWHID

- **content.rs**: `swhid_with_config(&self, config: &HashConfig) -> Swhid` using `hash_swhid_object_generic("blob", ..., config.hash_function)` and `Swhid::new(..., config.version)`. `swhid()` calls `swhid_with_config(&HashConfig::v1())`.

### Batch 2.4: Directory — Entry.id as Vec&lt;u8&gt;, config-based SWHID

- **directory.rs**: `Entry.id` changed from `[u8; 20]` to `Vec<u8>`; `Entry::new(name, mode, id: impl Into<Vec<u8>>)`; `From<ManifestEntry>` uses `manifest.target` (Vec). Added `swhid_with_config(&self, config: &HashConfig)` (manifest + generic hash + `Swhid::new(..., config.version)`); `swhid()` delegates to v1 config.
- **git.rs**: `DirEntry::new(name, mode, id.to_vec())` when building tree entries.

### Batch 2.5: Revision, Release, Snapshot — Vec&lt;u8&gt; ids and config-based SWHID

- **revision.rs**: `Revision.directory` and `Revision.parents` are `Vec<u8>` and `Vec<Vec<u8>>`. `swhid_with_config` (manifest, config hasher, `Swhid::new(..., config.version)`); `swhid()` = `swhid_with_config(&HashConfig::v1())`. `rev_manifest` already uses `HexSerializer.encode` for directory/parents.
- **release.rs**: `Release.object` is `Vec<u8>`. Same pattern: `swhid_with_config`, `swhid()` delegates to v1 config.
- **snapshot.rs**: `BranchTarget` variants use `Option<Vec<u8>>` instead of `Option<[u8; 20]>`; `target_id()` uses `id.as_deref().unwrap_or(b"")`. Added `swhid_with_config` and delegated `swhid()` to v1 config.
- **git.rs**: `revision_from_git` uses `directory.to_vec()`, `parents.into_iter().map(|p| p.to_vec()).collect()`; `release_from_git` uses `object.to_vec()`; `reference_to_branch` uses `d.to_vec()` / `digest.to_vec()` for branch targets. Release object built from `revision_swhid`/`release_swhid`/etc. by binding the `Swhid` before calling `digest_bytes()` to avoid temporary-borrow errors.
- **Tests**: revision, release, snapshot, git tests updated for `Vec<u8>` / `Option<Vec<u8>>` and explicit types / slice comparisons where needed; `cargo test --features git` passes.
