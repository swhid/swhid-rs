//! SHA-512 fast path for SWHID (two updates, fixed [u8; 64] output).

use sha2::{Digest as DigestTrait, Sha512};

use super::{swhid_object_header, HashFunction};

#[derive(Debug, Clone, Copy, Default)]
pub struct Sha512Hash;

impl HashFunction for Sha512Hash {
    type Output = [u8; 64];

    fn hash_object(&self, typ: &str, payload: &[u8]) -> [u8; 64] {
        let header = swhid_object_header(typ, payload.len());
        let mut hasher = Sha512::new();
        hasher.update(&header);
        hasher.update(payload);
        hasher.finalize().into()
    }
}

/// Content SWHID digest (blob) for SHA-512.
pub fn hash_content_sha512(data: &[u8]) -> [u8; 64] {
    Sha512Hash.hash_object("blob", data)
}

/// Arbitrary SWHID object digest for SHA-512.
pub fn hash_swhid_object_sha512(typ: &str, payload: &[u8]) -> [u8; 64] {
    Sha512Hash.hash_object(typ, payload)
}
