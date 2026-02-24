#![doc = include_str!("../README.md")]

#[cfg(not(any(feature = "sha1", feature = "sha256", feature = "sha512")))]
compile_error!("At least one of sha1, sha256, sha512 must be enabled.");

pub mod config;
pub mod content;
pub mod core;
pub mod digest;
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
pub use digest::Digest;
pub use directory::{Directory, DiskDirectoryBuilder, Entry, WalkOptions};
pub use directory::{DirectoryBuildOptions, ManifestEntry};
pub use permissions::{
    resolve_file_permissions, EntryExec, EntryPerms, PermissionPolicy, PermissionsSource,
    PermissionsSourceKind,
};
pub use qualifier::{ByteRange, LineRange, QualifiedSwhid};
pub use release::{Release, ReleaseTargetType};
pub use revision::Revision;
#[cfg(feature = "encoding-base32hex")]
pub use serialization::Base32HexSerializer;
#[cfg(feature = "encoding-base32")]
pub use serialization::Base32Serializer;
#[cfg(feature = "encoding-base64")]
pub use serialization::Base64Serializer;
#[cfg(feature = "encoding-base64url")]
pub use serialization::Base64UrlSerializer;
pub use serialization::DigestSerializer;
#[cfg(feature = "encoding-hex")]
pub use serialization::HexSerializer;
#[cfg(feature = "encoding-z85")]
pub use serialization::Z85Serializer;
pub use snapshot::{Branch, BranchTarget, Snapshot};
pub use types::SwhidVersion;

#[cfg(feature = "serde")]
pub use serde::{Deserialize, Serialize};

type Bytestring = Box<[u8]>;
