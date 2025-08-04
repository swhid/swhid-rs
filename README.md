# SWHID - Minimal Reference Implementation

A minimal, clean reference implementation of Software Hash Identifier (SWHID) computation in Rust.

## Overview

This library provides the **SWHID functionality** according to the official SWHID specification v1.2, without extra features like archive processing, complex Git operations, or performance optimizations. It serves as a clean reference implementation that can be used as a dependency for the full-featured `swhid-rs` library.

## Core SWHID Types

SWHIDs are persistent identifiers for software artifacts that follow the format:
```
swh:1:<object_type>:<40_character_hex_hash>
```

Where:
- `swh` is the namespace (always "swh")
- `1` is the scheme version (always 1)
- `<object_type>` is one of: `cnt`, `dir`, `rev`, `rel`, `snp`
- `<40_character_hex_hash>` is the SHA1 hash of the object

### Supported Object Types

According to the official SWHID specification:

- **`cnt`** - **Content**: Individual files and their contents
- **`dir`** - **Directory**: Directory trees and their structure
- **`rev`** - **Revision**: Git revisions and commits
- **`rel`** - **Release**: Git releases and tags
- **`snp`** - **Snapshot**: Git snapshots and repository states

## Features

- **Complete Core SWHID Support**: All 5 core object types from the specification
- **Git-compatible**: Uses Git's object format for hashing
- **Minimal Dependencies**: Only essential crates (sha1-checked, hex)
- **Reference Implementation**: Clean, readable code for SWHID specification
- **Specification Compliant**: Follows SWHID v1.2 specification exactly

## What's NOT Included

- Archive processing (tar, zip, etc.)
- Git repository operations (snapshot, revision, release computation)
- Extended SWHID types (Origin, Raw Extrinsic Metadata) - these are NOT part of the core spec
- Qualified SWHIDs with anchors, paths, and line ranges
- Performance optimizations (caching, statistics)
- Command-line interface
- Complex recursive traversal

## Installation

### From Source

```bash
git clone <repository-url>
cd swhid-rs
git checkout minimal-reference-impl
cargo build
```

### Using Cargo

Add to your `Cargo.toml`:
```toml
[dependencies]
swhid-lib = "0.1.0"
```

## Usage

### Basic SWHID Computation

```rust
use swhid::{SwhidComputer, Swhid, ObjectType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let computer = SwhidComputer::new();
    
    // Compute SWHID for a file
    let file_swhid = computer.compute_file_swhid("/path/to/file.txt")?;
    println!("File SWHID: {}", file_swhid);
    
    // Compute SWHID for a directory
    let dir_swhid = computer.compute_directory_swhid("/path/to/directory")?;
    println!("Directory SWHID: {}", dir_swhid);
    
    // Auto-detect and compute SWHID
    let swhid = computer.compute_swhid("/path/to/object")?;
    println!("SWHID: {}", swhid);
    
    Ok(())
}
```

### Direct Object Usage

```rust
use swhid::{Content, Directory, Swhid, ObjectType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create content from data
    let content = Content::from_data(b"Hello, World!".to_vec());
    let content_swhid = content.swhid();
    println!("Content SWHID: {}", content_swhid);
    
    // Create directory from disk
    let mut dir = Directory::from_disk("/path/to/directory", &[], true)?;
    let dir_swhid = dir.swhid();
    println!("Directory SWHID: {}", dir_swhid);
    
    // Create other SWHID types manually
    let hash = [0u8; 20];
    let revision_swhid = Swhid::new(ObjectType::Revision, hash);
    let release_swhid = Swhid::new(ObjectType::Release, hash);
    let snapshot_swhid = Swhid::new(ObjectType::Snapshot, hash);
    
    Ok(())
}
```

## Architecture

```
src/
├── lib.rs          # library exports
├── swhid.rs        # Core SWHID types and formatting (all 5 object types)
├── hash.rs         # Basic hash computation
├── content.rs      # Content object handling
├── directory.rs    # Directory object handling
├── error.rs        # error types
└── computer.rs     # Minimal SWHIDComputer
```

## Testing

Run the core conformance tests:

```bash
cargo test --test core_tests
```

Run all tests including SWHID module tests:

```bash
cargo test
```

## Dependencies

- **sha1-checked**: Collision-resistant SHA1 hashing (SWHID requirement)
- **hex**: Hexadecimal encoding/decoding

## Use Cases

- **Reference Implementation**: Clean code for SWHID specification
- **Core Library**: Foundation for full-featured SWHID implementations
- **Testing**: Base implementation for conformance testing
- **Learning**: Simple, readable SWHID computation code
- **Specification Compliance**: Exact implementation of SWHID v1.2


## Specification Compliance

This implementation follows the **official SWHID specification v1.2** exactly:

- ✅ **Core Object Types**: All 5 types (cnt, dir, rev, rel, snp)
- ✅ **Format**: `swh:1:<object_type>:<40_character_hex_hash>`
- ✅ **Hash Algorithm**: SHA1 (Git-compatible)
- ✅ **Namespace**: Always "swh"
- ✅ **Version**: Always "1"

**Note**: Extended types like `ori` (origin) and `emd` (metadata) are **NOT part of the core specification** and are not included in this reference implementation.

## License

MIT License - see LICENSE file for details. 