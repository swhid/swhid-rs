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
