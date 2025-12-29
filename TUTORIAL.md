# SWHID Tutorial

## Introduction

This tutorial provides a practical guide to using `swhid-rs` for computing, parsing, and verifying Software Heritage Identifiers (SWHIDs). Whether you're a developer, DevOps engineer, or researcher, this tutorial will help you get started with SWHIDs.

### What is SWHID?

SWHID (Software Heritage Identifier) is a persistent identifier format defined by ISO/IEC 18670:2025. It provides a standardized way to uniquely identify software artifacts:

- **Content**: Individual files (e.g., source code, binaries)
- **Directories**: Directory structures
- **Revisions**: VCS commits/changesets
- **Releases**: VCS tags/releases
- **Snapshots**: Repository state snapshots

SWHIDs are deterministic: the same content always produces the same identifier, making them ideal for:
- Content integrity verification
- Software artifact tracking
- Cross-tool compatibility
- Reproducible builds

### Use Cases

- **Content Integrity**: Verify files haven't been modified
- **Artifact Tracking**: Track software artifacts across systems
- **Build Reproducibility**: Ensure consistent builds
- **Cross-Implementation Validation**: Compare results across tools

### Installation

**From Source**:
```bash
git clone https://example.org/swhid-rs
cd swhid-rs
cargo build --release
```

**Using Cargo**:
```bash
cargo install --path .
```

The `swhid` binary will be available in your PATH.

## Quick Start

### Your First SWHID

Let's compute a SWHID for a simple string:

```bash
echo "Hello, World!" | swhid content
```

**Output**:
```
swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684
```

This is a **v1 content SWHID**:
- `swh:1`: Version 1
- `cnt`: Content object type
- `b45ef6fec89518d314f546fd6c3025367b721684`: 40-character hex digest (SHA1)

### From a File

Compute a SWHID for a file:

```bash
swhid content README.md
```

**Output**:
```
swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391
```

### Parsing a SWHID

Parse and display a SWHID:

```bash
swhid parse 'swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684'
```

**Output**:
```
swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684
```

The parse command validates the format and pretty-prints the SWHID.

## Content SWHIDs

### Basic Usage

**From stdin**:
```bash
echo "test content" | swhid content
```

When no file argument is provided, the command automatically reads from stdin. You do not need to use `-` as a filename argument.

**From file**:
```bash
swhid content path/to/file.txt
```

**Binary files**:
```bash
swhid content image.png
```

SWHIDs work with any file type - text, binary, images, etc.

### Version 1 (Default)

Version 1 uses SHA1 + hex encoding (40 characters):

```bash
echo "Hello" | swhid content
# Output: swh:1:cnt:8b1a9953c4611296a827abf8c47804d7
```

This is the default and maintains compatibility with existing SWHID implementations.

### Version 2 with Different Serialization Formats

Version 2 uses SHA256 (32 bytes) with configurable serialization. The same content produces the same digest bytes, but different string representations.

**Important**: Hex is the canonical encoding format for SWHID identifiers. The `Display` trait always uses hex. Alternative formats (base64, base32, z85) are available via `to_string_with()` for presentation, but hex remains the standard.

#### Hex (Canonical Format for v2)

```bash
echo "Hello" | swhid --version 2 --hash sha256 --serialization hex content
# Output: swh:2:cnt:185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969
```

**Characteristics**:
- 64 characters for SHA256
- Git-compatible
- Human-readable
- **Canonical format**: This is the standard encoding used by `Display` and `FromStr`

#### Base64

```bash
echo "Hello" | swhid --version 2 --hash sha256 --serialization base64 content
# Output: swh:2:cnt:GF+NcyJx/iX1Ya/JOL4mQ2MOwwTtpRgAfRdkgmOBaQ==
```

**Characteristics**:
- 44 characters for SHA256 (more compact than hex)
- Standard Base64 encoding
- Includes padding (`=`)

#### Base64URL

```bash
echo "Hello" | swhid --version 2 --hash sha256 --serialization base64url content
# Output: swh:2:cnt:GF-NcyJx_iX1Ya_JOL4mQ2MOwwTtpRgAfRdkgmOBaQ
```

