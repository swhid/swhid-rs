use swhid::digest::Digest;
use swhid::release::*;

fn bs(s: &'static str) -> Box<[u8]> {
    s.as_bytes().into()
}

#[test]
fn simple_rel_hash() {
    let tree_hash: [u8; 20] = hex::decode("0efb37b28c53c7e4fbd253bb04a4df14008f63fe")
        .unwrap()
        .try_into()
        .unwrap();

    let rel = Release {
        object: Digest::from(tree_hash),
        object_type: ReleaseTargetType::Directory,
        name: bs("v1.0"),
        author: Some(bs("Test User <test@example.com>")),
        author_timestamp: Some(1763027354),
        author_timestamp_offset: Some(bs("+0100")),
        extra_headers: Vec::new(),
        message: Some(bs("Test tag")),
    };

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    // using this script:
    // >>> from swh.model.model import *
    // >>> from swh.model.git_objects import *
    // >>> person = Person.from_fullname(b"Test User <test@example.com>")
    // >>> ts = TimestampWithTimezone(timestamp=Timestamp(seconds=1763027354, microseconds=0), offset_bytes=b"+0100")
    // >>> rel = Release(name=b"v1.0", target_type=ObjectType.DIRECTORY, target=bytes.fromhex("0efb37b28c53c7e4fbd253bb04a4df14008f63fe"), message=b"Test tag", author=person, date=ts, synthetic=False)
    // >>> release_git_object(rel)
    assert_eq!(
        rel_manifest(&rel),
        b"\
        object 0efb37b28c53c7e4fbd253bb04a4df14008f63fe\n\
        type tree\n\
        tag v1.0\n\
        tagger Test User <test@example.com> 1763027354 +0100\n\
        \n\
        Test tag\
        "
    );

    // ditto
    assert_eq!(
        rel.swhid().to_string(),
        "swh:1:rel:46d326edb8bfc49b757ccd09930365595806bfc0",
    );
}
