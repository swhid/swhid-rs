//! Digest encoding/decoding for SWHID (pluggable serialization).

mod base32;
mod base64;
mod z85;

use crate::error::SwhidError;

/// Encodes and decodes digest bytes to/from string representation.
pub trait DigestSerializer: Send + Sync {
    /// Encode digest bytes to string (e.g. lowercase hex).
    fn encode(&self, digest: &[u8]) -> String;

    /// Decode string to digest bytes. Fails on invalid length or characters.
    fn decode(&self, s: &str) -> Result<Vec<u8>, SwhidError>;
}

/// Lowercase hexadecimal digest encoding (SWHID v1 default).
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

pub use base32::{Base32HexSerializer, Base32Serializer};
pub use base64::{Base64Serializer, Base64UrlSerializer};
pub use z85::Z85Serializer;
