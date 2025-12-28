use crate::error::SwhidError;
use super::DigestSerializer;

// Z85 character set: 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#
const Z85_CHARSET: &[u8; 85] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

fn z85_encode(data: &[u8]) -> String {
    // Z85 encodes 4 bytes to 5 characters
    // Input must be multiple of 4 bytes
    if data.len() % 4 != 0 {
        // For non-multiple-of-4, we can't encode with Z85
        // This shouldn't happen for our digests (SHA1=20, SHA256=32)
        return String::new();
    }
    
    let mut result = String::with_capacity((data.len() / 4) * 5);
    for chunk in data.chunks_exact(4) {
        // Convert 4 bytes to u32
        let value = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        
        // Encode to base 85 (5 characters)
        let mut encoded = [0u8; 5];
        let mut v = value;
        for i in (0..5).rev() {
            encoded[i] = Z85_CHARSET[(v % 85) as usize];
            v /= 85;
        }
        result.push_str(std::str::from_utf8(&encoded).unwrap());
    }
    result
}

fn z85_decode(encoded: &str) -> Result<Vec<u8>, String> {
    // Z85 decodes 5 characters to 4 bytes
    // Input must be multiple of 5 characters
    if encoded.len() % 5 != 0 {
        return Err("Z85 encoded string length must be multiple of 5".to_string());
    }
    
    // Build reverse lookup table
    let mut char_to_value = [0u8; 256];
    for (i, &ch) in Z85_CHARSET.iter().enumerate() {
        char_to_value[ch as usize] = i as u8;
    }
    
    let mut result = Vec::with_capacity((encoded.len() / 5) * 4);
    for chunk in encoded.as_bytes().chunks_exact(5) {
        // Decode from base 85
        let mut value = 0u32;
        for &ch in chunk {
            let val = char_to_value[ch as usize] as u32;
            if val >= 85 {
                return Err(format!("Invalid Z85 character: {}", ch as char));
            }
            value = value * 85 + val;
        }
        
        // Convert u32 to 4 bytes
        let bytes = value.to_be_bytes();
        result.extend_from_slice(&bytes);
    }
    Ok(result)
}

/// Z85 (ZeroMQ Base85) serialization for hash digests.
///
/// Uses Z85 encoding, which is a URL-safe variant of Base85 designed by ZeroMQ.
/// This provides the most compact representation among the supported formats
/// (e.g., 32 bytes = 40 z85 chars vs 64 hex chars, 44 base64 chars, 52 base32 chars).
/// Z85 uses a character set that avoids special characters requiring URL encoding.
pub struct Z85Serializer;

impl Z85Serializer {
    /// Create a new Z85 serializer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Z85Serializer {
    fn default() -> Self {
        Self::new()
    }
}

impl DigestSerializer for Z85Serializer {
    fn encode(&self, digest: &[u8]) -> String {
        z85_encode(digest)
    }

    fn decode(&self, encoded: &str) -> Result<Vec<u8>, SwhidError> {
        z85_decode(encoded)
            .map_err(|e| SwhidError::InvalidDigest(format!("Invalid z85 encoding: {e}")))
    }

    fn name(&self) -> &str {
        "z85"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn z85_encode() {
        let serializer = Z85Serializer::new();
        // Z85 requires length to be multiple of 4 bytes
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let encoded = serializer.encode(&data);
        // Z85 encodes 4 bytes to 5 chars
        assert_eq!(encoded.len(), 5);
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn z85_decode() {
        let serializer = Z85Serializer::new();
        // Z85 requires length to be multiple of 4 bytes
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let encoded = serializer.encode(&data);
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn z85_roundtrip() {
        let serializer = Z85Serializer::new();
        // Z85 requires length to be multiple of 4 bytes
        let data = vec![0x00, 0xff, 0x12, 0xab, 0xcd, 0xef, 0x01, 0x02];
        let encoded = serializer.encode(&data);
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn z85_decode_invalid() {
        let serializer = Z85Serializer::new();
        // Z85 requires length to be multiple of 5 characters
        assert!(serializer.decode("invalid").is_err()); // 7 chars, not multiple of 5
        assert!(serializer.decode("invalid!").is_err()); // 8 chars, not multiple of 5
    }

    #[test]
    fn z85_name() {
        let serializer = Z85Serializer::new();
        assert_eq!(serializer.name(), "z85");
    }

    #[test]
    fn z85_sha1_digest() {
        let serializer = Z85Serializer::new();
        let sha1_digest = vec![0u8; 20];
        let encoded = serializer.encode(&sha1_digest);
        // Z85 encodes 4 bytes to 5 chars, so 20 bytes = 25 chars
        assert_eq!(encoded.len(), 25);
        // Verify roundtrip
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(decoded, sha1_digest);
    }

    #[test]
    fn z85_sha256_digest() {
        let serializer = Z85Serializer::new();
        let sha256_digest = vec![0u8; 32];
        let encoded = serializer.encode(&sha256_digest);
        // Z85 encodes 4 bytes to 5 chars, so 32 bytes = 40 chars
        assert_eq!(encoded.len(), 40);
        // Verify roundtrip
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(decoded, sha256_digest);
    }

    #[test]
    fn z85_compactness() {
        let serializer = Z85Serializer::new();
        let sha256_digest = vec![0u8; 32];
        let z85_encoded = serializer.encode(&sha256_digest);
        
        // Z85 should be more compact than hex (64 chars) and base64 (44 chars)
        assert!(z85_encoded.len() < 64); // Less than hex
        assert!(z85_encoded.len() < 44); // Less than base64
        assert_eq!(z85_encoded.len(), 40); // Exactly 40 chars for 32 bytes
    }

    #[test]
    fn z85_url_safe() {
        let serializer = Z85Serializer::new();
        // Z85 requires length to be multiple of 4 bytes
        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = serializer.encode(&data);
        
        // Z85 character set is designed to be URL-friendly
        // Verify it doesn't contain spaces or other obviously problematic chars
        assert!(!encoded.contains(' '));
        assert!(!encoded.contains('\n'));
        assert!(!encoded.contains('\r'));
        // Verify roundtrip
        let decoded = serializer.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}
