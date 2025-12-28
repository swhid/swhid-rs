use sha1collisiondetection::{Digest, Sha1CD};
use super::hash_function::HashFunction;

/// SHA1 hash function implementation using collision-detecting SHA1.
///
/// This implements the HashFunction trait for SHA1, which is used
/// in SWHID v1. The implementation uses sha1collisiondetection for
/// enhanced security.
pub struct Sha1Hash;

impl Sha1Hash {
    /// Create a new SHA1 hash function instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Sha1Hash {
    fn default() -> Self {
        Self::new()
    }
}

impl HashFunction for Sha1Hash {
    fn hash(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha1CD::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    fn digest_size(&self) -> usize {
        20
    }

    fn name(&self) -> &str {
        "sha1"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_digest_size() {
        let hasher = Sha1Hash::new();
        assert_eq!(hasher.digest_size(), 20);
    }

    #[test]
    fn sha1_name() {
        let hasher = Sha1Hash::new();
        assert_eq!(hasher.name(), "sha1");
    }

    #[test]
    fn sha1_empty_input() {
        let hasher = Sha1Hash::new();
        let digest = hasher.hash(&[]);
        assert_eq!(digest.len(), 20);
        // Known SHA1 of empty input: da39a3ee5e6b4b0d3255bfef95601890afd80709
        assert_eq!(hex::encode(&digest), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[test]
    fn sha1_known_value() {
        let hasher = Sha1Hash::new();
        let digest = hasher.hash(b"Hello, World!");
        assert_eq!(digest.len(), 20);
        // Known SHA1 of "Hello, World!"
        assert_eq!(hex::encode(&digest), "0a0a9f2a6772942557ab5355d76af442f8f65e01");
    }

    #[test]
    fn sha1_deterministic() {
        let hasher = Sha1Hash::new();
        let data = b"test data";
        let hash1 = hasher.hash(data);
        let hash2 = hasher.hash(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn sha1_different_inputs() {
        let hasher = Sha1Hash::new();
        let hash1 = hasher.hash(b"data1");
        let hash2 = hasher.hash(b"data2");
        assert_ne!(hash1, hash2);
    }
}

