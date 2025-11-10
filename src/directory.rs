use std::fs;
use std::io;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::core::{ObjectType, Swhid};
use crate::hash::{hash_content, hash_swhid_object};

/// Options for SWHID v1.2 directory walking and hashing.
#[derive(Debug, Clone, Default)]
pub struct WalkOptions {
    /// Whether to follow symlinks (note: not recommended; SWHID v1.2 uses link targets)
    pub follow_symlinks: bool,
    /// Exclude glob patterns (very minimal: literal suffix match)
    pub exclude_suffixes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Entry {
    name: Vec<u8>, // raw bytes (no encoding assumptions)
    mode: u32,     // SWHID v1.2 tree mode (compatible with Git tree mode)
    id: [u8; 20],  // SWHID object id
}

fn is_excluded(name: &[u8], opts: &WalkOptions) -> bool {
    if opts.exclude_suffixes.is_empty() {
        return false;
    }
    let s = String::from_utf8_lossy(name);
    opts.exclude_suffixes.iter().any(|suf| s.ends_with(suf))
}

/// Compute the SWHID v1.2 directory payload (concatenation of entries).
///
/// This implements the SWHID v1.2 directory tree format, which is compatible
/// with Git's tree format for directory objects.
fn dir_payload(mut children: Vec<Entry>) -> Vec<u8> {
    // SWHID v1.2 sorts by raw bytes of filename (no path separators here).
    children.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = Vec::new();
    for e in children {
        // "<mode> <name>\0<id-bytes>"
        let mut mode = format!("{:06o}", e.mode).into_bytes();
        out.append(&mut mode);
        out.push(b' ');
        out.extend_from_slice(&e.name);
        out.push(0);
        out.extend_from_slice(&e.id);
    }
    out
}

fn path_file_mode(meta: &fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        let m = meta.permissions().mode();
        let exec = (m & 0o111) != 0;
        if meta.is_dir() {
            0o040000
        } else if meta.is_file() {
            if exec {
                0o100755
            } else {
                0o100644
            }
        } else {
            0o100644
        }
    }
    #[cfg(not(unix))]
    {
        if meta.is_dir() {
            0o040000
        } else {
            0o100644
        }
    }
}

fn symlink_mode() -> u32 {
    0o120000
}

fn read_dir(path: &Path, opts: &WalkOptions) -> io::Result<Vec<Entry>> {
    let mut children: Vec<Entry> = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name_bytes = file_name.as_os_str().as_encoded_bytes().to_vec();

        if is_excluded(&name_bytes, opts) {
            continue;
        }

        let md = if opts.follow_symlinks {
            fs::metadata(entry.path())?
        } else {
            fs::symlink_metadata(entry.path())?
        };
        let ft = md.file_type();

        if ft.is_dir() {
            let id = hash_swhid_object("tree", &dir_payload(read_dir(&entry.path(), opts)?));
            children.push(Entry {
                name: name_bytes,
                mode: 0o040000,
                id,
            });
        } else if ft.is_symlink() {
            // The content is the link target bytes
            let target = fs::read_link(entry.path())?;
            let bytes = target.as_os_str().as_encoded_bytes();
            let id = hash_content(bytes);
            children.push(Entry {
                name: name_bytes,
                mode: symlink_mode(),
                id,
            });
        } else if ft.is_file() {
            let bytes = fs::read(entry.path())?;
            let id = hash_content(&bytes);
            let mode = path_file_mode(&md);
            children.push(Entry {
                name: name_bytes,
                mode,
                id,
            });
        } else {
            // ignore special files
            continue;
        }
    }
    Ok(children)
}

/// SWHID v1.2 directory object for computing directory SWHIDs.
///
/// This struct represents a directory tree and provides methods to compute
/// SWHID v1.2 compliant directory identifiers according to the specification.
#[derive(Debug, Clone)]
pub struct Directory {
    entries: Vec<Entry>,
}

