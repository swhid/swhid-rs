//! Base32 and Base32hex digest encoding (RFC 4648).

use data_encoding::{BASE32, BASE32HEX};

use crate::error::SwhidError;

use super::DigestSerializer;

#[derive(Debug, Clone, Copy, Default)]
pub struct Base32Serializer;

impl DigestSerializer for Base32Serializer {
    fn encode(&self, digest: &[u8]) -> String {
        BASE32.encode(digest)
    }

    fn decode(&self, s: &str) -> Result<Vec<u8>, SwhidError> {
        BASE32
            .decode(s.as_bytes())
            .map_err(|e| SwhidError::InvalidDigest(format!("base32: {e}")))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Base32HexSerializer;

impl DigestSerializer for Base32HexSerializer {
    fn encode(&self, digest: &[u8]) -> String {
        BASE32HEX.encode(digest)
    }

    fn decode(&self, s: &str) -> Result<Vec<u8>, SwhidError> {
        BASE32HEX
            .decode(s.as_bytes())
            .map_err(|e| SwhidError::InvalidDigest(format!("base32hex: {e}")))
    }
}
