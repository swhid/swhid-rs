#![doc = include_str!("../README.md")]

pub mod config;
pub mod content;
pub mod core;
pub mod directory;
pub mod error;
#[cfg(feature = "git")]
pub mod git;
pub mod hash;
pub mod permissions;
pub mod qualifier;
pub mod release;
pub mod revision;
pub mod serialization;
pub mod snapshot;
pub mod types;
mod utils;

pub use config::HashConfig;
pub use content::Content;
pub use core::{ObjectType, Swhid};
pub use directory::{Directory, DiskDirectoryBuilder, Entry, WalkOptions};
pub use directory::{DirectoryBuildOptions, ManifestEntry};
pub use hash::{HashFunction, Sha1Hash, Sha256Hash};
pub use permissions::{
    resolve_file_permissions, EntryExec, EntryPerms, PermissionPolicy, PermissionsSource,
    PermissionsSourceKind,
};
pub use qualifier::{ByteRange, LineRange, QualifiedSwhid};
pub use release::{Release, ReleaseTargetType};
pub use revision::Revision;
pub use serialization::{
    Base32HexSerializer, Base32Serializer, Base64Serializer, Base64UrlSerializer, DigestSerializer,
    HexSerializer, Z85Serializer,
};
pub use snapshot::{Branch, BranchTarget, Snapshot};
pub use types::{Encoding, HashAlgorithm, SwhidVersion};

#[cfg(feature = "serde")]
pub use serde::{Deserialize, Serialize};

type Bytestring = Box<[u8]>;
