/// Trait for hash functions used in SWHID computation.
///
/// This trait abstracts over different hash algorithms (SHA1, SHA256, etc.)
/// to allow pluggable hash functions for SWHID v2 experimentation.
pub trait HashFunction: Send + Sync {
    /// Compute the hash digest of the given data.
    ///
    /// Returns the raw digest bytes as a boxed slice. The length should match `digest_size()`.
    /// Using `Box<[u8]>` instead of `Vec<u8>` avoids capacity slack since hash digests
    /// have a fixed size and don't need to grow.
    fn hash(&self, data: &[u8]) -> Box<[u8]>;

    /// Return the size of the digest in bytes.
    ///
    /// For SHA1, this is 20 bytes. For SHA256, this is 32 bytes.
    fn digest_size(&self) -> usize;

    /// Return the name of the hash function.
    ///
    /// Examples: "sha1", "sha256"
    fn name(&self) -> &str;
}

