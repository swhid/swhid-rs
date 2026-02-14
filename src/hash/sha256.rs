//! SHA-256 fast path for SWHID (two updates, fixed [u8; 32] output).

use sha2::{Digest as DigestTrait, Sha256};

use super::{swhid_object_header, HashFunction};

#[derive(Debug, Clone, Copy, Default)]
pub struct Sha256Hash;

impl HashFunction for Sha256Hash {
    type Output = [u8; 32];

    fn hash_object(&self, typ: &str, payload: &[u8]) -> [u8; 32] {
        let header = swhid_object_header(typ, payload.len());
        let mut hasher = Sha256::new();
        hasher.update(&header);
        hasher.update(payload);
        hasher.finalize().into()
    }
}

/// Content SWHID digest (blob) for SHA-256.
pub fn hash_content_sha256(data: &[u8]) -> [u8; 32] {
    Sha256Hash.hash_object("blob", data)
}

/// Arbitrary SWHID object digest for SHA-256.
pub fn hash_swhid_object_sha256(typ: &str, payload: &[u8]) -> [u8; 32] {
    Sha256Hash.hash_object(typ, payload)
}
