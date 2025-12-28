use swhid::content::*;
use swhid::{ObjectType, Swhid};

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

#[test]
fn content_swhid_v2_all_serializers() {
    use swhid::config::HashConfig;
    let content = Content::from_bytes(b"test data for v2");

    // Test all v2 serialization formats
    let hex_config = HashConfig::v2_sha256_hex();
    let base64_config = HashConfig::v2_sha256_base64();
    let base64url_config = HashConfig::v2_sha256_base64url();
    let base32_config = HashConfig::v2_sha256_base32();
    let base32hex_config = HashConfig::v2_sha256_base32hex();
    let z85_config = HashConfig::v2_sha256_z85();

    let hex_swhid = content.swhid_with_config(&hex_config);
    let base64_swhid = content.swhid_with_config(&base64_config);
    let base64url_swhid = content.swhid_with_config(&base64url_config);
    let base32_swhid = content.swhid_with_config(&base32_config);
    let base32hex_swhid = content.swhid_with_config(&base32hex_config);
    let z85_swhid = content.swhid_with_config(&z85_config);

    // All should have version 2
    assert_eq!(hex_swhid.version(), "2");
    assert_eq!(base64_swhid.version(), "2");
    assert_eq!(base64url_swhid.version(), "2");
    assert_eq!(base32_swhid.version(), "2");
    assert_eq!(base32hex_swhid.version(), "2");
    assert_eq!(z85_swhid.version(), "2");

    // All should have 32-byte digests (SHA256)
    assert_eq!(hex_swhid.digest_bytes().len(), 32);
    assert_eq!(base64_swhid.digest_bytes().len(), 32);
    assert_eq!(base64url_swhid.digest_bytes().len(), 32);
    assert_eq!(base32_swhid.digest_bytes().len(), 32);
    assert_eq!(base32hex_swhid.digest_bytes().len(), 32);
    assert_eq!(z85_swhid.digest_bytes().len(), 32);

    // All should produce the same digest bytes (same hash function)
    assert_eq!(hex_swhid.digest_bytes(), base64_swhid.digest_bytes());
    assert_eq!(hex_swhid.digest_bytes(), base64url_swhid.digest_bytes());
    assert_eq!(hex_swhid.digest_bytes(), base32_swhid.digest_bytes());
    assert_eq!(hex_swhid.digest_bytes(), base32hex_swhid.digest_bytes());
    assert_eq!(hex_swhid.digest_bytes(), z85_swhid.digest_bytes());
}

#[test]
fn content_swhid_v2_compactness() {
    use swhid::config::HashConfig;
    let content = Content::from_bytes(b"test data");
    let swhid = content.swhid_with_config(&HashConfig::v2_sha256_hex());
    let sha256_digest = swhid.digest_bytes().to_vec();

    // Encode the same digest with different serializers via HashConfig
    let hex_config = HashConfig::v2_sha256_hex();
    let base64_config = HashConfig::v2_sha256_base64();
    let base32_config = HashConfig::v2_sha256_base32();
    let z85_config = HashConfig::v2_sha256_z85();
    
    let hex_encoded = hex_config.serializer.encode(&sha256_digest);
    let base64_encoded = base64_config.serializer.encode(&sha256_digest);
    let base32_encoded = base32_config.serializer.encode(&sha256_digest);
    let z85_encoded = z85_config.serializer.encode(&sha256_digest);

    // Verify compactness ordering: z85 < base64 < base32 < hex
    // Note: Base32 may have padding, so we check ranges
    assert_eq!(z85_encoded.len(), 40); // Z85: 32 bytes = 40 chars (exact, no padding)
    assert!(base64_encoded.len() >= 43 && base64_encoded.len() <= 44); // Base64: 32 bytes = 43-44 chars
    assert!(base32_encoded.len() >= 52 && base32_encoded.len() <= 56); // Base32: 32 bytes = 52-56 chars (with padding)
    assert_eq!(hex_encoded.len(), 64); // Hex: 32 bytes = 64 chars (exact)
    
    // Verify strict ordering (allowing for padding)
    assert!(z85_encoded.len() < base64_encoded.len() || z85_encoded.len() == 40);
    assert!(base64_encoded.len() < base32_encoded.len() || base64_encoded.len() <= 44);
    assert!(base32_encoded.len() < hex_encoded.len());
}

#[test]
fn content_swhid_v1_vs_v2() {
    use swhid::config::HashConfig;
    let content = Content::from_bytes(b"Hello, World!");

    let v1_swhid = content.swhid();
    let v2_swhid = content.swhid_with_config(&HashConfig::v2_sha256_hex());

    // Different versions
    assert_eq!(v1_swhid.version(), "1");
    assert_eq!(v2_swhid.version(), "2");

    // Different digest lengths
    assert_eq!(v1_swhid.digest_bytes().len(), 20); // SHA1
    assert_eq!(v2_swhid.digest_bytes().len(), 32); // SHA256

    // Different digest values (different hash functions)
    assert_ne!(v1_swhid.digest_bytes(), v2_swhid.digest_bytes());
}
