//! Permission handling for cross-platform directory SWHID computation.
//!
//! This module provides types and traits for handling file permissions across
//! different platforms, particularly addressing the Windows executable bit issue.

use std::path::Path;

use crate::error::SwhidError;

/// Entry permissions as specified in SWHID/Git tree format.
///
/// This represents the canonical permission modes that are part of the
/// directory manifest and affect the directory identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntryPerms {
    /// Regular file with executable bit
    File { executable: bool },
    /// Directory
    Directory,
    /// Symlink
    Symlink,
    /// Revision reference (submodule)
    RevisionRef,
}

impl EntryPerms {
    /// Convert to Git mode string (e.g., "100644", "100755").
    pub fn to_git_mode_string(&self) -> &'static str {
        match self {
            EntryPerms::File { executable: false } => "100644",
            EntryPerms::File { executable: true } => "100755",
            EntryPerms::Directory => "040000",
            EntryPerms::Symlink => "120000",
            EntryPerms::RevisionRef => "160000",
        }
    }

    /// Convert to SWHID mode as u32 (octal representation).
    pub fn to_swh_mode_u32(&self) -> u32 {
        match self {
            EntryPerms::File { executable: false } => 0o100644,
            EntryPerms::File { executable: true } => 0o100755,
            EntryPerms::Directory => 0o040000,
            EntryPerms::Symlink => 0o120000,
            EntryPerms::RevisionRef => 0o160000,
        }
    }

    /// Create from a u32 mode (Git tree mode).
    pub fn from_mode(mode: u32) -> Result<Self, SwhidError> {
        match mode {
            0o100644 => Ok(EntryPerms::File { executable: false }),
            0o100755 => Ok(EntryPerms::File { executable: true }),
            0o040000 => Ok(EntryPerms::Directory),
            0o120000 => Ok(EntryPerms::Symlink),
            0o160000 => Ok(EntryPerms::RevisionRef),
            _ => Err(SwhidError::InvalidFormat(format!(
                "Invalid entry mode: {:o}",
                mode
            ))),
        }
    }
}

/// Executable bit status at the probe layer.
///
/// This represents whether the executable bit is known or unknown.
/// Unknown typically occurs on Windows when filesystem metadata doesn't
/// provide executable information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryExec {
    /// Executable bit is known
    Known(bool),
    /// Executable bit cannot be determined
    Unknown,
}

/// Policy for handling unknown permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionPolicy {
    /// Strict: return error if permissions cannot be determined
    Strict,
    /// Best-effort: default to non-executable and emit warning
    BestEffort,
}

/// Kind of permission source to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionsSourceKind {
    /// Automatic detection (Git → Unix FS → Windows FS)
    Auto,
    /// Unix filesystem metadata
    Filesystem,
    /// Git index entries
    GitIndex,
    /// Git tree objects (HEAD)
    GitTree,
    /// Sidecar permission manifest file
    Manifest,
    /// Heuristic-based (extension/shebang, opt-in only)
    Heuristic,
}

/// Trait for permission sources that can determine executable status.
pub trait PermissionsSource {
    /// Determine if a file is executable.
    ///
    /// Returns `EntryExec::Known(bool)` if the executable status can be determined,
    /// or `EntryExec::Unknown` if it cannot be determined.
    fn executable_of(&self, path: &Path) -> Result<EntryExec, SwhidError>;
}

/// Helper function to map `EntryExec` + `PermissionPolicy` → `EntryPerms::File`.
///
/// This handles the policy logic for converting probe results into canonical permissions.
pub fn resolve_file_permissions(
    exec: EntryExec,
    policy: PermissionPolicy,
    path: &Path,
) -> Result<EntryPerms, SwhidError> {
    match (exec, policy) {
        (EntryExec::Known(executable), _) => Ok(EntryPerms::File { executable }),
        (EntryExec::Unknown, PermissionPolicy::Strict) => {
            Err(SwhidError::InvalidFormat(format!(
                "Cannot determine executable bit for {} on this platform. \
                Options: use --permissions-source git-index, provide a permission manifest, \
                or use --permissions-policy best-effort",
                path.display()
            )))
        }
        (EntryExec::Unknown, PermissionPolicy::BestEffort) => {
            // Default to non-executable
            Ok(EntryPerms::File { executable: false })
        }
    }
}

