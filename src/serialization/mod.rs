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
/// This trait abstracts over different encoding schemes (hex, base64, etc.)
/// to allow pluggable serialization formats for SWHID v2 experimentation.
pub trait DigestSerializer: Send + Sync {
    /// Encode a digest byte array to a string representation.
    ///
    /// The input is a raw digest (e.g., 20 bytes for SHA1, 32 bytes for SHA256).
    /// Returns a string representation suitable for use in SWHID identifiers.
    fn encode(&self, digest: &[u8]) -> String;

    /// Decode a string representation back to a digest byte array.
    ///
    /// Returns an error if the input is not valid for this serialization format.
    fn decode(&self, encoded: &str) -> Result<Vec<u8>, SwhidError>;

    /// Return the name of the serialization format.
    ///
    /// Examples: "hex", "base64", "base64url"
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_serializer_roundtrip() {
        let serializer = HexSerializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = serializer.encode(&data);
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn base64_serializer_roundtrip() {
        let serializer = Base64Serializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = serializer.encode(&data);
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn base64url_serializer_roundtrip() {
        let serializer = Base64UrlSerializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = serializer.encode(&data);
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn base32_serializer_roundtrip() {
        let serializer = Base32Serializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = serializer.encode(&data);
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn base32hex_serializer_roundtrip() {
        let serializer = Base32HexSerializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = serializer.encode(&data);
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn z85_serializer_roundtrip() {
        let serializer = Z85Serializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = serializer.encode(&data);
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }
}

