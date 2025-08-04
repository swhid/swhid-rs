use std::io;

#[derive(Debug)]
pub enum SwhidError {
    Io(io::Error),
    InvalidFormat(String),
    InvalidNamespace(String),
    InvalidVersion(String),
    InvalidObjectType(String),
    InvalidHash(String),
    InvalidHashLength(usize),
    InvalidPath(String),
    DuplicateEntry(String),
    UnsupportedOperation(String),
    InvalidQualifier(String),
    InvalidQualifierValue(String),
    UnknownQualifier(String),
    InvalidInput(String),
}

impl From<io::Error> for SwhidError {
    fn from(err: io::Error) -> Self {
        SwhidError::Io(err)
    }
}

impl std::fmt::Display for SwhidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwhidError::Io(e) => write!(f, "I/O error: {}", e),
            SwhidError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            SwhidError::InvalidNamespace(s) => write!(f, "Invalid namespace: {}", s),
            SwhidError::InvalidVersion(s) => write!(f, "Invalid version: {}", s),
            SwhidError::InvalidObjectType(s) => write!(f, "Invalid object type: {}", s),
            SwhidError::InvalidHash(s) => write!(f, "Invalid hash: {}", s),
            SwhidError::InvalidHashLength(len) => write!(f, "Invalid hash length: {} (expected 40)", len),
            SwhidError::InvalidPath(s) => write!(f, "Invalid path: {}", s),
            SwhidError::DuplicateEntry(s) => write!(f, "Duplicate entry: {}", s),
            SwhidError::UnsupportedOperation(s) => write!(f, "Unsupported operation: {}", s),
            SwhidError::InvalidQualifier(s) => write!(f, "Invalid qualifier: {}", s),
            SwhidError::InvalidQualifierValue(s) => write!(f, "Invalid qualifier value: {}", s),
            SwhidError::UnknownQualifier(s) => write!(f, "Unknown qualifier: {}", s),
            SwhidError::InvalidInput(s) => write!(f, "Invalid input: {}", s),
        }
    }
}

impl std::error::Error for SwhidError {} 