**Characteristics**:
- 43 characters for SHA256
- URL-safe (no special characters)
- No padding

#### Base32

```bash
echo "Hello" | swhid --version 2 --hash sha256 --serialization base32 content
# Output: swh:2:cnt:GF2NQYJX7IX1YA7JOL4MQ2MOQWQTT5RQGAFRDKGMOBAQ====
```

**Characteristics**:
- 52 characters for SHA256
- RFC 4648 standard
- Case-insensitive

#### Base32hex

```bash
echo "Hello" | swhid --version 2 --hash sha256 --serialization base32hex content
# Output: swh:2:cnt:GF2NQ0JX7IX1YA7JOL4MQ2MOQWQTT5RQGAFRDKGMOBAQ====
```

**Characteristics**:
- 52 characters for SHA256
- Base32hex variant (uses 0-9, A-V)
- Case-insensitive

#### Z85 (Most Compact)

```bash
echo "Hello" | swhid --version 2 --hash sha256 --serialization z85 content
# Output: swh:2:cnt:GF+NcyJx/iX1Ya/JOL4mQ2MOwwTtpRgAfRdkgmOBaQ
```

**Characteristics**:
- 40 characters for SHA256 (most compact)
- ZeroMQ Base85 encoding
- URL-safe character set

### Serialization Format Comparison

For a SHA256 digest (32 bytes), here's the character count comparison:

| Format     | Length | Compactness | Use Case                    |
|------------|--------|-------------|-----------------------------|
| **z85**    | 40     | Most        | Maximum compactness         |
| **base64url** | 43  | High        | URLs, APIs                  |
| **base64** | 44     | High        | General purpose             |
| **base32** | 52     | Medium      | Case-insensitive systems    |
| **base32hex** | 52  | Medium      | Case-insensitive, hex-like  |
| **hex**    | 64     | Least       | Git compatibility, readable |

**Recommendation**:
- **Git repositories**: Use `hex` (matches Git OID format)
- **APIs/URLs**: Use `base64url` (URL-safe, compact)
- **Maximum compactness**: Use `z85` (shortest representation)
- **General purpose**: Use `base64` (good balance)

### Verifying Content Integrity

Use the `verify` command to check if a file matches an expected SWHID:

```bash
# Compute and store SWHID
swhid content important.txt > expected.swhid

# Later, verify the file hasn't changed
swhid verify --file important.txt --expected "$(cat expected.swhid)"
```

**Output on success**:
```
✓ Verification successful: important.txt matches swh:1:cnt:...
```

**Output on failure**:
```
✗ Verification failed: important.txt does not match swh:1:cnt:...
  Expected: swh:1:cnt:abc123...
  Actual:   swh:1:cnt:def456...
```

## Directory SWHIDs

### Basic Usage

Compute a SWHID for an entire directory:

```bash
swhid dir /path/to/project
```

This recursively processes all files in the directory and computes a single SWHID representing the entire directory structure.

### Excluding Files

Exclude files by suffix (e.g., build artifacts, temporary files):

```bash
swhid dir /path/to/project \
  --exclude-suffix .tmp \
  --exclude-suffix .log \
  --exclude-suffix .o
```

**Common exclusions**:
- Build artifacts: `.o`, `.obj`, `.exe`, `.so`, `.dylib`
- Temporary files: `.tmp`, `.bak`, `.swp`
- IDE files: `.idea/`, `.vscode/`
- OS files: `.DS_Store`, `Thumbs.db`

### Following Symlinks

**Warning**: Following symlinks can lead to unexpected results (infinite loops, external files).

```bash
swhid dir /path/to/project --follow-symlinks
```

**Recommendation**: Only use `--follow-symlinks` if you understand the directory structure.

### Version 2

Use v2 with different serialization formats:

```bash
# Most compact
swhid --version 2 --hash sha256 --serialization z85 dir /path/to/project

# Git-compatible
swhid --version 2 --hash sha256 --serialization hex dir /path/to/project
```

### Real-World Example

Compute SWHID for a Rust project, excluding build artifacts:

