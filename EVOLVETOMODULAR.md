# Evolve to modular hash and serialization (v2-plugins)

This document tracks the refactor to make hash and serialization **pluggable** inside the core SWHID computation, with a single code path and v1 as the default config.

## Stage 1 — SHA1 + Hex only (replicate current behavior)

Goal: introduce plugin types and implementations (HashFunction, DigestSerializer) with only SHA1 and Hex; all existing behavior unchanged; validate with `cargo test` and swhid-rs-tools.

### Batch 1: Types and serialization module

- **Branch**: `v2-plugins` from `main`.
- **Added** `src/types.rs`: `SwhidVersion::V1`, `HashAlgorithm::Sha1`, `Encoding::Hex` (minimal enums for Stage 1).
- **Added** `src/serialization/mod.rs`: trait `DigestSerializer` (`encode`, `decode`), `HexSerializer` using the `hex` crate (lowercase; decode expects 40 hex chars).
- **Updated** `src/lib.rs`: `pub mod types`, `pub mod serialization`; re-exports for `DigestSerializer`, `HexSerializer`, `Encoding`, `HashAlgorithm`, `SwhidVersion`.
