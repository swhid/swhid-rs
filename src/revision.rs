use crate::config::HashConfig;
use crate::hash::hash_swhid_object_generic;
use crate::serialization::{DigestSerializer, HexSerializer};
use crate::utils::HeaderWriter;
use crate::{Bytestring, Swhid};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision {
    pub directory: Vec<u8>,
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
    /// Compute the SWHID revision identifier using the given config.
    pub fn swhid_with_config(&self, config: &HashConfig) -> Swhid {
        let manifest = rev_manifest(self);
        let digest = hash_swhid_object_generic(
            "commit",
            &manifest,
            config.hash_function.as_ref(),
        );
        Swhid::new(crate::ObjectType::Revision, digest, config.version)
    }

    /// Compute the SWHID v1 revision identifier (SHA-1, hex).
    ///
    /// Equivalent to `swhid_with_config(&HashConfig::v1())`.
    pub fn swhid(&self) -> Swhid {
        self.swhid_with_config(&HashConfig::v1())
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
