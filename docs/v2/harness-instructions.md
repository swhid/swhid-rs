# Instructions for swhid-rs-tools Test Harness Evolution

## Overview

The swhid-rs implementation is adding support for SWHID v2 with SHA256 hash functions and alternative serialization formats. The test harness needs to be extended to support testing these new features while maintaining full backward compatibility with v1.

## Objective

Create an `experimental` branch in the swhid-rs-tools repository that:
1. Contains expected SWHID results for each test payload as computed by Git using SHA256 object format
2. Extends the test harness to support running tests with SHA256 configuration
3. Maintains full backward compatibility with existing v1 tests

## Scope

**Included**: All test payloads that can be computed by Git:
- Content objects (blobs)
- Directory objects (trees)
- Revision objects (commits)
- Release objects (tags)

**Excluded**: 
- Snapshot objects (not supported by Git's object format)

## Tasks

### 1. Create Experimental Branch

```bash
cd /home/dicosmo/code/swhid-rs-tools
git checkout -b experimental
```

### 2. Generate Expected SHA256 Results

For each test payload in `config.yaml` (excluding snapshots), compute the expected SWHID using Git with SHA256 object format:

**For Content Objects**:
```bash
# Create a Git repo with SHA256 object format
git init --object-format=sha256
# Add the file
git add <payload_file>
# Get the blob hash (this is the SWHID digest)
git ls-files --stage <payload_file>
# Format as: swh:2:cnt:<64-char-hex-digest>
```

**For Directory Objects**:
```bash
# Create a Git repo with SHA256 object format
git init --object-format=sha256
# Add directory contents
git add <payload_directory>
# Get the tree hash
git write-tree
# Format as: swh:2:dir:<64-char-hex-digest>
```

**For Revision Objects**:
```bash
# In a SHA256 Git repo, get commit hash
git rev-parse <commit>
# Format as: swh:2:rev:<64-char-hex-digest>
```

**For Release Objects**:
```bash
# In a SHA256 Git repo, get tag object hash
git rev-parse <tag>^{}
# Format as: swh:2:rel:<64-char-hex-digest>
```

**Output Format**: Add expected SHA256 results to `config.yaml`:
```yaml
content:
  - name: hello_world
    path: payloads/content/hello.txt
    expected_swhid: swh:1:cnt:...  # existing v1
    expected_swhid_sha256: swh:2:cnt:...  # new v2 SHA256
```

### 3. Extend Rust Implementation Plugin

Update `implementations/rust/implementation.py` to:
- Accept `--version 2` and `--hash sha256` flags when calling swhid binary
- Support configuration via harness config or command-line options
- Pass these flags to the swhid binary: `swhid --version 2 --hash sha256 <command> <args>`

**Configuration Example**:
```yaml
# In harness config or test payload
rust_config:
  version: 2
  hash: sha256
  serialization: hex  # default for now
```

### 4. Update Test Harness Logic

Modify harness to:
- Support running tests with SHA256 configuration
- Compare results against `expected_swhid_sha256` when using SHA256 config
- Maintain existing v1 behavior (compare against `expected_swhid`)
- Support both v1 and v2 in the same test run

### 5. Test Execution

**Running v1 tests** (backward compatibility):
```bash
swhid-harness --impl rust --dashboard-output results-v1.json
```

**Running v2 SHA256 tests**:
```bash
swhid-harness --impl rust --config sha256-config.yaml --dashboard-output results-v2-sha256.json
```

## Requirements

1. **Backward Compatibility**: All existing v1 tests must continue to work unchanged
2. **Isolation**: Changes in `experimental` branch must not affect main branch
3. **Completeness**: Expected SHA256 results for all supported payload types
4. **Documentation**: Update DEVELOPER_GUIDE.md with v2 testing instructions

## Validation

- Verify all v1 tests still pass
- Verify SHA256 tests produce expected results matching Git's SHA256 object hashes
- Ensure harness can run both v1 and v2 tests independently

## Dependencies

- swhid-rs implementation must support `--version 2 --hash sha256` CLI flags
- Git 2.42+ with SHA256 repository support
- Test payloads must be accessible

## Notes

- Snapshot objects are excluded as Git doesn't support snapshot object format
- Focus on SHA256 + hex serialization initially (base64 can be added later)
- Expected results should match Git's actual SHA256 object hashes exactly

