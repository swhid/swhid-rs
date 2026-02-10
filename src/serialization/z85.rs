use crate::error::SwhidError;

use super::DigestSerializer;

// Z85 character set: 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#
const Z85_CHARSET: &[u8; 85] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

fn z85_encode(data: &[u8]) -> String {
    // Z85 encodes 4 bytes to 5 characters
    // Input must be multiple of 4 bytes
    // For digests (20 or 32 bytes), this is always true
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
#[derive(Debug, Clone, Copy, Default)]
pub struct Z85Serializer;

impl DigestSerializer for Z85Serializer {
    fn encode(&self, digest: &[u8]) -> String {
        z85_encode(digest)
    }

    fn decode(&self, s: &str) -> Result<Vec<u8>, SwhidError> {
        z85_decode(s).map_err(|e| SwhidError::InvalidDigest(format!("Invalid z85: {e}")))
    }
}
