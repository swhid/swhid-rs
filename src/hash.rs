use sha1collisiondetection::{Digest, Sha1CD};
use sha2::Sha256;

/// Pluggable hash function for SWHID digest computation.
pub trait HashFunction: Send + Sync {
    /// Compute digest of data.
    fn hash(&self, data: &[u8]) -> Vec<u8>;
    /// Digest length in bytes (e.g. 20 for SHA-1).
    fn digest_size(&self) -> usize;
    /// Algorithm name for diagnostics.
    fn name(&self) -> &'static str;
}

/// SHA-1 hash (SWHID v1 default).
#[derive(Debug, Clone, Copy, Default)]
pub struct Sha1Hash;

impl HashFunction for Sha1Hash {
    fn hash(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha1CD::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
    fn digest_size(&self) -> usize {
        20
    }
    fn name(&self) -> &'static str {
        "sha1"
    }
}

/// SHA-256 hash (SWHID v2).
#[derive(Debug, Clone, Copy, Default)]
pub struct Sha256Hash;

impl HashFunction for Sha256Hash {
    fn hash(&self, data: &[u8]) -> Vec<u8> {
        use sha2::Digest;
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
    fn digest_size(&self) -> usize {
        32
    }
    fn name(&self) -> &'static str {
        "sha256"
    }
}

/// Build SWHID v1.2 object header bytes: `<type> <len>\0`
///
/// This implements the object header format specified in SWHID v1.2,
/// which is compatible with Git's object format for content and directory objects.
pub fn swhid_object_header(typ: &str, len: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(typ.len() + 1 + 20 + 1 + len);
    v.extend_from_slice(typ.as_bytes());
    v.push(b' ');
    v.extend_from_slice(len.to_string().as_bytes());
    v.push(0);
    v
}

/// Internal: hash SWHID object with any hasher. Public API remains hash_content / hash_swhid_object.
pub(crate) fn hash_swhid_object_generic(
    typ: &str,
    payload: &[u8],
    hasher: &dyn HashFunction,
) -> Vec<u8> {
    let header = swhid_object_header(typ, payload.len());
    let mut input = header;
    input.extend_from_slice(payload);
    hasher.hash(&input)
}

/// Hash content data according to SWHID v1.2 specification.
///
/// This computes the SHA-1 digest of content data using the SWHID v1.2
/// object format, which is compatible with Git's blob format.
pub fn hash_content(data: &[u8]) -> [u8; 20] {
    let v = hash_swhid_object_generic("blob", data, &Sha1Hash);
    v.try_into().expect("SHA-1 digest is 20 bytes")
}

/// Hash arbitrary SWHID v1.2 object given its type and payload bytes.
///
/// This implements the SWHID v1.2 object hashing algorithm for any
/// object type (blob, tree, commit, tag, snapshot).
pub fn hash_swhid_object(typ: &str, payload: &[u8]) -> [u8; 20] {
    let v = hash_swhid_object_generic(typ, payload, &Sha1Hash);
    v.try_into().expect("SHA-1 digest is 20 bytes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_content_is_swhid_known_value() {
        let h = hash_content(&[]);
        // e69de29bb2d1d6434b8b29ae775ad8c2e48c5391
        assert_eq!(hex::encode(h), "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
    }

    #[test]
    fn hello_world_content() {
        let h = hash_content(b"Hello, World!");
        assert_eq!(hex::encode(h), "b45ef6fec89518d314f546fd6c3025367b721684");
    }

    #[test]
    fn swhid_object_header_format() {
        let header = swhid_object_header("blob", 0);
        assert_eq!(header, b"blob 0\0");

        let header = swhid_object_header("tree", 1234);
        assert_eq!(header, b"tree 1234\0");
    }

    #[test]
    fn hash_swhid_object_consistency() {
        let data = b"test data";
        let blob_hash = hash_swhid_object("blob", data);
        let direct_hash = hash_content(data);
        assert_eq!(blob_hash, direct_hash);
    }

    #[test]
    fn hash_different_object_types() {
        let data = b"same data";
        let blob_hash = hash_swhid_object("blob", data);
        let tree_hash = hash_swhid_object("tree", data);
        assert_ne!(blob_hash, tree_hash);
    }

    #[test]
    fn hash_empty_vs_non_empty() {
        let empty_hash = hash_content(&[]);
        let non_empty_hash = hash_content(b"x");
        assert_ne!(empty_hash, non_empty_hash);
    }

    #[test]
    fn hash_deterministic() {
        let data = b"deterministic test";
        let hash1 = hash_content(data);
        let hash2 = hash_content(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn hash_different_data() {
        let hash1 = hash_content(b"data1");
        let hash2 = hash_content(b"data2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn hash_large_data() {
        let large_data = vec![0u8; 10000];
        let hash = hash_content(&large_data);
        assert_eq!(hash.len(), 20);
    }

    #[test]
    fn hash_unicode_data() {
        let unicode_data = "Hello, 世界! 🌍".as_bytes();
        let hash = hash_content(unicode_data);
        assert_eq!(hash.len(), 20);
    }

    #[test]
    fn hash_newline_variations() {
        let unix_data = b"line1\nline2\n";
        let windows_data = b"line1\r\nline2\r\n";
        let mac_data = b"line1\rline2\r";

        let unix_hash = hash_content(unix_data);
        let windows_hash = hash_content(windows_data);
        let mac_hash = hash_content(mac_data);

        assert_ne!(unix_hash, windows_hash);
        assert_ne!(unix_hash, mac_hash);
        assert_ne!(windows_hash, mac_hash);
    }

    #[test]
    fn hash_binary_data() {
        let binary_data = vec![0x00, 0x01, 0xFF, 0xFE, 0x80, 0x7F];
        let hash = hash_content(&binary_data);
        assert_eq!(hash.len(), 20);
    }

    #[test]
    fn hash_known_swhid_objects() {
        // Test with known SWHID v1.2 object hashes
        let empty_tree = hash_swhid_object("tree", &[]);
        let empty_commit = hash_swhid_object("commit", &[]);
        let empty_tag = hash_swhid_object("tag", &[]);

        assert_ne!(empty_tree, empty_commit);
        assert_ne!(empty_tree, empty_tag);
        assert_ne!(empty_commit, empty_tag);
    }

    #[test]
    fn hash_object_header_edge_cases() {
        let header_zero = swhid_object_header("blob", 0);
        assert_eq!(header_zero, b"blob 0\0");

        let header_large = swhid_object_header("tree", 999999);
        assert_eq!(header_large, b"tree 999999\0");
    }

    #[test]
    fn hash_consistency_across_calls() {
        let data = b"consistency test data";
        let mut hashes = Vec::new();

        for _ in 0..10 {
            hashes.push(hash_content(data));
        }

        // All hashes should be identical
        for i in 1..hashes.len() {
            assert_eq!(hashes[0], hashes[i]);
        }
    }
}
