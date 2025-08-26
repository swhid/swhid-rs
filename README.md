# SWHID - Minimal Reference Implementation

A minimal, clean reference implementation of Software Hash Identifier (SWHID) computation in Rust.

## Overview

This library provides the **SWHID functionality** according to the official SWHID specification v1.2. It serves as a clean reference implementation that can be used as a dependency for full-featured toos. A minimal CLI interface is provided as `swhid-rs`, as well as a test harness to compare with other implementations: these components are for your convenience and are not part of the reference implementation.

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

### Qualified SWHIDs

The library also supports **Qualified SWHIDs** with qualifiers according to the specification:

- **`origin`** - Software origin URI where the object was found
- **`visit`** - Snapshot SWHID corresponding to a specific repository visit
- **`anchor`** - Reference node (dir, rev, rel, or snp) for path resolution
- **`path`** - Absolute file path relative to the anchor
- **`lines`** - Line range (start-end or single line) within content
- **`bytes`** - Byte range (start-end or single byte) within content

**Format**: `swh:1:<object_type>:<hash>[;qualifier=value]*`

**Example**: `swh:1:cnt:abc123...;origin=https://github.com/user/repo;path=/src/main.rs;lines=10-20;bytes=5-10`

## Features

- **Complete Core SWHID Support**: All 5 core object types from the specification
- **Qualified SWHID Support**: Full qualifier support (origin, visit, anchor, path, lines, bytes)
- **Minimal Dependencies**: Only essential crates (sha1-checked, hex)
- **Reference Implementation**: Clean, readable code for SWHID specification
- **Specification Compliant**: Follows SWHID v1.2 specification exactly

## What's Included

In the core library
- **Core SWHID computation** for all 5 object types
- **Qualified SWHID support** with all 6 qualifiers

In the CLI as a default
- **File and directory processing** with file attributes and symlink support (specification-compliant)
- **Exclude patterns** for directory traversal
- **SWHID verification** functionality
- **Stdin support** for content processing

In the cli via conditional compilation (CLI features)
- **Git support** to compute SWHID on git repositories (snapshot, revision, release computation)

## What's NOT Included

The CLI is provided as a minimal interface to the library, and it does not feature functionality that you may find in other tools, like:

- Archive processing (tar, zip, etc.)
- Recursive directory printing
- Performance optimizations (caching, statistics)

## Installation

### From Source

```bash
git clone <repository-url>
cd swhid-rs
git checkout minimal-reference-impl

# Build minimal version
cargo build

# Build with Git support
cargo build --features git
```

### Using Cargo

Add to your `Cargo.toml`:
```toml
[dependencies]
swhid-lib = "0.1.0"

# For Git support in CLI
swhid-lib = { version = "0.1.0", features = ["git"] }
```

## Features

Rust features allow conditional compilation of additional functionality:

- **Default**: Minimal SWHID functionality
- **`git`**: Enable Git support in CLI (revision, release, snapshot SWHIDs)

### Building with Features

```bash
# Minimal build (default)
cargo build

# With Git support
cargo build --features git

# CLI with Git support
cargo build --bin swhid-cli --features git
```

## Usage

### Command-Line Interface

The library includes a CLI tool for easy SWHID computation:

```bash
# Build the CLI (minimal version)
cargo build --bin swhid-cli

# Build the CLI with Git support
cargo build --bin swhid-cli --features git
```

#### Basic Usage

```bash
# Compute SWHID for a file
./target/debug/swhid-cli file.txt

# Compute SWHID for a directory
./target/debug/swhid-cli directory/

# Compute SWHID from stdin
echo "Hello, World!" | ./target/debug/swhid-cli -

# Verify a SWHID
./target/debug/swhid-cli -v "swh:1:cnt:abc123..." file.txt

# Exclude certain files from directory processing
./target/debug/swhid-cli -e "*.tmp" -e "*.log" directory/

# Get help
./target/debug/swhid-cli --help
```

