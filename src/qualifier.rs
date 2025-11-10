use std::fmt::{self, Display};
use std::str::FromStr;

use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet};

use crate::core::Swhid;
use crate::error::SwhidError;

/// Fragment sub‑selectors
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LineRange {
    pub start: u64,
    pub end: Option<u64>, // inclusive range like "9-15", or single "9"
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ByteRange {
    pub start: u64,
    pub end: Option<u64>,
}

fn parse_range(s: &str) -> Result<(u64, Option<u64>), SwhidError> {
    if let Some((a, b)) = s.split_once('-') {
        let start: u64 = a.parse().map_err(|_| SwhidError::InvalidQualifierValue {
            key: "range".into(),
            value: s.into(),
        })?;
        let end: u64 = b.parse().map_err(|_| SwhidError::InvalidQualifierValue {
            key: "range".into(),
            value: s.into(),
        })?;
        if end < start {
            return Err(SwhidError::InvalidQualifierValue {
                key: "range".into(),
                value: s.into(),
            });
        }
        Ok((start, Some(end)))
    } else {
        let start: u64 = s.parse().map_err(|_| SwhidError::InvalidQualifierValue {
            key: "range".into(),
            value: s.into(),
        })?;
        Ok((start, None))
    }
}

impl Display for LineRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.end {
            Some(e) => write!(f, "{}-{}", self.start, e),
            None => write!(f, "{}", self.start),
        }
    }
}
impl Display for ByteRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.end {
            Some(e) => write!(f, "{}-{}", self.start, e),
            None => write!(f, "{}", self.start),
        }
    }
}

/// Known qualifier keys (order in output is canonicalized).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnownKey {
    Origin,
    Visit,
    Anchor,
    Path,
    Lines,
    Bytes,
}

impl KnownKey {
    pub fn as_str(self) -> &'static str {
        match self {
            KnownKey::Origin => "origin",
            KnownKey::Visit => "visit",
            KnownKey::Anchor => "anchor",
            KnownKey::Path => "path",
            KnownKey::Lines => "lines",
            KnownKey::Bytes => "bytes",
        }
    }
}

/// A qualified SWHID with optional qualifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualifiedSwhid {
    core: Swhid,
    origin: Option<String>,
    visit: Option<Swhid>,
    anchor: Option<Swhid>,
    path: Option<String>,
    lines: Option<LineRange>,
    bytes: Option<ByteRange>,
    // future‑proof: unknown qualifiers we preserve round‑trip
    others: Vec<(String, String)>,
}

impl QualifiedSwhid {
    pub fn new(core: Swhid) -> Self {
        Self {
            core,
            origin: None,
            visit: None,
            anchor: None,
            path: None,
            lines: None,
            bytes: None,
            others: vec![],
        }
    }
    pub fn core(&self) -> &Swhid {
        &self.core
    }

    pub fn with_origin(mut self, url: impl Into<String>) -> Self {
        self.origin = Some(url.into());
        self
    }
    pub fn with_visit(mut self, id: Swhid) -> Self {
        self.visit = Some(id);
        self
    }
    pub fn with_anchor(mut self, id: Swhid) -> Self {
        self.anchor = Some(id);
        self
    }
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
    pub fn with_lines(mut self, lines: LineRange) -> Self {
        self.lines = Some(lines);
        self
    }
    pub fn with_bytes(mut self, bytes: ByteRange) -> Self {
        self.bytes = Some(bytes);
        self
    }

    pub fn push_unknown(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.others.push((key.into(), value.into()));
        self
    }
}

const ESCAPED: &AsciiSet = &AsciiSet::EMPTY.add(b';');

