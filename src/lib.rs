//! SWHID v1.2 reference implementation
//!
//! This crate provides a **clean, small, dependency‑light** implementation
//! of the SWHID (SoftWare Hash IDentifier) format as defined in
//! **ISO/IEC 18670:2025** and detailed in the SWHID v1.2 specification.
//!
//! This implementation is **fully compliant** with SWHID v1.2 and provides:
//! - Core identifier representation and parsing/printing (`swh:1:<tag>:<id>`)
//! - All SWHID v1.2 object types: contents (`cnt`), directories (`dir`), revisions (`rev`),
//!   releases (`rel`), snapshots (`snp`)
//! - Qualified identifiers (origin, visit, anchor, path, lines, bytes)
//! - SWHID v1.2 compliant hash computation for **content** and **directory** objects
//!
//! VCS Integration (optional):
//! - Computing `rev`, `rel`, `snp` SWHIDs from VCS metadata (requires `git` feature)
//! - Git repository support for revision, release, and snapshot SWHID computation
//!
//! The hashing algorithms implement the SWHID v1.2 specification using SHA‑1
//! with collision detection for enhanced security when processing untrusted data.
//!
//! ## Example
//!
//! ```no_run
//! use swhid::{Content, Directory, ObjectType, Swhid, QualifiedSwhid};
//!
//! // Content hash
//! let blob = Content::from_bytes(b"Hello, world!");
//! let s = blob.swhid();
//! assert_eq!(s.to_string(), "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684");
//!
//! // Parse & format
//! let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684".parse().unwrap();
//! assert_eq!(core.object_type(), ObjectType::Content);
//!
//! // Qualified identifiers
//! let q = QualifiedSwhid::new(core).with_path("/src/lib.rs");
//! assert!(q.to_string().contains(";path=/src/lib.rs"));
//! ```
//!
pub mod content;
pub mod core;
pub mod directory;
pub mod error;
pub mod git;
pub mod hash;
pub mod qualifier;

pub use crate::content::Content;
pub use crate::core::{ObjectType, Swhid};
pub use crate::directory::{Directory, DiskDirectoryBuilder, WalkOptions};
pub use crate::qualifier::{ByteRange, LineRange, QualifiedSwhid};

#[cfg(feature = "serde")]
pub use serde::{Deserialize, Serialize};

mod readme_test {
    #![doc = include_str!("../README.md")]
}
