//! Fixed-size digest variants for SWHID (one per hash algorithm).
//! Variants are feature-gated; at least one hash feature must be enabled.

use crate::error::SwhidError;

/// Digest bytes for a SWHID; variant depends on the hash algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Digest {
    #[cfg(feature = "sha1")]
    Sha1([u8; 20]),
    #[cfg(feature = "sha256")]
    Sha256([u8; 32]),
    #[cfg(feature = "sha512")]
    Sha512([u8; 64]),
}

impl Digest {
    /// Borrow the raw digest bytes.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            #[cfg(feature = "sha1")]
            Digest::Sha1(a) => a.as_slice(),
            #[cfg(feature = "sha256")]
            Digest::Sha256(a) => a.as_slice(),
            #[cfg(feature = "sha512")]
            Digest::Sha512(a) => a.as_slice(),
        }
    }

    /// Build a digest from bytes; length must match an enabled hash (20, 32, or 64).
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, SwhidError> {
        match bytes.len() {
            #[cfg(feature = "sha1")]
            20 => Ok(Digest::Sha1(
                bytes.try_into().map_err(|_| SwhidError::InvalidDigest("expected 20 bytes".into()))?,
            )),
            #[cfg(feature = "sha256")]
            32 => Ok(Digest::Sha256(
                bytes.try_into().map_err(|_| SwhidError::InvalidDigest("expected 32 bytes".into()))?,
            )),
            #[cfg(feature = "sha512")]
            64 => Ok(Digest::Sha512(
                bytes.try_into().map_err(|_| SwhidError::InvalidDigest("expected 64 bytes".into()))?,
            )),
            _ => Err(SwhidError::InvalidDigest(format!(
                "unsupported digest length: {}",
                bytes.len()
            ))),
        }
    }
}

#[cfg(feature = "sha1")]
impl From<[u8; 20]> for Digest {
    fn from(a: [u8; 20]) -> Self {
        Digest::Sha1(a)
    }
}

#[cfg(feature = "sha256")]
impl From<[u8; 32]> for Digest {
    fn from(a: [u8; 32]) -> Self {
        Digest::Sha256(a)
    }
}

#[cfg(feature = "sha512")]
impl From<[u8; 64]> for Digest {
    fn from(a: [u8; 64]) -> Self {
        Digest::Sha512(a)
    }
}
