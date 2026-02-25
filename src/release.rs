use crate::config::HashConfig;
use crate::hash::hash_swhid_object_generic;
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
    pub object: Vec<u8>,
    pub object_type: ReleaseTargetType,
    pub name: Bytestring,
    pub author: Option<Bytestring>,
    pub author_timestamp: Option<i64>,
    pub author_timestamp_offset: Option<Bytestring>,
    pub extra_headers: Vec<(Bytestring, Bytestring)>,
    pub message: Option<Bytestring>,
}

impl Release {
    /// Compute the SWHID release identifier using the given config.
    pub fn swhid_with_config(&self, config: &HashConfig) -> Swhid {
        let manifest = rel_manifest(self);
        let digest = hash_swhid_object_generic("tag", &manifest, config.hash_function.as_ref());
        Swhid::new(crate::ObjectType::Release, digest, config.version)
    }

    /// Compute the SWHID v1 release identifier (SHA-1, hex).
    ///
    /// Equivalent to `swhid_with_config(&HashConfig::v1())`.
    pub fn swhid(&self) -> Swhid {
        self.swhid_with_config(&HashConfig::v1())
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
