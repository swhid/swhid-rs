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

/// Qualified Software Hash Identifier
/// Format: swh:1:<object_type>:<hash>[;qualifier=value]*
/// According to the official SWHID specification v1.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualifiedSwhid {
    core: Swhid,
    origin: Option<String>,
    visit: Option<Swhid>,
    anchor: Option<Swhid>,
    path: Option<Vec<u8>>,
    lines: Option<(u32, Option<u32>)>,
    bytes: Option<(u32, Option<u32>)>,
}

impl QualifiedSwhid {
    /// Create a new QualifiedSWHID from a core SWHID
    pub fn new(core: Swhid) -> Self {
        Self {
            core,
            origin: None,
            visit: None,
            anchor: None,
            path: None,
            lines: None,
            bytes: None,
        }
    }

    /// Get the core SWHID
    pub fn core(&self) -> &Swhid {
        &self.core
    }

    /// Get the object type
    pub fn object_type(&self) -> ObjectType {
        self.core.object_type()
    }

    /// Get the hash
    pub fn hash(&self) -> &[u8; 20] {
        self.core.hash()
    }

    /// Set the origin qualifier
    pub fn with_origin(mut self, origin: String) -> Self {
        self.origin = Some(origin);
        self
    }

    /// Set the visit qualifier (must be a snapshot SWHID)
    pub fn with_visit(mut self, visit: Swhid) -> Result<Self, SwhidError> {
        if visit.object_type() != ObjectType::Snapshot {
            return Err(SwhidError::InvalidQualifier(
                "Visit qualifier must be a snapshot SWHID".to_string()
            ));
        }
        self.visit = Some(visit);
        Ok(self)
    }

    /// Set the anchor qualifier (must be dir, rev, rel, or snp)
    pub fn with_anchor(mut self, anchor: Swhid) -> Result<Self, SwhidError> {
        match anchor.object_type() {
            ObjectType::Directory | ObjectType::Revision | 
            ObjectType::Release | ObjectType::Snapshot => {
                self.anchor = Some(anchor);
                Ok(self)
            }
            _ => Err(SwhidError::InvalidQualifier(
                "Anchor qualifier must be dir, rev, rel, or snp SWHID".to_string()
            )),
        }
    }

    /// Set the path qualifier
    pub fn with_path(mut self, path: Vec<u8>) -> Self {
        self.path = Some(path);
        self
    }

    /// Set the lines qualifier
    pub fn with_lines(mut self, start: u32, end: Option<u32>) -> Self {
        self.lines = Some((start, end));
        self
    }

    /// Set the bytes qualifier
    pub fn with_bytes(mut self, start: u32, end: Option<u32>) -> Self {
        self.bytes = Some((start, end));
        self
    }

    /// Get the origin qualifier
    pub fn origin(&self) -> Option<&str> {
        self.origin.as_deref()
    }

    /// Get the visit qualifier
    pub fn visit(&self) -> Option<&Swhid> {
        self.visit.as_ref()
    }

    /// Get the anchor qualifier
    pub fn anchor(&self) -> Option<&Swhid> {
        self.anchor.as_ref()
    }

    /// Get the path qualifier
    pub fn path(&self) -> Option<&[u8]> {
        self.path.as_ref().map(|v| v.as_slice())
    }

    /// Get the lines qualifier
    pub fn lines(&self) -> Option<(u32, Option<u32>)> {
        self.lines
    }

    /// Get the bytes qualifier
    pub fn bytes(&self) -> Option<(u32, Option<u32>)> {
        self.bytes
    }

