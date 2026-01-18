use clap::{Parser, Subcommand};
use std::path::PathBuf;

use swhid::{
    Content, DirectoryBuildOptions, DiskDirectoryBuilder, PermissionPolicy, PermissionsSourceKind,
    WalkOptions,
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
            let s = Content::from_bytes(bytes).swhid();
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
            let swhid = dir.swhid()?;
            println!("{swhid}");
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
            let actual = if path.is_file() {
                let bytes = std::fs::read(&path)?;
                Content::from_bytes(bytes).swhid()
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
                dir.swhid()?
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
        Command::Git { cmd } => match cmd {
            GitCommand::Revision { repo, commit } => {
                let repo = git::open_repo(&repo)?;
                let commit_oid = if let Some(commit_str) = commit {
                    git2::Oid::from_str(&commit_str)
                        .map_err(|e| format!("Invalid commit hash: {e}"))?
                } else {
                    git::get_head_commit(&repo)?
                };
                let swhid = git::revision_swhid(&repo, &commit_oid)?;
                println!("{swhid}");
            }
            GitCommand::Release { repo, tag } => {
                let repo = git::open_repo(&repo)?;
                let tag_oid = repo
                    .refname_to_id(&format!("refs/tags/{tag}"))
                    .map_err(|e| format!("Tag not found: {e}"))?;
                let swhid = git::release_swhid(&repo, &tag_oid)?;
                println!("{swhid}");
            }
            GitCommand::Snapshot { repo } => {
                let repo = git::open_repo(&repo)?;
                let swhid = git::snapshot_swhid(&repo)?;
                println!("{swhid}");
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
