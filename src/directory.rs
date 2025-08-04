use std::fs;
use std::os::unix::fs::MetadataExt;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use crate::swhid::{Swhid, ObjectType};
use crate::content::Content;
use crate::hash::hash_git_object;
use crate::error::SwhidError;

/// Directory entry types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
}

impl EntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryType::File => "file",
            EntryType::Directory => "dir",
            EntryType::Symlink => "symlink",
        }
    }
}

/// Directory entry permissions (Git-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permissions {
    File = 0o100644,
    Executable = 0o100755,
    Symlink = 0o120000,
    Directory = 0o040000,
}

impl Permissions {
    pub fn from_mode(mode: u32) -> Self {
        match mode & 0o170000 {
            0o040000 => Permissions::Directory,
            0o120000 => Permissions::Symlink,
            _ => {
                if mode & 0o111 != 0 {
                    Permissions::Executable
                } else {
                    Permissions::File
                }
            }
        }
    }

    pub fn as_octal(&self) -> u32 {
        *self as u32
    }
}

/// Directory entry
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: Vec<u8>,
    pub entry_type: EntryType,
    pub permissions: Permissions,
    pub target: [u8; 20], // SHA1 hash of the target object
}

impl DirectoryEntry {
    pub fn new(name: Vec<u8>, entry_type: EntryType, permissions: Permissions, target: [u8; 20]) -> Self {
        Self {
            name,
            entry_type,
            permissions,
            target,
        }
    }
}

/// Directory object
#[derive(Debug, Clone)]
pub struct Directory {
    entries: Vec<DirectoryEntry>,
    hash: Option<[u8; 20]>,
    path: Option<PathBuf>,
}

impl Directory {
    /// Create a new empty directory
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            hash: None,
            path: None,
        }
    }

    /// Create directory from disk path
    pub fn from_disk<P: AsRef<Path>>(
        path: P,
        exclude_patterns: &[String],
        follow_symlinks: bool,
    ) -> Result<Self, SwhidError> {
        let path = path.as_ref();
        let mut entries = Vec::new();

        // Collect and sort directory entries
        let mut raw_entries: Vec<_> = fs::read_dir(path)?.collect();
        raw_entries.sort_by(|a, b| {
            let name_a = a.as_ref().unwrap().file_name();
            let name_b = b.as_ref().unwrap().file_name();
            name_a.cmp(&name_b)
        });

        for entry_result in raw_entries {
            let entry = entry_result?;
            let name = entry.file_name();
            let name_bytes = name.to_string_lossy().as_bytes().to_vec();

            // Skip excluded files and directories
            if Self::should_exclude(&name_bytes, exclude_patterns) {
                continue;
            }

            let metadata = if follow_symlinks {
                entry.metadata()?
            } else {
                entry.metadata()?
            };

            let entry_type = if metadata.is_dir() {
                EntryType::Directory
            } else if metadata.is_symlink() {
                EntryType::Symlink
            } else {
                EntryType::File
            };

            let permissions = Permissions::from_mode(metadata.mode());

            // Compute the target hash
            let target = if entry_type == EntryType::File {
                let content = Content::from_file(entry.path())?;
                *content.sha1_git()
            } else if entry_type == EntryType::Symlink {
                // Handle symlinks - read the symlink target as content
                if let Ok(target_path) = fs::read_link(entry.path()) {
                    let target_bytes = target_path.to_string_lossy().as_bytes().to_vec();
                    let content = Content::from_data(target_bytes);
                    *content.sha1_git()
                } else {
                    // Skip broken symlinks
                    continue;
                }
            } else {
                // Directory - use dummy hash for now, will be computed later
                [0u8; 20]
            };

            let dir_entry = DirectoryEntry::new(name_bytes, entry_type, permissions, target);
            entries.push(dir_entry);
        }

        // Sort entries according to Git's tree sorting rules
        entries.sort_by(|a, b| a.name.cmp(&b.name));

        // For directories, we need to compute their hashes recursively
        for entry in &mut entries {
            if entry.entry_type == EntryType::Directory {
                let child_path = path.join(std::ffi::OsStr::from_bytes(&entry.name));
                let mut child_dir = Directory::from_disk(child_path, exclude_patterns, follow_symlinks)?;
                entry.target = child_dir.compute_hash();
            }
        }

        let mut dir = Self {
            entries,
            hash: None,
            path: Some(path.to_path_buf()),
        };
        dir.path = Some(path.to_path_buf());
        Ok(dir)
    }

    /// Get directory entries
    pub fn entries(&self) -> &[DirectoryEntry] {
        &self.entries
    }

    /// Compute the directory hash
    pub fn compute_hash(&mut self) -> [u8; 20] {
        if let Some(hash) = self.hash {
            return hash;
        }

        let mut components = Vec::new();

        for entry in &self.entries {
            // Format: perms + space + name + null + target
            // Use exact string format as per SWHID specification
            let perms_str = match entry.permissions {
                Permissions::File => "100644",
                Permissions::Executable => "100755", 
                Permissions::Symlink => "120000",
                Permissions::Directory => "40000",
            };
            components.extend_from_slice(perms_str.as_bytes());
            components.push(b' ');
            components.extend_from_slice(&entry.name);
            components.push(0);
            components.extend_from_slice(&entry.target);
        }

        let hash = hash_git_object("tree", &components);
        self.hash = Some(hash);
        hash
    }

    /// Compute SWHID for this directory
    pub fn swhid(&mut self) -> Swhid {
        let hash = self.compute_hash();
        Swhid::new(ObjectType::Directory, hash)
    }

    /// Get the path associated with this directory
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Check if entry should be excluded based on patterns
    fn should_exclude(name: &[u8], patterns: &[String]) -> bool {
        let name_str = String::from_utf8_lossy(name);
        should_exclude_str(&name_str, patterns)
    }
}

/// Check if entry should be excluded based on patterns (string version)
/// Uses shell pattern matching like Python's fnmatch
fn should_exclude_str(name: &str, patterns: &[String]) -> bool {
    for pattern in patterns {
        // Simple shell pattern matching - for now just exact match
        // TODO: Implement full shell pattern matching like Python's fnmatch
        if name == pattern {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_directory_new() {
        let dir = Directory::new();
        assert_eq!(dir.entries().len(), 0);
        assert!(dir.path().is_none());
    }

    #[test]
    fn test_directory_from_disk() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(sub_dir.join("file.txt"), b"test").unwrap();

        let dir = Directory::from_disk(temp_dir.path(), &[], true).unwrap();
        assert_eq!(dir.entries().len(), 1);
        assert_eq!(dir.entries()[0].entry_type, EntryType::Directory);
    }

    #[test]
    fn test_directory_swhid() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("file.txt"), b"test").unwrap();

        let mut dir = Directory::from_disk(temp_dir.path(), &[], true).unwrap();
        let swhid = dir.swhid();
        
        assert_eq!(swhid.object_type(), ObjectType::Directory);
        assert_eq!(swhid.hash().len(), 20);
    }
} 