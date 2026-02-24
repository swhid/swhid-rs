//! Z85 (ZeroMQ Base85) digest encoding.
//!
//! Z85 encodes 4 bytes to 5 characters. Digest lengths 20, 32, 64 are all
//! divisible by 4, so all supported hash outputs encode without padding.

const Z85_CHARSET: &[u8; 85] =
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

use crate::error::SwhidError;

use super::DigestSerializer;

#[derive(Debug, Clone, Copy, Default)]
pub struct Z85Serializer;

impl DigestSerializer for Z85Serializer {
    fn encode(&self, digest: &[u8]) -> String {
        assert!(
            digest.len() % 4 == 0,
            "Z85 requires length multiple of 4; digest lengths 20, 32, 64 are supported"
        );
        let mut result = String::with_capacity((digest.len() / 4) * 5);
        for chunk in digest.chunks_exact(4) {
            let value = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
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

    fn decode(&self, s: &str) -> Result<Vec<u8>, SwhidError> {
        if s.len() % 5 != 0 {
            return Err(SwhidError::InvalidDigest(format!(
                "z85: encoded length must be multiple of 5, got {}",
                s.len()
            )));
        }
        let mut char_to_value = [0u8; 256];
        for (i, &ch) in Z85_CHARSET.iter().enumerate() {
            char_to_value[ch as usize] = i as u8;
        }
        let mut result = Vec::with_capacity((s.len() / 5) * 4);
        for chunk in s.as_bytes().chunks_exact(5) {
            let mut value = 0u32;
            for &ch in chunk {
                let val = char_to_value[ch as usize] as u32;
                if val >= 85 {
                    return Err(SwhidError::InvalidDigest(format!(
                        "z85: invalid character '{}'",
                        ch as char
                    )));
                }
                value = value * 85 + val;
            }
            result.extend_from_slice(&value.to_be_bytes());
        }
        Ok(result)
    }
}
