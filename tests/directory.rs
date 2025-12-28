use assert_fs::prelude::*;

use swhid::directory::*;
use swhid::hash::hash_content;
use swhid::ObjectType;

fn name(s: &'static str) -> Box<[u8]> {
    s.as_bytes().into()
}

#[test]
fn simple_dir_hash() {
    let dir = Directory::new(vec![
        Entry::new(name("a.txt"), 0o100644, [1; 20]),
        Entry::new(name("b.txt"), 0o100755, [2; 20]),
        Entry::new(name("c.txt"), 0o100644, [0; 20]),
    ])
    .unwrap();

    assert_eq!(
        dir_manifest(dir.entries().into()).unwrap(),
        b"\
        100644 a.txt\0\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\
        100755 b.txt\0\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\
        100644 c.txt\0\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
        "
    );

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    assert_eq!(
        dir.swhid().unwrap().to_string(),
        "swh:1:dir:8863dfedee16d4f5eae8c796f57b90b165e5bd8d"
    );
}

#[test]
fn dir_order() {
    let dir = Directory::new(vec![
        Entry::new(name("a.txt"), 0o100644, [1; 20]),
        Entry::new(name("c.txt"), 0o100644, [0; 20]),
        Entry::new(name("b.txt"), 0o100755, [2; 20]),
    ])
    .unwrap();

    assert_eq!(
        dir_manifest(dir.entries().into()).unwrap(),
        b"\
        100644 a.txt\0\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\
        100755 b.txt\0\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\
        100644 c.txt\0\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
        "
    );

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    assert_eq!(
        dir.swhid().unwrap().to_string(),
        "swh:1:dir:8863dfedee16d4f5eae8c796f57b90b165e5bd8d"
    );
}

#[test]
fn empty_dir_hash() {
    let dir = Directory::new(vec![]).unwrap();

    assert_eq!(dir_manifest(dir.entries().into()).unwrap(), b"");

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    assert_eq!(
        dir.swhid().unwrap().to_string(),
        "swh:1:dir:4b825dc642cb6eb9a060e54bf8d69288fbee4904"
    );
}

#[test]
fn dir_with_symlinks() {
    let dir = Directory::new(vec![
        Entry::new(name("a.txt"), 0o100644, [1; 20]),
        Entry::new(name("b.txt"), 0o120000, [2; 20]),
    ])
    .unwrap();

    assert_eq!(
        dir_manifest(dir.entries().into()).unwrap(),
        b"\
        100644 a.txt\0\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\
        120000 b.txt\0\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\
        "
    );

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    assert_eq!(
        dir.swhid().unwrap().to_string(),
        "swh:1:dir:277f7807173d7053469ccbab70958b3bc9d5c9f6"
    );
}

#[test]
fn dir_with_subdir() {
    let dir = Directory::new(vec![
        Entry::new(name("a.txt"), 0o100644, [1; 20]),
        Entry::new(name("b"), 0o040000, [2; 20]),
    ])
    .unwrap();

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    assert_eq!(
        dir_manifest(dir.entries().into()).unwrap(),
        b"\
        100644 a.txt\x00\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\
        40000 b\x00\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\
        "
    );

    // ditto
    assert_eq!(
        dir.swhid().unwrap().to_string(),
        "swh:1:dir:c890b32febf94c3163b67778ae8b26bb631610a3",
    );
}

#[test]
fn read_empty_dir() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let dir = DiskDirectoryBuilder::new(tmp.path()).build().unwrap();

    assert_eq!(dir.entries(), vec![]);
}

#[test]
fn read_simple_dir() {
    let tmp = assert_fs::TempDir::new().unwrap();
    tmp.child("a.txt").write_str("A").unwrap();
    tmp.child("b.txt").write_str("B").unwrap();

    let dir = DiskDirectoryBuilder::new(tmp.path()).build().unwrap();

    let expected_dir = Directory::new(vec![
        Entry::new(name("a.txt"), 0o100644, hash_content(b"A")),
        Entry::new(name("b.txt"), 0o100644, hash_content(b"B")),
    ])
    .unwrap();

    assert_eq!(dir.entries(), expected_dir.entries());
}

#[test]
fn read_dir_with_unicode_filenames() {
    let tmp = assert_fs::TempDir::new().unwrap();
    tmp.child("文件.txt").write_str("unicode filename").unwrap();
    tmp.child("файл.txt")
        .write_str("cyrillic filename")
        .unwrap();

    let dir = DiskDirectoryBuilder::new(tmp.path()).build().unwrap();

    assert_eq!(
        dir.entries(),
        vec![
            Entry::new(
                name("файл.txt"),
                0o100644,
                hash_content(b"cyrillic filename"),
            ),
            Entry::new(
                name("文件.txt"),
                0o100644,
                hash_content(b"unicode filename"),
            ),
        ]
    );
}