impl Display for QualifiedSwhid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.core)?;
        let mut sep = ';';
        let mut write_kv = |k: &str, v: String, f: &mut fmt::Formatter<'_>| -> fmt::Result {
            write!(f, "{sep}{k}={v}")?;
            sep = ';';
            Ok(())
        };
        if let Some(o) = &self.origin {
            write_kv("origin", utf8_percent_encode(o, ESCAPED).to_string(), f)?;
        }
        if let Some(v) = &self.visit {
            write_kv("visit", v.to_string(), f)?;
        }
        if let Some(a) = &self.anchor {
            write_kv("anchor", a.to_string(), f)?;
        }
        if let Some(p) = &self.path {
            write_kv("path", utf8_percent_encode(p, ESCAPED).to_string(), f)?;
        }
        if let Some(l) = &self.lines {
            write_kv("lines", l.to_string(), f)?;
        }
        if let Some(b) = &self.bytes {
            write_kv("bytes", b.to_string(), f)?;
        }
        for (k, v) in &self.others {
            write_kv(k, v.clone(), f)?;
        }
        Ok(())
    }
}

fn parse_string_qualifier(key: &'static str, value: &str) -> Result<String, SwhidError> {
    Ok(percent_decode_str(value)
        .decode_utf8()
        .map_err(|_| SwhidError::InvalidQualifierValue {
            key: key.to_string(),
            value: value.to_owned(),
        })?
        .into_owned())
}

impl FromStr for QualifiedSwhid {
    type Err = SwhidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (core_str, qstr) = match s.split_once(';') {
            Some((c, rest)) => (c, Some(rest)),
            None => (s, None),
        };
        let core: Swhid = core_str.parse()?;
        let mut q = QualifiedSwhid::new(core);
        if let Some(qstr) = qstr {
            for item in qstr.split(';') {
                if item.is_empty() {
                    continue;
                }
                let (k, v) = item
                    .split_once('=')
                    .ok_or_else(|| SwhidError::InvalidFormat(item.into()))?;
                if k.is_empty() {
                    return Err(SwhidError::InvalidFormat(item.into()));
                }
                match k {
                    "origin" => q.origin = Some(parse_string_qualifier("origin", v)?),
                    "visit" => q.visit = Some(v.parse()?),
                    "anchor" => q.anchor = Some(v.parse()?),
                    "path" => q.path = Some(parse_string_qualifier("path", v)?),
                    "lines" => {
                        let (s, e) = super::qualifier::parse_range(v)?;
                        q.lines = Some(LineRange { start: s, end: e });
                    }
                    "bytes" => {
                        let (s, e) = super::qualifier::parse_range(v)?;
                        q.bytes = Some(ByteRange { start: s, end: e });
                    }
                    other => q.others.push((other.to_owned(), v.to_owned())),
                }
            }
        }
        Ok(q)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for QualifiedSwhid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

#[cfg(feature = "serde")]
struct QualifiedSwhidVisitor;

#[cfg(feature = "serde")]
impl serde::de::Visitor<'_> for QualifiedSwhidVisitor {
    type Value = QualifiedSwhid;

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
impl<'de> serde::Deserialize<'de> for QualifiedSwhid {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        deserializer.deserialize_str(QualifiedSwhidVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_qualified() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let q = QualifiedSwhid::new(core.clone())
            .with_origin("https://example.org/repo.git")
            .with_path("/src/lib.rs")
            .with_lines(LineRange {
                start: 9,
                end: Some(15),
            });
        let s = q.to_string();
        assert_eq!(s, "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;origin=https://example.org/repo.git;path=/src/lib.rs;lines=9-15");

        let parsed: QualifiedSwhid = s.parse().unwrap();
        assert_eq!(parsed.core(), &core);
    }

    #[test]
    fn line_range_display() {
        let single = LineRange {
            start: 10,
            end: None,
        };
        assert_eq!(single.to_string(), "10");

        let range = LineRange {
            start: 10,
            end: Some(20),
        };
        assert_eq!(range.to_string(), "10-20");
    }

    #[test]
    fn byte_range_display() {
        let single = ByteRange {
            start: 100,
            end: None,
        };
        assert_eq!(single.to_string(), "100");

        let range = ByteRange {
            start: 100,
            end: Some(200),
        };
        assert_eq!(range.to_string(), "100-200");
    }

