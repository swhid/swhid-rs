use crate::config::HashConfig;
use crate::core::{ObjectType, Swhid};
use crate::digest::Digest;
use crate::hash::{hash_content, HashFunction};
use crate::serialization::DigestSerializer;

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

    /// Compute the SWHID using the given hash and encoding config.
    pub fn swhid_with_config<H, E>(&self, config: &HashConfig<H, E>) -> Swhid
    where
        H: HashFunction,
        E: DigestSerializer,
        H::Output: Into<Digest>,
    {
        let digest = config.hash.hash_object("blob", self.bytes.as_ref()).into();
        Swhid::new(ObjectType::Content, digest, config.version)
    }
}
