/// SWHID version enumeration.
///
/// Represents the version of a SWHID identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SwhidVersion {
    /// SWHID v1: SHA1 + hex (20 bytes, 40 hex chars)
    V1,
    /// SWHID v2: SHA256 + configurable serialization (32 bytes, variable encoding)
    V2,
}

impl SwhidVersion {
    /// Convert version to string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            SwhidVersion::V1 => "1",
            SwhidVersion::V2 => "2",
        }
    }

    /// Parse version from string.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "1" => Ok(SwhidVersion::V1),
            "2" => Ok(SwhidVersion::V2),
            _ => Err(format!("Invalid SWHID version: {}", s)),
        }
    }
}

impl std::str::FromStr for SwhidVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s)
    }
}

impl Default for SwhidVersion {
    fn default() -> Self {
        SwhidVersion::V1
    }
}

/// Hash algorithm enumeration.
///
/// Represents the hash function used for computing SWHID digests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HashAlgorithm {
    /// SHA1 hash function (20-byte digests)
    Sha1,
    /// SHA256 hash function (32-byte digests)
    Sha256,
}

impl HashAlgorithm {
    /// Convert hash algorithm to string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            HashAlgorithm::Sha1 => "sha1",
            HashAlgorithm::Sha256 => "sha256",
        }
    }

    /// Parse hash algorithm from string.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "sha1" => Ok(HashAlgorithm::Sha1),
            "sha256" => Ok(HashAlgorithm::Sha256),
            _ => Err(format!("Invalid hash algorithm: {}", s)),
        }
    }
}

impl std::str::FromStr for HashAlgorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        HashAlgorithm::from_str(s)
    }
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        HashAlgorithm::Sha1
    }
}

/// Serialization format enumeration.
///
/// Represents the encoding format used for SWHID digest strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Encoding {
    /// Hexadecimal encoding (canonical format)
    Hex,
    /// Base64 encoding (standard, with padding)
    Base64,
    /// Base64URL encoding (URL-safe, without padding)
    Base64Url,
    /// Base32 encoding (RFC 4648 standard)
    Base32,
    /// Base32hex encoding (RFC 4648 variant)
    Base32Hex,
    /// Z85 encoding (ZeroMQ Base85, most compact)
    Z85,
}

impl Encoding {
    /// Convert encoding to string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Encoding::Hex => "hex",
            Encoding::Base64 => "base64",
            Encoding::Base64Url => "base64url",
            Encoding::Base32 => "base32",
            Encoding::Base32Hex => "base32hex",
            Encoding::Z85 => "z85",
        }
    }

    /// Parse encoding from string.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "hex" => Ok(Encoding::Hex),
            "base64" => Ok(Encoding::Base64),
            "base64url" => Ok(Encoding::Base64Url),
            "base32" => Ok(Encoding::Base32),
            "base32hex" => Ok(Encoding::Base32Hex),
            "z85" => Ok(Encoding::Z85),
            _ => Err(format!("Invalid encoding: {}", s)),
        }
    }
}

impl std::str::FromStr for Encoding {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Encoding::from_str(s)
    }
}

impl Default for Encoding {
    fn default() -> Self {
        Encoding::Hex
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swhid_version_roundtrip() {
        assert_eq!(SwhidVersion::V1.as_str(), "1");
        assert_eq!(SwhidVersion::V2.as_str(), "2");
        assert_eq!(SwhidVersion::from_str("1").unwrap(), SwhidVersion::V1);
        assert_eq!(SwhidVersion::from_str("2").unwrap(), SwhidVersion::V2);
        assert!(SwhidVersion::from_str("3").is_err());
    }

    #[test]
    fn hash_algorithm_roundtrip() {
        assert_eq!(HashAlgorithm::Sha1.as_str(), "sha1");
        assert_eq!(HashAlgorithm::Sha256.as_str(), "sha256");
        assert_eq!(HashAlgorithm::from_str("sha1").unwrap(), HashAlgorithm::Sha1);
        assert_eq!(HashAlgorithm::from_str("sha256").unwrap(), HashAlgorithm::Sha256);
        assert!(HashAlgorithm::from_str("sha3").is_err());
    }

    #[test]
    fn encoding_roundtrip() {
        assert_eq!(Encoding::Hex.as_str(), "hex");
        assert_eq!(Encoding::Base64.as_str(), "base64");
        assert_eq!(Encoding::Base64Url.as_str(), "base64url");
        assert_eq!(Encoding::Base32.as_str(), "base32");
        assert_eq!(Encoding::Base32Hex.as_str(), "base32hex");
        assert_eq!(Encoding::Z85.as_str(), "z85");
        
        assert_eq!(Encoding::from_str("hex").unwrap(), Encoding::Hex);
        assert_eq!(Encoding::from_str("base64").unwrap(), Encoding::Base64);
        assert_eq!(Encoding::from_str("base64url").unwrap(), Encoding::Base64Url);
        assert_eq!(Encoding::from_str("base32").unwrap(), Encoding::Base32);
        assert_eq!(Encoding::from_str("base32hex").unwrap(), Encoding::Base32Hex);
        assert_eq!(Encoding::from_str("z85").unwrap(), Encoding::Z85);
        
        assert!(Encoding::from_str("invalid").is_err());
    }
}

