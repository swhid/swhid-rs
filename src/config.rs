//! Type-level config for hash and encoding (no dyn, no runtime dispatch).

use crate::hash::HashFunction;
use crate::serialization::DigestSerializer;
use crate::types::SwhidVersion;

/// Config for SWHID computation: concrete hasher and encoder (fixed at compile time).
#[derive(Clone)]
pub struct HashConfig<H, E> {
    /// Hash implementation.
    pub hash: H,
    /// Digest encoding for display/parse.
    pub encoder: E,
    /// SWHID version.
    pub version: SwhidVersion,
}

impl<H, E> HashConfig<H, E>
where
    H: HashFunction,
    E: DigestSerializer,
{
    /// Build a config from concrete hash and encoder.
    pub fn new(hash: H, encoder: E, version: SwhidVersion) -> Self {
        Self { hash, encoder, version }
    }
}

// Constructors gated by (hash, encoding) features.

#[cfg(all(feature = "sha1", feature = "encoding-hex"))]
impl HashConfig<crate::hash::Sha1Hash, crate::serialization::HexSerializer> {
    /// V1 config: SHA-1 + hex.
    pub fn v1() -> Self {
        Self::new(
            crate::hash::Sha1Hash,
            crate::serialization::HexSerializer,
            SwhidVersion::V1,
        )
    }
}

#[cfg(all(feature = "sha256", feature = "encoding-base64url"))]
impl HashConfig<crate::hash::Sha256Hash, crate::serialization::Base64UrlSerializer> {
    /// V2 config: SHA-256 + base64url.
    pub fn v2() -> Self {
        Self::new(
            crate::hash::Sha256Hash,
            crate::serialization::Base64UrlSerializer,
            SwhidVersion::V2,
        )
    }
}
