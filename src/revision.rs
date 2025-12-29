use crate::utils::HeaderWriter;
use crate::{Bytestring, Swhid};
use crate::hash::{hash_swhid_object, hash_swhid_object_with};
use crate::config::HashConfig;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision {
    /// Directory/tree digest (20 bytes for SHA1, 32 bytes for SHA256)
    pub directory: Vec<u8>,
    /// Parent commit digests (20 bytes for SHA1, 32 bytes for SHA256)
    pub parents: Vec<Vec<u8>>,
    pub author: Bytestring,
    pub author_timestamp: i64,
    pub author_timestamp_offset: Bytestring,
    pub committer: Bytestring,
    pub committer_timestamp: i64,
    pub committer_timestamp_offset: Bytestring,
    pub extra_headers: Vec<(Bytestring, Bytestring)>,
    pub message: Option<Bytestring>,
}

impl Revision {
    /// Compute a SWHID v1.2 revision identifier from a Git commit
    ///
    /// This implements the SWHID v1.2 revision hashing algorithm for Git commits,
    /// creating a `swh:1:rev:<digest>` identifier according to the specification.
    pub fn swhid(&self) -> Swhid {
        let manifest = rev_manifest(self);
        let digest = hash_swhid_object("commit", &manifest);

        Swhid::new_v1(crate::ObjectType::Revision, digest)
    }

    /// Compute the SWHID revision identifier using the specified hash configuration.
    ///
    /// This allows computing SWHIDs with different hash functions (SHA1, SHA256, etc.)
    /// and serialization formats (hex, base64, etc.) for v2 experimentation.
    ///
    /// This method uses the same manifest format as v1, but with the specified hash function.
    /// The directory and parents fields contain variable-length digests which are converted
    /// to hex for the manifest.
    pub fn swhid_with_config(&self, config: &HashConfig) -> Swhid {
        let manifest = rev_manifest(self);
        let digest = hash_swhid_object_with("commit", &manifest, config.hash_function.as_ref());
        Swhid::new(crate::ObjectType::Revision, digest, config.version)
    }
}

pub fn rev_manifest(rev: &Revision) -> Vec<u8> {
    let Revision {
        directory,
        parents,
        author,
        author_timestamp,
        author_timestamp_offset,
        committer,
        committer_timestamp,
        committer_timestamp_offset,
        extra_headers,
        message,
    } = rev;
    let mut writer = HeaderWriter::default();
    writer.push(b"tree", hex::encode(directory));

    for parent in parents {
        writer.push(b"parent", hex::encode(parent));
    }

    writer.push_authorship(
        b"author",
        author,
        *author_timestamp,
        author_timestamp_offset,
    );
    writer.push_authorship(
        b"committer",
        committer,
        *committer_timestamp,
        committer_timestamp_offset,
    );

    for (key, value) in extra_headers {
        writer.push(key, value)
    }

    writer.build(message.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::HashConfig;
    use crate::types::SwhidVersion;

    #[test]
    fn revision_swhid_v1() {
        let rev = Revision {
            directory: vec![0u8; 20],
            parents: vec![],
            author: b"Test Author".as_ref().into(),
            author_timestamp: 1234567890,
            author_timestamp_offset: b"+0000".as_ref().into(),
            committer: b"Test Committer".as_ref().into(),
            committer_timestamp: 1234567890,
            committer_timestamp_offset: b"+0000".as_ref().into(),
            extra_headers: vec![],
            message: None,
        };
        let swhid = rev.swhid();
        assert_eq!(swhid.version(), SwhidVersion::V1);
        assert_eq!(swhid.digest_bytes().len(), 20);
    }

    #[test]
    fn revision_swhid_with_config_v2() {
        let rev = Revision {
            directory: vec![0u8; 20],
            parents: vec![],
            author: b"Test Author".as_ref().into(),
            author_timestamp: 1234567890,
            author_timestamp_offset: b"+0000".as_ref().into(),
            committer: b"Test Committer".as_ref().into(),
            committer_timestamp: 1234567890,
            committer_timestamp_offset: b"+0000".as_ref().into(),
            extra_headers: vec![],
            message: None,
        };
        let config = HashConfig::v2_sha256_hex();
        let swhid = rev.swhid_with_config(&config);
        assert_eq!(swhid.version(), SwhidVersion::V2);
        assert_eq!(swhid.digest_bytes().len(), 32);
    }
}
