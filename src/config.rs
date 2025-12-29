use crate::error::SwhidError;
use crate::hash::{HashFunction, Sha1Hash, Sha256Hash};
use crate::serialization::{DigestSerializer, HexSerializer, Base64Serializer, Base64UrlSerializer, Base32Serializer, Base32HexSerializer, Z85Serializer};
use crate::types::{SwhidVersion, HashAlgorithm, Encoding};

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
    pub version: SwhidVersion,
    pub hash_algorithm: HashAlgorithm,
    pub encoding: Encoding,
}

impl HashConfig {
    /// Create a new HashConfig with the specified components.
    ///
    /// # Validation
    ///
    /// This method validates that the hash function's digest size matches the
    /// expected size for the specified hash algorithm:
    /// - SHA1: 20 bytes
    /// - SHA256: 32 bytes
    ///
    /// Returns an error if the digest size doesn't match.
    ///
    /// # Examples
    ///
    /// ```
    /// use swhid::config::HashConfig;
    /// use swhid::hash::{HashFunction, Sha1Hash};
    /// use swhid::serialization::HexSerializer;
    /// use swhid::types::{SwhidVersion, HashAlgorithm, Encoding};
    ///
    /// let hasher = Box::new(Sha1Hash::new());
    /// let serializer = Box::new(HexSerializer::new());
    /// let config = HashConfig::new(
    ///     hasher,
    ///     serializer,
    ///     SwhidVersion::V1,
    ///     HashAlgorithm::Sha1,
    ///     Encoding::Hex,
    /// )?;
    /// # Ok::<(), swhid::SwhidError>(())
    /// ```
    pub fn new(
        hash_function: Box<dyn HashFunction>,
        serializer: Box<dyn DigestSerializer>,
        version: SwhidVersion,
        hash_algorithm: HashAlgorithm,
        encoding: Encoding,
    ) -> Result<Self, SwhidError> {
        // Validate digest size matches expected size for hash algorithm
        let expected_size = match hash_algorithm {
            HashAlgorithm::Sha1 => 20,
            HashAlgorithm::Sha256 => 32,
        };
        let actual_size = hash_function.digest_size();
        if actual_size != expected_size {
            return Err(SwhidError::InvalidDigest(format!(
                "Hash function digest size {} does not match expected size {} for {}",
                actual_size, expected_size, hash_algorithm.as_str()
            )));
        }
        Ok(Self {
            hash_function,
            serializer,
            version,
            hash_algorithm,
            encoding,
        })
    }

    /// Encode a digest byte array using the configured serializer.
    ///
    /// This method applies the serialization format (hex, base64, etc.)
    /// to the raw digest bytes. The serializer does NOT affect the digest
    /// computation itself, only how it is encoded for display/storage.
    ///
    /// # Examples
    ///
    /// ```
    /// use swhid::config::HashConfig;
    ///
    /// let config = HashConfig::v2_sha256_z85();
    /// let digest = vec![0u8; 32]; // SHA256 digest
    /// let encoded = config.encode_digest(&digest);
    /// assert_eq!(encoded.len(), 40); // Z85 encoding
    /// ```
    pub fn encode_digest(&self, digest: &[u8]) -> Result<String, SwhidError> {
        self.serializer.encode(digest)
    }

    /// Decode an encoded digest string back to bytes using the configured serializer.
    ///
    /// This method decodes a serialized digest string (hex, base64, etc.)
    /// back to raw digest bytes. Returns an error if the encoded string
    /// is invalid for this serialization format.
    ///
    /// # Errors
    ///
    /// Returns `SwhidError::EncodingError` if the encoded string is invalid
    /// for the configured serialization format (e.g., invalid hex characters,
    /// malformed base64, or Z85 input not a multiple of 4 bytes).
    ///
    /// # Examples
    ///
    /// ```
    /// use swhid::config::HashConfig;
    ///
    /// let config = HashConfig::v2_sha256_hex();
    /// let encoded = "a0a477f1ecf419c7eaa7fe256c5c12fb03bee86df9a22aad25f85930de203e14";
    /// let decoded = config.decode_digest(encoded)?;
    /// assert_eq!(decoded.len(), 32); // SHA256 digest
    /// # Ok::<(), swhid::SwhidError>(())
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use swhid::config::HashConfig;
    ///
    /// let config = HashConfig::v2_sha256_hex();
    /// let encoded = "a0a477f1ecf419c7eaa7fe256c5c12fb03bee86df9a22aad25f85930de203e14";
    /// let digest = config.decode_digest(encoded).unwrap();
    /// assert_eq!(digest.len(), 32); // SHA256 digest size
    /// ```
    pub fn decode_digest(&self, encoded: &str) -> Result<Vec<u8>, crate::error::SwhidError> {
        let decoded = self.serializer.decode(encoded)?;
        // Validate decoded length matches expected digest size
        if decoded.len() != self.hash_function.digest_size() {
            return Err(crate::error::SwhidError::InvalidDigest(format!(
                "Decoded digest length {} does not match expected size {} for {}",
                decoded.len(),
                self.hash_function.digest_size(),
                self.hash_function.name()
            )));
        }
        Ok(decoded)
    }

