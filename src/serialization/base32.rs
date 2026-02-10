use crate::error::SwhidError;
use data_encoding::{BASE32, BASE32HEX};

use super::DigestSerializer;

/// Base32 serialization for hash digests (RFC 4648 standard).
///
/// Uses standard Base32 encoding with padding. This provides a more compact
/// representation than hex (e.g., 20 bytes = 32 base32 chars vs 40 hex chars,
/// 32 bytes = 52 base32 chars vs 64 hex chars).
#[derive(Debug, Clone, Copy, Default)]
pub struct Base32Serializer;

impl DigestSerializer for Base32Serializer {
    fn encode(&self, digest: &[u8]) -> String {
        BASE32.encode(digest)
    }

    fn decode(&self, s: &str) -> Result<Vec<u8>, SwhidError> {
        BASE32
            .decode(s.as_bytes())
            .map_err(|e| SwhidError::InvalidDigest(format!("Invalid base32: {e}")))
    }
}

/// Base32hex serialization for hash digests.
///
/// Uses Base32hex encoding (alternative character set: 0-9, A-V instead of A-Z, 2-7).
/// This variant uses a different character set that is more suitable for hexadecimal
/// representation. Provides the same compactness as standard Base32.
#[derive(Debug, Clone, Copy, Default)]
pub struct Base32HexSerializer;

impl DigestSerializer for Base32HexSerializer {
    fn encode(&self, digest: &[u8]) -> String {
        BASE32HEX.encode(digest)
    }

    fn decode(&self, s: &str) -> Result<Vec<u8>, SwhidError> {
        BASE32HEX
            .decode(s.as_bytes())
            .map_err(|e| SwhidError::InvalidDigest(format!("Invalid base32hex: {e}")))
    }
}
