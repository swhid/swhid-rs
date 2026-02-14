//! SWHID version and related types.

/// SWHID version: v1 (20-byte hex) or v2 (variable digest/encoding).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwhidVersion {
    /// Version 1: 20-byte SHA-1, lowercase hex.
    V1,
    /// Version 2: configurable hash and encoding.
    V2,
}

impl SwhidVersion {
    pub fn as_str(self) -> &'static str {
        match self {
            SwhidVersion::V1 => "1",
            SwhidVersion::V2 => "2",
        }
    }
}
