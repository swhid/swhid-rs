use std::fmt::{self, Display};
use std::str::FromStr;

use crate::error::SwhidError;

/// Known SWH object kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ObjectType {
    /// file contents (Git blob)
    Content, // "cnt"
    /// directory (Git tree)
    Directory, // "dir"
    /// VCS commit / changeset
    Revision, // "rev"
    /// VCS annotated tag / release
    Release, // "rel"
    /// Snapshot of repository refs
    Snapshot, // "snp"
}

impl ObjectType {
    pub fn as_tag(self) -> &'static str {
        match self {
            ObjectType::Content => "cnt",
            ObjectType::Directory => "dir",
            ObjectType::Revision => "rev",
            ObjectType::Release => "rel",
            ObjectType::Snapshot => "snp",
        }
    }
    pub fn from_tag(tag: &str) -> Result<Self, SwhidError> {
        match tag {
            "cnt" => Ok(Self::Content),
            "dir" => Ok(Self::Directory),
            "rev" => Ok(Self::Revision),
            "rel" => Ok(Self::Release),
            "snp" => Ok(Self::Snapshot),
            other => Err(SwhidError::InvalidObjectType(other.to_owned())),
        }
    }
}

/// A core SWHID: `swh:1:<tag>:<hex-digest>`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Swhid {
    object_type: ObjectType,
    /// Lowercase hex sha1 digest (20 bytes -> 40 hex chars)
    digest: [u8; 20],
}

impl Swhid {
    pub const VERSION: &'static str = "1";

    pub fn new(object_type: ObjectType, digest: [u8; 20]) -> Self {
        Self {
            object_type,
            digest,
        }
    }
    pub fn object_type(&self) -> ObjectType {
        self.object_type
    }
    pub fn digest_bytes(&self) -> &[u8; 20] {
        &self.digest
    }

    pub fn digest_hex(&self) -> String {
        hex::encode(self.digest)
    }
}

impl Display for Swhid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "swh:{}:{}:{}",
            Self::VERSION,
            self.object_type.as_tag(),
            self.digest_hex()
        )
    }
}

impl FromStr for Swhid {
    type Err = SwhidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Expect: swh:1:<tag>:<digest-hex>
        let mut it = s.split(':');
        let scheme = it
            .next()
            .ok_or_else(|| SwhidError::InvalidFormat(s.to_owned()))?;
        if scheme != "swh" {
            return Err(SwhidError::InvalidScheme(scheme.to_owned()));
        }
        let ver = it
            .next()
            .ok_or_else(|| SwhidError::InvalidFormat(s.to_owned()))?;
        if ver != Self::VERSION {
            return Err(SwhidError::InvalidVersion(ver.to_owned()));
        }
        let tag = it
            .next()
            .ok_or_else(|| SwhidError::InvalidFormat(s.to_owned()))?;
        let object_type = ObjectType::from_tag(tag)?;
        let digest_hex = it
            .next()
            .ok_or_else(|| SwhidError::InvalidFormat(s.to_owned()))?;

        if it.next().is_some() {
            // too many parts
            return Err(SwhidError::InvalidFormat(s.to_owned()));
        }
        if digest_hex.len() != 40
            || !digest_hex
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9'|b'a'..=b'f'))
        {
            return Err(SwhidError::InvalidDigest(digest_hex.to_owned()));
        }
        let mut raw = [0u8; 20];
        hex::decode_to_slice(digest_hex, &mut raw)
            .map_err(|_| SwhidError::InvalidDigest(digest_hex.to_owned()))?;
        Ok(Swhid::new(object_type, raw))
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Swhid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

#[cfg(feature = "serde")]
struct SwhidVisitor;

#[cfg(feature = "serde")]
impl serde::de::Visitor<'_> for SwhidVisitor {
    type Value = Swhid;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a SWHID")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        value.parse().map_err(E::custom)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Swhid {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        deserializer.deserialize_str(SwhidVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_core() {
        let id: Swhid = "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse()
            .unwrap();
        assert_eq!(id.object_type(), ObjectType::Content);
        assert_eq!(
            id.to_string(),
            "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
    }

    #[test]
    fn object_type_as_tag() {
        assert_eq!(ObjectType::Content.as_tag(), "cnt");
        assert_eq!(ObjectType::Directory.as_tag(), "dir");
        assert_eq!(ObjectType::Revision.as_tag(), "rev");
        assert_eq!(ObjectType::Release.as_tag(), "rel");
        assert_eq!(ObjectType::Snapshot.as_tag(), "snp");
    }

    #[test]
    fn object_type_from_tag() {
        assert_eq!(ObjectType::from_tag("cnt").unwrap(), ObjectType::Content);
        assert_eq!(ObjectType::from_tag("dir").unwrap(), ObjectType::Directory);
        assert_eq!(ObjectType::from_tag("rev").unwrap(), ObjectType::Revision);
        assert_eq!(ObjectType::from_tag("rel").unwrap(), ObjectType::Release);
        assert_eq!(ObjectType::from_tag("snp").unwrap(), ObjectType::Snapshot);
    }