impl Directory {
    pub fn new(entries: Vec<Entry>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Compute the SWHID v1.2 directory identifier for this directory.
    ///
    /// This implements the SWHID v1.2 directory hashing algorithm, which
    /// is compatible with Git's tree format for directory objects.
    pub fn swhid(&self) -> Result<Swhid, crate::error::SwhidError> {
        let payload = dir_payload(self.entries.clone());
        Ok(Swhid::new(
            ObjectType::Directory,
            hash_swhid_object("tree", &payload),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct DiskDirectoryBuilder<'a> {
    root: &'a Path,
    opts: WalkOptions,
}

impl<'a> DiskDirectoryBuilder<'a> {
    /// Create a new Directory object for the given path.
    ///
    /// This implements SWHID v1.2 directory object creation for any directory.
    pub fn new(root: &'a Path) -> Self {
        Self {
            root,
            opts: WalkOptions::default(),
        }
    }

    /// Configure directory walking options.
    pub fn with_options(mut self, opts: WalkOptions) -> Self {
        self.opts = opts;
        self
    }

    pub fn build(self) -> Result<Directory, io::Error> {
        Ok(Directory::new(read_dir(self.root, &self.opts)?))
    }

    /// Compute the SWHID v1.2 directory identifier for this directory.
    ///
    /// This implements the SWHID v1.2 directory hashing algorithm, which
    /// is compatible with Git's tree format for directory objects.
    pub fn swhid(&self) -> Result<Swhid, crate::error::SwhidError> {
        let entries = read_dir(self.root, &self.opts)
            .map_err(|e| crate::error::SwhidError::Io(e.to_string()))?;
        Directory::new(entries).swhid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn simple_dir_hash() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("a.txt").write_str("A").unwrap();
        tmp.child("b.txt").write_str("B").unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id = dir.swhid().unwrap();
        // Just sanity: should be 40 hex chars
        assert_eq!(id.to_string().len(), "swh:1:dir:".len() + 40);
    }

    #[test]
    fn empty_dir_hash() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id = dir.swhid().unwrap();
        assert_eq!(id.object_type(), ObjectType::Directory);
        assert_eq!(id.to_string().len(), "swh:1:dir:".len() + 40);
    }

    #[test]
    fn single_file_dir() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("single.txt")
            .write_str("single file content")
            .unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id = dir.swhid().unwrap();
        assert_eq!(id.object_type(), ObjectType::Directory);
    }

    #[test]
    fn nested_dir_structure() {
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
    fn dir_with_symlinks() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("target.txt").write_str("target content").unwrap();
        tmp.child("link.txt").symlink_to_file("target.txt").unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id = dir.swhid().unwrap();
        assert_eq!(id.object_type(), ObjectType::Directory);
    }

    #[test]
    fn dir_with_exclude_patterns() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("keep.txt").write_str("keep").unwrap();
        tmp.child("exclude.tmp").write_str("exclude").unwrap();
        tmp.child("also.tmp").write_str("also exclude").unwrap();

        let mut opts = WalkOptions::default();
        opts.exclude_suffixes.push(".tmp".to_string());

        let dir = DiskDirectoryBuilder::new(tmp.path()).with_options(opts);
        let id = dir.swhid().unwrap();
        assert_eq!(id.object_type(), ObjectType::Directory);
    }

    #[test]
    fn dir_consistency() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("a.txt").write_str("A").unwrap();
        tmp.child("b.txt").write_str("B").unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id1 = dir.swhid().unwrap();
        let id2 = dir.swhid().unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn dir_different_content() {
        let tmp1 = assert_fs::TempDir::new().unwrap();
        tmp1.child("file.txt").write_str("content1").unwrap();

        let tmp2 = assert_fs::TempDir::new().unwrap();
        tmp2.child("file.txt").write_str("content2").unwrap();

        let dir1 = DiskDirectoryBuilder::new(tmp1.path());
        let dir2 = DiskDirectoryBuilder::new(tmp2.path());

        let id1 = dir1.swhid().unwrap();
        let id2 = dir2.swhid().unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn dir_file_order_independence() {
        let expected_swhid = "swh:1:dir:063012ee77491cc23b0bbad86cfd47e8309c80cb";
        let dir = Directory::new(vec![
            Entry {
                name: b"c.txt".into(),
                mode: 0o100644,
                id: [0; 20],
            },
            Entry {
                name: b"b.txt".into(),
                mode: 0o100644,
                id: [2; 20],
            },
            Entry {
                name: b"a.txt".into(),
                mode: 0o100644,
                id: [1; 20],
            },
        ]);
        assert_eq!(dir.swhid().unwrap().to_string(), expected_swhid);

        let dir = Directory::new(vec![
            Entry {
                name: b"a.txt".into(),
                mode: 0o100644,
                id: [1; 20],
            },
            Entry {
                name: b"b.txt".into(),
                mode: 0o100644,
                id: [2; 20],
            },
            Entry {
                name: b"c.txt".into(),
                mode: 0o100644,
                id: [0; 20],
            },
        ]);
        assert_eq!(dir.swhid().unwrap().to_string(), expected_swhid);
    }

    #[test]
    fn dir_with_binary_files() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("binary.bin")
            .write_binary(&[0x00, 0x01, 0xFF, 0xFE])
            .unwrap();
        tmp.child("text.txt").write_str("text content").unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id = dir.swhid().unwrap();
        assert_eq!(id.object_type(), ObjectType::Directory);
    }

    #[test]
    fn dir_with_unicode_filenames() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("文件.txt").write_str("unicode filename").unwrap();
        tmp.child("файл.txt")
            .write_str("cyrillic filename")
            .unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id = dir.swhid().unwrap();
        assert_eq!(id.object_type(), ObjectType::Directory);
    }