    /// Return the expected digest size in bytes for this hash function.
    ///
    /// This is the size of the raw digest bytes before serialization.
    /// For SHA1, this is 20 bytes. For SHA256, this is 32 bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use swhid::config::HashConfig;
    ///
    /// let v1_config = HashConfig::v1();
    /// assert_eq!(v1_config.expected_digest_size(), 20);
    ///
    /// let v2_config = HashConfig::v2_sha256_hex();
    /// assert_eq!(v2_config.expected_digest_size(), 32);
    /// ```
    pub fn expected_digest_size(&self) -> usize {
        self.hash_function.digest_size()
    }

    /// Create v1 configuration (SHA1 + hex).
    ///
    /// This is the default configuration for SWHID v1 and maintains
    /// backward compatibility with existing implementations.
    pub fn v1() -> Self {
        Self::new(
            Box::new(Sha1Hash::new()),
            Box::new(HexSerializer::new()),
            SwhidVersion::V1,
            HashAlgorithm::Sha1,
            Encoding::Hex,
        ).expect("v1 config should always be valid")
    }

    /// Create v2 configuration with SHA256 + hex.
    ///
    /// This configuration uses SHA256 for enhanced security while
    /// maintaining hex serialization for compatibility with Git OIDs.
    pub fn v2_sha256_hex() -> Self {
        Self::new(
            Box::new(Sha256Hash::new()),
            Box::new(HexSerializer::new()),
            SwhidVersion::V2,
            HashAlgorithm::Sha256,
            Encoding::Hex,
        ).expect("v2_sha256_hex config should always be valid")
    }

    /// Create v2 configuration with SHA256 + base64.
    ///
    /// This configuration uses SHA256 with base64 serialization for
    /// a more compact representation (44 chars vs 64 hex chars).
    pub fn v2_sha256_base64() -> Self {
        Self::new(
            Box::new(Sha256Hash::new()),
            Box::new(Base64Serializer::new()),
            SwhidVersion::V2,
            HashAlgorithm::Sha256,
            Encoding::Base64,
        ).expect("v2_sha256_base64 config should always be valid")
    }

    /// Create v2 configuration with SHA256 + base64url.
    ///
    /// This configuration uses SHA256 with URL-safe base64 serialization
    /// without padding, suitable for use in URLs (43 chars for 32-byte digest).
    pub fn v2_sha256_base64url() -> Self {
        Self::new(
            Box::new(Sha256Hash::new()),
            Box::new(Base64UrlSerializer::new()),
            SwhidVersion::V2,
            HashAlgorithm::Sha256,
            Encoding::Base64Url,
        ).expect("v2_sha256_base64url config should always be valid")
    }

    /// Create v2 configuration with SHA256 + base32.
    ///
    /// This configuration uses SHA256 with Base32 (RFC 4648) serialization
    /// for a compact representation (52 chars for 32-byte digest).
    pub fn v2_sha256_base32() -> Self {
        Self::new(
            Box::new(Sha256Hash::new()),
            Box::new(Base32Serializer::new()),
            SwhidVersion::V2,
            HashAlgorithm::Sha256,
            Encoding::Base32,
        ).expect("v2_sha256_base32 config should always be valid")
    }

    /// Create v2 configuration with SHA256 + base32hex.
    ///
    /// This configuration uses SHA256 with Base32hex serialization
    /// for a compact representation (52 chars for 32-byte digest).
    pub fn v2_sha256_base32hex() -> Self {
        Self::new(
            Box::new(Sha256Hash::new()),
            Box::new(Base32HexSerializer::new()),
            SwhidVersion::V2,
            HashAlgorithm::Sha256,
            Encoding::Base32Hex,
        ).expect("v2_sha256_base32hex config should always be valid")
    }

    /// Create v2 configuration with SHA256 + z85.
    ///
    /// This configuration uses SHA256 with Z85 (ZeroMQ Base85) serialization
    /// for the most compact representation (40 chars for 32-byte digest).
    pub fn v2_sha256_z85() -> Self {
        Self::new(
            Box::new(Sha256Hash::new()),
            Box::new(Z85Serializer::new()),
            SwhidVersion::V2,
            HashAlgorithm::Sha256,
            Encoding::Z85,
        ).expect("v2_sha256_z85 config should always be valid")
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
                assert_eq!(config.version, SwhidVersion::V1);
                assert_eq!(config.hash_algorithm, HashAlgorithm::Sha1);
                assert_eq!(config.encoding, Encoding::Hex);
                assert_eq!(config.hash_function.name(), "sha1");
                assert_eq!(config.hash_function.digest_size(), 20);
                assert_eq!(config.serializer.name(), "hex");
            }

