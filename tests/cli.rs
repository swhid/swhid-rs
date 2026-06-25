use std::process::Command;

use assert_fs::prelude::*;

use swhid::DiskDirectoryBuilder;

#[test]
fn dir_recursive_prints_local_paths_and_swhids() {
    let tmp = assert_fs::TempDir::new().unwrap();
    tmp.child("a.txt").write_str("A").unwrap();
    tmp.child("subdir").create_dir_all().unwrap();
    tmp.child("subdir/b.txt").write_str("B").unwrap();

    let expected = DiskDirectoryBuilder::new(tmp.path())
        .recursive_swhids()
        .unwrap()
        .into_iter()
        .map(|entry| format!("{}\t{}", entry.swhid, entry.path.display()))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";

    let output = Command::new(env!("CARGO_BIN_EXE_swhid"))
        .args(["dir", "-R"])
        .arg(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
}