/// Filesystem-based permission source (Unix only).
///
/// On Unix systems, reads executable bit from filesystem metadata.
/// On non-Unix systems, returns `EntryExec::Unknown`.
#[derive(Debug, Clone)]
pub struct FilesystemPermissionsSource;

impl PermissionsSource for FilesystemPermissionsSource {
    fn executable_of(&self, path: &Path) -> Result<EntryExec, SwhidError> {
        #[cfg(unix)]
        {
            use std::fs;
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(path)
                .map_err(|e| SwhidError::Io(std::io::Error::other(format!(
                    "Failed to read metadata for {}: {}",
                    path.display(),
                    e
                ))))?;
            let mode = metadata.permissions().mode();
            let executable = (mode & 0o111) != 0;
            Ok(EntryExec::Known(executable))
        }
        #[cfg(not(unix))]
        {
            let _ = path; // suppress unused warning
            Ok(EntryExec::Unknown)
        }
    }
}

#[cfg(feature = "git")]
/// Git index-based permission source.
///
/// Reads executable bit from Git index entries.
/// This is the recommended source for Windows when working with Git repositories.
pub struct GitIndexPermissionsSource {
    repo: git2::Repository,
    root: std::path::PathBuf,
}

#[cfg(feature = "git")]
impl GitIndexPermissionsSource {
    pub fn new(repo: git2::Repository, root: std::path::PathBuf) -> Self {
        Self { repo, root }
    }
}

#[cfg(feature = "git")]
impl PermissionsSource for GitIndexPermissionsSource {
    fn executable_of(&self, path: &Path) -> Result<EntryExec, SwhidError> {
        // Get relative path from repo root
        let rel_path = path
            .strip_prefix(&self.root)
            .map_err(|_| SwhidError::InvalidFormat(format!(
                "Path {} is not under repository root {}",
                path.display(),
                self.root.display()
            )))?;

        // Convert to forward slashes for Git
        let git_path = rel_path.to_string_lossy().replace('\\', "/");

        let index = self.repo.index().map_err(|e| {
            SwhidError::Io(std::io::Error::other(format!("Failed to read Git index: {}", e)))
        })?;

        // Find entry in index
        if let Some(entry) = index.get_path(Path::new(&git_path), 0) {
            let mode = entry.mode();
            // Git modes: 100644 = regular, 100755 = executable
            let executable = (mode & 0o111) != 0 || mode == 0o100755;
            Ok(EntryExec::Known(executable))
        } else {
            // Not in index, return Unknown
            Ok(EntryExec::Unknown)
        }
    }
}

#[cfg(feature = "git")]
/// Git tree-based permission source (HEAD).
///
/// Reads executable bit from committed tree objects.
/// This reflects the committed state rather than the working directory.
pub struct GitTreePermissionsSource {
    repo: git2::Repository,
    root: std::path::PathBuf,
}

#[cfg(feature = "git")]
impl GitTreePermissionsSource {
    pub fn new(repo: git2::Repository, root: std::path::PathBuf) -> Self {
        Self { repo, root }
    }
}

#[cfg(feature = "git")]
impl PermissionsSource for GitTreePermissionsSource {
    fn executable_of(&self, path: &Path) -> Result<EntryExec, SwhidError> {
        use git2::ObjectType as GitObjectType;

        // Get relative path from repo root
        let rel_path = path
            .strip_prefix(&self.root)
            .map_err(|_| SwhidError::InvalidFormat(format!(
                "Path {} is not under repository root {}",
                path.display(),
                self.root.display()
            )))?;

        // Convert to forward slashes for Git
        let git_path = rel_path.to_string_lossy().replace('\\', "/");

        // Get HEAD tree
        let head = self.repo.head().map_err(|e| {
            SwhidError::Io(std::io::Error::other(format!("Failed to get HEAD: {}", e)))
        })?;
        let commit = head.peel_to_commit().map_err(|e| {
            SwhidError::Io(std::io::Error::other(format!("Failed to get commit: {}", e)))
        })?;
        let tree = commit.tree().map_err(|e| {
            SwhidError::Io(std::io::Error::other(format!("Failed to get tree: {}", e)))
        })?;

        // Walk tree to find entry
        let path_parts: Vec<&str> = git_path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_tree = tree;

        for (i, part) in path_parts.iter().enumerate() {
            if let Some(entry) = current_tree.get_path(Path::new(part)) {
                if i == path_parts.len() - 1 {
                    // This is the file we're looking for
                    let mode = entry.filemode();
                    let executable = (mode & 0o111) != 0 || mode == 0o100755;
                    return Ok(EntryExec::Known(executable));
                } else {
                    // Navigate into subdirectory
                    let obj = entry.to_object(&self.repo).map_err(|e| {
                        SwhidError::Io(std::io::Error::other(format!(
                            "Failed to get tree object: {}",
                            e
                        )))
                    })?;
                    current_tree = obj.as_tree().ok_or_else(|| {
                        SwhidError::InvalidFormat(format!(
                            "Expected tree at {}",
                            path_parts[..=i].join("/")
                        ))
                    })?;
                }
            } else {
                // Not found in tree
                return Ok(EntryExec::Unknown);
            }
        }

        Ok(EntryExec::Unknown)
    }
}

