use std::borrow::Cow;
use std::fs;
use std::io;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::core::{ObjectType, Swhid};
use crate::error::DirectoryError;
use crate::hash::{hash_content, hash_content_with, hash_swhid_object, hash_swhid_object_with};
use crate::config::HashConfig;
use crate::utils::check_unique;

const DIRECTORY_MODE: u32 = 0o040000;
const FILE_MODE: u32 = 0o100644;
#[allow(dead_code)] // Used on Unix systems, appears unused on Windows
const EXECUTABLE_FILE_MODE: u32 = 0o100755;

/// Options for SWHID v1.2 directory walking and hashing.
#[derive(Debug, Clone, Default)]
pub struct WalkOptions {
    /// Whether to follow symlinks (note: not recommended; SWHID v1.2 uses link targets)
    pub follow_symlinks: bool,
    /// Exclude glob patterns (very minimal: literal suffix match)
    pub exclude_suffixes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Item in a [`Directory`]
pub struct Entry {
    /// raw bytes (no encoding assumptions)
    name: Box<[u8]>,
    /// SWHID v1.2 tree mode (compatible with Git tree mode)
    mode: u32,
    /// SWHID object id (20 bytes for SHA1, 32 bytes for SHA256)
    id: Vec<u8>,
}

impl Entry {
    pub fn new(name: Box<[u8]>, mode: u32, id: Vec<u8>) -> Entry {
        Self { name, mode, id }
    }

    fn is_dir(&self) -> bool {
        self.mode & DIRECTORY_MODE != 0
    }

    fn name_for_sort(&self) -> Cow<'_, [u8]> {
        if self.is_dir() {
            let mut name = Vec::from(self.name.clone());
            name.push(b'/');
            Cow::Owned(name)
        } else {
            Cow::Borrowed(&self.name)
        }
    }
}

fn is_excluded(name: &[u8], opts: &WalkOptions) -> bool {
    if opts.exclude_suffixes.is_empty() {
        return false;
    }
    let s = String::from_utf8_lossy(name);
    opts.exclude_suffixes.iter().any(|suf| s.ends_with(suf))
}

/// Compute the SWHID v1.2 directory manifest (concatenation of entries).
///
/// This implements the SWHID v1.2 directory tree format, which is compatible
/// with Git's tree format for directory objects.
pub fn dir_manifest(mut children: Vec<Entry>) -> Result<Vec<u8>, DirectoryError> {
    sort_and_check_children(&mut children)?;

    Ok(dir_manifest_unchecked(&children))
}

/// Same as [`dir_manifest`] but assumes children are already sorted and validated with
/// [`sort_and_check_children`]
fn dir_manifest_unchecked(children: &[Entry]) -> Vec<u8> {
    let mut out = Vec::new();
    for e in children {
        // "<mode> <name>\0<id-bytes>"
        let mut mode = format!("{:o}", e.mode).into_bytes();
        out.append(&mut mode);
        out.push(b' ');
        out.extend_from_slice(&e.name);
        out.push(0);
        out.extend_from_slice(&e.id);
    }
    out
}

fn sort_and_check_children(children: &mut [Entry]) -> Result<(), DirectoryError> {
    children.sort_unstable_by(|a, b| a.name_for_sort().cmp(&b.name_for_sort()));

    check_unique(children.iter().map(|child| &child.name))
        .map_err(|name| DirectoryError::DuplicateEntryName(name.clone()))?;

    for entry in children {
        for byte in [b'\0', b'/'] {
            if entry.name.contains(&byte) {
                return Err(DirectoryError::InvalidByteInName {
                    byte,
                    name: entry.name.clone(),
                });
            }
        }
    }

    Ok(())
}

fn path_file_mode(meta: &fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        let m = meta.permissions().mode();
        let exec = (m & 0o111) != 0;
        if meta.is_dir() {
            DIRECTORY_MODE
        } else if meta.is_file() {
            if exec {
                EXECUTABLE_FILE_MODE
            } else {
                FILE_MODE
            }
        } else {
            FILE_MODE
        }
    }
    #[cfg(not(unix))]
    {
        if meta.is_dir() {
            DIRECTORY_MODE
        } else {
            FILE_MODE
        }
    }
}

fn symlink_mode() -> u32 {
    0o120000
}

