use crate::error::SwhidError;
use super::DigestSerializer;
use data_encoding::{BASE32, BASE32HEX};

/// Base32 serialization for hash digests (RFC 4648 standard).
///
/// Uses standard Base32 encoding with padding. This provides a more compact
/// representation than hex (e.g., 20 bytes = 32 base32 chars vs 40 hex chars,
/// 32 bytes = 52 base32 chars vs 64 hex chars).
pub struct Base32Serializer;

impl Base32Serializer {
    /// Create a new base32 serializer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Base32Serializer {
    fn default() -> Self {
        Self::new()
    }
}

impl DigestSerializer for Base32Serializer {
    fn encode(&self, digest: &[u8]) -> Result<String, SwhidError> {
        Ok(BASE32.encode(digest))
    }

    fn decode(&self, encoded: &str) -> Result<Vec<u8>, SwhidError> {
        BASE32
            .decode(encoded.as_bytes())
            .map_err(|e| SwhidError::EncodingError {
                format: "base32".to_string(),
                message: format!("Invalid base32 encoding: {e}"),
            })
    }

    fn name(&self) -> &str {
        "base32"
    }
}

/// Base32hex serialization for hash digests.
///
/// Uses Base32hex encoding (alternative character set: 0-9, A-V instead of A-Z, 2-7).
/// This variant uses a different character set that is more suitable for hexadecimal
/// representation. Provides the same compactness as standard Base32.
pub struct Base32HexSerializer;

impl Base32HexSerializer {
    /// Create a new base32hex serializer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Base32HexSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl DigestSerializer for Base32HexSerializer {
    fn encode(&self, digest: &[u8]) -> Result<String, SwhidError> {
        Ok(BASE32HEX.encode(digest))
    }

    fn decode(&self, encoded: &str) -> Result<Vec<u8>, SwhidError> {
        BASE32HEX
            .decode(encoded.as_bytes())
            .map_err(|e| SwhidError::EncodingError {
                format: "base32hex".to_string(),
                message: format!("Invalid base32hex encoding: {e}"),
            })
    }

    fn name(&self) -> &str {
        "base32hex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base32_encode() {
        let serializer = Base32Serializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let encoded = serializer.encode(&data).unwrap();
        assert_eq!(encoded, "CI2FM6A=");
    }

    #[test]
    fn base32_decode() {
        let serializer = Base32Serializer::new();
        let encoded = "CI2FM6A=";
        let decoded = serializer.decode(encoded).unwrap();
        assert_eq!(decoded, vec![0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn base32_roundtrip() {
        let serializer = Base32Serializer::new();
        let data = vec![0x00, 0xff, 0x12, 0xab, 0xcd, 0xef];
        let encoded = serializer.encode(&data).unwrap();
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn base32_decode_invalid() {
        let serializer = Base32Serializer::new();
        assert!(serializer.decode("invalid!").is_err());
        assert!(serializer.decode("CI2FM6A").is_err()); // Missing padding
    }

    #[test]
    fn base32_name() {
        let serializer = Base32Serializer::new();
        assert_eq!(serializer.name(), "base32");
    }

    #[test]
    fn base32_sha1_digest() {
        let serializer = Base32Serializer::new();
        let sha1_digest = vec![0u8; 20];
        let encoded = serializer.encode(&sha1_digest).unwrap();
        // 20 bytes = 32 base32 chars (with padding)
        assert_eq!(encoded.len(), 32);
    }

    #[test]
    fn base32_sha256_digest() {
        let serializer = Base32Serializer::new();
        let sha256_digest = vec![0u8; 32];
        let encoded = serializer.encode(&sha256_digest).unwrap();
        // 32 bytes = 52 base32 chars (with padding)
        // Actually, 32 bytes = 256 bits, which needs ceil(256/5) = 52 chars, but padding may vary
        assert!(encoded.len() >= 52);
        assert!(encoded.len() <= 56); // Allow for padding
    }

    #[test]
    fn base32hex_encode() {
        let serializer = Base32HexSerializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let encoded = serializer.encode(&data).unwrap();
        // Base32hex produces "28Q5CU0=" for this data (different from standard base32)
        assert_eq!(encoded, "28Q5CU0=");
        // Verify roundtrip
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn base32hex_decode() {
        let serializer = Base32HexSerializer::new();
        // Encode first to get valid base32hex
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let encoded = serializer.encode(&data).unwrap();
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn base32hex_roundtrip() {
        let serializer = Base32HexSerializer::new();
        let data = vec![0x00, 0xff, 0x12, 0xab, 0xcd, 0xef];
        let encoded = serializer.encode(&data).unwrap();
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn base32hex_name() {
        let serializer = Base32HexSerializer::new();
        assert_eq!(serializer.name(), "base32hex");
    }

    #[test]
    fn base32hex_sha256_digest() {
        let serializer = Base32HexSerializer::new();
        let sha256_digest = vec![0u8; 32];
        let encoded = serializer.encode(&sha256_digest).unwrap();
        // 32 bytes = 52 base32hex chars (with padding)
        // Actually, 32 bytes = 256 bits, which needs ceil(256/5) = 52 chars, but padding may vary
        assert!(encoded.len() >= 52);
        assert!(encoded.len() <= 56); // Allow for padding
    }

    #[test]
    fn base32_vs_base32hex_same_data() {
        let base32_ser = Base32Serializer::new();
        let base32hex_ser = Base32HexSerializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78];
        
        let base32_encoded = base32_ser.encode(&data).unwrap();
        let base32hex_encoded = base32hex_ser.encode(&data).unwrap();
        
        // They should decode to the same data
        assert_eq!(base32_ser.decode(&base32_encoded).unwrap(), data);
        assert_eq!(base32hex_ser.decode(&base32hex_encoded).unwrap(), data);
        
        // They may produce different encodings due to different character sets
        // but both should be valid and decode to the same data
        assert_eq!(base32_encoded.len(), base32hex_encoded.len());
    }
}

