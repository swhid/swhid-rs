use crate::core::{ObjectType, Swhid};
use crate::types::SwhidVersion;
use crate::hash::{hash_content, hash_content_with};
use crate::config::HashConfig;

/// SWHID v1.2 content object for computing content SWHIDs.
///
/// This struct represents file content data and provides methods to compute
/// SWHID v1.2 compliant content identifiers according to the specification.
#[derive(Debug, Clone)]
pub struct Content<B: AsRef<[u8]> = Box<[u8]>> {
    bytes: B,
}

impl<B: AsRef<[u8]>> Content<B> {
    /// Create a new Content object from byte data.
    ///
    /// This implements SWHID v1.2 content object creation for any byte data.
    pub fn from_bytes(bytes: B) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }

    pub fn len(&self) -> usize {
        self.bytes.as_ref().len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.as_ref().is_empty()
    }

    /// Compute the SWHID v1.2 content identifier for this content.
    ///
    /// This implements the SWHID v1.2 content hashing algorithm, which
    /// is compatible with Git's blob format for content objects.
    pub fn swhid(&self) -> Swhid {
        let digest = hash_content(self.bytes.as_ref());
        Swhid::new_v1(ObjectType::Content, digest)
    }

    /// Compute the SWHID content identifier using the specified hash configuration.
    ///
    /// This allows computing SWHIDs with different hash functions (SHA1, SHA256, etc.)
    /// and serialization formats (hex, base64, etc.) for v2 experimentation.
    pub fn swhid_with_config(&self, config: &HashConfig) -> Swhid {
        let digest = hash_content_with(self.bytes.as_ref(), config.hash_function.as_ref());
        Swhid::new(ObjectType::Content, digest, config.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::HashConfig;

    #[test]
    fn content_swhid_v1() {
        let content = Content::from_bytes(b"test");
        let swhid = content.swhid();
        assert_eq!(swhid.version(), SwhidVersion::V1);
        assert_eq!(swhid.digest_bytes().len(), 20);
    }

    #[test]
    fn content_swhid_with_config_v1() {
        let content = Content::from_bytes(b"test");
        let config = HashConfig::v1();
        let swhid = content.swhid_with_config(&config);
        assert_eq!(swhid.version(), SwhidVersion::V1);
        assert_eq!(swhid.digest_bytes().len(), 20);
        // Should match regular swhid() for v1
        assert_eq!(swhid.digest_bytes(), content.swhid().digest_bytes());
    }

    #[test]
    fn content_swhid_with_config_v2_sha256_hex() {
        let content = Content::from_bytes(b"test");
        let config = HashConfig::v2_sha256_hex();
        let swhid = content.swhid_with_config(&config);
        assert_eq!(swhid.version(), SwhidVersion::V2);
        assert_eq!(swhid.digest_bytes().len(), 32);
    }

    #[test]
    fn content_swhid_with_config_v2_sha256_base64() {
        let content = Content::from_bytes(b"test");
        let config = HashConfig::v2_sha256_base64();
        let swhid = content.swhid_with_config(&config);
        assert_eq!(swhid.version(), SwhidVersion::V2);
        assert_eq!(swhid.digest_bytes().len(), 32);
        // Both base64 and hex configs should produce the same digest (both SHA256)
        // The serializer in config affects how the digest is encoded in the SWHID string,
        // but for now Display always uses hex, so both will show hex in the string.
        let hex_config = HashConfig::v2_sha256_hex();
        let hex_swhid = content.swhid_with_config(&hex_config);
        // The digests should be identical (both SHA256 of the same content)
        assert_eq!(swhid.digest_bytes(), hex_swhid.digest_bytes());
    }

    #[test]
    fn content_swhid_different_hashes() {
        let content = Content::from_bytes(b"Hello, World!");
        let v1_swhid = content.swhid();
        let v2_config = HashConfig::v2_sha256_hex();
        let v2_swhid = content.swhid_with_config(&v2_config);
        
        assert_ne!(v1_swhid.digest_bytes(), v2_swhid.digest_bytes());
        assert_eq!(v1_swhid.version(), SwhidVersion::V1);
        assert_eq!(v2_swhid.version(), SwhidVersion::V2);
    }

    #[test]
    fn content_swhid_all_serializers() {
        let content = Content::from_bytes(b"test data");
        
        let hex_config = HashConfig::v2_sha256_hex();
        let base64_config = HashConfig::v2_sha256_base64();
        let base64url_config = HashConfig::v2_sha256_base64url();
        let base32_config = HashConfig::v2_sha256_base32();
        let base32hex_config = HashConfig::v2_sha256_base32hex();
        let z85_config = HashConfig::v2_sha256_z85();
        
        let hex_swhid = content.swhid_with_config(&hex_config);
        let base64_swhid = content.swhid_with_config(&base64_config);
        let base64url_swhid = content.swhid_with_config(&base64url_config);
        let base32_swhid = content.swhid_with_config(&base32_config);
        let base32hex_swhid = content.swhid_with_config(&base32hex_config);
        let z85_swhid = content.swhid_with_config(&z85_config);
        
        // All should have the same digest bytes (same hash function)
        assert_eq!(hex_swhid.digest_bytes(), base64_swhid.digest_bytes());
        assert_eq!(hex_swhid.digest_bytes(), base64url_swhid.digest_bytes());
        assert_eq!(hex_swhid.digest_bytes(), base32_swhid.digest_bytes());
        assert_eq!(hex_swhid.digest_bytes(), base32hex_swhid.digest_bytes());
        assert_eq!(hex_swhid.digest_bytes(), z85_swhid.digest_bytes());
        
        // All should be version 2
        assert_eq!(hex_swhid.version(), SwhidVersion::V2);
        assert_eq!(base64_swhid.version(), SwhidVersion::V2);
        assert_eq!(base64url_swhid.version(), SwhidVersion::V2);
        assert_eq!(base32_swhid.version(), SwhidVersion::V2);
        assert_eq!(base32hex_swhid.version(), SwhidVersion::V2);
        assert_eq!(z85_swhid.version(), SwhidVersion::V2);
    }
}
