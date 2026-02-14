//! Lowercase hex digest encoding.

use crate::error::SwhidError;

use super::DigestSerializer;

#[derive(Debug, Clone, Copy, Default)]
pub struct HexSerializer;

impl DigestSerializer for HexSerializer {
    fn encode(&self, digest: &[u8]) -> String {
        hex::encode(digest)
    }

    fn decode(&self, s: &str) -> Result<Vec<u8>, SwhidError> {
        if s.len() % 2 != 0 || !s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
            return Err(SwhidError::InvalidDigest(s.to_owned()));
        }
        hex::decode(s).map_err(|_| SwhidError::InvalidDigest(s.to_owned()))
    }
}