    #[test]
    fn line_range_equality() {
        let range1 = LineRange {
            start: 10,
            end: Some(20),
        };
        let range2 = LineRange {
            start: 10,
            end: Some(20),
        };
        let range3 = LineRange {
            start: 10,
            end: None,
        };
        let range4 = LineRange {
            start: 11,
            end: Some(20),
        };

        assert_eq!(range1, range2);
        assert_ne!(range1, range3);
        assert_ne!(range1, range4);
    }

    #[test]
    fn byte_range_equality() {
        let range1 = ByteRange {
            start: 100,
            end: Some(200),
        };
        let range2 = ByteRange {
            start: 100,
            end: Some(200),
        };
        let range3 = ByteRange {
            start: 100,
            end: None,
        };
        let range4 = ByteRange {
            start: 101,
            end: Some(200),
        };

        assert_eq!(range1, range2);
        assert_ne!(range1, range3);
        assert_ne!(range1, range4);
    }

    #[test]
    fn line_range_debug() {
        let range = LineRange {
            start: 10,
            end: Some(20),
        };
        let debug_str = format!("{range:?}");
        assert!(debug_str.contains("LineRange"));
        assert!(debug_str.contains("10"));
        assert!(debug_str.contains("20"));
    }

    #[test]
    fn byte_range_debug() {
        let range = ByteRange {
            start: 100,
            end: Some(200),
        };
        let debug_str = format!("{range:?}");
        assert!(debug_str.contains("ByteRange"));
        assert!(debug_str.contains("100"));
        assert!(debug_str.contains("200"));
    }

    #[test]
    fn line_range_clone() {
        let range1 = LineRange {
            start: 10,
            end: Some(20),
        };
        let range2 = range1.clone();
        assert_eq!(range1, range2);
    }

    #[test]
    fn byte_range_clone() {
        let range1 = ByteRange {
            start: 100,
            end: Some(200),
        };
        let range2 = range1.clone();
        assert_eq!(range1, range2);
    }

    #[test]
    fn known_key_as_str() {
        assert_eq!(KnownKey::Origin.as_str(), "origin");
        assert_eq!(KnownKey::Visit.as_str(), "visit");
        assert_eq!(KnownKey::Anchor.as_str(), "anchor");
        assert_eq!(KnownKey::Path.as_str(), "path");
        assert_eq!(KnownKey::Lines.as_str(), "lines");
        assert_eq!(KnownKey::Bytes.as_str(), "bytes");
    }

    #[test]
    fn known_key_equality() {
        assert_eq!(KnownKey::Origin, KnownKey::Origin);
        assert_ne!(KnownKey::Origin, KnownKey::Visit);
    }

    #[test]
    fn known_key_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(KnownKey::Origin, "origin");
        map.insert(KnownKey::Visit, "visit");
        assert_eq!(map.get(&KnownKey::Origin), Some(&"origin"));
        assert_eq!(map.get(&KnownKey::Visit), Some(&"visit"));
    }

    #[test]
    fn known_key_debug() {
        let debug_str = format!("{:?}", KnownKey::Origin);
        assert!(debug_str.contains("Origin"));
    }

    #[test]
    fn known_key_copy() {
        let original = KnownKey::Origin;
        let copied = original;
        assert_eq!(original, copied);
    }

