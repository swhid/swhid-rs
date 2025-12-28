# Contributing to swhid-rs

Thank you for your interest in contributing to **swhid-rs**.

This repository contains:

- A Rust **library** implementing SWHID parsing/formatting and object hashing.
- A Rust **CLI** for generating and inspecting SWHIDs.
- Optional **Git integration** behind a feature flag.

If you want to work on SWHID v2 (SHA256 + configurable digest encodings), start with `docs/v2/README.md` (or, if not present yet, begin with `src/types.rs`, `src/config.rs`, `src/core.rs`, and `src/git.rs`).

## Requirements

- Rust edition: **2021**
- Tooling: `rustc`, `cargo`, and `git`
- Optional: `rustfmt` and `clippy` (part of the standard Rust toolchain)

If the project declares an MSRV (minimum supported Rust version) in documentation, please respect it.

## Getting started

### Clone and run library tests

```sh
git clone https://github.com/swhid/swhid-rs.git
cd swhid-rs
cargo test --lib
```

### Build and run the CLI

```sh
cargo build
cargo run -- --help
```

### Feature flags

- `git`: enables Git repository integration (`src/git.rs`) and Git-related integration tests.

Run Git integration tests:

```sh
cargo test --tests --features git
```

Run everything with all features:

```sh
cargo test --all-features
```

## Repository map

Key entry points:

- `src/types.rs`  
  Type-safe enums used across the API (version, hash algorithm, encoding, object type, etc.).

- `src/config.rs`  
  Configuration and policy wiring (v1/v2, sha1/sha256, digest encoding selection).

- `src/core.rs`  
  Core SWHID type; parsing/formatting rules; canonical vs presentation encodings.

- `src/serialization/*`  
  Digest encoding/decoding (hex/base32/base64/z85, etc.) and encoding-specific errors.

- `src/git.rs` (**feature: `git`**)  
  Git repository integration, including SHA256 repositories where supported.

- `tests/*.rs`  
  Integration tests per object type and per feature.

Docs:

- `README.md`: quick overview and quick start
- `TUTORIAL.md`: extended examples (CLI and library)
- `DEVELOPER_GUIDE.md`: contributor-focused guide and architecture notes
- `REFERENCE.md`: reference behavior and invariants
- `MIGRATION.md`: API/behavior migration guidance
- `docs/v2/README.md`: v2 contributor guide (if present)

## How to contribute

### 1) Pick or propose work

- If there is an issue tracker, please pick an existing issue first.
- For new issues, include:
  - expected behavior
  - current behavior
  - reproduction steps (inputs/commands)
  - acceptance criteria

### 2) Branches and commits

Create a branch from `main`:

- `fix/<topic>` for bug fixes
- `feat/<topic>` for features
- `docs/<topic>` for documentation-only changes
- `chore/<topic>` for refactors/maintenance

Commit messages:
- Use imperative mood: `fix: ...`, `feat: ...`, `docs: ...`, `chore: ...`
- Keep commits small and reviewable.

### 3) Formatting and linting

Before submitting a PR:

```sh
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
```

### 4) Testing expectations

Minimum bar for most changes:

```sh
cargo test --lib
cargo test --all-features
cargo test --tests --features git
```

Add or adjust tests that:
- fail before your change, and
- pass after your change.

Prefer:
- unit tests for local invariants and edge cases
- integration tests for end-to-end behavior (library + CLI paths)

### 5) Documentation expectations

If a change affects user-facing behavior, update docs accordingly:

- CLI flags/output → `TUTORIAL.md` (and `README.md` if needed)
- Public API behavior → `REFERENCE.md`
- v2 semantics → `docs/v2/README.md`

### 6) Adding a new digest encoding

1. Implement `DigestSerializer` under `src/serialization/`.
2. Register it in `src/serialization/mod.rs`.
3. Expose it via the encoding enum in `src/types.rs`.
4. Wire it through config factories in `src/config.rs`.
5. Add tests:
   - encode/decode roundtrips
   - invalid input cases
   - CLI output changes when `--serialization` changes (if applicable)

### 7) Adding a new hash algorithm

1. Implement `HashFunction` under `src/hash/`.
2. Register it in `src/hash/mod.rs`.
3. Add enum wiring and config support (`src/types.rs`, `src/config.rs`).
4. Enforce digest-size invariants at parse/decode boundaries.
5. Add tests for each supported object type.

## Review checklist (for contributors)

Before requesting review:

- [ ] `cargo fmt` clean
- [ ] `cargo clippy --all-features` clean
- [ ] Tests pass for default and all features
- [ ] New behavior is documented
- [ ] New invariants have tests (not just comments)

Thank you for helping improve swhid-rs.