#### Git Support (Feature Flag)

When built with `--features git`, the CLI supports Git-based SWHIDs:

```bash
# Compute revision SWHID for a specific commit
./target/debug/swhid-cli --revision HEAD repository/

# Compute release SWHID for a specific tag
./target/debug/swhid-cli --release v1.0.0 repository/

# Compute snapshot SWHID for entire repository
./target/debug/swhid-cli --snapshot repository/
```

#### CLI Options

**Basic Options:**
- `-o, --obj-type <TYPE>`: Object type (auto, content, directory) [default: auto]
- `--dereference`: If the CLI is called on a symlink, follow it
- `--no-dereference`: If the CLI is called on a symlink, don't follow it
- `--filename`: Show filename in output [default: true]
- `-e, --exclude <PATTERN>`: Exclude directories using glob patterns
- `-v, --verify <SWHID>`: Reference identifier to compare with computed one
- `-h, --help`: Print help information

**Git Options (requires `--features git`):**
- `--revision <REVISION>`: Git revision to compute SWHID for
- `--release <RELEASE>`: Git release/tag to compute SWHID for
- `--snapshot`: Compute Git snapshot SWHID

### Symlink Handling and Specification Compliance

**Important**: The official SWHID specification v1.2 states that symlinks are treated as content objects where the "content" is the symlink target string itself, and this reference implementation strictly obeys that rule.

The CLI has a `--dereference` option, disabled by default, that comes handy if you want to compute the SWHID of the target of the symlink passed on the command line. This option only affects symlinks passed directly as arguments, not symlinks discovered during directory traversal, so the SWHID returned is the correct SWHID **for the symlink target**.



### Library Usage

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

### Qualified SWHID Usage

```rust
use swhid::{Swhid, ObjectType, QualifiedSwhid};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a core SWHID
    let hash = [0u8; 20];
    let core_swhid = Swhid::new(ObjectType::Content, hash);
    
    // Create a qualified SWHID with qualifiers
    let qualified = QualifiedSwhid::new(core_swhid)
        .with_origin("https://github.com/user/repo".to_string())
        .with_path(b"/src/main.rs".to_vec())
        .with_lines(10, Some(20))
        .with_bytes(5, Some(10));
    
    println!("Qualified SWHID: {}", qualified);
    
    // Parse a qualified SWHID from string
    let parsed = QualifiedSwhid::from_string(
        "swh:1:cnt:0000000000000000000000000000000000000000;origin=https://github.com/user/repo;path=/src/main.rs;lines=10-20;bytes=5-10"
    )?;
    
    println!("Origin: {:?}", parsed.origin());
    println!("Path: {:?}", parsed.path().map(|p| String::from_utf8_lossy(p)));
    println!("Lines: {:?}", parsed.lines());
    println!("Bytes: {:?}", parsed.bytes());
    
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
├── swhid.rs        # Core SWHID types, QualifiedSWHID, and formatting
├── hash.rs         # Basic hash computation
├── content.rs      # Content object handling
├── directory.rs    # Directory object handling
├── error.rs        # error types
├── computer.rs     # Minimal SWHIDComputer
└── main.rs         # CLI interface

Binaries:
├── swhid-cli       # Command-line interface for SWHID computation
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
- ✅ **Qualified SWHIDs**: Full qualifier support (origin, visit, anchor, path, lines, bytes)
- ✅ **Format**: `swh:1:<object_type>:<40_character_hex_hash>[;qualifier=value]*`
- ✅ **Hash Algorithm**: SHA1 (Git-compatible) with collision detection
- ✅ **Namespace**: Always "swh"
- ✅ **Version**: Always "1"
- ✅ **Qualifier Validation**: Proper type checking for visit/anchor qualifiers
- ✅ **Fragment Qualifiers**: Both lines and bytes qualifiers supported
- ✅ **Symlink Handling**: Never follows symlinks by default (specification-compliant)

## License

MIT License - see LICENSE file for details. 