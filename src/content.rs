use std::fs;
use std::path::Path;
use crate::swhid::{Swhid, ObjectType};
use crate::hash::sha1_git_hash;
use crate::error::SwhidError;

/// Content object representing a file
#[derive(Debug, Clone)]
pub struct Content {
    data: Vec<u8>,
    length: usize,
    sha1_git: [u8; 20],
}

impl Content {
    /// Create content from file data
    pub fn from_data(data: Vec<u8>) -> Self {
        let length = data.len();
        let sha1_git = sha1_git_hash(&data);
        
        Self {
            data,
            length,
            sha1_git,
        }
    }

    /// Create content from file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, SwhidError> {
        let data = fs::read(path)?;
        Ok(Self::from_data(data))
    }

    /// Get the raw data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the length
    pub fn length(&self) -> usize {
        self.length
    }

    /// Get the SHA1 Git hash
    pub fn sha1_git(&self) -> &[u8; 20] {
        &self.sha1_git
    }

    /// Compute SWHID for this content
    pub fn swhid(&self) -> Swhid {
        Swhid::new(ObjectType::Content, self.sha1_git)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_content_from_data() {
        let data = b"Hello, World!".to_vec();
        let content = Content::from_data(data.clone());
        
        assert_eq!(content.data(), data.as_slice());
        assert_eq!(content.length(), 13);
        assert_eq!(content.sha1_git().len(), 20);
    }

    #[test]
    fn test_content_from_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let data = b"Test file content";
        fs::write(&temp_file, data).unwrap();
        
        let content = Content::from_file(&temp_file).unwrap();
        assert_eq!(content.data(), data);
        assert_eq!(content.length(), data.len());
    }

    #[test]
    fn test_content_swhid() {
        let data = b"Hello, World!";
        let content = Content::from_data(data.to_vec());
        let swhid = content.swhid();
        
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(swhid.hash(), content.sha1_git());
    }

    #[test]
    fn test_content_known_hashes() {
        // Test known hash values for common content
        let empty_content = Content::from_data(b"".to_vec());
        let empty_swhid = empty_content.swhid();
        
        // Known hash for empty content
        assert_eq!(empty_swhid.hash(), &hex::decode("e69de29bb2d1d6434b8b29ae775ad8c2e48c5391").unwrap()[..]);
        
        let hello_content = Content::from_data(b"Hello, World!".to_vec());
        let hello_swhid = hello_content.swhid();
        
        // Known hash for "Hello, World!"
        assert_eq!(hello_swhid.hash(), &hex::decode("b45ef6fec89518d314f546fd6c3025367b721684").unwrap()[..]);
    }

    #[test]
    fn test_content_large_data() {
        let large_data = vec![b'a'; 10000];
        let content = Content::from_data(large_data.clone());
        
        assert_eq!(content.data(), large_data.as_slice());
        assert_eq!(content.length(), 10000);
        assert_eq!(content.sha1_git().len(), 20);
    }
} 