//! Minimal reference implementation of Software Hash Identifier (SWHID) computation
//! 
//! This library provides the SWHID functionality without extra features like
//! archive processing, complex Git operations, or performance optimizations.
//! 
//! ## Core SWHID Types
//! 
//! - **Content SWHID**: Compute SWHIDs for individual files
//! - **Directory SWHID**: Compute SWHIDs for directory trees
//! - **Basic SWHID**: Core SWHID format: `swh:1:obj_type:hash`
//! - **Qualified SWHID**: SWHIDs with qualifiers like origin, path, lines, bytes
//! 
//! ## Usage
//! 
//! ```rust
//! use swhid::{Swhid, ObjectType, QualifiedSwhid};
//! 
//! // Create a SWHID manually
//! let hash = [0u8; 20];
//! let swhid = Swhid::new(ObjectType::Content, hash);
//! assert_eq!(swhid.to_string(), "swh:1:cnt:0000000000000000000000000000000000000000");
//! 
//! // Create a qualified SWHID
//! let qualified = QualifiedSwhid::new(swhid)
//!     .with_origin("https://github.com/user/repo".to_string())
//!     .with_path(b"/src/main.rs".to_vec())
//!     .with_lines(10, Some(20))
//!     .with_bytes(5, Some(10));
//! 
//! // Parse a SWHID from string
//! let parsed = Swhid::from_string("swh:1:dir:0000000000000000000000000000000000000000").unwrap();
//! assert_eq!(parsed.object_type(), ObjectType::Directory);
//! ```
//! 

pub mod swhid;
pub mod hash;
pub mod content;
pub mod directory;
pub mod error;
pub mod computer;

pub use swhid::{Swhid, ObjectType, QualifiedSwhid};
pub use error::SwhidError;
pub use computer::SwhidComputer;
pub use content::Content;
pub use directory::Directory; 