    #[test]
    fn config_v2_sha256_hex() {
        let config = HashConfig::v2_sha256_hex();
        assert_eq!(config.version, SwhidVersion::V2);
        assert_eq!(config.hash_algorithm, HashAlgorithm::Sha256);
        assert_eq!(config.encoding, Encoding::Hex);
        assert_eq!(config.hash_function.name(), "sha256");
        assert_eq!(config.hash_function.digest_size(), 32);
        assert_eq!(config.serializer.name(), "hex");
    }

    #[test]
    fn config_v2_sha256_base64() {
        let config = HashConfig::v2_sha256_base64();
        assert_eq!(config.version, SwhidVersion::V2);
        assert_eq!(config.hash_algorithm, HashAlgorithm::Sha256);
        assert_eq!(config.encoding, Encoding::Base64);
        assert_eq!(config.hash_function.name(), "sha256");
        assert_eq!(config.hash_function.digest_size(), 32);
        assert_eq!(config.serializer.name(), "base64");
    }

    #[test]
    fn config_v2_sha256_base64url() {
        let config = HashConfig::v2_sha256_base64url();
        assert_eq!(config.version, SwhidVersion::V2);
        assert_eq!(config.hash_algorithm, HashAlgorithm::Sha256);
        assert_eq!(config.encoding, Encoding::Base64Url);
        assert_eq!(config.hash_function.name(), "sha256");
        assert_eq!(config.hash_function.digest_size(), 32);
        assert_eq!(config.serializer.name(), "base64url");
    }

    #[test]
    fn config_default_is_v1() {
        let config = HashConfig::default();
        assert_eq!(config.version, SwhidVersion::V1);
        assert_eq!(config.hash_algorithm, HashAlgorithm::Sha1);
        assert_eq!(config.hash_function.name(), "sha1");
    }

    #[test]
    fn config_encode_decode_roundtrip() {
        let config = HashConfig::v2_sha256_hex();
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let hash = config.hash_function.hash(&data);
        let encoded = config.serializer.encode(&hash).unwrap();
        let decoded = config.serializer.decode(&encoded).unwrap();
        assert_eq!(hash.as_ref(), decoded.as_slice());
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

        let hex_encoded = hex_config.serializer.encode(&hash).unwrap();
        let base64_encoded = base64_config.serializer.encode(&hash).unwrap();
        let base64url_encoded = base64url_config.serializer.encode(&hash).unwrap();
        let base32_encoded = base32_config.serializer.encode(&hash).unwrap();
        let base32hex_encoded = base32hex_config.serializer.encode(&hash).unwrap();
        let z85_encoded = z85_config.serializer.encode(&hash).unwrap();

        // All should decode to the same hash
        assert_eq!(hex_config.serializer.decode(&hex_encoded).unwrap(), hash.as_ref());
        assert_eq!(base64_config.serializer.decode(&base64_encoded).unwrap(), hash.as_ref());
        assert_eq!(base64url_config.serializer.decode(&base64url_encoded).unwrap(), hash.as_ref());
        assert_eq!(base32_config.serializer.decode(&base32_encoded).unwrap(), hash.as_ref());
        assert_eq!(base32hex_config.serializer.decode(&base32hex_encoded).unwrap(), hash.as_ref());
        assert_eq!(z85_config.serializer.decode(&z85_encoded).unwrap(), hash.as_ref());

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
        assert_eq!(config.version, SwhidVersion::V2);
        assert_eq!(config.hash_algorithm, HashAlgorithm::Sha256);
        assert_eq!(config.encoding, Encoding::Base32);
        assert_eq!(config.hash_function.name(), "sha256");
        assert_eq!(config.hash_function.digest_size(), 32);
        assert_eq!(config.serializer.name(), "base32");
    }

    #[test]
    fn config_v2_sha256_base32hex() {
        let config = HashConfig::v2_sha256_base32hex();
        assert_eq!(config.version, SwhidVersion::V2);
        assert_eq!(config.hash_algorithm, HashAlgorithm::Sha256);
        assert_eq!(config.encoding, Encoding::Base32Hex);
        assert_eq!(config.hash_function.name(), "sha256");
        assert_eq!(config.hash_function.digest_size(), 32);
        assert_eq!(config.serializer.name(), "base32hex");
    }

    #[test]
    fn config_v2_sha256_z85() {
        let config = HashConfig::v2_sha256_z85();
        assert_eq!(config.version, SwhidVersion::V2);
        assert_eq!(config.hash_algorithm, HashAlgorithm::Sha256);
        assert_eq!(config.encoding, Encoding::Z85);
        assert_eq!(config.hash_function.name(), "sha256");
        assert_eq!(config.hash_function.digest_size(), 32);
        assert_eq!(config.serializer.name(), "z85");
    }
}