    #[test]
    fn object_type_from_tag_invalid() {
        assert!(ObjectType::from_tag("invalid").is_err());
        assert!(ObjectType::from_tag("").is_err());
        assert!(ObjectType::from_tag("CNT").is_err());
    }

    #[test]
    fn object_type_equality() {
        assert_eq!(ObjectType::Content, ObjectType::Content);
        assert_ne!(ObjectType::Content, ObjectType::Directory);
    }

    #[test]
    fn object_type_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(ObjectType::Content, "content");
        map.insert(ObjectType::Directory, "directory");
        assert_eq!(map.get(&ObjectType::Content), Some(&"content"));
        assert_eq!(map.get(&ObjectType::Directory), Some(&"directory"));
    }

    #[test]
    fn object_type_debug() {
        let debug_str = format!("{:?}", ObjectType::Content);
        assert!(debug_str.contains("Content"));
    }

    #[test]
    fn object_type_copy() {
        let original = ObjectType::Content;
        let copied = original;
        assert_eq!(original, copied);
    }

    #[test]
    fn swhid_new() {
        let digest = [0u8; 20];
        let swhid = Swhid::new(ObjectType::Content, digest);
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(swhid.digest_bytes(), &digest);
    }

    #[test]
    fn swhid_version() {
        assert_eq!(Swhid::VERSION, "1");
    }

    #[test]
    fn swhid_digest_hex() {
        let digest = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC,
        ];
        let swhid = Swhid::new(ObjectType::Content, digest);
        assert_eq!(
            swhid.digest_hex(),
            "123456789abcdef0112233445566778899aabbcc"
        );
    }

    #[test]
    fn swhid_display() {
        let digest = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC,
        ];
        let swhid = Swhid::new(ObjectType::Content, digest);
        assert_eq!(
            swhid.to_string(),
            "swh:1:cnt:123456789abcdef0112233445566778899aabbcc"
        );
    }

    #[test]
    fn swhid_display_different_types() {
        let digest = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC,
        ];

        let content = Swhid::new(ObjectType::Content, digest);
        let directory = Swhid::new(ObjectType::Directory, digest);
        let revision = Swhid::new(ObjectType::Revision, digest);
        let release = Swhid::new(ObjectType::Release, digest);
        let snapshot = Swhid::new(ObjectType::Snapshot, digest);

        assert_eq!(
            content.to_string(),
            "swh:1:cnt:123456789abcdef0112233445566778899aabbcc"
        );
        assert_eq!(
            directory.to_string(),
            "swh:1:dir:123456789abcdef0112233445566778899aabbcc"
        );
        assert_eq!(
            revision.to_string(),
            "swh:1:rev:123456789abcdef0112233445566778899aabbcc"
        );
        assert_eq!(
            release.to_string(),
            "swh:1:rel:123456789abcdef0112233445566778899aabbcc"
        );
        assert_eq!(
            snapshot.to_string(),
            "swh:1:snp:123456789abcdef0112233445566778899aabbcc"
        );
    }

    #[test]
    fn swhid_parse_valid() {
        let swhid: Swhid = "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse()
            .unwrap();
        assert_eq!(swhid.object_type(), ObjectType::Content);
        assert_eq!(
            swhid.digest_hex(),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
    }

    #[test]
    fn swhid_parse_different_types() {
        let content: Swhid = "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse()
            .unwrap();
        let directory: Swhid = "swh:1:dir:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse()
            .unwrap();
        let revision: Swhid = "swh:1:rev:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse()
            .unwrap();
        let release: Swhid = "swh:1:rel:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse()
            .unwrap();
        let snapshot: Swhid = "swh:1:snp:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse()
            .unwrap();

        assert_eq!(content.object_type(), ObjectType::Content);
        assert_eq!(directory.object_type(), ObjectType::Directory);
        assert_eq!(revision.object_type(), ObjectType::Revision);
        assert_eq!(release.object_type(), ObjectType::Release);
        assert_eq!(snapshot.object_type(), ObjectType::Snapshot);
    }

    #[test]
    fn swhid_parse_invalid_scheme() {
        assert!("http:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse::<Swhid>()
            .is_err());
        assert!("ftp:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse::<Swhid>()
            .is_err());
    }

    #[test]
    fn swhid_parse_invalid_version() {
        assert!("swh:2:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse::<Swhid>()
            .is_err());
        assert!("swh:0:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse::<Swhid>()
            .is_err());
    }

    #[test]
    fn swhid_parse_invalid_object_type() {
        assert!("swh:1:invalid:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse::<Swhid>()
            .is_err());
        assert!("swh:1:CNT:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse::<Swhid>()
            .is_err());
    }

    #[test]
    fn swhid_parse_invalid_digest_length() {
        assert!("swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c539"
            .parse::<Swhid>()
            .is_err()); // too short
        assert!("swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391a"
            .parse::<Swhid>()
            .is_err()); // too long
    }

    #[test]
    fn swhid_parse_invalid_digest_chars() {
        assert!("swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c539g"
            .parse::<Swhid>()
            .is_err()); // invalid char
        assert!("swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c539!"
            .parse::<Swhid>()
            .is_err()); // invalid char
    }

    #[test]
    fn swhid_parse_invalid_format() {
        assert!("swh:1:cnt".parse::<Swhid>().is_err()); // missing digest
        assert!("swh:1".parse::<Swhid>().is_err()); // missing object type and digest
        assert!("swh".parse::<Swhid>().is_err()); // missing version, object type and digest
        assert!("".parse::<Swhid>().is_err()); // empty string
    }

    #[test]
    fn swhid_parse_too_many_parts() {
        assert!("swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391:extra"
            .parse::<Swhid>()
            .is_err());
    }

    #[test]
    fn swhid_parse_case_sensitive() {
        assert!("swh:1:CNT:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse::<Swhid>()
            .is_err());
        assert!("swh:1:cnt:E69DE29BB2D1D6434B8B29AE775AD8C2E48C5391"
            .parse::<Swhid>()
            .is_err());
    }

    #[test]
    fn swhid_equality() {
        let digest1 = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC,
        ];
        let digest2 = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCD,
        ];

        let swhid1 = Swhid::new(ObjectType::Content, digest1);
        let swhid2 = Swhid::new(ObjectType::Content, digest1);
        let swhid3 = Swhid::new(ObjectType::Content, digest2);
        let swhid4 = Swhid::new(ObjectType::Directory, digest1);

        assert_eq!(swhid1, swhid2);
        assert_ne!(swhid1, swhid3);
        assert_ne!(swhid1, swhid4);
    }

    #[test]
    fn swhid_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let digest = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC,
        ];
        let swhid = Swhid::new(ObjectType::Content, digest);
        map.insert(swhid.clone(), "content");
        assert_eq!(map.get(&swhid), Some(&"content"));
    }

    #[test]
    fn swhid_clone() {
        let digest = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC,
        ];
        let swhid1 = Swhid::new(ObjectType::Content, digest);
        let swhid2 = swhid1.clone();
        assert_eq!(swhid1, swhid2);
    }

    #[test]
    fn swhid_debug() {
        let digest = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC,
        ];
        let swhid = Swhid::new(ObjectType::Content, digest);
        let debug_str = format!("{swhid:?}");
        assert!(debug_str.contains("Swhid"));
        assert!(debug_str.contains("Content"));
    }

    #[test]
    fn swhid_roundtrip() {
        let original = "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
        let parsed: Swhid = original.parse().unwrap();
        let formatted = parsed.to_string();
        assert_eq!(original, formatted);
    }

    #[test]
    fn swhid_roundtrip_different_types() {
        let types = ["cnt", "dir", "rev", "rel", "snp"];
        let digest = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";

        for obj_type in &types {
            let original = format!("swh:1:{obj_type}:{digest}");
            let parsed: Swhid = original.parse().unwrap();
            let formatted = parsed.to_string();
            assert_eq!(original, formatted);
        }
    }

    #[test]
    fn swhid_roundtrip_different_digests() {
        let digests = [
            "0000000000000000000000000000000000000000",
            "ffffffffffffffffffffffffffffffffffffffff",
            "123456789abcdef0112233445566778899aabbcc",
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391",
        ];

        for digest in &digests {
            let original = format!("swh:1:cnt:{digest}");
            let parsed: Swhid = original.parse().unwrap();
            let formatted = parsed.to_string();
            assert_eq!(original, formatted);
        }
    }

    #[test]
    fn swhid_parse_whitespace() {
        assert!(" swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse::<Swhid>()
            .is_err());
        assert!("swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 "
            .parse::<Swhid>()
            .is_err());
        assert!(" swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 "
            .parse::<Swhid>()
            .is_err());
    }

    #[test]
    fn swhid_parse_uppercase_digest() {
        assert!("swh:1:cnt:E69DE29BB2D1D6434B8B29AE775AD8C2E48C5391"
            .parse::<Swhid>()
            .is_err());
    }

    #[test]
    fn swhid_parse_mixed_case_digest() {
        assert!("swh:1:cnt:E69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
            .parse::<Swhid>()
            .is_err());
    }

    #[test]
    fn swhid_parse_special_chars() {
        assert!("swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391\n"
            .parse::<Swhid>()
            .is_err());
        assert!("swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391\t"
            .parse::<Swhid>()
            .is_err());
    }
}
