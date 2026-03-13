//! Type-level config for hash and encoding (no dyn, no runtime dispatch).

use crate::hash::HashFunction;
use crate::serialization::DigestSerializer;
use crate::types::SwhidVersion;

/// Config for SWHID computation: concrete hasher and encoder (fixed at compile time).
#[derive(Clone)]
pub struct HashConfig<H, E> {
    /// Hash implementation.
    pub hash: H,
    /// Digest encoding for display/parse.
    pub encoder: E,
    /// SWHID version.
    pub version: SwhidVersion,
}

impl<H, E> HashConfig<H, E>
where
    H: HashFunction,
    E: DigestSerializer,
{
    /// Build a config from concrete hash and encoder.
    pub fn new(hash: H, encoder: E, version: SwhidVersion) -> Self {
        Self {
            hash,
            encoder,
            version,
        }
    }
}

// Constructors gated by (hash, encoding) features.

#[cfg(all(feature = "sha1", feature = "encoding-hex"))]
impl HashConfig<crate::hash::Sha1Hash, crate::serialization::HexSerializer> {
    /// V1 config: SHA-1 + hex.
    pub fn v1() -> Self {
        Self::new(
            crate::hash::Sha1Hash,
            crate::serialization::HexSerializer,
            SwhidVersion::V1,
        )
    }
}

#[cfg(all(feature = "sha256", feature = "encoding-base64url"))]
impl HashConfig<crate::hash::Sha256Hash, crate::serialization::Base64UrlSerializer> {
    /// V2 config: SHA-256 + base64url.
    pub fn v2() -> Self {
        Self::new(
            crate::hash::Sha256Hash,
            crate::serialization::Base64UrlSerializer,
            SwhidVersion::V2,
        )
    }
}

#[cfg(all(feature = "sha256", feature = "encoding-hex"))]
impl HashConfig<crate::hash::Sha256Hash, crate::serialization::HexSerializer> {
    /// V2 config: SHA-256 + hex.
    pub fn v2_hex() -> Self {
        Self::new(
            crate::hash::Sha256Hash,
            crate::serialization::HexSerializer,
            SwhidVersion::V2,
        )
    }
}

#[cfg(all(feature = "sha256", feature = "encoding-base64"))]
impl HashConfig<crate::hash::Sha256Hash, crate::serialization::Base64Serializer> {
    /// V2 config: SHA-256 + base64.
    pub fn v2_base64() -> Self {
        Self::new(
            crate::hash::Sha256Hash,
            crate::serialization::Base64Serializer,
            SwhidVersion::V2,
        )
    }
}

#[cfg(all(feature = "sha256", feature = "encoding-base32"))]
impl HashConfig<crate::hash::Sha256Hash, crate::serialization::Base32Serializer> {
    /// V2 config: SHA-256 + base32.
    pub fn v2_base32() -> Self {
        Self::new(
            crate::hash::Sha256Hash,
            crate::serialization::Base32Serializer,
            SwhidVersion::V2,
        )
    }
}

#[cfg(all(feature = "sha256", feature = "encoding-base32hex"))]
impl HashConfig<crate::hash::Sha256Hash, crate::serialization::Base32HexSerializer> {
    /// V2 config: SHA-256 + base32hex.
    pub fn v2_base32hex() -> Self {
        Self::new(
            crate::hash::Sha256Hash,
            crate::serialization::Base32HexSerializer,
            SwhidVersion::V2,
        )
    }
}

#[cfg(all(feature = "sha256", feature = "encoding-z85"))]
impl HashConfig<crate::hash::Sha256Hash, crate::serialization::Z85Serializer> {
    /// V2 config: SHA-256 + z85.
    pub fn v2_z85() -> Self {
        Self::new(
            crate::hash::Sha256Hash,
            crate::serialization::Z85Serializer,
            SwhidVersion::V2,
        )
    }
}

#[cfg(all(feature = "sha512", feature = "encoding-hex"))]
impl HashConfig<crate::hash::Sha512Hash, crate::serialization::HexSerializer> {
    /// SHA-512 + hex.
    pub fn sha512_hex() -> Self {
        Self::new(
            crate::hash::Sha512Hash,
            crate::serialization::HexSerializer,
            SwhidVersion::V2,
        )
    }
}

#[cfg(all(feature = "sha512", feature = "encoding-base64url"))]
impl HashConfig<crate::hash::Sha512Hash, crate::serialization::Base64UrlSerializer> {
    /// SHA-512 + base64url.
    pub fn sha512_base64url() -> Self {
        Self::new(
            crate::hash::Sha512Hash,
            crate::serialization::Base64UrlSerializer,
            SwhidVersion::V2,
        )
    }
}

