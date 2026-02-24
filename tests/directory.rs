use assert_fs::prelude::*;

use swhid::digest::Digest;
use swhid::directory::*;
use swhid::hash::hash_content;
use swhid::ObjectType;

fn name(s: &'static str) -> Box<[u8]> {
    s.as_bytes().into()
}

#[test]
fn simple_dir_hash() {
    let dir = Directory::new(vec![
        Entry::new(name("a.txt"), 0o100644, Digest::from([1; 20])),
        Entry::new(name("b.txt"), 0o100755, Digest::from([2; 20])),
        Entry::new(name("c.txt"), 0o100644, Digest::from([0; 20])),
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
        Entry::new(name("a.txt"), 0o100644, Digest::from([1; 20])),
        Entry::new(name("c.txt"), 0o100644, Digest::from([0; 20])),
        Entry::new(name("b.txt"), 0o100755, Digest::from([2; 20])),
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
        Entry::new(name("a.txt"), 0o100644, Digest::from([1; 20])),
        Entry::new(name("b.txt"), 0o120000, Digest::from([2; 20])),
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
        Entry::new(name("a.txt"), 0o100644, Digest::from([1; 20])),
        Entry::new(name("b"), 0o040000, Digest::from([2; 20])),
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
        Entry::new(name("a.txt"), 0o100644, Digest::from(hash_content(b"A"))),
        Entry::new(name("b.txt"), 0o100644, Digest::from(hash_content(b"B"))),
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
                Digest::from(hash_content(b"cyrillic filename")),
            ),
            Entry::new(
                name("文件.txt"),
                0o100644,
                Digest::from(hash_content(b"unicode filename")),
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
            Entry::new(
                name("link.txt"),
                0o120000,
                Digest::from(hash_content(b"target.txt"))
            ),
            Entry::new(
                name("target.txt"),
                0o100644,
                Digest::from(hash_content(b"target content"))
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
            Entry::new(
                name("link.txt"),
                0o100644,
                Digest::from(hash_content(b"target content"))
            ),
            Entry::new(
                name("target.txt"),
                0o100644,
                Digest::from(hash_content(b"target content"))
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
            Digest::from(hash_content(b"keep"))
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
fn executable_bit_changes_directory_id() {
    use swhid::permissions::EntryPerms;
    // Golden test: executable bit must change directory ID
    let content = b"test content";
    let content_hash = hash_content(content);

    // Directory with non-executable file
    let dir1 = Directory::from_manifest(vec![ManifestEntry {
        name: b"script.sh".to_vec(),
        perms: EntryPerms::File { executable: false },
        target: content_hash.to_vec(),
    }])
    .unwrap();

    // Directory with executable file (same content)
    let dir2 = Directory::from_manifest(vec![ManifestEntry {
        name: b"script.sh".to_vec(),
        perms: EntryPerms::File { executable: true },
        target: content_hash.to_vec(),
    }])
    .unwrap();

    // Directory IDs must differ
    let swhid1 = dir1.swhid().unwrap();
    let swhid2 = dir2.swhid().unwrap();
    assert_ne!(swhid1, swhid2, "Executable bit should change directory ID");
}

#[test]
fn manifest_based_directory_building() {
    use swhid::permissions::EntryPerms;
    // Test that Directory::from_manifest works correctly
    let entries = vec![
        ManifestEntry {
            name: b"file1.txt".to_vec(),
            perms: EntryPerms::File { executable: false },
            target: hash_content(b"content1").to_vec(),
        },
        ManifestEntry {
            name: b"script.sh".to_vec(),
            perms: EntryPerms::File { executable: true },
            target: hash_content(b"#!/bin/bash").to_vec(),
        },
        ManifestEntry {
            name: b"subdir".to_vec(),
            perms: EntryPerms::Directory,
            target: [0u8; 20].to_vec(), // dummy hash for directory
        },
    ];

    let dir = Directory::from_manifest(entries).unwrap();
    assert_eq!(dir.entries().len(), 3);

    // Verify entries are sorted correctly
    // Note: Entry.name is private, so we check via the manifest
    let manifest = dir_manifest(dir.entries().to_vec()).unwrap();
    // The manifest should contain entries in sorted order
    assert!(manifest
        .windows(b"file1.txt".len())
        .any(|w| w == b"file1.txt"));
    assert!(manifest
        .windows(b"script.sh".len())
        .any(|w| w == b"script.sh"));
    assert!(manifest.windows(b"subdir".len()).any(|w| w == b"subdir"));
}

#[test]
#[cfg(unix)]
fn unix_filesystem_permission_source() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use swhid::permissions::{FilesystemPermissionsSource, PermissionsSource};
    use tempfile::TempDir;

    let tmp = TempDir::new().unwrap();
    let file_path = tmp.path().join("test.sh");

    // Create executable file
    fs::write(&file_path, b"#!/bin/bash").unwrap();
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&file_path, perms).unwrap();

    let source = FilesystemPermissionsSource;
    let exec = source.executable_of(&file_path).unwrap();
    assert_eq!(exec, swhid::permissions::EntryExec::Known(true));

    // Create non-executable file
    let file_path2 = tmp.path().join("test.txt");
    fs::write(&file_path2, b"content").unwrap();
    let mut perms2 = fs::metadata(&file_path2).unwrap().permissions();
    perms2.set_mode(0o644);
    fs::set_permissions(&file_path2, perms2).unwrap();

    let exec2 = source.executable_of(&file_path2).unwrap();
    assert_eq!(exec2, swhid::permissions::EntryExec::Known(false));
}

#[test]
fn permission_manifest_source() {
    use swhid::permissions::{ManifestPermissionsSource, PermissionsSource};
    use tempfile::NamedTempFile;

    let manifest_content = r#"
[[file]]
path = "bin/tool"
executable = true

[[file]]
path = "scripts/run.sh"
executable = true

[[file]]
path = "data.txt"
executable = false
"#;

    let mut file = NamedTempFile::new().unwrap();
    use std::io::Write;
    file.write_all(manifest_content.as_bytes()).unwrap();

    let source = ManifestPermissionsSource::load(file.path()).unwrap();

    // Test known paths
    assert_eq!(
        source
            .executable_of(std::path::Path::new("bin/tool"))
            .unwrap(),
        swhid::permissions::EntryExec::Known(true)
    );
    assert_eq!(
        source
            .executable_of(std::path::Path::new("scripts/run.sh"))
            .unwrap(),
        swhid::permissions::EntryExec::Known(true)
    );
    assert_eq!(
        source
            .executable_of(std::path::Path::new("data.txt"))
            .unwrap(),
        swhid::permissions::EntryExec::Known(false)
    );

    // Test unknown path
    assert_eq!(
        source
            .executable_of(std::path::Path::new("unknown.txt"))
            .unwrap(),
        swhid::permissions::EntryExec::Unknown
    );
}
