use sha2::{Digest, Sha256};
use super::hash_function::HashFunction;

/// SHA256 hash function implementation.
///
/// This implements the HashFunction trait for SHA256, which is used
/// in SWHID v2 for enhanced security and compatibility with Git SHA256
/// repositories.
pub struct Sha256Hash;

impl Sha256Hash {
    /// Create a new SHA256 hash function instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Sha256Hash {
    fn default() -> Self {
        Self::new()
    }
}

impl HashFunction for Sha256Hash {
    fn hash(&self, data: &[u8]) -> Box<[u8]> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec().into_boxed_slice()
    }

    fn digest_size(&self) -> usize {
        32
    }

    fn name(&self) -> &str {
        "sha256"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_digest_size() {
        let hasher = Sha256Hash::new();
        assert_eq!(hasher.digest_size(), 32);
    }

    #[test]
    fn sha256_name() {
        let hasher = Sha256Hash::new();
        assert_eq!(hasher.name(), "sha256");
    }

    #[test]
    fn sha256_empty_input() {
        let hasher = Sha256Hash::new();
        let digest = hasher.hash(&[]);
        assert_eq!(digest.len(), 32);
        // Known SHA256 of empty input: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(hex::encode(&digest), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn sha256_known_value() {
        let hasher = Sha256Hash::new();
        let digest = hasher.hash(b"Hello, World!");
        assert_eq!(digest.len(), 32);
        // Known SHA256 of "Hello, World!"
        assert_eq!(hex::encode(&digest), "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f");
    }

    #[test]
    fn sha256_deterministic() {
        let hasher = Sha256Hash::new();
        let data = b"test data";
        let hash1 = hasher.hash(data);
        let hash2 = hasher.hash(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn sha256_different_inputs() {
        let hasher = Sha256Hash::new();
        let hash1 = hasher.hash(b"data1");
        let hash2 = hasher.hash(b"data2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn sha256_vs_sha1_different_size() {
        use super::super::sha1::Sha1Hash;
        let sha1 = Sha1Hash::new();
        let sha256 = Sha256Hash::new();
        assert_ne!(sha1.digest_size(), sha256.digest_size());
    }
}

