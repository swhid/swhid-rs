//! Digest encoding/decoding for SWHID display and parsing.

mod hex;
#[cfg(any(feature = "encoding-base64", feature = "encoding-base64url"))]
mod base64;
#[cfg(any(feature = "encoding-base32", feature = "encoding-base32hex"))]
mod base32;
#[cfg(feature = "encoding-z85")]
mod base85;

use crate::error::SwhidError;

/// Encodes and decodes digest bytes to/from string representation.
pub trait DigestSerializer: Send + Sync {
    /// Encode digest bytes to string (e.g. lowercase hex).
    fn encode(&self, digest: &[u8]) -> String;
    /// Decode string to digest bytes.
    fn decode(&self, s: &str) -> Result<Vec<u8>, SwhidError>;
}

#[cfg(feature = "encoding-hex")]
pub use hex::HexSerializer;
#[cfg(feature = "encoding-base64")]
pub use base64::Base64Serializer;
#[cfg(feature = "encoding-base64url")]
pub use base64::Base64UrlSerializer;
#[cfg(feature = "encoding-base32")]
pub use base32::Base32Serializer;
#[cfg(feature = "encoding-base32hex")]
pub use base32::Base32HexSerializer;
#[cfg(feature = "encoding-z85")]
pub use base85::Z85Serializer;
