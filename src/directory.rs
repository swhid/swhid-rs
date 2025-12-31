use std::borrow::Cow;
use std::fs;
use std::path::Path;

use std::path::PathBuf;

use crate::core::{ObjectType, Swhid};
use crate::error::DirectoryError;
use crate::hash::{hash_content, hash_swhid_object};
use crate::permissions::{
    EntryPerms, PermissionPolicy, PermissionsSource, PermissionsSourceKind, resolve_file_permissions,
};
use crate::utils::check_unique;

const DIRECTORY_MODE: u32 = 0o040000;

/// Options for SWHID v1.2 directory walking and hashing.
#[derive(Debug, Clone, Default)]
pub struct WalkOptions {
    /// Whether to follow symlinks (note: not recommended; SWHID v1.2 uses link targets)
    pub follow_symlinks: bool,
    /// Exclude glob patterns (very minimal: literal suffix match)
    pub exclude_suffixes: Vec<String>,
}

/// Options for building directories with permission handling.
#[derive(Debug, Clone)]
pub struct DirectoryBuildOptions {
    /// Permission source to use
    pub permissions_source: PermissionsSourceKind,
    /// Policy for handling unknown permissions
    pub permissions_policy: PermissionPolicy,
    /// Path to permission manifest file (required when source=Manifest)
    pub permissions_manifest_path: Option<PathBuf>,
    /// Walk options (symlinks, excludes, etc.)
    pub walk_options: WalkOptions,
}

/// Manifest entry for building directories from explicit permissions.
///
/// This represents a directory entry with explicit permission information,
/// allowing platform-independent directory construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    /// Entry name (raw bytes, no encoding assumptions)
    pub name: Vec<u8>,
    /// Entry permissions (canonical SWHID/Git format)
    pub perms: EntryPerms,
    /// SWHID object id (digest bytes of the child object, 20 bytes for v1)
    pub target: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Item in a [`Directory`]
pub struct Entry {
    /// raw bytes (no encoding assumptions)
    name: Box<[u8]>,
    /// SWHID v1.2 tree mode (compatible with Git tree mode)
    mode: u32,
    /// SWHID object id
    id: [u8; 20],
}

