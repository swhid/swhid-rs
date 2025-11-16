# Signed Objects SWHID Discrepancy Analysis

## Summary

The Rust implementation (`swhid-rs`) produces different SWHIDs for signed Git objects (revisions and releases) compared to other implementations (git, git-cmd, pygit2). All other implementations agree on the expected values.

## Discrepancies Found

### 1. Signed Releases

#### Release v1.0.0
- **Expected SWHID**: `swh:1:rel:d6bc712db2ffad219e410155850770f2a6f80566`
- **Git tag object hash**: `d6bc712db2ffad219e410155850770f2a6f80566` ✓
- **Rust computed**: `swh:1:rel:e93aa4bf691e9ab23f24c075699fa344b2e8c7ec` ✗
- **Other implementations**: git, git-cmd, pygit2 all compute the expected value ✓

#### Release v2.0.0
- **Expected SWHID**: `swh:1:rel:90b798f42ee8c20dc94b119fc4139b79a03c3b7e`
- **Git tag object hash**: `90b798f42ee8c20dc94b119fc4139b79a03c3b7e` ✓
- **Rust computed**: `swh:1:rel:81a52ac4f8360f8563729dfacba066bb1887ee71` ✗
- **Other implementations**: git, git-cmd, pygit2 all compute the expected value ✓

#### Release v2.1.0
- **Expected SWHID**: `swh:1:rel:dc4a4d4c9110311ff03e0a6f218ecfcb3247ac0b`
- **Git tag object hash**: `dc4a4d4c9110311ff03e0a6f218ecfcb3247ac0b` ✓
- **Rust computed**: `swh:1:rel:637f6d6d1483dea2397b0869ab23dc0ead45bf7d` ✗
- **Other implementations**: git, git-cmd, pygit2 all compute the expected value ✓

### 2. Signed Revisions

#### Revision (main branch)
- **Expected SWHID**: `swh:1:rev:8a1241cc9d81178d7c1c29201354b2cb309601fe`
- **Git commit hash**: `8a1241cc9d81178d7c1c29201354b2cb309601fe` ✓
- **Rust computed**: `swh:1:rev:e9d5358af9321e508c4fbc02cec6152dc94b1cd6` ✗
- **Other implementations**: git, git-cmd, pygit2 all compute the expected value ✓

#### Revision (signed-feature branch)
- **Expected SWHID**: `swh:1:rev:8a1241cc9d81178d7c1c29201354b2cb309601fe`
- **Git commit hash**: `8a1241cc9d81178d7c1c29201354b2cb309601fe` ✓
- **Rust computed**: `swh:1:rev:e9d5358af9321e508c4fbc02cec6152dc94b1cd6` ✗
- **Other implementations**: git, git-cmd, pygit2 all compute the expected value ✓

## Analysis

### Expected Behavior

According to the SWHID specification:
- **For releases**: The SWHID should be computed from the **tag object hash**, which includes the GPG signature.
- **For revisions**: The SWHID should be computed from the **commit object hash**, which includes the GPG signature.

The Git object hashes (`git rev-parse <tag>` and `git rev-parse <commit>`) include the GPG signatures, and this is what all other implementations use.

### Rust Implementation Issue

The Rust implementation is computing different hashes that don't match any Git object:
- The computed hashes (`e93aa4bf...`, `81a52ac4...`, `637f6d6d...`, `e9d5358a...`) are not valid Git object hashes.
- This suggests the Rust implementation may be:
  1. Stripping GPG signatures before hashing (incorrect)
  2. Using a different hashing method
  3. Not properly handling signed Git objects

### Verification

**Tag object hash (includes signature):**
```bash
$ git rev-parse v1.0.0
d6bc712db2ffad219e410155850770f2a6f80566
```

**Tag object without signature (hypothetical):**
```bash
$ git cat-file -p v1.0.0 | grep -v "^-----" | grep -v "^iQIz" | grep -v "^=" | git hash-object -t tag --stdin
7afa2363b72fbb8fc159d34366a2b7d4558ddb08
```

The Rust computed hash (`e93aa4bf...`) doesn't match either of these, indicating a different computation method.

**Commit object hash (includes signature):**
```bash
$ git rev-parse main
8a1241cc9d81178d7c1c29201354b2cb309601fe
```

**Commit object without signature (hypothetical):**
```bash
$ git cat-file -p 8a1241cc... | grep -v "^gpgsig" | git hash-object -t commit --stdin
b515ccb5be1aa9b24c1fd390967b372daf5e773c
```

The Rust computed hash (`e9d5358a...`) doesn't match either of these.

## Conclusion

The Rust implementation (`swhid-rs`) has a bug in how it computes SWHIDs for signed Git objects. It should compute the SWHID from the full Git object hash (including GPG signatures), as specified in the SWHID specification and as implemented by all other implementations.

## Test Repositories

- **Signed releases**: `payloads/git-repository/signed_releases.tar.gz`
- **Signed revisions**: `payloads/git-repository/signed_revisions.tar.gz`

## Next Steps

1. Investigate the Rust implementation's handling of signed Git objects
2. Verify that it uses the full Git object hash (including GPG signatures)
3. Compare with the SWHID specification requirements for signed objects
4. Fix the Rust implementation to match the expected behavior

