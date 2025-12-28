use swhid::release::*;

fn bs(s: &'static str) -> Box<[u8]> {
    s.as_bytes().into()
}

#[test]
fn simple_rel_hash() {
    let tree_hash = hex::decode("0efb37b28c53c7e4fbd253bb04a4df14008f63fe")
        .unwrap()
        .try_into()
        .unwrap();

    let rel = Release {
        object: tree_hash,
        object_type: ReleaseTargetType::Directory,
        name: bs("v1.0"),
        author: Some(bs("Test User <test@example.com>")),
        author_timestamp: Some(1763027354),
        author_timestamp_offset: Some(bs("+0100")),
        extra_headers: Vec::new(),
        message: Some(bs("Test tag")),
    };

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    // using this script:
    // >>> from swh.model.model import *
    // >>> from swh.model.git_objects import *
    // >>> person = Person.from_fullname(b"Test User <test@example.com>")
    // >>> ts = TimestampWithTimezone(timestamp=Timestamp(seconds=1763027354, microseconds=0), offset_bytes=b"+0100")
    // >>> rel = Release(name=b"v1.0", target_type=ObjectType.DIRECTORY, target=bytes.fromhex("0efb37b28c53c7e4fbd253bb04a4df14008f63fe"), message=b"Test tag", author=person, date=ts, synthetic=False)
    // >>> release_git_object(rel)
    assert_eq!(
        rel_manifest(&rel),
        b"\
        object 0efb37b28c53c7e4fbd253bb04a4df14008f63fe\n\
        type tree\n\
        tag v1.0\n\
        tagger Test User <test@example.com> 1763027354 +0100\n\
        \n\
        Test tag\
        "
    );

    // ditto
    assert_eq!(
        rel.swhid().to_string(),
        "swh:1:rel:46d326edb8bfc49b757ccd09930365595806bfc0",
    );
}

#[test]
fn release_swhid_v2_all_serializers() {
    use swhid::config::HashConfig;
    let tree_hash = hex::decode("0efb37b28c53c7e4fbd253bb04a4df14008f63fe")
        .unwrap()
        .try_into()
        .unwrap();

    let rel = Release {
        object: tree_hash,
        object_type: ReleaseTargetType::Directory,
        name: bs("v1.0"),
        author: Some(bs("Test User <test@example.com>")),
        author_timestamp: Some(1763027354),
        author_timestamp_offset: Some(bs("+0100")),
        extra_headers: Vec::new(),
        message: Some(bs("Test tag")),
    };

    // Test all v2 serialization formats
    let hex_config = HashConfig::v2_sha256_hex();
    let base64_config = HashConfig::v2_sha256_base64();
    let base64url_config = HashConfig::v2_sha256_base64url();
    let base32_config = HashConfig::v2_sha256_base32();
    let base32hex_config = HashConfig::v2_sha256_base32hex();
    let z85_config = HashConfig::v2_sha256_z85();

    let hex_swhid = rel.swhid_with_config(&hex_config);
    let base64_swhid = rel.swhid_with_config(&base64_config);
    let base64url_swhid = rel.swhid_with_config(&base64url_config);
    let base32_swhid = rel.swhid_with_config(&base32_config);
    let base32hex_swhid = rel.swhid_with_config(&base32hex_config);
    let z85_swhid = rel.swhid_with_config(&z85_config);

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
fn release_swhid_v1_backward_compatibility() {
    let tree_hash = hex::decode("0efb37b28c53c7e4fbd253bb04a4df14008f63fe")
        .unwrap()
        .try_into()
        .unwrap();

    let rel = Release {
        object: tree_hash,
        object_type: ReleaseTargetType::Directory,
        name: bs("v1.0"),
        author: Some(bs("Test User <test@example.com>")),
        author_timestamp: Some(1763027354),
        author_timestamp_offset: Some(bs("+0100")),
        extra_headers: Vec::new(),
        message: Some(bs("Test tag")),
    };

    // V1 should still work
    let v1_swhid = rel.swhid();
    assert_eq!(v1_swhid.version(), "1");
    assert_eq!(v1_swhid.digest_bytes().len(), 20);

    // V1 and V2 should produce different digests (different hash functions)
    use swhid::config::HashConfig;
    let v2_swhid = rel.swhid_with_config(&HashConfig::v2_sha256_hex());
    assert_ne!(v1_swhid.digest_bytes(), v2_swhid.digest_bytes());
}