#[cfg(all(feature = "blake3", feature = "encoding-hex"))]
impl HashConfig<crate::hash::Blake3Hash, crate::serialization::HexSerializer> {
    /// BLAKE3 + hex.
    pub fn blake3_hex() -> Self {
        Self::new(
            crate::hash::Blake3Hash,
            crate::serialization::HexSerializer,
            SwhidVersion::V2,
        )
    }
}

#[cfg(all(feature = "blake3", feature = "encoding-base64"))]
impl HashConfig<crate::hash::Blake3Hash, crate::serialization::Base64Serializer> {
    /// BLAKE3 + base64.
    pub fn blake3_base64() -> Self {
        Self::new(
            crate::hash::Blake3Hash,
            crate::serialization::Base64Serializer,
            SwhidVersion::V2,
        )
    }
}

#[cfg(all(feature = "blake3", feature = "encoding-base64url"))]
impl HashConfig<crate::hash::Blake3Hash, crate::serialization::Base64UrlSerializer> {
    /// BLAKE3 + base64url.
    pub fn blake3_base64url() -> Self {
        Self::new(
            crate::hash::Blake3Hash,
            crate::serialization::Base64UrlSerializer,
            SwhidVersion::V2,
        )
    }
}

#[cfg(all(feature = "blake3", feature = "encoding-base32"))]
impl HashConfig<crate::hash::Blake3Hash, crate::serialization::Base32Serializer> {
    /// BLAKE3 + base32.
    pub fn blake3_base32() -> Self {
        Self::new(
            crate::hash::Blake3Hash,
            crate::serialization::Base32Serializer,
            SwhidVersion::V2,
        )
    }
}

#[cfg(all(feature = "blake3", feature = "encoding-base32hex"))]
impl HashConfig<crate::hash::Blake3Hash, crate::serialization::Base32HexSerializer> {
    /// BLAKE3 + base32hex.
    pub fn blake3_base32hex() -> Self {
        Self::new(
            crate::hash::Blake3Hash,
            crate::serialization::Base32HexSerializer,
            SwhidVersion::V2,
        )
    }
}

#[cfg(all(feature = "blake3", feature = "encoding-z85"))]
impl HashConfig<crate::hash::Blake3Hash, crate::serialization::Z85Serializer> {
    /// BLAKE3 + z85.
    pub fn blake3_z85() -> Self {
        Self::new(
            crate::hash::Blake3Hash,
            crate::serialization::Z85Serializer,
            SwhidVersion::V2,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Content;

    #[cfg(all(feature = "sha1", feature = "encoding-hex"))]
    #[test]
    fn v1_content_roundtrip() {
        let config = HashConfig::v1();
        let content = Content::from_bytes(b"hello");
        let swhid = content.swhid_with_config(&config);
        assert_eq!(swhid.version(), SwhidVersion::V1);
        let s = swhid.to_string_encoded(&config.encoder);
        assert!(s.starts_with("swh:1:cnt:"));
    }

    #[cfg(all(feature = "sha256", feature = "encoding-hex"))]
    #[test]
    fn v2_hex_content_roundtrip() {
        let config = HashConfig::v2_hex();
        let content = Content::from_bytes(b"hello");
        let swhid = content.swhid_with_config(&config);
        assert_eq!(swhid.version(), SwhidVersion::V2);
        let s = swhid.to_string_encoded(&config.encoder);
        assert!(s.starts_with("swh:2:cnt:"));
    }

    #[cfg(all(feature = "sha256", feature = "encoding-base64url"))]
    #[test]
    fn v2_base64url_content_roundtrip() {
        let config = HashConfig::v2();
        let content = Content::from_bytes(b"hello");
        let swhid = content.swhid_with_config(&config);
        assert_eq!(swhid.version(), SwhidVersion::V2);
        let s = swhid.to_string_encoded(&config.encoder);
        assert!(s.starts_with("swh:2:cnt:"));
    }

    #[cfg(all(feature = "sha256", feature = "encoding-z85"))]
    #[test]
    fn v2_z85_content_roundtrip() {
        let config = HashConfig::v2_z85();
        let content = Content::from_bytes(b"hello");
        let swhid = content.swhid_with_config(&config);
        assert_eq!(swhid.version(), SwhidVersion::V2);
        let s = swhid.to_string_encoded(&config.encoder);
        assert!(s.starts_with("swh:2:cnt:"));
    }

    #[cfg(all(feature = "blake3", feature = "encoding-hex"))]
    #[test]
    fn blake3_hex_content_roundtrip() {
        let config = HashConfig::blake3_hex();
        let content = Content::from_bytes(b"hello");
        let swhid = content.swhid_with_config(&config);
        assert_eq!(swhid.version(), SwhidVersion::V2);
        let s = swhid.to_string_encoded(&config.encoder);
        assert!(s.starts_with("swh:2:cnt:"));
    }
}