    #[test]
    fn dir_with_empty_files() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("empty.txt").write_str("").unwrap();
        tmp.child("nonempty.txt").write_str("content").unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id = dir.swhid().unwrap();
        assert_eq!(id.object_type(), ObjectType::Directory);
    }

    #[test]
    fn dir_with_special_characters() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("file with spaces.txt")
            .write_str("content")
            .unwrap();
        tmp.child("file-with-dashes.txt")
            .write_str("content")
            .unwrap();
        tmp.child("file_with_underscores.txt")
            .write_str("content")
            .unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id = dir.swhid().unwrap();
        assert_eq!(id.object_type(), ObjectType::Directory);
    }

    #[test]
    fn dir_swhid_format() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("test.txt").write_str("test").unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id = dir.swhid().unwrap();
        let swhid_str = id.to_string();
        assert!(swhid_str.starts_with("swh:1:dir:"));
        assert_eq!(swhid_str.len(), "swh:1:dir:".len() + 40);
    }

    #[test]
    fn dir_swhid_roundtrip() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("test.txt").write_str("test").unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id = dir.swhid().unwrap();
        let swhid_str = id.to_string();
        let parsed: Swhid = swhid_str.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn dir_swhid_digest_hex() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("test.txt").write_str("test").unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id = dir.swhid().unwrap();
        let hex = id.digest_hex();
        assert_eq!(hex.len(), 40);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn dir_swhid_digest_bytes() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("test.txt").write_str("test").unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id = dir.swhid().unwrap();
        let bytes = id.digest_bytes();
        assert_eq!(bytes.len(), 20);
    }

    #[test]
    fn dir_swhid_object_type() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("test.txt").write_str("test").unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id = dir.swhid().unwrap();
        assert_eq!(id.object_type(), ObjectType::Directory);
    }

    #[test]
    fn dir_swhid_version() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("test.txt").write_str("test").unwrap();

        let _dir = DiskDirectoryBuilder::new(tmp.path());
        assert_eq!(Swhid::VERSION, "1");
    }

    #[test]
    fn dir_swhid_equality() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("test.txt").write_str("test").unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id1 = dir.swhid().unwrap();
        let id2 = dir.swhid().unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn dir_swhid_hash_consistency() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("test.txt").write_str("test").unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id1 = dir.swhid().unwrap();
        let id2 = dir.swhid().unwrap();
        assert_eq!(id1, id2);
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
    fn dir_walk_options_exclude_patterns() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("keep.txt").write_str("keep").unwrap();
        tmp.child("exclude.tmp").write_str("exclude").unwrap();
        tmp.child("also.log").write_str("also exclude").unwrap();
        tmp.child("keep.log").write_str("keep").unwrap();

        let mut opts = WalkOptions::default();
        opts.exclude_suffixes.push(".tmp".to_string());
        opts.exclude_suffixes.push(".log".to_string());

        let dir = DiskDirectoryBuilder::new(tmp.path()).with_options(opts);
        let id = dir.swhid().unwrap();
        assert_eq!(id.object_type(), ObjectType::Directory);
    }

    #[test]
    fn dir_walk_options_follow_symlinks() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("target.txt").write_str("target content").unwrap();
        tmp.child("link.txt").symlink_to_file("target.txt").unwrap();

        let mut opts = WalkOptions::default();
        opts.follow_symlinks = true;

        let dir = DiskDirectoryBuilder::new(tmp.path()).with_options(opts);
        let id = dir.swhid().unwrap();
        assert_eq!(id.object_type(), ObjectType::Directory);
    }

    #[test]
    fn dir_walk_options_clone() {
        let opts1 = WalkOptions::default();
        let opts2 = opts1.clone();
        assert_eq!(opts1.follow_symlinks, opts2.follow_symlinks);
        assert_eq!(opts1.exclude_suffixes, opts2.exclude_suffixes);
    }

    #[test]
    fn dir_walk_options_debug() {
        let opts = WalkOptions::default();
        let debug_str = format!("{opts:?}");
        assert!(debug_str.contains("WalkOptions"));
    }

    #[test]
    fn dir_walk_options_with_options() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("test.txt").write_str("test").unwrap();

        let mut opts = WalkOptions::default();
        opts.follow_symlinks = true;

        let dir = DiskDirectoryBuilder::new(tmp.path()).with_options(opts);
        let id = dir.swhid().unwrap();
        assert_eq!(id.object_type(), ObjectType::Directory);
    }

    #[test]
    fn dir_walk_options_new() {
        let tmp = assert_fs::TempDir::new().unwrap();
        tmp.child("test.txt").write_str("test").unwrap();

        let dir = DiskDirectoryBuilder::new(tmp.path());
        let id = dir.swhid().unwrap();
        assert_eq!(id.object_type(), ObjectType::Directory);
    }

    #[test]
    fn dir_walk_options_debug_clone() {
        let opts = WalkOptions::default();
        let cloned = opts.clone();
        let debug_str = format!("{cloned:?}");
        assert!(debug_str.contains("WalkOptions"));
    }
}