    #[test]
    fn qualified_swhid_new() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let q = QualifiedSwhid::new(core.clone());
        assert_eq!(q.core(), &core);
        assert!(q.origin.is_none());
        assert!(q.visit.is_none());
        assert!(q.anchor.is_none());
        assert!(q.path.is_none());
        assert!(q.lines.is_none());
        assert!(q.bytes.is_none());
        assert!(q.others.is_empty());
    }

    #[test]
    fn qualified_swhid_with_origin() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let q = QualifiedSwhid::new(core).with_origin("https://example.org/repo.git");
        assert_eq!(q.origin, Some("https://example.org/repo.git".to_string()));
    }

    #[test]
    fn qualified_swhid_with_origin_no_escape() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let q =
            QualifiedSwhid::new(core).with_origin("https://example.org/repo.git?foo=bar:baz qux");
        assert_eq!(
            q.origin,
            Some("https://example.org/repo.git?foo=bar:baz qux".to_string())
        );
    }

    #[test]
    fn qualified_swhid_with_visit() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let visit: Swhid = "swh:1:snp:123456789abcdef0112233445566778899aabbcc"
            .parse()
            .unwrap();
        let q = QualifiedSwhid::new(core).with_visit(visit.clone());
        assert_eq!(q.visit, Some(visit));
    }

    #[test]
    fn qualified_swhid_with_anchor() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let anchor: Swhid = "swh:1:dir:123456789abcdef0112233445566778899aabbcc"
            .parse()
            .unwrap();
        let q = QualifiedSwhid::new(core).with_anchor(anchor.clone());
        assert_eq!(q.anchor, Some(anchor));
    }

    #[test]
    fn qualified_swhid_with_path() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let q = QualifiedSwhid::new(core).with_path("/src/lib.rs");
        assert_eq!(q.path, Some("/src/lib.rs".to_string()));
    }

    #[test]
    fn qualified_swhid_with_lines() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let lines = LineRange {
            start: 10,
            end: Some(20),
        };
        let q = QualifiedSwhid::new(core).with_lines(lines.clone());
        assert_eq!(q.lines, Some(lines));
    }

    #[test]
    fn qualified_swhid_with_bytes() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let bytes = ByteRange {
            start: 100,
            end: Some(200),
        };
        let q = QualifiedSwhid::new(core).with_bytes(bytes.clone());
        assert_eq!(q.bytes, Some(bytes));
    }

    #[test]
    fn qualified_swhid_push_unknown() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let q = QualifiedSwhid::new(core).push_unknown("custom", "value");
        assert_eq!(q.others.len(), 1);
        assert_eq!(q.others[0], ("custom".to_string(), "value".to_string()));
    }

    #[test]
    fn qualified_swhid_chaining() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let q = QualifiedSwhid::new(core)
            .with_origin("https://example.org/repo.git")
            .with_path("/src/lib.rs")
            .with_lines(LineRange {
                start: 10,
                end: Some(20),
            })
            .push_unknown("custom", "value");

        assert_eq!(q.origin, Some("https://example.org/repo.git".to_string()));
        assert_eq!(q.path, Some("/src/lib.rs".to_string()));
        assert_eq!(
            q.lines,
            Some(LineRange {
                start: 10,
                end: Some(20)
            })
        );
        assert_eq!(q.others.len(), 1);
    }

    #[test]
    fn qualified_swhid_display() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let q = QualifiedSwhid::new(core);
        let s = q.to_string();
        assert_eq!(s, "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684");
    }

    #[test]
    fn qualified_swhid_display_with_qualifiers() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let q = QualifiedSwhid::new(core)
            .with_origin("https://example.org/repo.git")
            .with_path("/src/lib.rs");
        let s = q.to_string();
        assert_eq!(s, "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;origin=https://example.org/repo.git;path=/src/lib.rs");
    }

    #[test]
    fn qualified_swhid_with_escaped_qualifiers() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let q = QualifiedSwhid::new(core)
            .with_origin("https://example.org/repo.git?foo=bar:baz qux;quux")
            .with_path("/this;is\u{00A0}not?a=very:good file\0name");
        let s = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;origin=https://example.org/repo.git?foo=bar:baz qux%3Bquux;path=/this%3Bis%C2%A0not?a=very:good file\0name";
        assert_eq!(q.to_string(), s);
        assert_eq!(s.parse::<QualifiedSwhid>().unwrap(), q);
    }

    #[test]
    fn qualified_swhid_parse_basic() {
        let s = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684";
        let q: QualifiedSwhid = s.parse().unwrap();
        assert_eq!(q.core().to_string(), s);
        assert!(q.origin.is_none());
        assert!(q.visit.is_none());
        assert!(q.anchor.is_none());
        assert!(q.path.is_none());
        assert!(q.lines.is_none());
        assert!(q.bytes.is_none());
        assert!(q.others.is_empty());
    }

    #[test]
    fn qualified_swhid_parse_with_qualifiers() {
        let s = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;origin=https://example.org/repo.git;path=/src/lib.rs";
        let q: QualifiedSwhid = s.parse().unwrap();
        assert_eq!(q.origin, Some("https://example.org/repo.git".to_string()));
        assert_eq!(q.path, Some("/src/lib.rs".to_string()));
    }

    #[test]
    fn qualified_swhid_parse_with_visit() {
        let s = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;visit=swh:1:snp:123456789abcdef0112233445566778899aabbcc";
        let q: QualifiedSwhid = s.parse().unwrap();
        assert_eq!(
            q.visit,
            Some(
                "swh:1:snp:123456789abcdef0112233445566778899aabbcc"
                    .parse()
                    .unwrap()
            )
        );
    }

    #[test]
    fn qualified_swhid_parse_with_anchor() {
        let s = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;anchor=swh:1:dir:123456789abcdef0112233445566778899aabbcc";
        let q: QualifiedSwhid = s.parse().unwrap();
        assert_eq!(
            q.anchor,
            Some(
                "swh:1:dir:123456789abcdef0112233445566778899aabbcc"
                    .parse()
                    .unwrap()
            )
        );
    }

    #[test]
    fn qualified_swhid_parse_with_lines() {
        let s = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;lines=10-20";
        let q: QualifiedSwhid = s.parse().unwrap();
        assert_eq!(
            q.lines,
            Some(LineRange {
                start: 10,
                end: Some(20)
            })
        );
    }

    #[test]
    fn qualified_swhid_parse_with_lines_single() {
        let s = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;lines=10";
        let q: QualifiedSwhid = s.parse().unwrap();
        assert_eq!(
            q.lines,
            Some(LineRange {
                start: 10,
                end: None
            })
        );
    }

    #[test]
    fn qualified_swhid_parse_with_bytes() {
        let s = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;bytes=100-200";
        let q: QualifiedSwhid = s.parse().unwrap();
        assert_eq!(
            q.bytes,
            Some(ByteRange {
                start: 100,
                end: Some(200)
            })
        );
    }

    #[test]
    fn qualified_swhid_parse_with_bytes_single() {
        let s = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;bytes=100";
        let q: QualifiedSwhid = s.parse().unwrap();
        assert_eq!(
            q.bytes,
            Some(ByteRange {
                start: 100,
                end: None
            })
        );
    }

    #[test]
    fn qualified_swhid_parse_with_unknown() {
        let s = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;custom=value";
        let q: QualifiedSwhid = s.parse().unwrap();
        assert_eq!(q.others.len(), 1);
        assert_eq!(q.others[0], ("custom".to_string(), "value".to_string()));
    }

    #[test]
    fn qualified_swhid_parse_with_multiple_unknown() {
        let s = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;custom1=value1;custom2=value2";
        let q: QualifiedSwhid = s.parse().unwrap();
        assert_eq!(q.others.len(), 2);
        assert!(q
            .others
            .contains(&("custom1".to_string(), "value1".to_string())));
        assert!(q
            .others
            .contains(&("custom2".to_string(), "value2".to_string())));
    }

    #[test]
    fn qualified_swhid_parse_empty_qualifiers() {
        let s = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;";
        let q: QualifiedSwhid = s.parse().unwrap();
        assert_eq!(
            q.core().to_string(),
            "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
        );
    }

    #[test]
    fn qualified_swhid_parse_invalid_format() {
        assert!("swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;invalid"
            .parse::<QualifiedSwhid>()
            .is_err());
        assert!("swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;=value"
            .parse::<QualifiedSwhid>()
            .is_err());
    }

    #[test]
    fn qualified_swhid_parse_invalid_visit() {
        assert!(
            "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;visit=invalid"
                .parse::<QualifiedSwhid>()
                .is_err()
        );
    }

    #[test]
    fn qualified_swhid_parse_invalid_anchor() {
        assert!(
            "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;anchor=invalid"
                .parse::<QualifiedSwhid>()
                .is_err()
        );
    }

    #[test]
    fn qualified_swhid_parse_invalid_lines() {
        assert!(
            "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;lines=invalid"
                .parse::<QualifiedSwhid>()
                .is_err()
        );
        assert!(
            "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;lines=20-10"
                .parse::<QualifiedSwhid>()
                .is_err()
        );
    }

    #[test]
    fn qualified_swhid_parse_invalid_bytes() {
        assert!(
            "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;bytes=invalid"
                .parse::<QualifiedSwhid>()
                .is_err()
        );
        assert!(
            "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;bytes=200-100"
                .parse::<QualifiedSwhid>()
                .is_err()
        );
    }

    #[test]
    fn qualified_swhid_equality() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let q1 = QualifiedSwhid::new(core.clone()).with_origin("https://example.org/repo.git");
        let q2 = QualifiedSwhid::new(core.clone()).with_origin("https://example.org/repo.git");
        let q3 = QualifiedSwhid::new(core).with_origin("https://example.org/other.git");

        assert_eq!(q1, q2);
        assert_ne!(q1, q3);
    }

    #[test]
    fn qualified_swhid_clone() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let q1 = QualifiedSwhid::new(core).with_origin("https://example.org/repo.git");
        let q2 = q1.clone();
        assert_eq!(q1, q2);
    }

    #[test]
    fn qualified_swhid_debug() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let q = QualifiedSwhid::new(core).with_origin("https://example.org/repo.git");
        let debug_str = format!("{q:?}");
        assert!(debug_str.contains("QualifiedSwhid"));
    }

    #[test]
    fn qualified_swhid_roundtrip() {
        let original = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;origin=https://example.org/repo.git;path=/src/lib.rs;lines=10-20";
        let parsed: QualifiedSwhid = original.parse().unwrap();
        let formatted = parsed.to_string();
        assert_eq!(original, formatted);
    }

    #[test]
    fn qualified_swhid_roundtrip_complex() {
        let core: Swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684"
            .parse()
            .unwrap();
        let visit: Swhid = "swh:1:snp:123456789abcdef0112233445566778899aabbcc"
            .parse()
            .unwrap();
        let anchor: Swhid = "swh:1:dir:123456789abcdef0112233445566778899aabbcc"
            .parse()
            .unwrap();

        let q = QualifiedSwhid::new(core)
            .with_origin("https://example.org/repo.git")
            .with_visit(visit)
            .with_anchor(anchor)
            .with_path("/src/lib.rs")
            .with_lines(LineRange {
                start: 10,
                end: Some(20),
            })
            .with_bytes(ByteRange {
                start: 100,
                end: Some(200),
            })
            .push_unknown("custom1", "value1")
            .push_unknown("custom2", "value2");

        let formatted = q.to_string();
        let parsed: QualifiedSwhid = formatted.parse().unwrap();
        assert_eq!(q, parsed);
    }

    #[test]
    fn parse_range_valid() {
        assert_eq!(parse_range("10").unwrap(), (10, None));
        assert_eq!(parse_range("10-20").unwrap(), (10, Some(20)));
        assert_eq!(parse_range("0").unwrap(), (0, None));
        assert_eq!(parse_range("0-0").unwrap(), (0, Some(0)));
    }

    #[test]
    fn parse_range_invalid() {
        assert!(parse_range("invalid").is_err());
        assert!(parse_range("10-5").is_err()); // end < start
        assert!(parse_range("-10").is_err());
        assert!(parse_range("10-").is_err());
    }

    #[test]
    fn parse_range_edge_cases() {
        assert_eq!(parse_range("0").unwrap(), (0, None));
        assert_eq!(parse_range("0-0").unwrap(), (0, Some(0)));
        assert_eq!(parse_range("1-1").unwrap(), (1, Some(1)));
    }
}