    /// Parse QualifiedSWHID from string
    pub fn from_string(s: &str) -> Result<Self, SwhidError> {
        // Split by semicolon to separate core SWHID from qualifiers
        let parts: Vec<&str> = s.split(';').collect();
        if parts.is_empty() {
            return Err(SwhidError::InvalidFormat("Empty SWHID string".to_string()));
        }

        // Parse the core SWHID part
        let core = Swhid::from_string(parts[0])?;
        
        let mut qualified = Self::new(core);

        // Parse qualifiers
        for qualifier in &parts[1..] {
            let qualifier_parts: Vec<&str> = qualifier.split('=').collect();
            if qualifier_parts.len() != 2 {
                return Err(SwhidError::InvalidFormat(format!(
                    "Invalid qualifier format: {}", qualifier
                )));
            }

            let key = qualifier_parts[0];
            let value = qualifier_parts[1];

            match key {
                "origin" => {
                    qualified.origin = Some(value.to_string());
                }
                "visit" => {
                    let visit_swhid = Swhid::from_string(value)?;
                    qualified = qualified.with_visit(visit_swhid)?;
                }
                "anchor" => {
                    let anchor_swhid = Swhid::from_string(value)?;
                    qualified = qualified.with_anchor(anchor_swhid)?;
                }
                "path" => {
                    qualified.path = Some(value.as_bytes().to_vec());
                }
                "lines" => {
                    let lines_parts: Vec<&str> = value.split('-').collect();
                    if lines_parts.len() > 2 {
                        return Err(SwhidError::InvalidFormat(format!(
                            "Invalid lines format: {}", value
                        )));
                    }
                    
                    let start = lines_parts[0].parse::<u32>()
                        .map_err(|_| SwhidError::InvalidFormat(format!(
                            "Invalid start line: {}", lines_parts[0]
                        )))?;
                    
                    let end = if lines_parts.len() == 2 {
                        Some(lines_parts[1].parse::<u32>()
                            .map_err(|_| SwhidError::InvalidFormat(format!(
                                "Invalid end line: {}", lines_parts[1]
                            )))?)
                    } else {
                        None
                    };
                    
                    qualified.lines = Some((start, end));
                }
                "bytes" => {
                    let bytes_parts: Vec<&str> = value.split('-').collect();
                    if bytes_parts.len() > 2 {
                        return Err(SwhidError::InvalidFormat(format!(
                            "Invalid bytes format: {}", value
                        )));
                    }
                    
                    let start = bytes_parts[0].parse::<u32>()
                        .map_err(|_| SwhidError::InvalidFormat(format!(
                            "Invalid start byte: {}", bytes_parts[0]
                        )))?;
                    
                    let end = if bytes_parts.len() == 2 {
                        Some(bytes_parts[1].parse::<u32>()
                            .map_err(|_| SwhidError::InvalidFormat(format!(
                                "Invalid end byte: {}", bytes_parts[1]
                            )))?)
                    } else {
                        None
                    };
                    
                    qualified.bytes = Some((start, end));
                }
                _ => {
                    return Err(SwhidError::UnknownQualifier(key.to_string()));
                }
            }
        }

        Ok(qualified)
    }
}

impl fmt::Display for QualifiedSwhid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Start with core SWHID
        write!(f, "{}", self.core)?;

        // Add qualifiers in the specified order: origin, visit, anchor, path, lines
        if let Some(ref origin) = self.origin {
            write!(f, ";origin={}", origin)?;
        }

        if let Some(ref visit) = self.visit {
            write!(f, ";visit={}", visit)?;
        }

        if let Some(ref anchor) = self.anchor {
            write!(f, ";anchor={}", anchor)?;
        }

        if let Some(ref path) = self.path {
            write!(f, ";path={}", String::from_utf8_lossy(path))?;
        }

        if let Some((start, end)) = self.lines {
            match end {
                Some(end) => write!(f, ";lines={}-{}", start, end)?,
                None => write!(f, ";lines={}", start)?,
            }
        }

        if let Some((start, end)) = self.bytes {
            match end {
                Some(end) => write!(f, ";bytes={}-{}", start, end)?,
                None => write!(f, ";bytes={}", start)?,
            }
        }

        Ok(())
    }
}

