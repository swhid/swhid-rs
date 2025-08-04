use std::path::Path;
use crate::swhid::Swhid;
use crate::error::SwhidError;
use crate::content::Content;
use crate::directory::Directory;

/// Minimal SWHID computer for core functionality
#[derive(Clone, Default)]
pub struct SwhidComputer {
    pub follow_symlinks: bool,
    pub exclude_patterns: Vec<String>,
}

impl SwhidComputer {
    /// Create a new SWHID computer with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to follow symlinks
    pub fn with_follow_symlinks(mut self, follow_symlinks: bool) -> Self {
        self.follow_symlinks = follow_symlinks;
        self
    }

    /// Set exclude patterns
    pub fn with_exclude_patterns(mut self, exclude_patterns: &[String]) -> Self {
        self.exclude_patterns = exclude_patterns.to_vec();
        self
    }

    /// Compute SWHID for content bytes
    pub fn compute_content_swhid(&self, content: &[u8]) -> Result<Swhid, SwhidError> {
        let content_obj = Content::from_data(content.to_vec());
        Ok(content_obj.swhid())
    }

    /// Compute SWHID for a file
    pub fn compute_file_swhid<P: AsRef<Path>>(&self, path: P) -> Result<Swhid, SwhidError> {
        let content = Content::from_file(path)?;
        Ok(content.swhid())
    }

    /// Compute SWHID for a directory
    pub fn compute_directory_swhid<P: AsRef<Path>>(&self, path: P) -> Result<Swhid, SwhidError> {
        let mut dir = Directory::from_disk(path, &self.exclude_patterns, self.follow_symlinks)?;
        Ok(dir.swhid())
    }

    /// Auto-detect object type and compute SWHID
    pub fn compute_swhid<P: AsRef<Path>>(&self, path: P) -> Result<Swhid, SwhidError> {
        let path = path.as_ref();
        
        if path.is_symlink() {
            if self.follow_symlinks {
                // Follow the symlink and compute SWHID of the target
                let target = std::fs::read_link(path)?;
                let resolved_target = if target.is_relative() {
                    path.parent().unwrap().join(&target)
                } else {
                    target
                };
                self.compute_swhid(resolved_target)
            } else {
                // Hash the symlink target as content
                let target = std::fs::read_link(path)?;
                let target_bytes = target.to_string_lossy().as_bytes().to_vec();
                let content = Content::from_data(target_bytes);
                Ok(content.swhid())
            }
        } else if path.is_file() {
            self.compute_file_swhid(path)
        } else if path.is_dir() {
            self.compute_directory_swhid(path)
        } else {
            Err(SwhidError::InvalidInput("Path is neither file nor directory".to_string()))
        }
    }

    /// Verify that a SWHID matches the computed SWHID for a path
    pub fn verify_swhid<P: AsRef<Path>>(&self, path: P, expected_swhid: &str) -> Result<bool, SwhidError> {
        // Parse the expected SWHID
        let expected = Swhid::from_string(expected_swhid)?;
        
        // Compute the actual SWHID
        let actual = self.compute_swhid(path)?;
        
        Ok(expected == actual)
    }
}
