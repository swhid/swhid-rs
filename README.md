# swhid-rs: SWHID v1.2 reference and v2 exploration

This crate provides a minimal implementation of the SWHID (SoftWare Hash IDentifier) format as defined in **ISO/IEC 18670:2025** and detailed in the SWHID v1.2 specification.

## Exploration status

> **This branch (`v2-typespecialisation`)** is an experimental refactor. It adds a config-based pipeline (v1 and v2), multiple hash algorithms (SHA-1, SHA-256, SHA-512) and encodings (hex, base64url, base32, z85, etc.), and CLI options `--hash` / `--format`. For the **stable v1-only reference**, see the `main` branch.

This implementation is **fully compliant** with SWHID v1.2 and on this branch also explores v2-style identifiers:

- Core representation and parsing for `swh:1:...` and `swh:2:...`
- All object types: content, directory, revision, release, snapshot
- Qualified identifiers; optional Git integration (`git` feature)
- **Config-based pipeline:** `HashConfig::v1()`, `HashConfig::v2()`, `swhid_with_config(&config)` (feature-gated)

## Installing the CLI

- **Rust:** `cargo install swhid` (add `--features git` for VCS commands). Build with default features for v1 + v2 (sha1, sha256, encoding-hex, encoding-base64url).
- **Binaries:** [Releases](https://github.com/swhid/swhid-rs/releases) or [Actions](https://github.com/swhid/swhid-rs/actions/workflows/release-binaries.yml). Download for your OS/arch, extract, run (e.g. `./swhid --help`).
- **More:** [User guide](docs/user-guide.md) for install options, library usage, v1/v2 examples, and CLI reference.

## Quick start

**Library (v1):** `Content::from_bytes(b"data").swhid()` -> `swh:1:cnt:<hex>`.

**Library (v2):** `content.swhid_with_config(&HashConfig::v2())` then `swhid.to_string_encoded(&config.encoder)` (requires `sha256` + `encoding-base64url`).

**CLI:** `swhid content --file README.md` · `swhid --hash sha256 --format base64url content` · `swhid dir .` · `swhid parse "swh:1:cnt:..."`

See the [user guide](docs/user-guide.md) for full documentation.

## License

Licensed under **MIT**.

## References

- [SWHID specification](https://swhid.org/swhid-specification/v1.2/)
- **ISO/IEC 18670:2025** — Software Heritage Identifiers
- [Software Heritage](https://www.softwareheritage.org/)
