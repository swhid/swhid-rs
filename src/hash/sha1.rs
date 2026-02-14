//! SHA-1 fast path for SWHID (two updates, fixed [u8; 20] output).

use sha1collisiondetection::{Digest as DigestTrait, Sha1CD};

use super::{swhid_object_header, HashFunction};

#[derive(Debug, Clone, Copy, Default)]
pub struct Sha1Hash;

impl HashFunction for Sha1Hash {
    type Output = [u8; 20];

    fn hash_object(&self, typ: &str, payload: &[u8]) -> [u8; 20] {
        let header = swhid_object_header(typ, payload.len());
        let mut hasher = Sha1CD::new();
        hasher.update(&header);
        hasher.update(payload);
        hasher.finalize().into()
    }
}

/// Content SWHID digest (blob) for SHA-1.
pub fn hash_content_sha1(data: &[u8]) -> [u8; 20] {
    Sha1Hash.hash_object("blob", data)
}

/// Arbitrary SWHID object digest for SHA-1.
pub fn hash_swhid_object_sha1(typ: &str, payload: &[u8]) -> [u8; 20] {
    Sha1Hash.hash_object(typ, payload)
}
