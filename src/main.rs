use clap::{Parser, Subcommand};
use std::path::PathBuf;

use swhid::{
    Base32HexSerializer, Base32Serializer, Base64Serializer, Base64UrlSerializer, Content,
    DigestSerializer, DirectoryBuildOptions, DiskDirectoryBuilder, HashConfig, HashFunction,
    PermissionPolicy, PermissionsSourceKind, SwhidVersion, WalkOptions, Z85Serializer,
};
use swhid::{QualifiedSwhid, Swhid};

#[cfg(feature = "git")]
use swhid::git;

/// Small CLI for the SWHID reference implementation
#[derive(Parser, Debug)]
#[command(name = "swhid")]
#[command(about = "Compute and parse SWHIDs (ISO/IEC 18670)")]
#[command(version)]
struct Cli {
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
        /// SWHID version: 1 = SHA-1 (20 bytes), 2 = SHA-256 (32 bytes). V1 only with sha1+hex
        #[arg(long, value_name = "VERSION", default_value = "1")]
        version: String,
        /// Hash: sha1 or sha256. Overrides --version. sha1 => v1 (hex only), sha256 => v2
        #[arg(long, value_name = "HASH")]
        hash: Option<String>,
        /// Digest encoding: hex, base64, base64url, base32, base32hex, z85
        #[arg(long, value_name = "FORMAT", default_value = "hex")]
        format: String,
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
        /// SWHID version: 1 = SHA-1 (20 bytes), 2 = SHA-256 (32 bytes). V1 only with sha1+hex
        #[arg(long, value_name = "VERSION", default_value = "1")]
        version: String,
        /// Hash: sha1 or sha256. Overrides --version. sha1 => v1 (hex only), sha256 => v2
        #[arg(long, value_name = "HASH")]
        hash: Option<String>,
        /// Digest encoding: hex, base64, base64url, base32, base32hex, z85
        #[arg(long, value_name = "FORMAT", default_value = "hex")]
        format: String,
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
        /// SWHID version: 1 = SHA-1 (20 bytes), 2 = SHA-256 (32 bytes). V1 only with sha1+hex
        #[arg(long, value_name = "VERSION", default_value = "1")]
        version: String,
        /// Hash: sha1 or sha256. Overrides --version. sha1 => v1 (hex only), sha256 => v2
        #[arg(long, value_name = "HASH")]
        hash: Option<String>,
        /// Digest encoding: hex, base64, base64url, base32, base32hex, z85
        #[arg(long, value_name = "FORMAT", default_value = "hex")]
        format: String,
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

fn config_from_hash_and_format(
    hash: Option<&str>,
    format: &str,
    version: &str,
) -> Result<HashConfig, Box<dyn std::error::Error>> {
    use swhid::{HashAlgorithm, Sha1Hash, Sha256Hash};
    use swhid::HexSerializer;
    use swhid::SwhidVersion;

    // Determine hash algorithm
    let hash_algorithm = if let Some(h) = hash {
        match h.to_lowercase().as_str() {
            "sha1" | "sha-1" => HashAlgorithm::Sha1,
            "sha256" | "sha-256" => HashAlgorithm::Sha256,
            _ => {
                return Err(format!(
                    "Invalid hash algorithm: {}. Must be sha1 or sha256",
                    h
                )
                .into());
            }
        }
    } else {
        // Use version to determine hash
        match version {
            "1" => HashAlgorithm::Sha1,
            "2" => HashAlgorithm::Sha256,
            _ => {
                return Err(format!(
                    "Invalid SWHID version: {}. Must be 1 or 2",
                    version
                )
                .into());
            }
        }
    };

    // Determine serializer
    let format_lower = format.to_lowercase();
    let is_hex = format_lower == "hex";
    let serializer: Box<dyn DigestSerializer> = match format_lower.as_str() {
        "hex" => Box::new(HexSerializer),
        "base64" => Box::new(Base64Serializer),
        "base64url" => Box::new(Base64UrlSerializer),
        "base32" => Box::new(Base32Serializer),
        "base32hex" => Box::new(Base32HexSerializer),
        "z85" => Box::new(Z85Serializer),
        _ => {
            return Err(format!(
                "Invalid format: {}. Must be hex, base64, base64url, base32, base32hex, or z85",
                format
            )
            .into());
        }
    };

    // Version 1 is only used for sha1+hex; all other combinations use version 2
    let swhid_version = match (hash_algorithm, is_hex) {
        (HashAlgorithm::Sha1, true) => SwhidVersion::V1,
        _ => SwhidVersion::V2,
    };

    // Warn when user's --version doesn't match the effective version
    if version == "1" && swhid_version != SwhidVersion::V1 {
        eprintln!(
            "Warning: SWHID version 1 only applies to sha1+hex. Using version 2 for the selected hash/format."
        );
    } else if version == "2" && swhid_version == SwhidVersion::V1 {
        eprintln!(
            "Warning: sha1+hex always uses SWHID version 1; --version 2 is ignored."
        );
    }

    // Build config
    let hash_function: Box<dyn HashFunction> = match hash_algorithm {
        HashAlgorithm::Sha1 => Box::new(Sha1Hash),
        HashAlgorithm::Sha256 => Box::new(Sha256Hash),
    };

    Ok(HashConfig {
        hash_function,
        serializer,
        version: swhid_version,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.cmd {
        Command::Content {
            file,
            version,
            hash,
            format,
        } => {
            let config = config_from_hash_and_format(hash.as_deref(), &format, &version)?;
            let bytes = if let Some(p) = file {
                std::fs::read(p)?
            } else {
                use std::io::Read;
                let mut buf = Vec::new();
                std::io::stdin().read_to_end(&mut buf)?;
                buf
            };
            let s = Content::from_bytes(bytes).swhid_with_config(&config);
            let encoded = config.serializer.encode(s.digest_bytes());
            let version_str = match s.version() {
                SwhidVersion::V1 => "1",
                SwhidVersion::V2 => "2",
            };
            println!("swh:{}:{}:{}", version_str, s.object_type().as_tag(), encoded);
        }
        Command::Dir {
            path,
            follow_symlinks,
            exclude,
            permissions_source,
            permissions_policy,
            permissions_manifest,
            version,
            hash,
            format,
        } => {
            let config = config_from_hash_and_format(hash.as_deref(), &format, &version)?;
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
            let swhid = dir.swhid_with_config(&config)?;
            let encoded = config.serializer.encode(swhid.digest_bytes());
            let version_str = match swhid.version() {
                SwhidVersion::V1 => "1",
                SwhidVersion::V2 => "2",
            };
            println!("swh:{}:{}:{}", version_str, swhid.object_type().as_tag(), encoded);
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

            let expected: Swhid = swhid.parse()?;
            // Use the same version as the expected SWHID
            let config = match expected.version() {
                SwhidVersion::V1 => HashConfig::v1(),
                SwhidVersion::V2 => HashConfig::v2_sha256_hex(),
            };
            let actual = if path.is_file() {
                let bytes = std::fs::read(&path)?;
                Content::from_bytes(bytes).swhid_with_config(&config)
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
                let dir = DiskDirectoryBuilder::new(&path).with_build_options(build_opts);
                dir.swhid_with_config(&config)?
            } else {
                eprintln!(
                    "Error: {} is neither a file nor a directory",
                    path.display()
                );
                std::process::exit(1);
            };

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
        #[cfg(feature = "git")]
        Command::Git {
            version,
            hash,
            format,
            cmd,
        } => {
            let config = config_from_hash_and_format(hash.as_deref(), &format, &version)?;
            match cmd {
                GitCommand::Revision { repo, commit } => {
                    let repo = git::open_repo(&repo)?;
                    let commit_oid = if let Some(commit_str) = commit {
                        git2::Oid::from_str(&commit_str)
                            .map_err(|e| format!("Invalid commit hash: {e}"))?
                    } else {
                        git::get_head_commit(&repo)?
                    };
                    let swhid = git::revision_swhid_with_config(&repo, &commit_oid, &config)?;
                    let encoded = config.serializer.encode(swhid.digest_bytes());
                    let version_str = match swhid.version() {
                        SwhidVersion::V1 => "1",
                        SwhidVersion::V2 => "2",
                    };
                    println!("swh:{}:{}:{}", version_str, swhid.object_type().as_tag(), encoded);
                }
                GitCommand::Release { repo, tag } => {
                    let repo = git::open_repo(&repo)?;
                    let tag_oid = repo
                        .refname_to_id(&format!("refs/tags/{tag}"))
                        .map_err(|e| format!("Tag not found: {e}"))?;
                    let swhid = git::release_swhid_with_config(&repo, &tag_oid, &config)?;
                    let encoded = config.serializer.encode(swhid.digest_bytes());
                    let version_str = match swhid.version() {
                        SwhidVersion::V1 => "1",
                        SwhidVersion::V2 => "2",
                    };
                    println!("swh:{}:{}:{}", version_str, swhid.object_type().as_tag(), encoded);
                }
                GitCommand::Snapshot { repo } => {
                    let repo = git::open_repo(&repo)?;
                    let swhid = git::snapshot_swhid_with_config(&repo, &config)?;
                    let encoded = config.serializer.encode(swhid.digest_bytes());
                    let version_str = match swhid.version() {
                        SwhidVersion::V1 => "1",
                        SwhidVersion::V2 => "2",
                    };
                    println!("swh:{}:{}:{}", version_str, swhid.object_type().as_tag(), encoded);
                }
                GitCommand::Tags { repo } => {
                    let repo = git::open_repo(&repo)?;
                    let tags = git::get_tags(&repo)?;
                    for tag_oid in tags {
                        println!("{tag_oid}");
                    }
                }
            }
        }
    }
    Ok(())
}
