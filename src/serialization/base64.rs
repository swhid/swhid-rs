use crate::error::SwhidError;
use base64::Engine;

use super::DigestSerializer;

/// Base64 standard serialization for hash digests.
///
/// Uses standard Base64 encoding with padding. This provides a more compact
/// representation than hex (e.g., 32 bytes = 44 base64 chars vs 64 hex chars).
#[derive(Debug, Clone, Copy, Default)]
pub struct Base64Serializer;

impl DigestSerializer for Base64Serializer {
    fn encode(&self, digest: &[u8]) -> String {
        base64::engine::general_purpose::STANDARD.encode(digest)
    }

    fn decode(&self, s: &str) -> Result<Vec<u8>, SwhidError> {
        base64::engine::general_purpose::STANDARD
            .decode(s)
            .map_err(|e| SwhidError::InvalidDigest(format!("Invalid base64: {e}")))
    }
}

/// Base64 URL-safe serialization for hash digests.
///
/// Uses URL-safe Base64 encoding without padding. This is suitable for use
/// in URLs and provides a compact representation.
#[derive(Debug, Clone, Copy, Default)]
pub struct Base64UrlSerializer;

impl DigestSerializer for Base64UrlSerializer {
    fn encode(&self, digest: &[u8]) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
    }

    fn decode(&self, s: &str) -> Result<Vec<u8>, SwhidError> {
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|e| SwhidError::InvalidDigest(format!("Invalid base64url: {e}")))
    }
}