```bash
swhid dir /path/to/rust-project \
  --exclude-suffix .rs.bk \
  --exclude-suffix .pdb \
  --exclude-suffix .dSYM
```

This gives you a fingerprint of your source code that excludes compiler-generated files.

## Git Repository SWHIDs

### Prerequisites

Git integration requires building with the `git` feature:

```bash
cargo build --release --features git
```

### Revision SWHIDs (Commits)

Compute SWHID for a commit:

```bash
# HEAD commit
swhid git revision --repo /path/to/git/repo

# Specific commit
swhid git revision --repo /path/to/git/repo --commit abc123def456
```

**Automatic Detection**: The tool automatically detects whether the repository uses SHA1 or SHA256 object format and uses the appropriate hash function.

### Release SWHIDs (Tags)

List all tags:
```bash
swhid git tags --repo /path/to/git/repo
```

Compute SWHID for a specific tag:
```bash
swhid git release --repo /path/to/git/repo --tag v1.0.0
```

### Snapshot SWHIDs

Compute SWHID for the entire repository state:

```bash
swhid git snapshot --repo /path/to/git/repo
```

A snapshot SWHID represents the state of all branches and tags in the repository at a given time.

### SHA256 Git Repositories

Git repositories using SHA256 object format are automatically detected:

```bash
# In a SHA256 Git repo, this automatically uses SHA256
swhid git revision --repo /path/to/sha256-repo
```

The tool detects the repository's hash algorithm and uses the appropriate configuration.

## Parsing and Validation

### Parsing Core SWHIDs

Parse a simple SWHID:

```bash
swhid parse 'swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684'
```

### Parsing Qualified SWHIDs

Qualified SWHIDs include additional context (origin, path, lines, bytes):

```bash
swhid parse 'swh:1:cnt:abc123...;origin=https://github.com/user/repo;path=/src/main.rs;lines=10-20'
```

### Error Handling

Invalid SWHIDs produce clear error messages:

```bash
swhid parse 'invalid'
# Error: Invalid SWHID format
```

## Advanced Topics

### Serialization Format Selection Guide

**Choose based on your use case**:

1. **Canonical Format**: Use `hex` (standard, used by Display/FromStr)
2. **Git Integration**: Use `hex` (matches Git OID format)
3. **REST APIs**: Use `base64url` (URL-safe, compact) via `to_string_with()`
4. **Database Storage**: Use `z85` (most compact) via `to_string_with()`
5. **Human Readability**: Use `hex` (familiar format, canonical)
6. **Case-Insensitive Systems**: Use `base32` or `base32hex` via `to_string_with()`

**Note**: While alternative formats are available, hex is the canonical format. Use `to_string_with()` to format with alternative serializations, and `parse_with()` to parse them back.

### Version Selection

**When to use v1**:
- Compatibility with existing SWHID implementations
- Git repositories using SHA1
- Maximum compatibility

**When to use v2**:
- Enhanced security (SHA256)
- Need for compact serialization
- Git repositories using SHA256
- Future-proofing

### Qualified SWHIDs

Qualified SWHIDs add context to core SWHIDs:

```
swh:1:cnt:abc123...;origin=https://github.com/user/repo;path=/src/main.rs;lines=10-20;bytes=100-200
```

**Qualifiers**:
- `origin`: Source repository URL
- `path`: File path within repository
- `lines`: Line range (e.g., `10-20`)
- `bytes`: Byte range (e.g., `100-200`)

**Use cases**:
- Referencing specific code sections
- Tracking file locations
- Cross-repository references

## Real-World Examples

### Content Integrity Verification

**Scenario**: Verify a downloaded file matches the expected SWHID.

```bash
# Download file
wget https://example.com/file.tar.gz

# Verify against expected SWHID
swhid verify --file file.tar.gz --expected 'swh:1:cnt:expected-digest-here'
```

### Directory Fingerprinting

**Scenario**: Create a fingerprint of your project source code.

```bash
# Create fingerprint (excluding build artifacts)
swhid dir /path/to/project \
  --exclude-suffix .o \
  --exclude-suffix .so \
  --exclude-suffix .exe \
  > project-fingerprint.swhid

# Store fingerprint for later verification
cat project-fingerprint.swhid
```

