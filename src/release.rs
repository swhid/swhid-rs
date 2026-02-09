use crate::serialization::{DigestSerializer, HexSerializer};
use crate::utils::HeaderWriter;
use crate::{Bytestring, Swhid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReleaseTargetType {
    Revision,
    Directory,
    Release,
    Content,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Release {
    pub object: [u8; 20],
    pub object_type: ReleaseTargetType,
    pub name: Bytestring,
    pub author: Option<Bytestring>,
    pub author_timestamp: Option<i64>,
    pub author_timestamp_offset: Option<Bytestring>,
    pub extra_headers: Vec<(Bytestring, Bytestring)>,
    pub message: Option<Bytestring>,
}

impl Release {
    /// Compute a SWHID v1.2 revision identifier from a Git commit
    ///
    /// This implements the SWHID v1.2 revision hashing algorithm for Git commits,
    /// creating a `swh:1:rev:<digest>` identifier according to the specification.
    pub fn swhid(&self) -> Swhid {
        let manifest = rel_manifest(self);
        let digest = crate::hash::hash_swhid_object("tag", &manifest);

        Swhid::new_v1(crate::ObjectType::Release, digest)
    }
}

pub fn rel_manifest(rev: &Release) -> Vec<u8> {
    let Release {
        object,
        object_type,
        name,
        author,
        author_timestamp,
        author_timestamp_offset,
        extra_headers,
        message,
    } = rev;
    let mut writer = HeaderWriter::default();

    writer.push(b"object", HexSerializer.encode(object));
    writer.push(
        b"type",
        match object_type {
            ReleaseTargetType::Revision => b"commit".as_ref(),
            ReleaseTargetType::Directory => b"tree".as_ref(),
            ReleaseTargetType::Release => b"tag".as_ref(),
            ReleaseTargetType::Content => b"blob".as_ref(),
        },
    );
    writer.push(b"tag", name);

    match (author, author_timestamp, author_timestamp_offset) {
        (Some(author), Some(author_timestamp), Some(author_timestamp_offset)) => writer
            .push_authorship(
                b"tagger",
                author,
                *author_timestamp,
                author_timestamp_offset,
            ),
        (None, None, None) => (),
        _ => (), // unspecified, see https://github.com/swhid/specification/issues/62
    }

    for (key, value) in extra_headers {
        writer.push(key, value)
    }

    writer.build(message.as_ref())
}
