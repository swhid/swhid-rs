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
//! 
//! ## Usage
//! 
//! ```rust
//! use swhid::{Swhid, ObjectType};
//! 
//! // Create a SWHID manually
//! let hash = [0u8; 20];
//! let swhid = Swhid::new(ObjectType::Content, hash);
//! assert_eq!(swhid.to_string(), "swh:1:cnt:0000000000000000000000000000000000000000");
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

pub use swhid::{Swhid, ObjectType};
pub use error::SwhidError;
pub use computer::SwhidComputer;
pub use content::Content;
pub use directory::Directory; 