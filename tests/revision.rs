use swhid::revision::*;

fn bs(s: &'static str) -> Box<[u8]> {
    s.as_bytes().into()
}

#[test]
fn simple_rev_hash() {
    let tree_hash = hex::decode("0efb37b28c53c7e4fbd253bb04a4df14008f63fe")
        .unwrap()
        .try_into()
        .unwrap();

    let rev = Revision {
        directory: tree_hash,
        parents: Vec::new(),
        author: bs("Test User <test@example.com>"),
        author_timestamp: 1763027354,
        author_timestamp_offset: bs("+0100"),
        committer: bs("Test User <test@example.com>"),
        committer_timestamp: 1763027354,
        committer_timestamp_offset: bs("+0100"),
        extra_headers: Vec::new(),
        message: Some(bs("Test commit")),
    };

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    // using this script:
    // >>> from swh.model.model import *
    // >>> from swh.model.git_objects import *
    // >>> person = Person.from_fullname(b"Test User <test@example.com>")
    // >>> ts = TimestampWithTimezone(timestamp=Timestamp(seconds=1763027354, microseconds=0), offset_bytes=b"+0100")
    // >>> rev = Revision(directory=bytes.fromhex("0efb37b28c53c7e4fbd253bb04a4df14008f63fe"), message=b"Test commit", author=person, committer=person, date=ts, committer_date=ts, type=RevisionType.GIT, synthetic=False)
    // >>> revision_git_object(rev)
    assert_eq!(
        rev_manifest(&rev),
        b"\
        tree 0efb37b28c53c7e4fbd253bb04a4df14008f63fe\n\
        author Test User <test@example.com> 1763027354 +0100\n\
        committer Test User <test@example.com> 1763027354 +0100\n\
        \n\
        Test commit\
        ",
        "{}",
        String::from_utf8_lossy(&rev_manifest(&rev)),
    );

    // ditto
    assert_eq!(
        rev.swhid().to_string(),
        "swh:1:rev:07cde6575fb633ef9b5ecbe730e6eb97475a2fd9"
    );
}

#[test]
fn negative_timezone() {
    let tree_hash = hex::decode("0efb37b28c53c7e4fbd253bb04a4df14008f63fe")
        .unwrap()
        .try_into()
        .unwrap();

    let rev = Revision {
        directory: tree_hash,
        parents: Vec::new(),
        author: bs("Test User <test@example.com>"),
        author_timestamp: 1763027354,
        author_timestamp_offset: bs("-0700"),
        committer: bs("Test User <test@example.com>"),
        committer_timestamp: 1763027354,
        committer_timestamp_offset: bs("-0700"),
        extra_headers: Vec::new(),
        message: Some(bs("Test commit")),
    };

    assert_eq!(
        rev_manifest(&rev),
        b"\
        tree 0efb37b28c53c7e4fbd253bb04a4df14008f63fe\n\
        author Test User <test@example.com> 1763027354 -0700\n\
        committer Test User <test@example.com> 1763027354 -0700\n\
        \n\
        Test commit\
        ",
        "{}",
        String::from_utf8_lossy(&rev_manifest(&rev)),
    );

    assert_eq!(
        rev.swhid().to_string(),
        "swh:1:rev:927b3b4a8291765c874d26560da492de7b3d9091"
    );
}
