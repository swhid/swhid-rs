# swhid-rs: SWHID v1.2 reference implementation

This crate provides a minimal implementation of the SWHID (SoftWare Hash IDentifier) format as defined in **ISO/IEC 18670:2025** and detailed in the SWHID v1.2 specification.

This implementation is **fully compliant** with SWHID v1.2 and provides:

- Core identifier representation and parsing/printing (`swh:1:<tag>:<id>`)
- All SWHID v1.2 object types: contents (`cnt`), directories (`dir`), revisions (`rev`), releases (`rel`), snapshots (`snp`)
- Qualified identifiers (origin, visit, anchor, path, lines, bytes)
- SWHID v1.2 compliant hash computation for **content** and **directory** objects
- Optional VCS integration: computing `rev`, `rel`, `snp` SWHIDs from Git (requires `git` or `gitoxide` feature)

## Installing the CLI

- **Rust:** `cargo install swhid` (add `--features git` for VCS via libgit2, or `--features gitoxide` for VCS via gitoxide).
- **Binaries:** [Releases](https://github.com/swhid/swhid-rs/releases) or [Actions](https://github.com/swhid/swhid-rs/actions/workflows/release-binaries.yml). Download for your OS/arch, extract, run (e.g. `./swhid --help`).
- **More:** [User guide](docs/user-guide.md) for all install options, library usage, examples, and CLI reference.

## Quick start

**Library:** `Content::from_bytes(b"data").swhid()` -> `swh:1:cnt:<hex>`. Parse with `"swh:1:cnt:...".parse::<Swhid>()`.

**CLI:** `swhid content --file README.md` · `swhid dir .` · `swhid parse "swh:1:cnt:..."` · `swhid verify PATH SWHID`

See the [user guide](docs/user-guide.md) for full documentation.

## License

Licensed under **MIT**.

## References

- [SWHID specification](https://swhid.org/swhid-specification/v1.2/)
- **ISO/IEC 18670:2025** — Software Heritage Identifiers
- [Software Heritage](https://www.softwareheritage.org/)
