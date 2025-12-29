use crate::error::SwhidError;
use super::DigestSerializer;

/// Hex (hexadecimal) serialization for hash digests.
///
/// This is the default serialization format for SWHID v1 (SHA1 + hex).
/// Produces lowercase hexadecimal strings (e.g., "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391").
pub struct HexSerializer;

impl HexSerializer {
    /// Create a new hex serializer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HexSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl DigestSerializer for HexSerializer {
    fn encode(&self, digest: &[u8]) -> Result<String, SwhidError> {
        Ok(hex::encode(digest))
    }

    fn decode(&self, encoded: &str) -> Result<Vec<u8>, SwhidError> {
        hex::decode(encoded)
            .map_err(|e| SwhidError::EncodingError {
                format: "hex".to_string(),
                message: format!("Invalid hex encoding: {e}"),
            })
    }

    fn name(&self) -> &str {
        "hex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_encode() {
        let serializer = HexSerializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let encoded = serializer.encode(&data).unwrap();
        assert_eq!(encoded, "12345678");
    }

    #[test]
    fn hex_decode() {
        let serializer = HexSerializer::new();
        let encoded = "12345678";
        let decoded = serializer.decode(encoded).unwrap();
        assert_eq!(decoded, vec![0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn hex_roundtrip() {
        let serializer = HexSerializer::new();
        let data = vec![0x00, 0xff, 0x12, 0xab, 0xcd, 0xef];
        let encoded = serializer.encode(&data).unwrap();
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn hex_decode_invalid() {
        let serializer = HexSerializer::new();
        assert!(serializer.decode("invalid").is_err());
        assert!(serializer.decode("123g").is_err()); // 'g' is not valid hex
    }

    #[test]
    fn hex_name() {
        let serializer = HexSerializer::new();
        assert_eq!(serializer.name(), "hex");
    }

    #[test]
    fn hex_sha1_digest() {
        let serializer = HexSerializer::new();
        let sha1_digest = vec![0xe6, 0x9d, 0xe2, 0x9b, 0xb2, 0xd1, 0xd6, 0x43, 0x4b, 0x8b, 0x29, 0xae, 0x77, 0x5a, 0xd8, 0xc2, 0xe4, 0x8c, 0x53, 0x91];
        let encoded = serializer.encode(&sha1_digest).unwrap();
        assert_eq!(encoded, "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
        assert_eq!(encoded.len(), 40); // 20 bytes * 2 = 40 hex chars
    }

    #[test]
    fn hex_sha256_digest() {
        let serializer = HexSerializer::new();
        let sha256_digest = vec![0u8; 32];
        let encoded = serializer.encode(&sha256_digest).unwrap();
        assert_eq!(encoded.len(), 64); // 32 bytes * 2 = 64 hex chars
    }
}

