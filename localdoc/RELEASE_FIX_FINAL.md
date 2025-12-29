# Final Fix for Release SWHID Computation

## Summary of Changes

1. **Extract all fields from raw Git tag object** (`extract_tag_components`):
   - Extract `object` hash directly from raw object
   - Extract `type` string directly (preserves "tag" vs "release")
   - Extract `tag` name directly
   - Extract `tagger` line directly (exact bytes)
   - Extract `extra_headers` and `message` as before

2. **Store raw tagger line in Release struct**:
   - Added `raw_tagger_line: Option<Bytestring>` field
   - Use raw tagger line directly in manifest instead of reconstructing

3. **Fix type field in manifest**:
   - Changed from `b"release"` to `b"tag"` to match Git object format exactly

## Key Insight

For Git tag objects, the SWHID must match the Git object OID exactly. This requires:
- Using `type tag` (not `type release`) in the manifest
- Using the exact tagger line bytes from the raw object
- Extracting all fields from the raw object, not from git2's parsed values

## Testing

After these changes, the following tests should pass:
- `signed_release_v1` → `swh:1:rel:d6bc712db2ffad219e410155850770f2a6f80566`
- `signed_release_v2` → `swh:1:rel:90b798f42ee8c20dc94b119fc4139b79a03c3b7e`
- `signed_release_v2_1` → `swh:1:rel:dc4a4d4c9110311ff03e0a6f218ecfcb3247ac0b`

