use crate::config::HashConfig;
use crate::core::{ObjectType, Swhid};
use crate::hash::hash_swhid_object_generic;

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

    /// Compute the SWHID content identifier using the given config (hash + version).
    pub fn swhid_with_config(&self, config: &HashConfig) -> Swhid {
        let digest =
            hash_swhid_object_generic("blob", self.bytes.as_ref(), config.hash_function.as_ref());
        Swhid::new(ObjectType::Content, digest, config.version)
    }

    /// Compute the SWHID v1 content identifier (SHA-1, hex).
    ///
    /// Equivalent to `swhid_with_config(&HashConfig::v1())`.
    pub fn swhid(&self) -> Swhid {
        self.swhid_with_config(&HashConfig::v1())
    }
}