/// Manifest-based permission source.
///
/// Reads executable bit from a sidecar permission manifest file (TOML format).
pub struct ManifestPermissionsSource {
    manifest: std::collections::HashMap<String, bool>,
}

impl ManifestPermissionsSource {
    /// Load permission manifest from a TOML file.
    pub fn load(path: &Path) -> Result<Self, SwhidError> {
        use std::fs;
        use std::io::Read;

        let mut file = fs::File::open(path).map_err(|e| {
            SwhidError::Io(std::io::Error::other(format!(
                "Failed to open manifest file {}: {}",
                path.display(),
                e
            )))
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            SwhidError::Io(std::io::Error::other(format!(
                "Failed to read manifest file {}: {}",
                path.display(),
                e
            )))
        })?;

        Self::parse(&contents)
    }

    /// Parse permission manifest from TOML string.
    ///
    /// Expected format:
    /// ```toml
    /// [[file]]
    /// path = "bin/tool"
    /// executable = true
    ///
    /// [[file]]
    /// path = "scripts/run.sh"
    /// executable = true
    /// ```
    pub fn parse(toml: &str) -> Result<Self, SwhidError> {
        // Simple TOML parser for the specific format we need
        // This avoids adding a TOML dependency for now
        let mut manifest = std::collections::HashMap::new();

        // Split by [[file]] sections
        let sections: Vec<&str> = toml.split("[[file]]").collect();
        for section in sections.iter().skip(1) {
            // Extract path and executable
            let mut path: Option<String> = None;
            let mut executable: Option<bool> = None;

            for line in section.lines() {
                let line = line.trim();
                if line.starts_with("path") {
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line.rfind('"') {
                            if end > start {
                                path = Some(line[start + 1..end].to_string());
                            }
                        }
                    }
                } else if line.starts_with("executable") {
                    executable = Some(line.contains("true"));
                }
            }

            if let (Some(p), Some(exec)) = (path, executable) {
                // Normalize path (forward slashes, reject .. and absolute)
                let normalized = Self::normalize_path(&p)?;
                manifest.insert(normalized, exec);
            }
        }

        Ok(Self { manifest })
    }

    fn normalize_path(path: &str) -> Result<String, SwhidError> {
        // Reject absolute paths
        if path.starts_with('/') || (cfg!(windows) && path.contains(':')) {
            return Err(SwhidError::InvalidFormat(format!(
                "Manifest contains absolute path: {}",
                path
            )));
        }

        // Reject .. segments
        if path.contains("..") {
            return Err(SwhidError::InvalidFormat(format!(
                "Manifest contains '..' in path: {}",
                path
            )));
        }

        // Normalize to forward slashes
        Ok(path.replace('\\', "/"))
    }
}

impl PermissionsSource for ManifestPermissionsSource {
    fn executable_of(&self, path: &Path) -> Result<EntryExec, SwhidError> {
        // Normalize path for lookup
        let path_str = path.to_string_lossy().replace('\\', "/");
        if let Some(&executable) = self.manifest.get(&path_str) {
            Ok(EntryExec::Known(executable))
        } else {
            Ok(EntryExec::Unknown)
        }
    }
}

