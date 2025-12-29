pub mod hex;
pub mod base64;
pub mod base32;
pub mod base85;

pub use hex::HexSerializer;
pub use base64::{Base64Serializer, Base64UrlSerializer};
pub use base32::{Base32Serializer, Base32HexSerializer};
pub use base85::Z85Serializer;

use crate::error::SwhidError;

/// Trait for serializing and deserializing hash digests.
///
/// This trait abstracts over different encoding schemes to allow pluggable
/// serialization formats for SWHID v2 experimentation. Supported formats:
///
/// - **hex**: Hexadecimal encoding (default for v1, 64 chars for SHA256)
/// - **base64**: Standard Base64 encoding with padding (44 chars for SHA256)
/// - **base64url**: URL-safe Base64 without padding (43 chars for SHA256)
/// - **base32**: RFC 4648 Base32 encoding (52 chars for SHA256)
/// - **base32hex**: Base32hex variant with alternative character set (52 chars for SHA256)
/// - **z85**: ZeroMQ Base85 encoding (40 chars for SHA256, most compact)
///
/// All serializers support roundtrip encoding/decoding and are suitable for use
/// in SWHID identifiers. The choice of format depends on requirements for
/// compactness, URL-safety, and compatibility.
pub trait DigestSerializer: Send + Sync {
    /// Encode a digest byte array to a string representation.
    ///
    /// The input is a raw digest (e.g., 20 bytes for SHA1, 32 bytes for SHA256).
    /// Returns a string representation suitable for use in SWHID identifiers.
    ///
    /// # Errors
    ///
    /// Returns an error if the digest cannot be encoded (e.g., Z85 requires
    /// length to be a multiple of 4 bytes).
    ///
    /// # Examples
    ///
    /// ```
    /// use swhid::serialization::{HexSerializer, Base64Serializer, Z85Serializer};
    ///
    /// let digest = vec![0u8; 32]; // SHA256 digest
    /// let hex_encoded = HexSerializer::new().encode(&digest).unwrap();
    /// let base64_encoded = Base64Serializer::new().encode(&digest).unwrap();
    /// let z85_encoded = Z85Serializer::new().encode(&digest).unwrap();
    ///
    /// assert_eq!(hex_encoded.len(), 64);
    /// assert_eq!(base64_encoded.len(), 44);
    /// assert_eq!(z85_encoded.len(), 40);
    /// ```
    fn encode(&self, digest: &[u8]) -> Result<String, SwhidError>;

    /// Decode a string representation back to a digest byte array.
    ///
    /// Returns an error if the input is not valid for this serialization format.
    ///
    /// # Examples
    ///
    /// ```
    /// use swhid::serialization::HexSerializer;
    ///
    /// let serializer = HexSerializer::new();
    /// let encoded = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
    /// let decoded = serializer.decode(encoded).unwrap();
    /// assert_eq!(decoded.len(), 20);
    /// ```
    fn decode(&self, encoded: &str) -> Result<Vec<u8>, SwhidError>;

    /// Return the name of the serialization format.
    ///
    /// Examples: "hex", "base64", "base64url", "base32", "base32hex", "z85"
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_serializer_roundtrip() {
        let serializer = HexSerializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = serializer.encode(&data).unwrap();
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn base64_serializer_roundtrip() {
        let serializer = Base64Serializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = serializer.encode(&data).unwrap();
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn base64url_serializer_roundtrip() {
        let serializer = Base64UrlSerializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = serializer.encode(&data).unwrap();
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn base32_serializer_roundtrip() {
        let serializer = Base32Serializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = serializer.encode(&data).unwrap();
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn base32hex_serializer_roundtrip() {
        let serializer = Base32HexSerializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = serializer.encode(&data).unwrap();
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn z85_serializer_roundtrip() {
        let serializer = Z85Serializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = serializer.encode(&data).unwrap();
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }
}

