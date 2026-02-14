//! Base64 and Base64url digest encoding.

use base64::Engine;

use crate::error::SwhidError;

use super::DigestSerializer;

#[derive(Debug, Clone, Copy, Default)]
pub struct Base64Serializer;

impl DigestSerializer for Base64Serializer {
    fn encode(&self, digest: &[u8]) -> String {
        base64::engine::general_purpose::STANDARD.encode(digest)
    }

    fn decode(&self, s: &str) -> Result<Vec<u8>, SwhidError> {
        base64::engine::general_purpose::STANDARD
            .decode(s)
            .map_err(|e| SwhidError::InvalidDigest(format!("base64: {e}")))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Base64UrlSerializer;

impl DigestSerializer for Base64UrlSerializer {
    fn encode(&self, digest: &[u8]) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
    }

    fn decode(&self, s: &str) -> Result<Vec<u8>, SwhidError> {
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|e| SwhidError::InvalidDigest(format!("base64url: {e}")))
    }
}
