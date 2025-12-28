use swhid::snapshot::*;

fn name(s: &'static str) -> Box<[u8]> {
    s.as_bytes().into()
}

#[test]
fn simple_snp_hash() {
    let snp = Snapshot::new(vec![
        Branch::new(
            name("refs/heads/develop"),
            BranchTarget::Revision(Some([2; 20])),
        ),
        Branch::new(
            name("refs/heads/main"),
            BranchTarget::Revision(Some([1; 20])),
        ),
    ])
    .unwrap();

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    assert_eq!(
        snp_manifest(snp.branches().into()).unwrap(),
        b"\
        revision refs/heads/develop\020:\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\
        revision refs/heads/main\020:\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\
        "
    );

    // ditto
    assert_eq!(
        snp.swhid().to_string(),
        "swh:1:snp:870148a17e00ea8bd84b727cd26104b8c6ac6a72"
    );
}

#[test]
fn snp_order() {
    let snp = Snapshot::new(vec![
        Branch::new(
            name("refs/heads/main"),
            BranchTarget::Revision(Some([1; 20])),
        ),
        Branch::new(
            name("refs/heads/develop"),
            BranchTarget::Revision(Some([2; 20])),
        ),
    ])
    .unwrap();

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    assert_eq!(
        snp_manifest(snp.branches().into()).unwrap(),
        b"\
        revision refs/heads/develop\020:\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\
        revision refs/heads/main\020:\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\
        "
    );

    // ditto
    assert_eq!(
        snp.swhid().to_string(),
        "swh:1:snp:870148a17e00ea8bd84b727cd26104b8c6ac6a72"
    );
}

#[test]
fn empty_snp_hash() {
    let snp = Snapshot::new(vec![]).unwrap();

    assert_eq!(snp_manifest(snp.branches().into()).unwrap(), b"");

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    assert_eq!(
        snp.swhid().to_string(),
        "swh:1:snp:1a8893e6a86f444e8be8e7bda6cb34fb1735a00e"
    );
}

#[test]
fn snp_with_alias() {
    let snp = Snapshot::new(vec![
        Branch::new(
            name("refs/heads/main"),
            BranchTarget::Revision(Some([1; 20])),
        ),
        Branch::new(
            name("refs/heads/develop"),
            BranchTarget::Revision(Some([2; 20])),
        ),
        Branch::new(
            name("HEAD"),
            BranchTarget::Alias(Some(name("refs/heads/main"))),
        ),
    ])
    .unwrap();

    assert_eq!(
        snp_manifest(snp.branches().into()).unwrap(),
        b"\
        alias HEAD\x0015:refs/heads/main\
        revision refs/heads/develop\x0020:\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\
        revision refs/heads/main\x0020:\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\
        "
    );

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    assert_eq!(
        snp.swhid().to_string(),
        "swh:1:snp:9ecd7950d10ed3d02bfcf9c4a534f173697ab9f3"
    );
}

#[test]
fn snapshot_swhid_v2_all_serializers() {
    use swhid::config::HashConfig;
    let snp = Snapshot::new(vec![
        Branch::new(
            name("refs/heads/develop"),
            BranchTarget::Revision(Some([2; 20])),
        ),
        Branch::new(
            name("refs/heads/main"),
            BranchTarget::Revision(Some([1; 20])),
        ),
    ])
    .unwrap();

    // Test all v2 serialization formats
    let hex_config = HashConfig::v2_sha256_hex();
    let base64_config = HashConfig::v2_sha256_base64();
    let base64url_config = HashConfig::v2_sha256_base64url();
    let base32_config = HashConfig::v2_sha256_base32();
    let base32hex_config = HashConfig::v2_sha256_base32hex();
    let z85_config = HashConfig::v2_sha256_z85();

    let hex_swhid = snp.swhid_with_config(&hex_config);
    let base64_swhid = snp.swhid_with_config(&base64_config);
    let base64url_swhid = snp.swhid_with_config(&base64url_config);
    let base32_swhid = snp.swhid_with_config(&base32_config);
    let base32hex_swhid = snp.swhid_with_config(&base32hex_config);
    let z85_swhid = snp.swhid_with_config(&z85_config);

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
fn snapshot_swhid_v1_backward_compatibility() {
    let snp = Snapshot::new(vec![
        Branch::new(
            name("refs/heads/main"),
            BranchTarget::Revision(Some([1; 20])),
        ),
    ])
    .unwrap();

    // V1 should still work
    let v1_swhid = snp.swhid();
    assert_eq!(v1_swhid.version(), "1");
    assert_eq!(v1_swhid.digest_bytes().len(), 20);

    // V1 and V2 should produce different digests (different hash functions)
    use swhid::config::HashConfig;
    let v2_swhid = snp.swhid_with_config(&HashConfig::v2_sha256_hex());
    assert_ne!(v1_swhid.digest_bytes(), v2_swhid.digest_bytes());
}