impl From<Swhid> for QualifiedSwhid {
    fn from(swhid: Swhid) -> Self {
        Self::new(swhid)
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

    // QualifiedSWHID tests
    #[test]
    fn test_qualified_swhid_new() {
        let core = Swhid::new(ObjectType::Content, [0u8; 20]);
        let qualified = QualifiedSwhid::new(core.clone());
        
        assert_eq!(qualified.core(), &core);
        assert_eq!(qualified.origin(), None);
        assert_eq!(qualified.visit(), None);
        assert_eq!(qualified.anchor(), None);
        assert_eq!(qualified.path(), None);
        assert_eq!(qualified.lines(), None);
        assert_eq!(qualified.bytes(), None);
    }

    #[test]
    fn test_qualified_swhid_with_origin() {
        let core = Swhid::new(ObjectType::Content, [0u8; 20]);
        let qualified = QualifiedSwhid::new(core)
            .with_origin("https://github.com/user/repo".to_string());
        
        assert_eq!(qualified.origin(), Some("https://github.com/user/repo"));
    }

    #[test]
    fn test_qualified_swhid_with_visit() {
        let core = Swhid::new(ObjectType::Content, [0u8; 20]);
        let visit = Swhid::new(ObjectType::Snapshot, [1u8; 20]);
        
        let qualified = QualifiedSwhid::new(core)
            .with_visit(visit.clone()).unwrap();
        
        assert_eq!(qualified.visit(), Some(&visit));
    }

    #[test]
    fn test_qualified_swhid_with_invalid_visit() {
        let core = Swhid::new(ObjectType::Content, [0u8; 20]);
        let invalid_visit = Swhid::new(ObjectType::Content, [1u8; 20]);
        
        let result = QualifiedSwhid::new(core)
            .with_visit(invalid_visit);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_qualified_swhid_with_anchor() {
        let core = Swhid::new(ObjectType::Content, [0u8; 20]);
        let anchor = Swhid::new(ObjectType::Directory, [1u8; 20]);
        
        let qualified = QualifiedSwhid::new(core)
            .with_anchor(anchor.clone()).unwrap();
        
        assert_eq!(qualified.anchor(), Some(&anchor));
    }

    #[test]
    fn test_qualified_swhid_with_invalid_anchor() {
        let core = Swhid::new(ObjectType::Content, [0u8; 20]);
        let invalid_anchor = Swhid::new(ObjectType::Content, [1u8; 20]);
        
        let result = QualifiedSwhid::new(core)
            .with_anchor(invalid_anchor);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_qualified_swhid_with_path() {
        let core = Swhid::new(ObjectType::Content, [0u8; 20]);
        let path = b"/src/main.rs".to_vec();
        
        let qualified = QualifiedSwhid::new(core)
            .with_path(path.clone());
        
        assert_eq!(qualified.path(), Some(path.as_slice()));
    }

    #[test]
    fn test_qualified_swhid_with_lines() {
        let core = Swhid::new(ObjectType::Content, [0u8; 20]);
        
        // Single line
        let qualified = QualifiedSwhid::new(core.clone())
            .with_lines(10, None);
        assert_eq!(qualified.lines(), Some((10, None)));
        
        // Line range
        let qualified = QualifiedSwhid::new(core)
            .with_lines(10, Some(20));
        assert_eq!(qualified.lines(), Some((10, Some(20))));
    }

    #[test]
    fn test_qualified_swhid_with_bytes() {
        let core = Swhid::new(ObjectType::Content, [0u8; 20]);
        
        // Single byte
        let qualified = QualifiedSwhid::new(core.clone())
            .with_bytes(10, None);
        assert_eq!(qualified.bytes(), Some((10, None)));
        
        // Byte range
        let qualified = QualifiedSwhid::new(core)
            .with_bytes(10, Some(20));
        assert_eq!(qualified.bytes(), Some((10, Some(20))));
    }

    #[test]
    fn test_qualified_swhid_display() {
        let core = Swhid::new(ObjectType::Content, [0u8; 20]);
        let qualified = QualifiedSwhid::new(core)
            .with_origin("https://github.com/user/repo".to_string())
            .with_path(b"/src/main.rs".to_vec())
            .with_lines(10, Some(20))
            .with_bytes(5, Some(10));
        
        let expected = "swh:1:cnt:0000000000000000000000000000000000000000;origin=https://github.com/user/repo;path=/src/main.rs;lines=10-20;bytes=5-10";
        assert_eq!(qualified.to_string(), expected);
    }

    #[test]
    fn test_qualified_swhid_from_string() {
        let s = "swh:1:cnt:0000000000000000000000000000000000000000;origin=https://github.com/user/repo;path=/src/main.rs;lines=10-20;bytes=5-10";
        let qualified = QualifiedSwhid::from_string(s).unwrap();
        
        assert_eq!(qualified.origin(), Some("https://github.com/user/repo"));
        assert_eq!(qualified.path(), Some(b"/src/main.rs".as_slice()));
        assert_eq!(qualified.lines(), Some((10, Some(20))));
        assert_eq!(qualified.bytes(), Some((5, Some(10))));
    }

    #[test]
    fn test_qualified_swhid_from_string_invalid() {
        // Invalid qualifier format
        assert!(QualifiedSwhid::from_string("swh:1:cnt:0000000000000000000000000000000000000000;invalid").is_err());
        
        // Invalid lines format
        assert!(QualifiedSwhid::from_string("swh:1:cnt:0000000000000000000000000000000000000000;lines=invalid").is_err());
        
        // Invalid bytes format
        assert!(QualifiedSwhid::from_string("swh:1:cnt:0000000000000000000000000000000000000000;bytes=invalid").is_err());
        
        // Unknown qualifier
        assert!(QualifiedSwhid::from_string("swh:1:cnt:0000000000000000000000000000000000000000;unknown=value").is_err());
    }

    #[test]
    fn test_qualified_swhid_from_swhid() {
        let core = Swhid::new(ObjectType::Content, [0u8; 20]);
        let qualified: QualifiedSwhid = core.clone().into();
        
        assert_eq!(qualified.core(), &core);
        assert_eq!(qualified.object_type(), ObjectType::Content);
    }
} 