#[test]
fn read_nested_dir_structure() {
    let tmp = assert_fs::TempDir::new().unwrap();
    tmp.child("file1.txt").write_str("content1").unwrap();
    tmp.child("subdir").create_dir_all().unwrap();
    tmp.child("subdir/file2.txt").write_str("content2").unwrap();
    tmp.child("subdir/file3.txt").write_str("content3").unwrap();

    let dir = DiskDirectoryBuilder::new(tmp.path());
    let id = dir.swhid().unwrap();
    assert_eq!(id.object_type(), ObjectType::Directory);
}

#[test]
fn read_dir_with_symlinks() {
    let tmp = assert_fs::TempDir::new().unwrap();
    tmp.child("target.txt").write_str("target content").unwrap();
    tmp.child("link.txt").symlink_to_file("target.txt").unwrap();

    let dir = DiskDirectoryBuilder::new(tmp.path()).build().unwrap();

    assert_eq!(
        dir.entries(),
        vec![
            Entry::new(name("link.txt"), 0o120000, hash_content(b"target.txt")),
            Entry::new(
                name("target.txt"),
                0o100644,
                hash_content(b"target content")
            ),
        ]
    );
}

#[test]
fn read_dir_with_followed_symlinks() {
    let tmp = assert_fs::TempDir::new().unwrap();
    tmp.child("target.txt").write_str("target content").unwrap();
    tmp.child("link.txt").symlink_to_file("target.txt").unwrap();

    let mut opts = WalkOptions::default();
    opts.follow_symlinks = true;

    let dir = DiskDirectoryBuilder::new(tmp.path())
        .with_options(opts)
        .build()
        .unwrap();

    assert_eq!(
        dir.entries(),
        vec![
            Entry::new(name("link.txt"), 0o100644, hash_content(b"target content")),
            Entry::new(
                name("target.txt"),
                0o100644,
                hash_content(b"target content")
            ),
        ]
    );
}

#[test]
fn read_dir_with_exclude_patterns() {
    let tmp = assert_fs::TempDir::new().unwrap();
    tmp.child("keep.txt").write_str("keep").unwrap();
    tmp.child("exclude.tmp").write_str("exclude").unwrap();
    tmp.child("also.tmp").write_str("also exclude").unwrap();

    let mut opts = WalkOptions::default();
    opts.exclude_suffixes.push(".tmp".to_string());

    let dir = DiskDirectoryBuilder::new(tmp.path())
        .with_options(opts)
        .build()
        .unwrap();

    assert_eq!(
        dir.entries(),
        vec![Entry::new(
            name("keep.txt"),
            0o100644,
            hash_content(b"keep")
        ),]
    );
}

#[test]
fn dir_walk_options_default() {
    let opts = WalkOptions::default();
    assert!(!opts.follow_symlinks);
    assert!(opts.exclude_suffixes.is_empty());
}

#[test]
fn dir_walk_options_custom() {
    let mut opts = WalkOptions::default();
    opts.follow_symlinks = true;
    opts.exclude_suffixes.push(".tmp".to_string());
    opts.exclude_suffixes.push(".log".to_string());

    assert!(opts.follow_symlinks);
    assert_eq!(opts.exclude_suffixes.len(), 2);
}

#[test]
fn dir_swhid_v2_all_serializers() {
    use swhid::config::HashConfig;
    let dir = Directory::new(vec![
        Entry::new(name("a.txt"), 0o100644, [1; 20]),
        Entry::new(name("b.txt"), 0o100755, [2; 20]),
    ])
    .unwrap();

    // Test all v2 serialization formats
    let hex_config = HashConfig::v2_sha256_hex();
    let base64_config = HashConfig::v2_sha256_base64();
    let base64url_config = HashConfig::v2_sha256_base64url();
    let base32_config = HashConfig::v2_sha256_base32();
    let base32hex_config = HashConfig::v2_sha256_base32hex();
    let z85_config = HashConfig::v2_sha256_z85();

    let hex_swhid = dir.swhid_with_config(&hex_config).unwrap();
    let base64_swhid = dir.swhid_with_config(&base64_config).unwrap();
    let base64url_swhid = dir.swhid_with_config(&base64url_config).unwrap();
    let base32_swhid = dir.swhid_with_config(&base32_config).unwrap();
    let base32hex_swhid = dir.swhid_with_config(&base32hex_config).unwrap();
    let z85_swhid = dir.swhid_with_config(&z85_config).unwrap();

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
fn dir_swhid_v1_backward_compatibility() {
    let dir = Directory::new(vec![
        Entry::new(name("a.txt"), 0o100644, [1; 20]),
    ])
    .unwrap();

    // V1 should still work
    let v1_swhid = dir.swhid().unwrap();
    assert_eq!(v1_swhid.version(), "1");
    assert_eq!(v1_swhid.digest_bytes().len(), 20);

    // V1 and V2 should produce different digests (different hash functions)
    use swhid::config::HashConfig;
    let v2_swhid = dir.swhid_with_config(&HashConfig::v2_sha256_hex()).unwrap();
    assert_ne!(v1_swhid.digest_bytes(), v2_swhid.digest_bytes());
}
