# Detailed Analysis: Release SWHID Computation Issue

## Problem

The Rust implementation computes incorrect SWHIDs for Git tag objects (releases):
- `signed_release_v1`: Computes `e93aa4bf691e9ab23f24c075699fa344b2e8c7ec`, expected `d6bc712db2ffad219e410155850770f2a6f80566`
- `signed_release_v2`: Computes `81a52ac4f8360f8563729dfacba066bb1887ee71`, expected `90b798f42ee8c20dc94b119fc4139b79a03c3b7e`
- `signed_release_v2_1`: Computes `637f6d6d1483dea2397b0869ab23dc0ead45bf7d`, expected `dc4a4d4c9110311ff03e0a6f218ecfcb3247ac0b`

The expected values are the actual Git tag object OIDs, which means the SWHID for Git tag objects should match the Git object hash.

## Root Cause Analysis

For Git objects, the SWHID specification requires that the SWHID matches the Git object OID. The Git object OID is computed as:
```
SHA1("tag " + len(content) + "\0" + content)
```

The current implementation:
1. Extracts components from the raw Git tag object ✓
2. Reconstructs a manifest using `HeaderWriter` ✗
3. Hashes the manifest with `hash_swhid_object("tag", manifest)` ✗

The problem is that the reconstructed manifest must match the original Git tag object content **exactly**, byte-for-byte. Any difference will result in a different hash.

## Key Issues to Fix

### Issue 1: Message Extraction
The message extraction code splits by `\n` and rejoins, which should preserve the format. However, we need to verify that:
- Trailing newlines are preserved correctly
- The exact byte sequence matches the original

### Issue 2: HeaderWriter Format
The `HeaderWriter` adds newlines after each header and handles multi-line values. We need to ensure:
- Headers are formatted exactly as Git stores them
- The empty line separator is correct
- Message is appended correctly

### Issue 3: Comparison with Git Object Format
The reconstructed manifest must match the Git tag object content exactly. The format is:
```
object <hash>\n
type <type>\n
tag <name>\n
tagger <name> <email> <timestamp> <offset>\n
<extra headers>\n (if any)
\n
<message>
```

## Solution

The fix requires ensuring that:
1. Message extraction preserves exact bytes (including trailing newline if present)
2. HeaderWriter produces the exact same format as Git
3. The manifest matches the original Git tag object content byte-for-byte

## Testing Strategy

1. Extract a tag object from Git
2. Parse it and reconstruct the manifest
3. Compare the reconstructed manifest with the original content byte-for-byte
4. Verify that hashing the manifest produces the correct Git OID
5. Test with all three failing tags