impl Entry {
    pub fn new(name: Box<[u8]>, mode: u32, id: [u8; 20]) -> Entry {
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

impl From<ManifestEntry> for Entry {
    fn from(manifest: ManifestEntry) -> Self {
        // Convert Vec<u8> to [u8; 20] for v1 compatibility
        let id_array: [u8; 20] = manifest.target.try_into()
            .unwrap_or_else(|v: Vec<u8>| {
                let mut arr = [0u8; 20];
                let len = v.len().min(20);
                arr[..len].copy_from_slice(&v[..len]);
                arr
            });
        Entry {
            name: manifest.name.into_boxed_slice(),
            mode: manifest.perms.to_swh_mode_u32(),
            id: id_array,
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

fn symlink_mode() -> u32 {
    0o120000
}

fn read_dir(
    path: &Path,
    root: &Path,
    opts: &DirectoryBuildOptions,
) -> Result<Vec<Entry>, crate::error::SwhidError> {
    use crate::permissions::{
        AutoPermissionsSource, FilesystemPermissionsSource, ManifestPermissionsSource,
    };
    #[cfg(feature = "git")]
    use crate::permissions::{GitIndexPermissionsSource, GitTreePermissionsSource};

    // Create permission source based on options
    let permission_source: Box<dyn PermissionsSource> = match opts.permissions_source {
        PermissionsSourceKind::Auto => {
            Box::new(AutoPermissionsSource::new(root)?)
        }
        PermissionsSourceKind::Filesystem => {
            Box::new(FilesystemPermissionsSource)
        }
        #[cfg(feature = "git")]
        PermissionsSourceKind::GitIndex => {
            let repo = git2::Repository::open(root).map_err(|e| {
                crate::error::SwhidError::Io(std::io::Error::other(format!(
                    "Failed to open Git repository: {}",
                    e
                )))
            })?;
            Box::new(GitIndexPermissionsSource::new(repo, root.to_path_buf()))
        }
        #[cfg(feature = "git")]
        PermissionsSourceKind::GitTree => {
            let repo = git2::Repository::open(root).map_err(|e| {
                crate::error::SwhidError::Io(std::io::Error::other(format!(
                    "Failed to open Git repository: {}",
                    e
                )))
            })?;
            Box::new(GitTreePermissionsSource::new(repo, root.to_path_buf()))
        }
        PermissionsSourceKind::Manifest => {
            let manifest_path = opts.permissions_manifest_path.as_ref().ok_or_else(|| {
                crate::error::SwhidError::InvalidFormat(
                    "permissions_manifest_path is required when using Manifest source".to_string(),
                )
            })?;
            Box::new(ManifestPermissionsSource::load(manifest_path)?)
        }
        #[cfg(not(feature = "git"))]
        PermissionsSourceKind::GitIndex | PermissionsSourceKind::GitTree => {
            return Err(crate::error::SwhidError::InvalidFormat(
                "Git permission sources require the 'git' feature".to_string(),
            ));
        }
        PermissionsSourceKind::Heuristic => {
            // Heuristic not implemented yet, fall back to filesystem
            Box::new(FilesystemPermissionsSource)
        }
    };
    let mut children: Vec<Entry> = Vec::new();
    for entry in fs::read_dir(path).map_err(|e| {
        crate::error::SwhidError::Io(std::io::Error::other(format!(
            "Failed to read directory {}: {}",
            path.display(),
            e
        )))
    })? {
        let entry = entry.map_err(|e| {
            crate::error::SwhidError::Io(std::io::Error::other(format!(
                "Failed to read directory entry: {}",
                e
            )))
        })?;
        let file_name = entry.file_name();
        let name_bytes = Box::from(file_name.as_os_str().as_encoded_bytes());

        if is_excluded(&name_bytes, &opts.walk_options) {
            continue;
        }

        let md = if opts.walk_options.follow_symlinks {
            fs::metadata(entry.path()).map_err(|e| {
                crate::error::SwhidError::Io(std::io::Error::other(format!(
                    "Failed to read metadata for {}: {}",
                    entry.path().display(),
                    e
                )))
            })?
        } else {
            fs::symlink_metadata(entry.path()).map_err(|e| {
                crate::error::SwhidError::Io(std::io::Error::other(format!(
                    "Failed to read symlink metadata for {}: {}",
                    entry.path().display(),
                    e
                )))
            })?
        };
        let ft = md.file_type();

        if ft.is_dir() {
            let nested_entries = read_dir(&entry.path(), root, opts)?;
            let manifest = dir_manifest(nested_entries)
                .map_err(|e: DirectoryError| {
                    crate::error::SwhidError::Io(std::io::Error::other(format!(
                        "Failed to build directory manifest: {}",
                        e
                    )))
                })?;
            let id = hash_swhid_object("tree", &manifest);
            children.push(Entry {
                name: name_bytes,
                mode: 0o040000,
                id,
            });
        } else if ft.is_symlink() {
            // The content is the link target bytes
            let target = fs::read_link(entry.path()).map_err(|e| {
                crate::error::SwhidError::Io(std::io::Error::other(format!(
                    "Failed to read symlink {}: {}",
                    entry.path().display(),
                    e
                )))
            })?;
            let bytes = target.as_os_str().as_encoded_bytes();
            let id = hash_content(bytes);
            children.push(Entry {
                name: name_bytes,
                mode: symlink_mode(),
                id,
            });
        } else if ft.is_file() {
            let bytes = fs::read(entry.path()).map_err(|e| {
                crate::error::SwhidError::Io(std::io::Error::other(format!(
                    "Failed to read file {}: {}",
                    entry.path().display(),
                    e
                )))
            })?;
            let id = hash_content(&bytes);

            // Use permission source to determine executable bit
            let exec = permission_source.executable_of(&entry.path())?;
            let perms = resolve_file_permissions(
                exec,
                opts.permissions_policy,
                &entry.path(),
            )?;
            let mode = perms.to_swh_mode_u32();

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

    /// Create a Directory from manifest entries with explicit permissions.
    ///
    /// This is the pure manifest-based path that is platform-independent.
    /// All permission information must be provided in the manifest entries.
    pub fn from_manifest(
        entries: impl IntoIterator<Item = ManifestEntry>,
    ) -> Result<Self, DirectoryError> {
        let entries: Vec<Entry> = entries.into_iter().map(Entry::from).collect();
        Self::new(entries)
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
        Ok(Swhid::new(
            ObjectType::Directory,
            hash_swhid_object("tree", &manifest),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct DiskDirectoryBuilder<'a> {
    root: &'a Path,
    opts: DirectoryBuildOptions,
}

impl<'a> DiskDirectoryBuilder<'a> {
    /// Create a new Directory object for the given path.
    ///
    /// This implements SWHID v1.2 directory object creation for any directory.
    /// Uses default options (best-effort policy, auto permission source).
    pub fn new(root: &'a Path) -> Self {
        Self {
            root,
            opts: DirectoryBuildOptions {
                permissions_source: PermissionsSourceKind::Auto,
                permissions_policy: PermissionPolicy::BestEffort,
                permissions_manifest_path: None,
                walk_options: WalkOptions::default(),
            },
        }
    }

    /// Configure directory building options.
    pub fn with_build_options(mut self, opts: DirectoryBuildOptions) -> Self {
        self.opts = opts;
        self
    }

    /// Configure directory walking options (backward compatibility).
    pub fn with_options(mut self, walk_opts: WalkOptions) -> Self {
        self.opts.walk_options = walk_opts;
        self
    }

    pub fn build(self) -> Result<Directory, crate::error::SwhidError> {
        let entries = read_dir(self.root, self.root, &self.opts)?;
        Directory::new(entries)
            .map_err(|e| crate::error::SwhidError::Io(std::io::Error::other(e)))
    }

    /// Compute the SWHID v1.2 directory identifier for this directory.
    ///
    /// This implements the SWHID v1.2 directory hashing algorithm, which
    /// is compatible with Git's tree format for directory objects.
    pub fn swhid(&self) -> Result<Swhid, crate::error::SwhidError> {
        let entries = read_dir(self.root, self.root, &self.opts)?;
        Directory::new(entries)
            .map_err(|e| crate::error::SwhidError::Io(std::io::Error::other(e)))?
            .swhid()
    }
}