/// Auto-detecting permission source.
///
/// Automatically selects the best available source:
/// 1. Git index (if Git repo and git feature enabled)
/// 2. Filesystem (Unix)
/// 3. Filesystem (Windows, will return Unknown)
pub struct AutoPermissionsSource {
    inner: Box<dyn PermissionsSource>,
}

impl AutoPermissionsSource {
    /// Create an auto-detecting permission source.
    ///
    /// Attempts to discover a Git repository starting from `root` and walking up.
    /// If found and `git` feature is enabled, uses Git index source.
    /// Otherwise, falls back to filesystem source.
    pub fn new(_root: &Path) -> Result<Self, SwhidError> {
        // Try to find Git repository
        #[cfg(feature = "git")]
        {
            let mut current = Some(_root);
            while let Some(path) = current {
                let git_dir = path.join(".git");
                if git_dir.exists() {
                    match git2::Repository::open(path) {
                        Ok(repo) => {
                            return Ok(Self {
                                inner: Box::new(GitIndexPermissionsSource::new(
                                    repo,
                                    path.to_path_buf(),
                                )),
                            });
                        }
                        Err(_) => {
                            // Continue searching
                        }
                    }
                }
                current = path.parent();
            }
        }

        // Fall back to filesystem
        Ok(Self {
            inner: Box::new(FilesystemPermissionsSource),
        })
    }
}

impl PermissionsSource for AutoPermissionsSource {
    fn executable_of(&self, path: &Path) -> Result<EntryExec, SwhidError> {
        self.inner.executable_of(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_perms_to_git_mode_string() {
        assert_eq!(
            EntryPerms::File { executable: false }.to_git_mode_string(),
            "100644"
        );
        assert_eq!(
            EntryPerms::File { executable: true }.to_git_mode_string(),
            "100755"
        );
        assert_eq!(EntryPerms::Directory.to_git_mode_string(), "040000");
        assert_eq!(EntryPerms::Symlink.to_git_mode_string(), "120000");
        assert_eq!(EntryPerms::RevisionRef.to_git_mode_string(), "160000");
    }

    #[test]
    fn entry_perms_to_swh_mode_u32() {
        assert_eq!(
            EntryPerms::File { executable: false }.to_swh_mode_u32(),
            0o100644
        );
        assert_eq!(
            EntryPerms::File { executable: true }.to_swh_mode_u32(),
            0o100755
        );
        assert_eq!(EntryPerms::Directory.to_swh_mode_u32(), 0o040000);
        assert_eq!(EntryPerms::Symlink.to_swh_mode_u32(), 0o120000);
        assert_eq!(EntryPerms::RevisionRef.to_swh_mode_u32(), 0o160000);
    }

    #[test]
    fn entry_perms_from_mode() {
        assert_eq!(
            EntryPerms::from_mode(0o100644).unwrap(),
            EntryPerms::File { executable: false }
        );
        assert_eq!(
            EntryPerms::from_mode(0o100755).unwrap(),
            EntryPerms::File { executable: true }
        );
        assert_eq!(
            EntryPerms::from_mode(0o040000).unwrap(),
            EntryPerms::Directory
        );
        assert_eq!(
            EntryPerms::from_mode(0o120000).unwrap(),
            EntryPerms::Symlink
        );
        assert_eq!(
            EntryPerms::from_mode(0o160000).unwrap(),
            EntryPerms::RevisionRef
        );
    }

    #[test]
    fn resolve_file_permissions_known() {
        let path = Path::new("test.txt");
        assert_eq!(
            resolve_file_permissions(EntryExec::Known(true), PermissionPolicy::Strict, path)
                .unwrap(),
            EntryPerms::File { executable: true }
        );
        assert_eq!(
            resolve_file_permissions(EntryExec::Known(false), PermissionPolicy::Strict, path)
                .unwrap(),
            EntryPerms::File { executable: false }
        );
    }

    #[test]
    fn resolve_file_permissions_unknown_strict() {
        let path = Path::new("test.txt");
        let result = resolve_file_permissions(EntryExec::Unknown, PermissionPolicy::Strict, path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot determine executable bit"));
    }

    #[test]
    fn resolve_file_permissions_unknown_best_effort() {
        let path = Path::new("test.txt");
        assert_eq!(
            resolve_file_permissions(EntryExec::Unknown, PermissionPolicy::BestEffort, path)
                .unwrap(),
            EntryPerms::File { executable: false }
        );
    }
}

