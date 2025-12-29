use clap::{Parser, Subcommand};
use std::path::PathBuf;

use swhid::{Content, DiskDirectoryBuilder, WalkOptions};
use swhid::{QualifiedSwhid, Swhid, ObjectType};
use swhid::config::HashConfig;
use swhid::types::{SwhidVersion, HashAlgorithm, Encoding};

#[cfg(feature = "git")]
use swhid::git;

/// Small CLI for the SWHID reference implementation
#[derive(Parser, Debug)]
#[command(name = "swhid")]
#[command(about = "Compute and parse SWHIDs (ISO/IEC 18670)")]
struct Cli {
    /// SWHID version (1 or 2)
    #[arg(long, value_name = "VERSION", default_value = "1")]
    version: SwhidVersion,
    /// Hash function (sha1 or sha256)
    #[arg(long, value_name = "HASH", default_value = "sha1")]
    hash: HashAlgorithm,
    /// Serialization format (hex, base64, base64url, base32, base32hex, or z85)
    #[arg(long, value_name = "FORMAT", default_value = "hex")]
    serialization: Encoding,
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

fn get_hash_config(version: SwhidVersion, hash: HashAlgorithm, serialization: Encoding) -> Result<HashConfig, Box<dyn std::error::Error>> {
    // Create appropriate config based on enum values
    match (version, hash, serialization) {
        (SwhidVersion::V1, HashAlgorithm::Sha1, Encoding::Hex) => Ok(HashConfig::v1()),
        (SwhidVersion::V2, HashAlgorithm::Sha256, Encoding::Hex) => Ok(HashConfig::v2_sha256_hex()),
        (SwhidVersion::V2, HashAlgorithm::Sha256, Encoding::Base64) => Ok(HashConfig::v2_sha256_base64()),
        (SwhidVersion::V2, HashAlgorithm::Sha256, Encoding::Base64Url) => Ok(HashConfig::v2_sha256_base64url()),
        (SwhidVersion::V2, HashAlgorithm::Sha256, Encoding::Base32) => Ok(HashConfig::v2_sha256_base32()),
        (SwhidVersion::V2, HashAlgorithm::Sha256, Encoding::Base32Hex) => Ok(HashConfig::v2_sha256_base32hex()),
        (SwhidVersion::V2, HashAlgorithm::Sha256, Encoding::Z85) => Ok(HashConfig::v2_sha256_z85()),
        (SwhidVersion::V2, HashAlgorithm::Sha1, Encoding::Hex) => Ok(HashConfig::v1()), // v2 with sha1+hex is same as v1
        _ => Err(format!(
            "Invalid combination: version={}, hash={}, serialization={}. \
            v1 only supports sha1+hex. v2 supports sha256 with hex/base64/base64url/base32/base32hex/z85",
            version.as_str(), hash.as_str(), serialization.as_str()
        ).into()),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = get_hash_config(cli.version, cli.hash, cli.serialization)?;
    let use_v2 = cli.version == SwhidVersion::V2;
    
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
            let content = Content::from_bytes(bytes);
            let swhid = if use_v2 {
                content.swhid_with_config(&config)
            } else {
                content.swhid()
            };
            // Use serializer from config for output (P0.1 fix)
            if use_v2 {
                println!("{}", swhid.to_string_with(config.serializer.as_ref())?);
            } else {
                println!("{swhid}");
            }
        }
        Command::Dir {
            path,
            follow_symlinks,
            exclude,
        } => {
            let mut opts = WalkOptions {
                follow_symlinks,
                ..Default::default()
            };
            opts.exclude_suffixes = exclude;
            let dir = DiskDirectoryBuilder::new(&path)
                .with_options(opts);
            let swhid = if use_v2 {
                dir.swhid_with_config(&config)?
            } else {
                dir.swhid()?
            };
            // Use serializer from config for output (P0.1 fix)
            if use_v2 {
                println!("{}", swhid.to_string_with(config.serializer.as_ref())?);
            } else {
                println!("{swhid}");
            }
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
        } => {
            // Try to parse with config serializer if v2, otherwise use default hex
            let expected = if use_v2 {
                // For v2, try to decode using the config serializer
                // First try parsing as hex (canonical), then try with config serializer
                match swhid.parse::<Swhid>() {
                    Ok(s) => s,
                    Err(_) => {
                        // If hex parse fails, try decoding with config serializer
                        // Extract version, type, and digest from string
                        let parts: Vec<&str> = swhid.split(':').collect();
                        if parts.len() == 4 && parts[0] == "swh" && parts[1] == "2" {
                            let digest_bytes = config.decode_digest(parts[3])?;
                            Swhid::new(
                                ObjectType::from_tag(parts[2])?,
                                digest_bytes,
                                SwhidVersion::V2,
                            )
                        } else {
                            return Err(format!("Invalid SWHID format: {}", swhid).into());
                        }
                    }
                }
            } else {
                swhid.parse::<Swhid>()?
            };
            
            let actual = if path.is_file() {
                let bytes = std::fs::read(&path)?;
                let content = Content::from_bytes(bytes);
                if use_v2 {
                    content.swhid_with_config(&config)
                } else {
                    content.swhid()
                }
            } else if path.is_dir() {
                let mut opts = WalkOptions {
                    follow_symlinks,
                    ..Default::default()
                };
                opts.exclude_suffixes = exclude;
                let dir = DiskDirectoryBuilder::new(&path)
                    .with_options(opts);
                if use_v2 {
                    dir.swhid_with_config(&config)?
                } else {
                    dir.swhid()?
                }
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
                    if use_v2 {
                        actual.to_string_with(config.serializer.as_ref())?
                    } else {
                        actual.to_string()
                    }
                );
                std::process::exit(0);
            } else {
                println!(
                    "✗ Verification failed: {} does not match {}",
                    path.display(),
                    if use_v2 {
                        expected.to_string_with(config.serializer.as_ref())?
                    } else {
                        expected.to_string()
                    }
                );
                println!(
                    "  Expected: {}",
                    if use_v2 {
                        expected.to_string_with(config.serializer.as_ref())?
                    } else {
                        expected.to_string()
                    }
                );
                println!(
                    "  Actual:   {}",
                    if use_v2 {
                        actual.to_string_with(config.serializer.as_ref())?
                    } else {
                        actual.to_string()
                    }
                );
                std::process::exit(1);
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
                let swhid = if use_v2 {
                    git::revision_swhid_with_config(&repo, &commit_oid, &config)?
                } else {
                    git::revision_swhid(&repo, &commit_oid)?
                };
                if use_v2 {
                    println!("{}", swhid.to_string_with(config.serializer.as_ref())?);
                } else {
                    println!("{swhid}");
                }
            }
            GitCommand::Release { repo, tag } => {
                let repo = git::open_repo(&repo)?;
                let tag_oid = repo
                    .refname_to_id(&format!("refs/tags/{tag}"))
                    .map_err(|e| format!("Tag not found: {e}"))?;
                let swhid = if use_v2 {
                    git::release_swhid_with_config(&repo, &tag_oid, &config)?
                } else {
                    git::release_swhid(&repo, &tag_oid)?
                };
                if use_v2 {
                    println!("{}", swhid.to_string_with(config.serializer.as_ref())?);
                } else {
                    println!("{swhid}");
                }
            }
            GitCommand::Snapshot { repo } => {
                let repo = git::open_repo(&repo)?;
                let swhid = if use_v2 {
                    git::snapshot_swhid_with_config(&repo, &config)?
                } else {
                    git::snapshot_swhid(&repo)?
                };
                if use_v2 {
                    println!("{}", swhid.to_string_with(config.serializer.as_ref())?);
                } else {
                    println!("{swhid}");
                }
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
