use crate::hash::{HashFunction, Sha1Hash, Sha256Hash};
use crate::serialization::{DigestSerializer, HexSerializer, Base64Serializer, Base64UrlSerializer, Base32Serializer, Base32HexSerializer, Z85Serializer};

/// Configuration bundling a hash function and serialization format.
///
/// This struct combines a hash function (SHA1, SHA256, etc.) with a
/// serialization format (hex, base64, base32, z85, etc.) to define how SWHIDs are
/// computed and encoded. This enables SWHID v2 experimentation while
/// maintaining backward compatibility with v1.
///
/// # Serialization Format Compactness
///
/// For a 32-byte SHA256 digest, the encoded length varies by format:
///
/// | Format     | Length | Use Case                          |
/// |------------|--------|-----------------------------------|
/// | hex        | 64     | Default, Git-compatible           |
/// | base64     | 44     | Standard Base64, compact          |
/// | base64url  | 43     | URL-safe, no padding              |
/// | base32     | 52     | RFC 4648 standard                 |
/// | base32hex  | 52     | Base32hex variant                 |
/// | z85        | 40     | Most compact, ZeroMQ variant      |
///
/// # Examples
///
/// ```
/// use swhid::config::HashConfig;
/// use swhid::Content;
///
/// // V1 (default): SHA1 + hex
/// let v1_config = HashConfig::v1();
/// let content = Content::from_bytes(b"Hello");
/// let v1_swhid = content.swhid_with_config(&v1_config);
///
/// // V2 with different serialization formats
/// let hex_config = HashConfig::v2_sha256_hex();
/// let base64_config = HashConfig::v2_sha256_base64();
/// let z85_config = HashConfig::v2_sha256_z85();
///
/// // All produce the same digest bytes (same hash function)
/// let hex_swhid = content.swhid_with_config(&hex_config);
/// let base64_swhid = content.swhid_with_config(&base64_config);
/// let z85_swhid = content.swhid_with_config(&z85_config);
///
/// assert_eq!(hex_swhid.digest_bytes(), base64_swhid.digest_bytes());
/// assert_eq!(hex_swhid.digest_bytes(), z85_swhid.digest_bytes());
/// ```
pub struct HashConfig {
    pub hash_function: Box<dyn HashFunction>,
    pub serializer: Box<dyn DigestSerializer>,
    pub version: String,
}

impl HashConfig {
    /// Create a new HashConfig with the specified components.
    pub fn new(
        hash_function: Box<dyn HashFunction>,
        serializer: Box<dyn DigestSerializer>,
        version: String,
    ) -> Self {
        Self {
            hash_function,
            serializer,
            version,
        }
    }

    /// Create v1 configuration (SHA1 + hex).
    ///
    /// This is the default configuration for SWHID v1 and maintains
    /// backward compatibility with existing implementations.
    pub fn v1() -> Self {
        Self::new(
            Box::new(Sha1Hash::new()),
            Box::new(HexSerializer::new()),
            "1".to_string(),
        )
    }

    /// Create v2 configuration with SHA256 + hex.
    ///
    /// This configuration uses SHA256 for enhanced security while
    /// maintaining hex serialization for compatibility with Git OIDs.
    pub fn v2_sha256_hex() -> Self {
        Self::new(
            Box::new(Sha256Hash::new()),
            Box::new(HexSerializer::new()),
            "2".to_string(),
        )
    }

    /// Create v2 configuration with SHA256 + base64.
    ///
    /// This configuration uses SHA256 with base64 serialization for
    /// a more compact representation (44 chars vs 64 hex chars).
    pub fn v2_sha256_base64() -> Self {
        Self::new(
            Box::new(Sha256Hash::new()),
            Box::new(Base64Serializer::new()),
            "2".to_string(),
        )
    }

    /// Create v2 configuration with SHA256 + base64url.
    ///
    /// This configuration uses SHA256 with URL-safe base64 serialization
    /// without padding, suitable for use in URLs (43 chars for 32-byte digest).
    pub fn v2_sha256_base64url() -> Self {
        Self::new(
            Box::new(Sha256Hash::new()),
            Box::new(Base64UrlSerializer::new()),
            "2".to_string(),
        )
    }

    /// Create v2 configuration with SHA256 + base32.
    ///
    /// This configuration uses SHA256 with Base32 (RFC 4648) serialization
    /// for a compact representation (52 chars for 32-byte digest).
    pub fn v2_sha256_base32() -> Self {
        Self::new(
            Box::new(Sha256Hash::new()),
            Box::new(Base32Serializer::new()),
            "2".to_string(),
        )
    }

    /// Create v2 configuration with SHA256 + base32hex.
    ///
    /// This configuration uses SHA256 with Base32hex serialization
    /// for a compact representation (52 chars for 32-byte digest).
    pub fn v2_sha256_base32hex() -> Self {
        Self::new(
            Box::new(Sha256Hash::new()),
            Box::new(Base32HexSerializer::new()),
            "2".to_string(),
        )
    }

