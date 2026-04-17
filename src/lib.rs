#![doc = include_str!("../README.md")]

pub mod content;
pub mod core;
pub mod directory;
pub mod error;
#[cfg(feature = "git")]
pub mod git;
#[cfg(feature = "gitoxide")]
pub mod git_gix;
pub mod hash;
pub mod permissions;
pub mod qualifier;
pub mod release;
pub mod revision;
pub mod snapshot;
mod utils;

pub use content::Content;
pub use core::{ObjectType, Swhid};
pub use directory::{Directory, DiskDirectoryBuilder, Entry, WalkOptions};
pub use directory::{DirectoryBuildOptions, ManifestEntry};
pub use permissions::{
    resolve_file_permissions, EntryExec, EntryPerms, GitIndexPermissionsSource,
    GitTreePermissionsSource, PermissionPolicy, PermissionsSource, PermissionsSourceKind,
};
pub use qualifier::{ByteRange, LineRange, QualifiedSwhid};
pub use release::{Release, ReleaseTargetType};
pub use revision::Revision;
pub use snapshot::{Branch, BranchTarget, Snapshot};

#[cfg(feature = "serde")]
pub use serde::{Deserialize, Serialize};

type Bytestring = Box<[u8]>;
