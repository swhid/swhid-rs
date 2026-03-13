//! BLAKE3 fast path for SWHID (two updates, fixed [u8; 32] output).

use crate::Digest;

use super::{swhid_object_header, HashFunction};

/// Newtype for BLAKE3 digest output to avoid `From<[u8; 32]>` conflict with SHA-256.
#[derive(Debug, Clone, Copy)]
pub struct Blake3Digest(pub(crate) [u8; 32]);


impl AsRef<[u8]> for Blake3Digest {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Blake3Digest> for Digest {
    fn from(d: Blake3Digest) -> Self {
        Digest::Blake3(d.0)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Blake3Hash;

impl HashFunction for Blake3Hash {
    type Output = Blake3Digest;

    fn hash_object(&self, typ: &str, payload: &[u8]) -> Blake3Digest {
        let header = swhid_object_header(typ, payload.len());
        let mut hasher = blake3::Hasher::new();
        hasher.update(&header);
        hasher.update(payload);
        Blake3Digest(hasher.finalize().into())
    }
}
