use std::path::PathBuf;

use clap::{Parser, Subcommand};

use swhid::{
    Content, DirectoryBuildOptions, DiskDirectoryBuilder, PermissionPolicy, PermissionsSourceKind,
    WalkOptions,
};
use swhid::{HashConfig, QualifiedSwhid, Swhid};

#[cfg(feature = "git")]
use swhid::git;

/// Small CLI for the SWHID reference implementation
#[derive(Parser, Debug)]
#[command(name = "swhid")]
#[command(about = "Compute and parse SWHIDs (ISO/IEC 18670)")]
#[command(version)]
#[command(disable_version_flag = true)]
struct Cli {
    /// SWHID version (1 or 2). Accepted for harness compatibility; use --hash and --format instead.
    #[arg(long, global = true, value_name = "VERSION")]
    version: Option<u8>,
    /// Hash algorithm (sha1, sha256, sha512). Requires matching feature. Default: sha1.
    #[arg(long, global = true, value_name = "HASH")]
    hash: Option<String>,
    /// Digest encoding (hex, base64, base64url, base32, base32hex, z85). Requires matching feature. Default: hex.
    #[arg(long, global = true, value_name = "FORMAT")]
    format: Option<String>,
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Compute a content SWHID from stdin or a file
    Content {
        /// Path to file (if omitted, read stdin)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Compute a directory SWHID recursively
    Dir {
        /// Directory root
        path: PathBuf,
        /// Follow symlinks (not recommended)
        #[arg(long)]
        follow_symlinks: bool,
        /// Exclude files matching these suffixes (e.g., .tmp, .log)
        #[arg(long, value_name = "SUFFIX")]
        exclude: Vec<String>,
        /// Permission source (auto, fs, git-index, git-tree, manifest, heuristic)
        #[arg(long, value_name = "SOURCE", default_value = "auto")]
        permissions_source: String,
        /// Permission policy (strict, best-effort)
        #[arg(long, value_name = "POLICY", default_value = "best-effort")]
        permissions_policy: String,
        /// Path to permission manifest file (required when source=manifest)
        #[arg(long, value_name = "PATH")]
        permissions_manifest: Option<PathBuf>,
    },
    /// Parse/pretty-print a (qualified) SWHID
    Parse {
        /// The SWHID string
        swhid: String,
    },
    /// Verify that a file or directory matches a given SWHID
    Verify {
        /// Path to file or directory
        path: PathBuf,
        /// Expected SWHID
        swhid: String,
        /// Follow symlinks (not recommended)
        #[arg(long)]
        follow_symlinks: bool,
        /// Exclude files matching these suffixes (e.g., .tmp, .log)
        #[arg(long, value_name = "SUFFIX")]
        exclude: Vec<String>,
        /// Permission source (auto, fs, git-index, git-tree, manifest, heuristic)
        #[arg(long, value_name = "SOURCE", default_value = "auto")]
        permissions_source: String,
        /// Permission policy (strict, best-effort)
        #[arg(long, value_name = "POLICY", default_value = "best-effort")]
        permissions_policy: String,
        /// Path to permission manifest file (required when source=manifest)
        #[arg(long, value_name = "PATH")]
        permissions_manifest: Option<PathBuf>,
    },
    /// Git repository SWHID computation (requires --features git)
    #[cfg(feature = "git")]
    Git {
        #[command(subcommand)]
        cmd: GitCommand,
    },
}

#[cfg(feature = "git")]
#[derive(Subcommand, Debug)]
enum GitCommand {
    /// Compute revision SWHID for a commit
    Revision {
        /// Git repository path
        repo: PathBuf,
        /// Commit hash (if omitted, use HEAD)
        commit: Option<String>,
    },
    /// Compute release SWHID for a tag
    Release {
        /// Git repository path
        repo: PathBuf,
        /// Tag name
        tag: String,
    },
    /// Compute snapshot SWHID for a repository
    Snapshot {
        /// Git repository path
        repo: PathBuf,
    },
    /// List all tags in a repository
    Tags {
        /// Git repository path
        repo: PathBuf,
    },
}

/// Build a helpful error for unsupported hash/format combinations.
fn hash_format_error(hash: Option<&str>, format: Option<&str>) -> String {
    let (h, f) = (hash.unwrap_or("(default)"), format.unwrap_or("(default)"));
    format!(
        "unsupported --hash/--format (hash={h}, format={f}). \
        Supported combinations (require matching features): \
        sha1+hex, sha256+hex, sha256+base64, sha256+base64url, sha256+base32, sha256+base32hex, sha256+z85, sha512+hex, sha512+base64url"
    )
}

/// Compute content SWHID string; when hash/format are set use that config.
fn content_swhid_string(
    bytes: Vec<u8>,
    hash: Option<&str>,
    format: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    match (hash, format) {
        (None, None) => Ok(Content::from_bytes(bytes).swhid().to_string()),
        (Some("sha1"), Some("hex")) => {
            #[cfg(all(feature = "sha1", feature = "encoding-hex"))]
            {
                let config = HashConfig::v1();
                let swhid = Content::from_bytes(bytes).swhid_with_config(&config);
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha1", feature = "encoding-hex")))]
            Err("sha1/hex not enabled (compile with sha1 and encoding-hex)".into())
        }
        (Some("sha256"), Some("hex")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-hex"))]
            {
                let config = HashConfig::v2_hex();
                let swhid = Content::from_bytes(bytes).swhid_with_config(&config);
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-hex")))]
            Err("sha256/hex not enabled (compile with sha256 and encoding-hex)".into())
        }
        (Some("sha256"), Some("base64")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base64"))]
            {
                let config = HashConfig::v2_base64();
                let swhid = Content::from_bytes(bytes).swhid_with_config(&config);
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base64")))]
            Err("sha256/base64 not enabled (compile with sha256 and encoding-base64)".into())
        }
        (Some("sha256"), Some("base64url")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base64url"))]
            {
                let config = HashConfig::v2();
                let swhid = Content::from_bytes(bytes).swhid_with_config(&config);
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base64url")))]
            Err("sha256/base64url not enabled (compile with sha256 and encoding-base64url)".into())
        }
        (Some("sha256"), Some("base32")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base32"))]
            {
                let config = HashConfig::v2_base32();
                let swhid = Content::from_bytes(bytes).swhid_with_config(&config);
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base32")))]
            Err("sha256/base32 not enabled (compile with sha256 and encoding-base32)".into())
        }
        (Some("sha256"), Some("base32hex")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base32hex"))]
            {
                let config = HashConfig::v2_base32hex();
                let swhid = Content::from_bytes(bytes).swhid_with_config(&config);
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base32hex")))]
            Err("sha256/base32hex not enabled (compile with sha256 and encoding-base32hex)".into())
        }
        (Some("sha256"), Some("z85")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-z85"))]
            {
                let config = HashConfig::v2_z85();
                let swhid = Content::from_bytes(bytes).swhid_with_config(&config);
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-z85")))]
            Err("sha256/z85 not enabled (compile with sha256 and encoding-z85)".into())
        }
        (Some("sha512"), Some("hex")) => {
            #[cfg(all(feature = "sha512", feature = "encoding-hex"))]
            {
                let config = HashConfig::sha512_hex();
                let swhid = Content::from_bytes(bytes).swhid_with_config(&config);
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha512", feature = "encoding-hex")))]
            Err("sha512/hex not enabled (compile with sha512 and encoding-hex)".into())
        }
        (Some("sha512"), Some("base64url")) => {
            #[cfg(all(feature = "sha512", feature = "encoding-base64url"))]
            {
                let config = HashConfig::sha512_base64url();
                let swhid = Content::from_bytes(bytes).swhid_with_config(&config);
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha512", feature = "encoding-base64url")))]
            Err("sha512/base64url not enabled (compile with sha512 and encoding-base64url)".into())
        }
        _ => Err(hash_format_error(hash, format).into()),
    }
}

/// Compute directory SWHID string; when hash/format are set use that config.
fn dir_swhid_string(
    dir: &swhid::Directory,
    hash: Option<&str>,
    format: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    match (hash, format) {
        (None, None) => Ok(dir.swhid()?.to_string()),
        (Some("sha1"), Some("hex")) => {
            #[cfg(all(feature = "sha1", feature = "encoding-hex"))]
            {
                let config = HashConfig::v1();
                let swhid = dir.swhid_with_config(&config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha1", feature = "encoding-hex")))]
            Err("sha1/hex not enabled".into())
        }
        (Some("sha256"), Some("hex")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-hex"))]
            {
                let config = HashConfig::v2_hex();
                let swhid = dir.swhid_with_config(&config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-hex")))]
            Err("sha256/hex not enabled".into())
        }
        (Some("sha256"), Some("base64")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base64"))]
            {
                let config = HashConfig::v2_base64();
                let swhid = dir.swhid_with_config(&config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base64")))]
            Err("sha256/base64 not enabled".into())
        }
        (Some("sha256"), Some("base64url")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base64url"))]
            {
                let config = HashConfig::v2();
                let swhid = dir.swhid_with_config(&config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base64url")))]
            Err("sha256/base64url not enabled".into())
        }
        (Some("sha256"), Some("base32")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base32"))]
            {
                let config = HashConfig::v2_base32();
                let swhid = dir.swhid_with_config(&config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base32")))]
            Err("sha256/base32 not enabled".into())
        }
        (Some("sha256"), Some("base32hex")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base32hex"))]
            {
                let config = HashConfig::v2_base32hex();
                let swhid = dir.swhid_with_config(&config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base32hex")))]
            Err("sha256/base32hex not enabled".into())
        }
        (Some("sha256"), Some("z85")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-z85"))]
            {
                let config = HashConfig::v2_z85();
                let swhid = dir.swhid_with_config(&config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-z85")))]
            Err("sha256/z85 not enabled".into())
        }
        (Some("sha512"), Some("hex")) => {
            #[cfg(all(feature = "sha512", feature = "encoding-hex"))]
            {
                let config = HashConfig::sha512_hex();
                let swhid = dir.swhid_with_config(&config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha512", feature = "encoding-hex")))]
            Err("sha512/hex not enabled".into())
        }
        (Some("sha512"), Some("base64url")) => {
            #[cfg(all(feature = "sha512", feature = "encoding-base64url"))]
            {
                let config = HashConfig::sha512_base64url();
                let swhid = dir.swhid_with_config(&config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha512", feature = "encoding-base64url")))]
            Err("sha512/base64url not enabled".into())
        }
        _ => Err(hash_format_error(hash, format).into()),
    }
}

#[cfg(feature = "git")]
fn git_revision_swhid_string(
    repo: &git2::Repository,
    commit_oid: &git2::Oid,
    hash: Option<&str>,
    format: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    match (hash, format) {
        (None, None) => {
            let swhid = git::revision_swhid(repo, commit_oid)?;
            Ok(swhid.to_string())
        }
        (Some("sha1"), Some("hex")) => {
            #[cfg(all(feature = "sha1", feature = "encoding-hex"))]
            {
                let config = HashConfig::v1();
                let swhid = git::revision_swhid_with_config(repo, commit_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha1", feature = "encoding-hex")))]
            Err("sha1/hex not enabled".into())
        }
        (Some("sha256"), Some("hex")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-hex"))]
            {
                let config = HashConfig::v2_hex();
                let swhid = git::revision_swhid_with_config(repo, commit_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-hex")))]
            Err("sha256/hex not enabled".into())
        }
        (Some("sha256"), Some("base64")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base64"))]
            {
                let config = HashConfig::v2_base64();
                let swhid = git::revision_swhid_with_config(repo, commit_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base64")))]
            Err("sha256/base64 not enabled".into())
        }
        (Some("sha256"), Some("base64url")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base64url"))]
            {
                let config = HashConfig::v2();
                let swhid = git::revision_swhid_with_config(repo, commit_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base64url")))]
            Err("sha256/base64url not enabled".into())
        }
        (Some("sha256"), Some("base32")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base32"))]
            {
                let config = HashConfig::v2_base32();
                let swhid = git::revision_swhid_with_config(repo, commit_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base32")))]
            Err("sha256/base32 not enabled".into())
        }
        (Some("sha256"), Some("base32hex")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base32hex"))]
            {
                let config = HashConfig::v2_base32hex();
                let swhid = git::revision_swhid_with_config(repo, commit_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base32hex")))]
            Err("sha256/base32hex not enabled".into())
        }
        (Some("sha256"), Some("z85")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-z85"))]
            {
                let config = HashConfig::v2_z85();
                let swhid = git::revision_swhid_with_config(repo, commit_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-z85")))]
            Err("sha256/z85 not enabled".into())
        }
        (Some("sha512"), Some("hex")) => {
            #[cfg(all(feature = "sha512", feature = "encoding-hex"))]
            {
                let config = HashConfig::sha512_hex();
                let swhid = git::revision_swhid_with_config(repo, commit_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha512", feature = "encoding-hex")))]
            Err("sha512/hex not enabled".into())
        }
        (Some("sha512"), Some("base64url")) => {
            #[cfg(all(feature = "sha512", feature = "encoding-base64url"))]
            {
                let config = HashConfig::sha512_base64url();
                let swhid = git::revision_swhid_with_config(repo, commit_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha512", feature = "encoding-base64url")))]
            Err("sha512/base64url not enabled".into())
        }
        _ => Err(hash_format_error(hash, format).into()),
    }
}

#[cfg(feature = "git")]
fn git_release_swhid_string(
    repo: &git2::Repository,
    tag_oid: &git2::Oid,
    hash: Option<&str>,
    format: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    match (hash, format) {
        (None, None) => Ok(git::release_swhid(repo, tag_oid)?.to_string()),
        (Some("sha1"), Some("hex")) => {
            #[cfg(all(feature = "sha1", feature = "encoding-hex"))]
            {
                let config = HashConfig::v1();
                let swhid = git::release_swhid_with_config(repo, tag_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha1", feature = "encoding-hex")))]
            Err("sha1/hex not enabled".into())
        }
        (Some("sha256"), Some("hex")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-hex"))]
            {
                let config = HashConfig::v2_hex();
                let swhid = git::release_swhid_with_config(repo, tag_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-hex")))]
            Err("sha256/hex not enabled".into())
        }
        (Some("sha256"), Some("base64")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base64"))]
            {
                let config = HashConfig::v2_base64();
                let swhid = git::release_swhid_with_config(repo, tag_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base64")))]
            Err("sha256/base64 not enabled".into())
        }
        (Some("sha256"), Some("base64url")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base64url"))]
            {
                let config = HashConfig::v2();
                let swhid = git::release_swhid_with_config(repo, tag_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base64url")))]
            Err("sha256/base64url not enabled".into())
        }
        (Some("sha256"), Some("base32")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base32"))]
            {
                let config = HashConfig::v2_base32();
                let swhid = git::release_swhid_with_config(repo, tag_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base32")))]
            Err("sha256/base32 not enabled".into())
        }
        (Some("sha256"), Some("base32hex")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base32hex"))]
            {
                let config = HashConfig::v2_base32hex();
                let swhid = git::release_swhid_with_config(repo, tag_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base32hex")))]
            Err("sha256/base32hex not enabled".into())
        }
        (Some("sha256"), Some("z85")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-z85"))]
            {
                let config = HashConfig::v2_z85();
                let swhid = git::release_swhid_with_config(repo, tag_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-z85")))]
            Err("sha256/z85 not enabled".into())
        }
        (Some("sha512"), Some("hex")) => {
            #[cfg(all(feature = "sha512", feature = "encoding-hex"))]
            {
                let config = HashConfig::sha512_hex();
                let swhid = git::release_swhid_with_config(repo, tag_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha512", feature = "encoding-hex")))]
            Err("sha512/hex not enabled".into())
        }
        (Some("sha512"), Some("base64url")) => {
            #[cfg(all(feature = "sha512", feature = "encoding-base64url"))]
            {
                let config = HashConfig::sha512_base64url();
                let swhid = git::release_swhid_with_config(repo, tag_oid, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha512", feature = "encoding-base64url")))]
            Err("sha512/base64url not enabled".into())
        }
        _ => Err(hash_format_error(hash, format).into()),
    }
}

#[cfg(feature = "git")]
fn git_snapshot_swhid_string(
    repo: &git2::Repository,
    hash: Option<&str>,
    format: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    match (hash, format) {
        (None, None) => Ok(git::snapshot_swhid(repo)?.to_string()),
        (Some("sha1"), Some("hex")) => {
            #[cfg(all(feature = "sha1", feature = "encoding-hex"))]
            {
                let config = HashConfig::v1();
                let swhid = git::snapshot_swhid_with_config(repo, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha1", feature = "encoding-hex")))]
            Err("sha1/hex not enabled".into())
        }
        (Some("sha256"), Some("hex")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-hex"))]
            {
                let config = HashConfig::v2_hex();
                let swhid = git::snapshot_swhid_with_config(repo, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-hex")))]
            Err("sha256/hex not enabled".into())
        }
        (Some("sha256"), Some("base64")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base64"))]
            {
                let config = HashConfig::v2_base64();
                let swhid = git::snapshot_swhid_with_config(repo, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base64")))]
            Err("sha256/base64 not enabled".into())
        }
        (Some("sha256"), Some("base64url")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base64url"))]
            {
                let config = HashConfig::v2();
                let swhid = git::snapshot_swhid_with_config(repo, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base64url")))]
            Err("sha256/base64url not enabled".into())
        }
        (Some("sha256"), Some("base32")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base32"))]
            {
                let config = HashConfig::v2_base32();
                let swhid = git::snapshot_swhid_with_config(repo, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base32")))]
            Err("sha256/base32 not enabled".into())
        }
        (Some("sha256"), Some("base32hex")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-base32hex"))]
            {
                let config = HashConfig::v2_base32hex();
                let swhid = git::snapshot_swhid_with_config(repo, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-base32hex")))]
            Err("sha256/base32hex not enabled".into())
        }
        (Some("sha256"), Some("z85")) => {
            #[cfg(all(feature = "sha256", feature = "encoding-z85"))]
            {
                let config = HashConfig::v2_z85();
                let swhid = git::snapshot_swhid_with_config(repo, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha256", feature = "encoding-z85")))]
            Err("sha256/z85 not enabled".into())
        }
        (Some("sha512"), Some("hex")) => {
            #[cfg(all(feature = "sha512", feature = "encoding-hex"))]
            {
                let config = HashConfig::sha512_hex();
                let swhid = git::snapshot_swhid_with_config(repo, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha512", feature = "encoding-hex")))]
            Err("sha512/hex not enabled".into())
        }
        (Some("sha512"), Some("base64url")) => {
            #[cfg(all(feature = "sha512", feature = "encoding-base64url"))]
            {
                let config = HashConfig::sha512_base64url();
                let swhid = git::snapshot_swhid_with_config(repo, &config)?;
                Ok(swhid.to_string_encoded(&config.encoder))
            }
            #[cfg(not(all(feature = "sha512", feature = "encoding-base64url")))]
            Err("sha512/base64url not enabled".into())
        }
        _ => Err(hash_format_error(hash, format).into()),
    }
}

fn parse_permissions_source(s: &str) -> Result<PermissionsSourceKind, Box<dyn std::error::Error>> {
    match s {
        "auto" => Ok(PermissionsSourceKind::Auto),
        "fs" | "filesystem" => Ok(PermissionsSourceKind::Filesystem),
        "git-index" => Ok(PermissionsSourceKind::GitIndex),
        "git-tree" => Ok(PermissionsSourceKind::GitTree),
        "manifest" => Ok(PermissionsSourceKind::Manifest),
        "heuristic" => Ok(PermissionsSourceKind::Heuristic),
        _ => Err(format!(
            "Invalid permissions source: {}. Must be auto, fs, git-index, git-tree, manifest, or heuristic",
            s
        ).into()),
    }
}

fn parse_permissions_policy(s: &str) -> Result<PermissionPolicy, Box<dyn std::error::Error>> {
    match s {
        "strict" => Ok(PermissionPolicy::Strict),
        "best-effort" | "besteffort" => Ok(PermissionPolicy::BestEffort),
        _ => Err(format!(
            "Invalid permissions policy: {}. Must be strict or best-effort",
            s
        )
        .into()),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let hash = cli.hash.as_deref();
    let format = cli.format.as_deref();
    match cli.cmd {
        Command::Content { file } => {
            let bytes = if let Some(p) = file {
                std::fs::read(p)?
            } else {
                use std::io::Read;
                let mut buf = Vec::new();
                std::io::stdin().read_to_end(&mut buf)?;
                buf
            };
            let s = content_swhid_string(bytes, hash, format)?;
            println!("{s}");
        }
        Command::Dir {
            path,
            follow_symlinks,
            exclude,
            permissions_source,
            permissions_policy,
            permissions_manifest,
        } => {
            let perm_source = parse_permissions_source(&permissions_source)?;
            let perm_policy = parse_permissions_policy(&permissions_policy)?;

            if perm_source == PermissionsSourceKind::Manifest && permissions_manifest.is_none() {
                return Err(
                    "--permissions-manifest is required when --permissions-source=manifest".into(),
                );
            }

            let build_opts = DirectoryBuildOptions {
                permissions_source: perm_source,
                permissions_policy: perm_policy,
                permissions_manifest_path: permissions_manifest,
                walk_options: WalkOptions {
                    follow_symlinks,
                    exclude_suffixes: exclude,
                },
            };

            let dir = DiskDirectoryBuilder::new(&path).with_build_options(build_opts);
            let dir = dir.build()?;
            let s = dir_swhid_string(&dir, hash, format)?;
            println!("{s}");
        }
        Command::Parse { swhid } => {
            // Try qualified first, fallback to core
            match swhid.parse::<QualifiedSwhid>() {
                Ok(q) => println!("{q}"),
                Err(_) => {
                    let core: Swhid = swhid.parse()?;
                    println!("{core}");
                }
            }
        }
        Command::Verify {
            path,
            swhid,
            follow_symlinks,
            exclude,
            permissions_source,
            permissions_policy,
            permissions_manifest,
        } => {
            let perm_source = parse_permissions_source(&permissions_source)?;
            let perm_policy = parse_permissions_policy(&permissions_policy)?;

            if perm_source == PermissionsSourceKind::Manifest && permissions_manifest.is_none() {
                return Err(
                    "--permissions-manifest is required when --permissions-source=manifest".into(),
                );
            }

            let expected_str = &swhid;
            let actual_str = if path.is_file() {
                let bytes = std::fs::read(&path)?;
                content_swhid_string(bytes, hash, format)?
            } else if path.is_dir() {
                let build_opts = DirectoryBuildOptions {
                    permissions_source: perm_source,
                    permissions_policy: perm_policy,
                    permissions_manifest_path: permissions_manifest,
                    walk_options: WalkOptions {
                        follow_symlinks,
                        exclude_suffixes: exclude,
                    },
                };
                let dir = DiskDirectoryBuilder::new(&path)
                    .with_build_options(build_opts)
                    .build()?;
                dir_swhid_string(&dir, hash, format)?
            } else {
                eprintln!(
                    "Error: {} is neither a file nor a directory",
                    path.display()
                );
                std::process::exit(1);
            };
            if hash.is_some() && format.is_some() {
                if actual_str == *expected_str {
                    println!(
                        "✓ Verification successful: {} matches {}",
                        path.display(),
                        expected_str
                    );
                    std::process::exit(0);
                } else {
                    println!(
                        "✗ Verification failed: {} does not match {}",
                        path.display(),
                        expected_str
                    );
                    println!("  Expected: {expected_str}");
                    println!("  Actual:   {actual_str}");
                    std::process::exit(1);
                }
            } else {
                let expected: Swhid = expected_str.parse()?;
                let actual: Swhid = actual_str
                    .parse()
                    .map_err(|e| format!("Computed SWHID did not parse: {e}"))?;
                if actual == expected {
                    println!(
                        "✓ Verification successful: {} matches {}",
                        path.display(),
                        expected
                    );
                    std::process::exit(0);
                } else {
                    println!(
                        "✗ Verification failed: {} does not match {}",
                        path.display(),
                        expected
                    );
                    println!("  Expected: {expected}");
                    println!("  Actual:   {actual}");
                    std::process::exit(1);
                }
            }
        }
        #[cfg(feature = "git")]
        Command::Git { cmd } => match cmd {
            GitCommand::Revision { repo, commit } => {
                let repo = git::open_repo(&repo)?;
                let commit_oid = if let Some(commit_str) = commit {
                    git2::Oid::from_str(&commit_str)
                        .map_err(|e| format!("Invalid commit hash: {e}"))?
                } else {
                    git::get_head_commit(&repo)?
                };
                let s = git_revision_swhid_string(&repo, &commit_oid, hash, format)?;
                println!("{s}");
            }
            GitCommand::Release { repo, tag } => {
                let repo = git::open_repo(&repo)?;
                let tag_oid = repo
                    .refname_to_id(&format!("refs/tags/{tag}"))
                    .map_err(|e| format!("Tag not found: {e}"))?;
                let s = git_release_swhid_string(&repo, &tag_oid, hash, format)?;
                println!("{s}");
            }
            GitCommand::Snapshot { repo } => {
                let repo = git::open_repo(&repo)?;
                let s = git_snapshot_swhid_string(&repo, hash, format)?;
                println!("{s}");
            }
            GitCommand::Tags { repo } => {
                let repo = git::open_repo(&repo)?;
                let tags = git::get_tags(&repo)?;
                for tag_oid in tags {
                    println!("{tag_oid}");
                }
            }
        },
    }
    Ok(())
}