### CI/CD Integration

**Scenario**: Verify source code integrity in CI pipeline.

```bash
#!/bin/bash
# ci-verify.sh

EXPECTED_SWHID="swh:1:dir:expected-digest-here"
ACTUAL_SWHID=$(swhid dir . --exclude-suffix .tmp)

if [ "$ACTUAL_SWHID" = "$EXPECTED_SWHID" ]; then
    echo "✓ Source code verified"
    exit 0
else
    echo "✗ Source code verification failed"
    exit 1
fi
```

### Cross-Implementation Validation

**Scenario**: Compare SWHIDs computed by different tools.

```bash
# Compute with swhid-rs
swhid content test.txt > swhid-rs-result.txt

# Compute with Python implementation
python -c "from swh.model import *; print(Content.from_bytes(open('test.txt', 'rb').read()).swhid())" > python-result.txt

# Compare (should be identical for v1)
diff swhid-rs-result.txt python-result.txt
```

## Troubleshooting

### Common Errors

**"Invalid SWHID format"**:
- Check SWHID string format: `swh:<version>:<type>:<digest>`
- Ensure digest length matches version (40 chars for v1 hex, 64 for v2 hex)

**"File not found"**:
- Verify file path is correct
- Check file permissions

**"Git repository errors"**:
- Ensure repository path is correct
- Verify `git` feature is enabled: `cargo build --features git`
- Check Git repository is not corrupted

**"Feature not available"**:
- Git commands require `--features git` build flag
- Rebuild with: `cargo build --release --features git`

### Performance Tips

**Large Files**:
- SWHID computation is efficient even for large files
- Memory usage is minimal (streaming hash computation)

**Large Directories**:
- Directory SWHID computation processes files sequentially
- Exclude unnecessary files to speed up computation
- Consider using v2 with compact serialization to reduce storage

**Memory Considerations**:
- SWHID computation uses minimal memory
- Large files are hashed in chunks (not loaded entirely)

### Getting Help

- Check this tutorial for examples
- Review README.md for quick reference
- See DEVELOPER_GUIDE.md for implementation details
- Check REFERENCE.md for architectural information

## CLI Reference Quick Guide

### Global Options

- `--version <VERSION>`: SWHID version (1 or 2, default: 1)
- `--hash <HASH>`: Hash function (sha1 or sha256, default: sha1)
- `--serialization <FORMAT>`: Serialization format (hex, base64, base64url, base32, base32hex, z85, default: hex)

### Commands

**Content**:
```bash
swhid content [<FILE>]
```

**Directory**:
```bash
swhid dir <PATH> [--follow-symlinks] [--exclude-suffix <SUFFIX>...]
```

**Parse**:
```bash
swhid parse <SWHID>
```

**Verify**:
```bash
swhid verify --file <PATH> --expected <SWHID> [--follow-symlinks] [--exclude-suffix <SUFFIX>...]
```

**Git** (requires `--features git`):
```bash
swhid git revision --repo <PATH> [--commit <HASH>]
swhid git release --repo <PATH> --tag <TAG>
swhid git snapshot --repo <PATH>
swhid git tags --repo <PATH>
```

### Valid Option Combinations

- **v1**: `sha1` + `hex` (only)
- **v2**: `sha256` + `hex`/`base64`/`base64url`/`base32`/`base32hex`/`z85`

### Examples Summary

```bash
# V1 content (default)
echo "test" | swhid content

# V2 content with z85 (most compact)
echo "test" | swhid --version 2 --hash sha256 --serialization z85 content

# Directory with exclusions
swhid dir . --exclude-suffix .tmp --exclude-suffix .log

# Git revision
swhid git revision --repo /path/to/repo

# Verify file
swhid verify --file test.txt --expected 'swh:1:cnt:...'
```

---

**Next Steps**:
- Try the examples in this tutorial
- Explore different serialization formats
- Integrate SWHIDs into your workflow
- Check DEVELOPER_GUIDE.md if you want to contribute

