use crate::serialization::{DigestSerializer, HexSerializer};
use crate::utils::HeaderWriter;
use crate::{Bytestring, Swhid};

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
        let digest = crate::hash::hash_swhid_object("commit", &manifest);

        Swhid::new_v1(crate::ObjectType::Revision, digest)
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
    writer.push(b"tree", HexSerializer.encode(directory));

    for parent in parents {
        writer.push(b"parent", HexSerializer.encode(parent));
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
