use swhid::digest::Digest;
use swhid::snapshot::*;

fn name(s: &'static str) -> Box<[u8]> {
    s.as_bytes().into()
}

#[test]
fn simple_snp_hash() {
    let snp = Snapshot::new(vec![
        Branch::new(
            name("refs/heads/develop"),
            BranchTarget::Revision(Some(Digest::from([2; 20]))),
        ),
        Branch::new(
            name("refs/heads/main"),
            BranchTarget::Revision(Some(Digest::from([1; 20]))),
        ),
    ])
    .unwrap();

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    assert_eq!(
        snp_manifest(snp.branches().into()).unwrap(),
        b"\
        revision refs/heads/develop\020:\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\
        revision refs/heads/main\020:\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\
        "
    );

    // ditto
    assert_eq!(
        snp.swhid().to_string(),
        "swh:1:snp:870148a17e00ea8bd84b727cd26104b8c6ac6a72"
    );
}

#[test]
fn snp_order() {
    let snp = Snapshot::new(vec![
        Branch::new(
            name("refs/heads/main"),
            BranchTarget::Revision(Some(Digest::from([1; 20]))),
        ),
        Branch::new(
            name("refs/heads/develop"),
            BranchTarget::Revision(Some(Digest::from([2; 20]))),
        ),
    ])
    .unwrap();

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    assert_eq!(
        snp_manifest(snp.branches().into()).unwrap(),
        b"\
        revision refs/heads/develop\020:\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\
        revision refs/heads/main\020:\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\
        "
    );

    // ditto
    assert_eq!(
        snp.swhid().to_string(),
        "swh:1:snp:870148a17e00ea8bd84b727cd26104b8c6ac6a72"
    );
}

#[test]
fn empty_snp_hash() {
    let snp = Snapshot::new(vec![]).unwrap();

    assert_eq!(snp_manifest(snp.branches().into()).unwrap(), b"");

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    assert_eq!(
        snp.swhid().to_string(),
        "swh:1:snp:1a8893e6a86f444e8be8e7bda6cb34fb1735a00e"
    );
}

#[test]
fn snp_with_alias() {
    let snp = Snapshot::new(vec![
        Branch::new(
            name("refs/heads/main"),
            BranchTarget::Revision(Some(Digest::from([1; 20]))),
        ),
        Branch::new(
            name("refs/heads/develop"),
            BranchTarget::Revision(Some(Digest::from([2; 20]))),
        ),
        Branch::new(
            name("HEAD"),
            BranchTarget::Alias(Some(name("refs/heads/main"))),
        ),
    ])
    .unwrap();

    assert_eq!(
        snp_manifest(snp.branches().into()).unwrap(),
        b"\
        alias HEAD\x0015:refs/heads/main\
        revision refs/heads/develop\x0020:\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\x02\
        revision refs/heads/main\x0020:\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\
        "
    );

    // Checked against the implementation in https://archive.softwareheritage.org/swh:1:dir:60e683f48069373ee85227f2d7ab2eb1a8873ddb;origin=https://gitlab.softwareheritage.org/swh/devel/swh-model.git;visit=swh:1:snp:291aefbdccd43abac57629431201c2fd55284df7;anchor=swh:1:rev:9e54500902fc00ab1e6400431e2803b9bb41cc0a
    assert_eq!(
        snp.swhid().to_string(),
        "swh:1:snp:9ecd7950d10ed3d02bfcf9c4a534f173697ab9f3"
    );
}
