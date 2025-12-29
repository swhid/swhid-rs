use crate::core::{ObjectType, Swhid};
use crate::types::SwhidVersion;
use crate::error::SnapshotError;
use crate::hash::{hash_swhid_object, hash_swhid_object_with};
use crate::config::HashConfig;
use crate::utils::check_unique;
use crate::Bytestring;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BranchTarget {
    /// Content digest (20 bytes for SHA1, 32 bytes for SHA256)
    Content(Option<Vec<u8>>),
    /// Directory digest (20 bytes for SHA1, 32 bytes for SHA256)
    Directory(Option<Vec<u8>>),
    /// Revision digest (20 bytes for SHA1, 32 bytes for SHA256)
    Revision(Option<Vec<u8>>),
    /// Release digest (20 bytes for SHA1, 32 bytes for SHA256)
    Release(Option<Vec<u8>>),
    /// Snapshot digest (20 bytes for SHA1, 32 bytes for SHA256)
    Snapshot(Option<Vec<u8>>),
    Alias(Option<Bytestring>),
}

impl BranchTarget {
    fn target_id(&self) -> &[u8] {
        match self {
            BranchTarget::Content(id)
            | BranchTarget::Directory(id)
            | BranchTarget::Revision(id)
            | BranchTarget::Release(id)
            | BranchTarget::Snapshot(id) => id.as_deref().unwrap_or(b""),
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

    /// Compute the SWHID snapshot identifier using the specified hash configuration.
    ///
    /// This allows computing SWHIDs with different hash functions (SHA1, SHA256, etc.)
    /// and serialization formats (hex, base64, etc.) for v2 experimentation.
    ///
    /// Note: This method currently uses the same manifest format as v1, but with
    /// the specified hash function. The branch target IDs still contain [u8; 20] digests
    /// which are converted to hex for the manifest.
    pub fn swhid_with_config(&self, config: &HashConfig) -> Swhid {
        let manifest = snp_manifest_unchecked(&self.branches);
        let digest = hash_swhid_object_with("snapshot", &manifest, config.hash_function.as_ref());
        Swhid::new(ObjectType::Snapshot, digest, config.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::HashConfig;

    #[test]
    fn snapshot_swhid_v1() {
        let snapshot = Snapshot::new(vec![]).unwrap();
        let swhid = snapshot.swhid();
        assert_eq!(swhid.version(), SwhidVersion::V1);
        assert_eq!(swhid.digest_bytes().len(), 20);
    }

    #[test]
    fn snapshot_swhid_with_config_v2() {
        let snapshot = Snapshot::new(vec![]).unwrap();
        let config = HashConfig::v2_sha256_hex();
        let swhid = snapshot.swhid_with_config(&config);
        assert_eq!(swhid.version(), SwhidVersion::V2);
        assert_eq!(swhid.digest_bytes().len(), 32);
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
