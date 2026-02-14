use crate::config::HashConfig;
use crate::core::{ObjectType, Swhid};
use crate::digest::Digest;
use crate::error::SnapshotError;
use crate::hash::{hash_swhid_object, HashFunction};
use crate::serialization::DigestSerializer;
use crate::utils::check_unique;
use crate::Bytestring;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BranchTarget {
    Content(Option<Digest>),
    Directory(Option<Digest>),
    Revision(Option<Digest>),
    Release(Option<Digest>),
    Snapshot(Option<Digest>),
    Alias(Option<Bytestring>),
}

impl BranchTarget {
    fn target_id(&self) -> &[u8] {
        match self {
            BranchTarget::Content(id)
            | BranchTarget::Directory(id)
            | BranchTarget::Revision(id)
            | BranchTarget::Release(id)
            | BranchTarget::Snapshot(id) => id.as_ref().map(Digest::as_bytes).unwrap_or(b""),
            BranchTarget::Alias(id) => id.as_ref().map(AsRef::as_ref).unwrap_or(b""),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Item in a [`Snapshot`]
pub struct Branch {
    pub name: Bytestring,
    pub target: BranchTarget,
}

impl Branch {
    pub fn new(name: Bytestring, target: BranchTarget) -> Self {
        Self { name, target }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Snapshot {
    /// sorted
    branches: Vec<Branch>,
}

impl Snapshot {
    pub fn new(mut branches: Vec<Branch>) -> Result<Self, SnapshotError> {
        sort_and_check_branches(&mut branches)?;

        Ok(Self { branches })
    }

    pub fn branches(&self) -> &[Branch] {
        &self.branches
    }

    /// Compute the SWHID v1.2 snapshot identifier for this snapshot.
    pub fn swhid(&self) -> Swhid {
        let manifest = snp_manifest_unchecked(&self.branches);
        Swhid::new_v1(
            ObjectType::Snapshot,
            hash_swhid_object("snapshot", &manifest),
        )
    }

    /// Compute the SWHID using the given hash and encoding config.
    pub fn swhid_with_config<H, E>(&self, config: &HashConfig<H, E>) -> Swhid
    where
        H: HashFunction,
        E: DigestSerializer,
        H::Output: Into<Digest>,
    {
        let manifest = snp_manifest_unchecked(&self.branches);
        let digest = config.hash.hash_object("snapshot", &manifest).into();
        Swhid::new(ObjectType::Snapshot, digest, config.version)
    }
}

/// Compute the SWHID v1.2 snapshot manifest (concatenation of branches).
///
/// This implements the SWHID v1.2 directory tree format, which is compatible
/// with Git's tree format for directory objects.
pub fn snp_manifest(mut branches: Vec<Branch>) -> Result<Vec<u8>, SnapshotError> {
    sort_and_check_branches(&mut branches)?;
    Ok(snp_manifest_unchecked(&branches))
}

fn sort_and_check_branches(branches: &mut [Branch]) -> Result<(), SnapshotError> {
    branches.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    check_unique(branches.iter().map(|branch| &branch.name))
        .map_err(|name| SnapshotError::DuplicateBranchName(name.clone()))?;

    for branch in branches {
        for byte in [b'\0'] {
            if branch.name.contains(&byte) {
                return Err(SnapshotError::InvalidByteInName {
                    byte,
                    name: branch.name.clone(),
                });
            }
        }
    }

    Ok(())
}

/// Same as [`snp_manifest`] but assumes children are already sorted and validated with
/// [`sort_and_check_branches`]
fn snp_manifest_unchecked(branches: &[Branch]) -> Vec<u8> {
    let mut out = Vec::new();
    for branch in branches {
        out.extend_from_slice(match branch.target {
            BranchTarget::Content(_) => b"content",
            BranchTarget::Directory(_) => b"directory",
            BranchTarget::Revision(_) => b"revision",
            BranchTarget::Release(_) => b"release",
            BranchTarget::Snapshot(_) => b"snapshot",
            BranchTarget::Alias(_) => b"alias",
        });
        out.push(b' ');
        out.extend_from_slice(&branch.name);
        out.push(b'\0');
        out.extend_from_slice(format!("{}", branch.target.target_id().len()).as_bytes());
        out.push(b':');
        out.extend_from_slice(branch.target.target_id());
    }

    out
}
