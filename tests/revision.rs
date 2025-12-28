use swhid::revision::*;

fn bs(s: &'static str) -> Box<[u8]> {
    s.as_bytes().into()
}

#[test]
fn simple_rev_hash() {
    let tree_hash = hex::decode("0efb37b28c53c7e4fbd253bb04a4df14008f63fe")
        .unwrap()
        .try_into()
        .unwrap();

    let rev = Revision {
        directory: tree_hash,
        parents: Vec::new(),
        author: bs("Test User <test@example.com>"),
        author_timestamp: 1763027354,
        author_timestamp_offset: bs("+0100"),
        committer: bs("Test User <test@example.com>"),
        committer_timestamp: 1763027354,
        committer_timestamp_offset: bs("+0100"),
        extra_headers: Vec::new(),
        message: Some(bs("Test commit")),
    };

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    // using this script:
    // >>> from swh.model.model import *
    // >>> from swh.model.git_objects import *
    // >>> person = Person.from_fullname(b"Test User <test@example.com>")
    // >>> ts = TimestampWithTimezone(timestamp=Timestamp(seconds=1763027354, microseconds=0), offset_bytes=b"+0100")
    // >>> rev = Revision(directory=bytes.fromhex("0efb37b28c53c7e4fbd253bb04a4df14008f63fe"), message=b"Test commit", author=person, committer=person, date=ts, committer_date=ts, type=RevisionType.GIT, synthetic=False)
    // >>> revision_git_object(rev)
    assert_eq!(
        rev_manifest(&rev),
        b"\
        tree 0efb37b28c53c7e4fbd253bb04a4df14008f63fe\n\
        author Test User <test@example.com> 1763027354 +0100\n\
        committer Test User <test@example.com> 1763027354 +0100\n\
        \n\
        Test commit\
        "
    );

    // ditto
    assert_eq!(
        rev.swhid().to_string(),
        "swh:1:rev:07cde6575fb633ef9b5ecbe730e6eb97475a2fd9"
    );
}

#[test]
fn revision_swhid_v2_all_serializers() {
    use swhid::config::HashConfig;
    let tree_hash = hex::decode("0efb37b28c53c7e4fbd253bb04a4df14008f63fe")
        .unwrap()
        .try_into()
        .unwrap();

    let rev = Revision {
        directory: tree_hash,
        parents: Vec::new(),
        author: bs("Test User <test@example.com>"),
        author_timestamp: 1763027354,
        author_timestamp_offset: bs("+0100"),
        committer: bs("Test User <test@example.com>"),
        committer_timestamp: 1763027354,
        committer_timestamp_offset: bs("+0100"),
        extra_headers: Vec::new(),
        message: Some(bs("Test commit")),
    };

    // Test all v2 serialization formats
    let hex_config = HashConfig::v2_sha256_hex();
    let base64_config = HashConfig::v2_sha256_base64();
    let base64url_config = HashConfig::v2_sha256_base64url();
    let base32_config = HashConfig::v2_sha256_base32();
    let base32hex_config = HashConfig::v2_sha256_base32hex();
    let z85_config = HashConfig::v2_sha256_z85();

    let hex_swhid = rev.swhid_with_config(&hex_config);
    let base64_swhid = rev.swhid_with_config(&base64_config);
    let base64url_swhid = rev.swhid_with_config(&base64url_config);
    let base32_swhid = rev.swhid_with_config(&base32_config);
    let base32hex_swhid = rev.swhid_with_config(&base32hex_config);
    let z85_swhid = rev.swhid_with_config(&z85_config);

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
fn revision_swhid_v1_backward_compatibility() {
    let tree_hash = hex::decode("0efb37b28c53c7e4fbd253bb04a4df14008f63fe")
        .unwrap()
        .try_into()
        .unwrap();

    let rev = Revision {
        directory: tree_hash,
        parents: Vec::new(),
        author: bs("Test User <test@example.com>"),
        author_timestamp: 1763027354,
        author_timestamp_offset: bs("+0100"),
        committer: bs("Test User <test@example.com>"),
        committer_timestamp: 1763027354,
        committer_timestamp_offset: bs("+0100"),
        extra_headers: Vec::new(),
        message: Some(bs("Test commit")),
    };

    // V1 should still work
    let v1_swhid = rev.swhid();
    assert_eq!(v1_swhid.version(), "1");
    assert_eq!(v1_swhid.digest_bytes().len(), 20);

    // V1 and V2 should produce different digests (different hash functions)
    use swhid::config::HashConfig;
    let v2_swhid = rev.swhid_with_config(&HashConfig::v2_sha256_hex());
    assert_ne!(v1_swhid.digest_bytes(), v2_swhid.digest_bytes());
}
