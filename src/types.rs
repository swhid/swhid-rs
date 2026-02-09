//! SWHID version, hash algorithm, and encoding types for pluggable pipeline.
//! Stage 1: only V1, Sha1, Hex. Extended in Stage 2.

/// SWHID spec version (v1 = 20-byte SHA1 hex; v2 = variable hash/encoding).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SwhidVersion {
    /// SWHID v1: SHA-1 digest, 40 hex chars
    V1,
    /// SWHID v2: configurable hash and encoding (e.g. SHA-256, hex/base64)
    V2,
}

/// Hash algorithm for digest computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HashAlgorithm {
    /// SHA-1 (20 bytes)
    Sha1,
    /// SHA-256 (32 bytes)
    Sha256,
}

/// Encoding for digest string representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Encoding {
    /// Lowercase hexadecimal
    Hex,
}
