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
        let mut dir = Directory::from_disk(path, &self.exclude_patterns)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use crate::swhid::ObjectType;

    #[test]
    fn test_swhid_computer_new() {
        let computer = SwhidComputer::new();
        assert_eq!(computer.follow_symlinks, false);
        assert!(computer.exclude_patterns.is_empty());
    }

    #[test]
    fn test_swhid_computer_with_follow_symlinks() {
        let computer = SwhidComputer::new().with_follow_symlinks(true);
        assert_eq!(computer.follow_symlinks, true);
    }

    #[test]
    fn test_swhid_computer_with_exclude_patterns() {
        let patterns = vec!["*.tmp".to_string(), "*.log".to_string()];
        let computer = SwhidComputer::new().with_exclude_patterns(&patterns);
        assert_eq!(computer.exclude_patterns, patterns);
    }

    #[test]
    fn test_swhid_computer_compute_content_swhid() {
        let computer = SwhidComputer::new();
        let content = b"test content";
        let swhid = computer.compute_content_swhid(content).unwrap();
        
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(swhid.hash().len(), 20);
    }

    #[test]
    fn test_swhid_computer_compute_file_swhid() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"test content").unwrap();

        let computer = SwhidComputer::new();
        let swhid = computer.compute_file_swhid(&file_path).unwrap();
        
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(swhid.hash().len(), 20);
    }

    #[test]
    fn test_swhid_computer_compute_directory_swhid() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"test content").unwrap();

        let computer = SwhidComputer::new();
        let swhid = computer.compute_directory_swhid(temp_dir.path()).unwrap();
        
        assert_eq!(swhid.object_type(), ObjectType::Directory);
        assert_eq!(swhid.hash().len(), 20);
    }

    #[test]
    fn test_swhid_computer_auto_detect_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"test content").unwrap();

        let computer = SwhidComputer::new();
        let swhid = computer.compute_swhid(&file_path).unwrap();
        
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(swhid.hash().len(), 20);
    }

    #[test]
    fn test_swhid_computer_auto_detect_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"test content").unwrap();

        let computer = SwhidComputer::new();
        let swhid = computer.compute_swhid(temp_dir.path()).unwrap();
        
        assert_eq!(swhid.object_type(), ObjectType::Directory);
        assert_eq!(swhid.hash().len(), 20);
    }

    #[test]
    fn test_swhid_computer_symlink_handling() {
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().join("target.txt");
        fs::write(&target_path, b"target content").unwrap();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let link_path = temp_dir.path().join("link.txt");
            symlink("target.txt", &link_path).unwrap();

            let computer = SwhidComputer::new();
            let swhid = computer.compute_swhid(&link_path).unwrap();
            
            // Should hash the symlink target string by default
            assert_eq!(swhid.object_type(), ObjectType::Content);
            assert_eq!(swhid.hash().len(), 20);
        }
    }

    #[test]
    fn test_swhid_computer_verification() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"test content").unwrap();

        let computer = SwhidComputer::new();
        let swhid = computer.compute_file_swhid(&file_path).unwrap();
        
        let is_valid = computer.verify_swhid(&file_path, &swhid.to_string()).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_swhid_computer_verification_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"test content").unwrap();

        let computer = SwhidComputer::new();
        let wrong_swhid = "swh:1:cnt:0000000000000000000000000000000000000000";
        
        let is_valid = computer.verify_swhid(&file_path, wrong_swhid).unwrap();
        assert!(!is_valid);
    }
}
