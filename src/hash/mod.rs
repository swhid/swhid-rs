//! Per-hash fast paths for SWHID object hashing (no generic buffer, fixed output).

use crate::Digest;

/// Build SWHID v1.2 object header: `<type> <len>\0`
pub fn swhid_object_header(typ: &str, len: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(typ.len() + 1 + 20 + 1);
    v.extend_from_slice(typ.as_bytes());
    v.push(b' ');
    v.extend_from_slice(len.to_string().as_bytes());
    v.push(0);
    v
}

/// Hash function with fixed-size output (no heap, no dyn).
pub trait HashFunction: Send + Sync {
    /// Fixed-size digest; must implement `Into<Digest>` for the enabled variant.
    type Output: Into<Digest> + AsRef<[u8]> + Copy + Send + Sync;
    /// Hash SWHID object (header + payload) with two updates; no combined buffer.
    fn hash_object(&self, typ: &str, payload: &[u8]) -> Self::Output;
}

#[cfg(feature = "sha1")]
pub use sha1::{
    hash_content_sha1 as hash_content, hash_swhid_object_sha1 as hash_swhid_object, Sha1Hash,
};
#[cfg(feature = "sha256")]
pub use sha256::Sha256Hash;
#[cfg(feature = "sha512")]
pub use sha512::Sha512Hash;
#[cfg(feature = "blake3")]
pub use blake3::Blake3Hash;

#[cfg(feature = "sha1")]
mod sha1;
#[cfg(feature = "sha256")]
mod sha256;
#[cfg(feature = "sha512")]
mod sha512;
#[cfg(feature = "blake3")]
mod blake3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swhid_object_header_format() {
        let header = swhid_object_header("blob", 0);
        assert_eq!(header, b"blob 0\0");
        let header = swhid_object_header("tree", 1234);
        assert_eq!(header, b"tree 1234\0");
    }

    #[cfg(feature = "sha1")]
    #[test]
    fn empty_content_is_swhid_known_value() {
        let h = hash_content(&[]);
        assert_eq!(hex::encode(h), "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
    }

    #[cfg(feature = "sha1")]
    #[test]
    fn hello_world_content() {
        let h = hash_content(b"Hello, World!");
        assert_eq!(hex::encode(h), "b45ef6fec89518d314f546fd6c3025367b721684");
    }

    #[cfg(feature = "sha1")]
    #[test]
    fn hash_swhid_object_consistency() {
        let data = b"test data";
        assert_eq!(hash_swhid_object("blob", data), hash_content(data));
    }

    #[cfg(feature = "sha1")]
    #[test]
    fn hash_different_object_types() {
        let data = b"same data";
        assert_ne!(
            hash_swhid_object("blob", data),
            hash_swhid_object("tree", data)
        );
    }

    #[cfg(feature = "sha1")]
    #[test]
    fn hash_deterministic() {
        let data = b"deterministic test";
        assert_eq!(hash_content(data), hash_content(data));
    }

    #[cfg(feature = "sha1")]
    #[test]
    fn hash_known_swhid_objects() {
        let empty_tree = hash_swhid_object("tree", &[]);
        let empty_commit = hash_swhid_object("commit", &[]);
        assert_ne!(empty_tree, empty_commit);
    }

    #[cfg(all(feature = "blake3", feature = "encoding-hex"))]
    #[test]
    fn blake3_empty_content_known_value() {
        use crate::{Content, HashConfig};
        let config = HashConfig::blake3_hex();
        let content = Content::from_bytes(&[]);
        let swhid = content.swhid_with_config(&config);
        let s = swhid.to_string_encoded(&config.encoder);
        assert!(s.starts_with("swh:2:cnt:"));
        let digest_part = s.strip_prefix("swh:2:cnt:").unwrap();
        assert_eq!(digest_part.len(), 64, "BLAKE3 hex digest should be 64 chars");
    }
}