fn read_dir(path: &Path, opts: &WalkOptions, hash_config: Option<&HashConfig>) -> io::Result<Vec<Entry>> {
    let mut children: Vec<Entry> = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name_bytes = Box::from(file_name.as_os_str().as_encoded_bytes());

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
            let nested_entries = read_dir(&entry.path(), opts, hash_config)?;
            let manifest = dir_manifest(nested_entries)
                .map_err(|e: DirectoryError| io::Error::other(e))?;
            let id = if let Some(config) = hash_config {
                hash_swhid_object_with("tree", &manifest, config.hash_function.as_ref())
            } else {
                hash_swhid_object("tree", &manifest).to_vec()
            };
            children.push(Entry {
                name: name_bytes,
                mode: 0o040000,
                id,
            });
        } else if ft.is_symlink() {
            // The content is the link target bytes
            let target = fs::read_link(entry.path())?;
            let bytes = target.as_os_str().as_encoded_bytes();
            let id = if let Some(config) = hash_config {
                hash_content_with(bytes, config.hash_function.as_ref())
            } else {
                hash_content(bytes).to_vec()
            };
            children.push(Entry {
                name: name_bytes,
                mode: symlink_mode(),
                id,
            });
        } else if ft.is_file() {
            let bytes = fs::read(entry.path())?;
            let id = if let Some(config) = hash_config {
                hash_content_with(&bytes, config.hash_function.as_ref())
            } else {
                hash_content(&bytes).to_vec()
            };
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
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Directory {
    /// sorted according to the SWHID v1.2 order (ie. `/` suffix to directory names)
    entries: Vec<Entry>,
}

impl Directory {
    pub fn new(mut entries: Vec<Entry>) -> Result<Self, DirectoryError> {
        sort_and_check_children(&mut entries)?;

        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Compute the SWHID v1.2 directory identifier for this directory.
    ///
    /// This implements the SWHID v1.2 directory hashing algorithm, which
    /// is compatible with Git's tree format for directory objects.
    pub fn swhid(&self) -> Result<Swhid, crate::error::SwhidError> {
        let manifest = dir_manifest_unchecked(&self.entries);
        Ok(Swhid::new_v1(
            ObjectType::Directory,
            hash_swhid_object("tree", &manifest),
        ))
    }

    /// Compute the SWHID directory identifier using the specified hash configuration.
    ///
    /// This allows computing SWHIDs with different hash functions (SHA1, SHA256, etc.)
    /// and serialization formats (hex, base64, etc.) for v2 experimentation.
    ///
    /// Note: This method currently uses the same manifest format as v1, but with
    /// the specified hash function. The entries still contain [u8; 20] digests
    /// which are converted to hex for the manifest.
    pub fn swhid_with_config(&self, config: &HashConfig) -> Result<Swhid, crate::error::SwhidError> {
        let manifest = dir_manifest_unchecked(&self.entries);
        let digest = hash_swhid_object_with("tree", &manifest, config.hash_function.as_ref());
        Ok(Swhid::new(ObjectType::Directory, digest, config.version))
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
        Directory::new(read_dir(self.root, &self.opts, None)?)
            .map_err(|e: DirectoryError| io::Error::other(e))
    }

    /// Compute the SWHID v1.2 directory identifier for this directory.
    ///
    /// This implements the SWHID v1.2 directory hashing algorithm, which
    /// is compatible with Git's tree format for directory objects.
    pub fn swhid(&self) -> Result<Swhid, crate::error::SwhidError> {
        let entries = read_dir(self.root, &self.opts, None).map_err(crate::error::SwhidError::Io)?;
        Directory::new(entries)
            .map_err(|e| crate::error::SwhidError::Io(std::io::Error::other(e)))?
            .swhid()
    }

    /// Compute the SWHID directory identifier using the specified hash configuration.
    ///
    /// This allows computing SWHIDs with different hash functions (SHA1, SHA256, etc.)
    /// and serialization formats (hex, base64, etc.) for v2 experimentation.
    pub fn swhid_with_config(&self, config: &HashConfig) -> Result<Swhid, crate::error::SwhidError> {
        let entries = read_dir(self.root, &self.opts, Some(config)).map_err(crate::error::SwhidError::Io)?;
        Directory::new(entries)
            .map_err(|e| crate::error::SwhidError::Io(std::io::Error::other(e)))?
            .swhid_with_config(config)
    }
}
