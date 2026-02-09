//! Hash and serialization config for the SWHID pipeline (pluggable v1/v2).
//! Stage 1: v1 only (SHA1 + Hex).

use crate::hash::{HashFunction, Sha1Hash};
use crate::serialization::{DigestSerializer, HexSerializer};
use crate::types::SwhidVersion;

/// Configuration for SWHID computation: hasher, digest encoding, and version.
/// Passed through the single pipeline; v1 is the default.
pub struct HashConfig {
    /// Hash implementation (e.g. SHA-1 for v1).
    pub hash_function: Box<dyn HashFunction>,
    /// Digest encoding for display/parse (e.g. hex for v1).
    pub serializer: Box<dyn DigestSerializer>,
    /// SWHID version (V1 = 20-byte hex; V2 = variable in Stage 2).
    pub version: SwhidVersion,
}

impl HashConfig {
    /// V1 config: SHA-1 digest, lowercase hex encoding.
    pub fn v1() -> Self {
        Self {
            hash_function: Box::new(Sha1Hash),
            serializer: Box::new(HexSerializer),
            version: SwhidVersion::V1,
        }
    }
}
