use crate::utils::HeaderWriter;
use crate::{Bytestring, Swhid};
use crate::hash::{hash_swhid_object, hash_swhid_object_with};
use crate::config::HashConfig;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision {
    pub directory: [u8; 20],
    pub parents: Vec<[u8; 20]>,
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
    /// Note: This method currently uses the same manifest format as v1, but with
    /// the specified hash function. The directory and parents fields still contain
    /// [u8; 20] digests which are converted to hex for the manifest.
    pub fn swhid_with_config(&self, config: &HashConfig) -> Swhid {
        let manifest = rev_manifest(self);
        let digest = hash_swhid_object_with("commit", &manifest, config.hash_function.as_ref());
        Swhid::new(crate::ObjectType::Revision, digest, config.version.clone())
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
