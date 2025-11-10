use crate::core::{ObjectType, Swhid};
use crate::hash::hash_content;

/// SWHID v1.2 content object for computing content SWHIDs.
///
/// This struct represents file content data and provides methods to compute
/// SWHID v1.2 compliant content identifiers according to the specification.
#[derive(Debug, Clone)]
pub struct Content<'a> {
    bytes: Cow<'a, [u8]>,
}

use std::borrow::Cow;

impl<'a> Content<'a> {
    /// Create a new Content object from byte data.
    ///
    /// This implements SWHID v1.2 content object creation for any byte data.
    pub fn from_bytes(bytes: impl Into<Cow<'a, [u8]>>) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }

    #[cfg(feature = "serde")]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Compute the SWHID v1.2 content identifier for this content.
    ///
    /// This implements the SWHID v1.2 content hashing algorithm, which
    /// is compatible with Git's blob format for content objects.
    pub fn swhid(&self) -> Swhid {
        let digest = hash_content(&self.bytes);
        Swhid::new(ObjectType::Content, digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_from_bytes() {
        let data = b"test content";
        let content = Content::from_bytes(data);
        assert_eq!(content.len(), 12); // "test content" is 12 bytes
        assert!(!content.is_empty());
    }

    #[test]
    fn content_from_vec() {
        let data = vec![1, 2, 3, 4, 5];
        let content = Content::from_bytes(data);
        assert_eq!(content.len(), 5);
    }

    #[test]
    fn content_from_slice() {
        let data = &[1, 2, 3, 4, 5];
        let content = Content::from_bytes(data);
        assert_eq!(content.len(), 5);
    }

    #[test]
    fn content_empty() {
        let content = Content::from_bytes(&[]);
        assert_eq!(content.len(), 0);
        assert!(content.is_empty());
    }

    #[test]
    fn content_swhid_consistency() {
        let data = b"consistent test";
        let content1 = Content::from_bytes(data);
        let content2 = Content::from_bytes(data);
        assert_eq!(content1.swhid(), content2.swhid());
    }

    #[test]
    fn content_swhid_different_data() {
        let content1 = Content::from_bytes(b"data1");
        let content2 = Content::from_bytes(b"data2");
        assert_ne!(content1.swhid(), content2.swhid());
    }

    #[test]
    fn content_swhid_empty() {
        let content = Content::from_bytes(&[]);
        let swhid = content.swhid();
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(
            swhid.to_string(),
            "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
    }

    #[test]
    fn content_swhid_hello_world() {
        let content = Content::from_bytes(b"Hello, World!");
        let swhid = content.swhid();
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(
            swhid.to_string(),
            "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
        );
    }

    #[test]
    fn content_unicode() {
        let unicode_data = "Hello, 世界! 🌍";
        let content = Content::from_bytes(unicode_data.as_bytes());
        let swhid = content.swhid();
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(swhid.digest_bytes().len(), 20);
    }

    #[test]
    fn content_large_data() {
        let large_data = vec![0u8; 10000];
        let content = Content::from_bytes(large_data);
        let swhid = content.swhid();
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(swhid.digest_bytes().len(), 20);
    }

    #[test]
    fn content_binary_data() {
        let binary_data = vec![0x00, 0x01, 0xFF, 0xFE, 0x80, 0x7F];
        let content = Content::from_bytes(binary_data);
        let swhid = content.swhid();
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(swhid.digest_bytes().len(), 20);
    }

    #[test]
    fn content_newline_variations() {
        let unix_content = Content::from_bytes(b"line1\nline2\n");
        let windows_content = Content::from_bytes(b"line1\r\nline2\r\n");
        let mac_content = Content::from_bytes(b"line1\rline2\r");

        assert_ne!(unix_content.swhid(), windows_content.swhid());
        assert_ne!(unix_content.swhid(), mac_content.swhid());
        assert_ne!(windows_content.swhid(), mac_content.swhid());
    }

    #[test]
    fn content_cow_borrowed() {
        let data = b"borrowed data";
        let content = Content::from_bytes(data);
        assert_eq!(content.len(), 13);
    }

    #[test]
    fn content_cow_owned() {
        let data = vec![1, 2, 3, 4, 5];
        let content = Content::from_bytes(data);
        assert_eq!(content.len(), 5);
    }

    #[test]
    fn content_swhid_roundtrip() {
        let data = b"roundtrip test";
        let content = Content::from_bytes(data);
        let swhid = content.swhid();
        let swhid_str = swhid.to_string();
        let parsed: Swhid = swhid_str.parse().unwrap();
        assert_eq!(swhid, parsed);
    }

    #[test]
    fn content_swhid_format() {
        let content = Content::from_bytes(b"test");
        let swhid = content.swhid();
        let swhid_str = swhid.to_string();
        assert!(swhid_str.starts_with("swh:1:cnt:"));
        assert_eq!(swhid_str.len(), "swh:1:cnt:".len() + 40);
    }

    #[test]
    fn content_swhid_digest_hex() {
        let content = Content::from_bytes(b"test");
        let swhid = content.swhid();
        let hex = swhid.digest_hex();
        assert_eq!(hex.len(), 40);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn content_swhid_digest_bytes() {
        let content = Content::from_bytes(b"test");
        let swhid = content.swhid();
        let bytes = swhid.digest_bytes();
        assert_eq!(bytes.len(), 20);
    }

    #[test]
    fn content_swhid_object_type() {
        let content = Content::from_bytes(b"test");
        let swhid = content.swhid();
        assert_eq!(swhid.object_type(), ObjectType::Content);
    }

    #[test]
    fn content_swhid_version() {
        let _content = Content::from_bytes(b"test");
        assert_eq!(Swhid::VERSION, "1");
    }

    #[test]
    fn content_swhid_equality() {
        let data = b"equality test";
        let content1 = Content::from_bytes(data);
        let content2 = Content::from_bytes(data);
        assert_eq!(content1.swhid(), content2.swhid());
    }

    #[test]
    fn content_swhid_hash_consistency() {
        let data = b"hash consistency test";
        let content = Content::from_bytes(data);
        let swhid1 = content.swhid();
        let swhid2 = content.swhid();
        assert_eq!(swhid1, swhid2);
    }
}
