# SWHID Specification: "tag" vs "release" in Manifests

## Summary

There are **two different contexts** where "tag" vs "release" matters:

1. **SWHID Identifier Format**: Uses `rel` (release) - `swh:1:rel:<digest>`
2. **Git Object Manifest Format**: Uses `tag` - when reconstructing the manifest for hashing

## SWHID Identifier Format

According to the SWHID specification:
- The identifier type is **`rel`** (release), not `tag`
- Format: `swh:1:rel:<digest>`
- This represents a release object, which corresponds to Git annotated tags

**Reference**: The SWHID specification uses "release" (`swh:1:rel:`) to represent versioned snapshots, not "tag".

## Git Object Manifest Format

When computing SWHIDs for Git tag objects, the **manifest** (the content that gets hashed) must match the Git object format exactly.

### Git Tag Object Format

Git stores tag objects with this structure:
```
object <hash>
type tag          ← Git uses "tag", not "release"
tag <name>
tagger <info>
<extra headers>
<message>
```

### Why "tag" in the Manifest?

For Git objects, the SWHID specification requires that:
- **The SWHID must match the Git object OID**
- The Git object OID is computed as: `SHA1("tag " + len + "\0" + content)`
- Therefore, the manifest must use `type tag` to match Git's format exactly

If we used `type release` in the manifest, the hash would be different, and the SWHID would not match the Git object OID.

## Specification References

- **SWHID v1.2 Specification Section 5.5**: Release objects
- The specification states that for Git objects, the SWHID should match the Git object OID
- This requires using Git's exact format in the manifest

## Implementation Decision

In `swhid-rs`, we use:
- **`rel`** in the SWHID identifier: `swh:1:rel:<digest>` ✓
- **`tag`** in the manifest: `type tag` (to match Git format) ✓

This ensures:
1. SWHID identifiers use the correct type (`rel`)
2. The manifest matches Git's format exactly
3. The computed SWHID matches the Git object OID

## Conclusion

The specification distinguishes between:
- **Identifier type**: Always use `rel` (release) in `swh:1:rel:...`
- **Manifest format**: Use `tag` when reconstructing Git tag objects to match Git's format

This is the correct approach and ensures full compliance with both the SWHID specification and Git's object format.