    /// Create v2 configuration with SHA256 + z85.
    ///
    /// This configuration uses SHA256 with Z85 (ZeroMQ Base85) serialization
    /// for the most compact representation (40 chars for 32-byte digest).
    pub fn v2_sha256_z85() -> Self {
        Self::new(
            Box::new(Sha256Hash::new()),
            Box::new(Z85Serializer::new()),
            "2".to_string(),
        )
    }
}

impl Default for HashConfig {
    fn default() -> Self {
        Self::v1()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_v1() {
        let config = HashConfig::v1();
        assert_eq!(config.version, "1");
        assert_eq!(config.hash_function.name(), "sha1");
        assert_eq!(config.hash_function.digest_size(), 20);
        assert_eq!(config.serializer.name(), "hex");
    }

    #[test]
    fn config_v2_sha256_hex() {
        let config = HashConfig::v2_sha256_hex();
        assert_eq!(config.version, "2");
        assert_eq!(config.hash_function.name(), "sha256");
        assert_eq!(config.hash_function.digest_size(), 32);
        assert_eq!(config.serializer.name(), "hex");
    }

    #[test]
    fn config_v2_sha256_base64() {
        let config = HashConfig::v2_sha256_base64();
        assert_eq!(config.version, "2");
        assert_eq!(config.hash_function.name(), "sha256");
        assert_eq!(config.hash_function.digest_size(), 32);
        assert_eq!(config.serializer.name(), "base64");
    }

    #[test]
    fn config_v2_sha256_base64url() {
        let config = HashConfig::v2_sha256_base64url();
        assert_eq!(config.version, "2");
        assert_eq!(config.hash_function.name(), "sha256");
        assert_eq!(config.hash_function.digest_size(), 32);
        assert_eq!(config.serializer.name(), "base64url");
    }

    #[test]
    fn config_default_is_v1() {
        let config = HashConfig::default();
        assert_eq!(config.version, "1");
        assert_eq!(config.hash_function.name(), "sha1");
    }

    #[test]
    fn config_encode_decode_roundtrip() {
        let config = HashConfig::v2_sha256_hex();
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let hash = config.hash_function.hash(&data);
        let encoded = config.serializer.encode(&hash);
        let decoded = config.serializer.decode(&encoded).unwrap();
        assert_eq!(hash, decoded);
    }

    #[test]
    fn config_different_serializers() {
        let hex_config = HashConfig::v2_sha256_hex();
        let base64_config = HashConfig::v2_sha256_base64();
        let base64url_config = HashConfig::v2_sha256_base64url();
        let base32_config = HashConfig::v2_sha256_base32();
        let base32hex_config = HashConfig::v2_sha256_base32hex();
        let z85_config = HashConfig::v2_sha256_z85();

        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let hash = hex_config.hash_function.hash(&data);

        let hex_encoded = hex_config.serializer.encode(&hash);
        let base64_encoded = base64_config.serializer.encode(&hash);
        let base64url_encoded = base64url_config.serializer.encode(&hash);
        let base32_encoded = base32_config.serializer.encode(&hash);
        let base32hex_encoded = base32hex_config.serializer.encode(&hash);
        let z85_encoded = z85_config.serializer.encode(&hash);

        // All should decode to the same hash
        assert_eq!(hex_config.serializer.decode(&hex_encoded).unwrap(), hash);
        assert_eq!(base64_config.serializer.decode(&base64_encoded).unwrap(), hash);
        assert_eq!(base64url_config.serializer.decode(&base64url_encoded).unwrap(), hash);
        assert_eq!(base32_config.serializer.decode(&base32_encoded).unwrap(), hash);
        assert_eq!(base32hex_config.serializer.decode(&base32hex_encoded).unwrap(), hash);
        assert_eq!(z85_config.serializer.decode(&z85_encoded).unwrap(), hash);

        // But encodings are different
        assert_ne!(hex_encoded, base64_encoded);
        assert_ne!(hex_encoded, base64url_encoded);
        assert_ne!(hex_encoded, base32_encoded);
        assert_ne!(hex_encoded, z85_encoded);
        
        // Verify compactness: z85 < base64 < base32 < hex
        assert!(z85_encoded.len() < base64_encoded.len());
        assert!(base64_encoded.len() < base32_encoded.len());
        assert!(base32_encoded.len() < hex_encoded.len());
    }

    #[test]
    fn config_v2_sha256_base32() {
        let config = HashConfig::v2_sha256_base32();
        assert_eq!(config.version, "2");
        assert_eq!(config.hash_function.name(), "sha256");
        assert_eq!(config.hash_function.digest_size(), 32);
        assert_eq!(config.serializer.name(), "base32");
    }

    #[test]
    fn config_v2_sha256_base32hex() {
        let config = HashConfig::v2_sha256_base32hex();
        assert_eq!(config.version, "2");
        assert_eq!(config.hash_function.name(), "sha256");
        assert_eq!(config.hash_function.digest_size(), 32);
        assert_eq!(config.serializer.name(), "base32hex");
    }

    #[test]
    fn config_v2_sha256_z85() {
        let config = HashConfig::v2_sha256_z85();
        assert_eq!(config.version, "2");
        assert_eq!(config.hash_function.name(), "sha256");
        assert_eq!(config.hash_function.digest_size(), 32);
        assert_eq!(config.serializer.name(), "z85");
    }
}

