use std::fmt;
use crate::error::SwhidError;

/// Software Hash object types (Core SWHID)
/// According to the official SWHID specification v1.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectType {
    Content,    // "cnt" - File contents
    Directory,  // "dir" - Directory trees
    Revision,   // "rev" - Git revisions
    Release,    // "rel" - Git releases
    Snapshot,   // "snp" - Git snapshots
}

impl ObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ObjectType::Content => "cnt",
            ObjectType::Directory => "dir",
            ObjectType::Revision => "rev",
            ObjectType::Release => "rel",
            ObjectType::Snapshot => "snp",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, SwhidError> {
        match s {
            "cnt" => Ok(ObjectType::Content),
            "dir" => Ok(ObjectType::Directory),
            "rev" => Ok(ObjectType::Revision),
            "rel" => Ok(ObjectType::Release),
            "snp" => Ok(ObjectType::Snapshot),
            _ => Err(SwhidError::InvalidObjectType(s.to_string())),
        }
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Core Software Hash Identifier
/// Format: swh:1:<object_type>:<40_character_hex_hash>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Swhid {
    object_type: ObjectType,
    hash: [u8; 20],
}

impl Swhid {
    /// Create a new SWHID
    pub fn new(object_type: ObjectType, hash: [u8; 20]) -> Self {
        Self {
            object_type,
            hash,
        }
    }

    /// Get the object type
    pub fn object_type(&self) -> ObjectType {
        self.object_type
    }

    /// Get the hash
    pub fn hash(&self) -> &[u8; 20] {
        &self.hash
    }

    /// Parse SWHID from string
    pub fn from_string(s: &str) -> Result<Self, SwhidError> {
        let parts: Vec<&str> = s.split(':').collect();
        
        if parts.len() != 4 {
            return Err(SwhidError::InvalidFormat(format!(
                "SWHID must have 4 parts, got {}: {}", parts.len(), s
            )));
        }

        // Check namespace
        if parts[0] != "swh" {
            return Err(SwhidError::InvalidNamespace(parts[0].to_string()));
        }

        // Check version
        if parts[1] != "1" {
            return Err(SwhidError::InvalidVersion(parts[1].to_string()));
        }

        // Parse object type
        let object_type = ObjectType::from_str(parts[2])?;

        // Parse hash
        let hash_bytes = hex::decode(parts[3])
            .map_err(|e| SwhidError::InvalidHash(e.to_string()))?;
        
        if hash_bytes.len() != 20 {
            return Err(SwhidError::InvalidHashLength(hash_bytes.len()));
        }

        let mut hash = [0u8; 20];
        hash.copy_from_slice(&hash_bytes);

        Ok(Swhid::new(object_type, hash))
    }
}

impl fmt::Display for Swhid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hash_hex = hex::encode(self.hash);
        write!(f, "swh:1:{}:{}", self.object_type, hash_hex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swhid_new() {
        let hash = [0u8; 20];
        let swhid = Swhid::new(ObjectType::Content, hash);
        
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(swhid.hash(), &hash);
    }

    #[test]
    fn test_swhid_display() {
        let hash = [0u8; 20];
        let swhid = Swhid::new(ObjectType::Directory, hash);
        
        assert_eq!(swhid.to_string(), "swh:1:dir:0000000000000000000000000000000000000000");
    }

    #[test]
    fn test_swhid_from_string() {
        let swhid = Swhid::from_string("swh:1:cnt:0000000000000000000000000000000000000000").unwrap();
        
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(swhid.hash(), &[0u8; 20]);
    }

    #[test]
    fn test_swhid_from_string_invalid() {
        // Invalid format
        assert!(Swhid::from_string("invalid").is_err());
        
        // Invalid namespace
        assert!(Swhid::from_string("invalid:1:cnt:0000000000000000000000000000000000000000").is_err());
        
        // Invalid version
        assert!(Swhid::from_string("swh:2:cnt:0000000000000000000000000000000000000000").is_err());
        
        // Invalid object type
        assert!(Swhid::from_string("swh:1:invalid:0000000000000000000000000000000000000000").is_err());
        
        // Invalid hash length
        assert!(Swhid::from_string("swh:1:cnt:00000000000000000000000000000000000000").is_err());
    }

    #[test]
    fn test_all_object_types() {
        // Test all core SWHID object types
        let hash = [0u8; 20];
        
        let content_swhid = Swhid::new(ObjectType::Content, hash);
        assert_eq!(content_swhid.to_string(), "swh:1:cnt:0000000000000000000000000000000000000000");
        
        let directory_swhid = Swhid::new(ObjectType::Directory, hash);
        assert_eq!(directory_swhid.to_string(), "swh:1:dir:0000000000000000000000000000000000000000");
        
        let revision_swhid = Swhid::new(ObjectType::Revision, hash);
        assert_eq!(revision_swhid.to_string(), "swh:1:rev:0000000000000000000000000000000000000000");
        
        let release_swhid = Swhid::new(ObjectType::Release, hash);
        assert_eq!(release_swhid.to_string(), "swh:1:rel:0000000000000000000000000000000000000000");
        
        let snapshot_swhid = Swhid::new(ObjectType::Snapshot, hash);
        assert_eq!(snapshot_swhid.to_string(), "swh:1:snp:0000000000000000000000000000000000000000");
    }

    #[test]
    fn test_object_type_parsing() {
        // Test parsing all valid object types
        assert_eq!(ObjectType::from_str("cnt").unwrap(), ObjectType::Content);
        assert_eq!(ObjectType::from_str("dir").unwrap(), ObjectType::Directory);
        assert_eq!(ObjectType::from_str("rev").unwrap(), ObjectType::Revision);
        assert_eq!(ObjectType::from_str("rel").unwrap(), ObjectType::Release);
        assert_eq!(ObjectType::from_str("snp").unwrap(), ObjectType::Snapshot);
        
        // Test invalid object type
        assert!(ObjectType::from_str("invalid").is_err());
    }
} 