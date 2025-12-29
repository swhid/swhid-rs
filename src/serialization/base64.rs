use crate::error::SwhidError;
use super::DigestSerializer;
use base64::Engine;

/// Base64 standard serialization for hash digests.
///
/// Uses standard Base64 encoding with padding. This provides a more compact
/// representation than hex (e.g., 32 bytes = 44 base64 chars vs 64 hex chars).
pub struct Base64Serializer;

impl Base64Serializer {
    /// Create a new base64 serializer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Base64Serializer {
    fn default() -> Self {
        Self::new()
    }
}

impl DigestSerializer for Base64Serializer {
    fn encode(&self, digest: &[u8]) -> Result<String, SwhidError> {
        Ok(base64::engine::general_purpose::STANDARD.encode(digest))
    }

    fn decode(&self, encoded: &str) -> Result<Vec<u8>, SwhidError> {
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| SwhidError::EncodingError {
                format: "base64".to_string(),
                message: format!("Invalid base64 encoding: {e}"),
            })
    }

    fn name(&self) -> &str {
        "base64"
    }
}

/// Base64 URL-safe serialization for hash digests.
///
/// Uses URL-safe Base64 encoding without padding. This is suitable for use
/// in URLs and provides a compact representation.
pub struct Base64UrlSerializer;

impl Base64UrlSerializer {
    /// Create a new base64url serializer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Base64UrlSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl DigestSerializer for Base64UrlSerializer {
    fn encode(&self, digest: &[u8]) -> Result<String, SwhidError> {
        Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest))
    }

    fn decode(&self, encoded: &str) -> Result<Vec<u8>, SwhidError> {
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|e| SwhidError::EncodingError {
                format: "base64url".to_string(),
                message: format!("Invalid base64url encoding: {e}"),
            })
    }

    fn name(&self) -> &str {
        "base64url"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encode() {
        let serializer = Base64Serializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let encoded = serializer.encode(&data).unwrap();
        assert_eq!(encoded, "EjRWeA==");
    }

    #[test]
    fn base64_decode() {
        let serializer = Base64Serializer::new();
        let encoded = "EjRWeA==";
        let decoded = serializer.decode(encoded).unwrap();
        assert_eq!(decoded, vec![0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn base64_roundtrip() {
        let serializer = Base64Serializer::new();
        let data = vec![0x00, 0xff, 0x12, 0xab, 0xcd, 0xef];
        let encoded = serializer.encode(&data).unwrap();
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn base64_decode_invalid() {
        let serializer = Base64Serializer::new();
        assert!(serializer.decode("invalid!").is_err());
    }

    #[test]
    fn base64_name() {
        let serializer = Base64Serializer::new();
        assert_eq!(serializer.name(), "base64");
    }

    #[test]
    fn base64_sha256_digest() {
        let serializer = Base64Serializer::new();
        let sha256_digest = vec![0u8; 32];
        let encoded = serializer.encode(&sha256_digest).unwrap();
        // 32 bytes = 44 base64 chars (with padding)
        assert_eq!(encoded.len(), 44);
    }

    #[test]
    fn base64url_encode() {
        let serializer = Base64UrlSerializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let encoded = serializer.encode(&data).unwrap();
        assert_eq!(encoded, "EjRWeA");
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
        assert!(!encoded.contains('='));
    }

    #[test]
    fn base64url_decode() {
        let serializer = Base64UrlSerializer::new();
        let encoded = "EjRWeA";
        let decoded = serializer.decode(encoded).unwrap();
        assert_eq!(decoded, vec![0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn base64url_roundtrip() {
        let serializer = Base64UrlSerializer::new();
        let data = vec![0x00, 0xff, 0x12, 0xab, 0xcd, 0xef];
        let encoded = serializer.encode(&data).unwrap();
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn base64url_name() {
        let serializer = Base64UrlSerializer::new();
        assert_eq!(serializer.name(), "base64url");
    }

    #[test]
    fn base64url_sha256_digest() {
        let serializer = Base64UrlSerializer::new();
        let sha256_digest = vec![0u8; 32];
        let encoded = serializer.encode(&sha256_digest).unwrap();
        // 32 bytes = 43 base64url chars (no padding)
        assert_eq!(encoded.len(), 43);
    }

    #[test]
    fn base64_vs_base64url_different() {
        let base64_ser = Base64Serializer::new();
        let base64url_ser = Base64UrlSerializer::new();
        let data = vec![0x12, 0x34, 0x56, 0x78];
        
        let base64_encoded = base64_ser.encode(&data).unwrap();
        let base64url_encoded = base64url_ser.encode(&data).unwrap();
        
        // They should decode to the same data
        assert_eq!(base64_ser.decode(&base64_encoded).unwrap(), data);
        assert_eq!(base64url_ser.decode(&base64url_encoded).unwrap(), data);
        
        // But the encodings may differ (padding, character set)
        // base64 has padding, base64url doesn't
        assert!(base64_encoded.contains('=') || !base64url_encoded.contains('='));
    }
}

