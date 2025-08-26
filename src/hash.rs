use sha1_checked::{Sha1, Digest};

/// Git-style SHA1 hash computation (collision-resistant)
/// Uses SHA1-checked to prevent SHATTERED-style attacks as required by SWHID spec
pub fn sha1_git_hash(data: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    let header = format!("blob {}\0", data.len());
    hasher.update(header.as_bytes());
    hasher.update(data);
    hasher.finalize().into()
}

/// Standard SHA1 hash computation (collision-resistant)
pub fn sha1_hash(data: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Git object header formatting
pub fn git_object_header(git_type: &str, length: usize) -> Vec<u8> {
    format!("{} {}\0", git_type, length).into_bytes()
}

/// Hash a Git object (header + data) using SHA1-checked
/// This ensures compatibility with Git's SHA1DC usage and SWHID spec compliance
pub fn hash_git_object(git_type: &str, data: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    let header = git_object_header(git_type, data.len());
    hasher.update(&header);
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_git_hash() {
        let data = b"Hello, World!";
        let hash = sha1_git_hash(data);
        
        // The hash should be different from a regular SHA1
        let regular_hash = sha1_hash(data);
        assert_ne!(hash, regular_hash);
        
        // Hash should be 20 bytes
        assert_eq!(hash.len(), 20);
    }

    #[test]
    fn test_sha1_git_hash_known_values() {
        // Test known Git hash values
        let empty_data = b"";
        let empty_hash = sha1_git_hash(empty_data);
        
        // Known hash for empty content (Git blob)
        let expected_empty: [u8; 20] = hex::decode("e69de29bb2d1d6434b8b29ae775ad8c2e48c5391").unwrap().try_into().unwrap();
        assert_eq!(empty_hash, expected_empty);
        
        let hello_data = b"Hello, World!";
        let hello_hash = sha1_git_hash(hello_data);
        
        // Known hash for "Hello, World!" (Git blob)
        let expected_hello: [u8; 20] = hex::decode("b45ef6fec89518d314f546fd6c3025367b721684").unwrap().try_into().unwrap();
        assert_eq!(hello_hash, expected_hello);
    }

    #[test]
    fn test_git_object_header() {
        let header = git_object_header("blob", 1234);
        assert_eq!(header, b"blob 1234\0");
        
        let header = git_object_header("tree", 0);
        assert_eq!(header, b"tree 0\0");
        
        let header = git_object_header("commit", 567);
        assert_eq!(header, b"commit 567\0");
    }

    #[test]
    fn test_hash_git_object() {
        let data = b"test data";
        let hash = hash_git_object("blob", data);
        
        // Should be same as sha1_git_hash for blob type
        let expected = sha1_git_hash(data);
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_hash_git_object_different_types() {
        let data = b"test data";
        let blob_hash = hash_git_object("blob", data);
        let tree_hash = hash_git_object("tree", data);
        
        // Different Git object types should produce different hashes
        assert_ne!(blob_hash, tree_hash);
    }

    #[test]
    fn test_sha1_hash() {
        let data = b"test data";
        let hash = sha1_hash(data);
        
        // Should be 20 bytes
        assert_eq!(hash.len(), 20);
        
        // Should be deterministic
        let hash2 = sha1_hash(data);
        assert_eq!(hash, hash2);
    }


} 