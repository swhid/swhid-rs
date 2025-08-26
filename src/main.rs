use clap::Parser;
use std::io::Read;
use std::path::Path;
use swhid::SwhidComputer;

#[derive(Parser)]
#[command(name = "swhid-cli")]
#[command(about = "Compute Software Hash Identifiers (SWHID)")]
struct Cli {
    /// Type of object to identify
    #[arg(short, long, default_value = "auto")]
    obj_type: String,

    /// Follow symlinks (violates SWHID specification)
    #[arg(long, default_value = "false")]
    dereference: bool,

    /// Don't follow symlinks (default: compliant with SWHID specification)
    #[arg(long, conflicts_with = "dereference")]
    no_dereference: bool,

    /// Show filename in output
    #[arg(long, default_value = "true")]
    filename: bool,

    /// Exclude directories using glob patterns
    #[arg(short, long)]
    exclude: Vec<String>,

    /// Reference identifier to be compared with computed one
    #[arg(short, long)]
    verify: Option<String>,

    #[cfg(feature = "git")]
    /// Git revision to compute SWHID for (requires --git feature)
    #[arg(long)]
    revision: Option<String>,

    #[cfg(feature = "git")]
    /// Git release/tag to compute SWHID for (requires --git feature)
    #[arg(long)]
    release: Option<String>,

    #[cfg(feature = "git")]
    /// Compute Git snapshot SWHID (requires --git feature)
    #[arg(long)]
    snapshot: bool,

    /// Objects to identify
    objects: Vec<String>,
}

#[cfg(feature = "git")]
mod git_support {
    use super::*;
    use git2::Repository;

    pub fn compute_git_revision_swhid(repo_path: &str, revision: &str) -> Result<String, Box<dyn std::error::Error>> {
        let repo = Repository::open(repo_path)?;
        let commit = repo.revparse_single(revision)?.peel_to_commit()?;
        
        // Format as Git commit object
        let tree_id = commit.tree_id().to_string();
        let parent_ids: Vec<String> = commit.parents().map(|p| p.id().to_string()).collect();
        let author = commit.author().to_string();
        let committer = commit.committer().to_string();
        let message = commit.message().unwrap_or("").to_string();
        
        // Create Git commit object format
        let mut commit_data = format!("tree {}\n", tree_id);
        for parent_id in parent_ids {
            commit_data.push_str(&format!("parent {}\n", parent_id));
        }
        commit_data.push_str(&format!("author {}\n", author));
        commit_data.push_str(&format!("committer {}\n", committer));
        commit_data.push_str("\n");
        commit_data.push_str(&message);
        
        // Compute SHA1 hash
        let swhid = swhid::hash::hash_git_object("commit", commit_data.as_bytes());
        Ok(format!("swh:1:rev:{}", hex::encode(swhid)))
    }

    pub fn compute_git_release_swhid(repo_path: &str, tag_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let repo = Repository::open(repo_path)?;
        let tag = repo.find_tag(repo.revparse_single(tag_name)?.id())?;
        
        // Format as Git tag object
        let target_id = tag.target_id().to_string();
        let target_type = if tag.target().unwrap().kind() == Some(git2::ObjectType::Commit) {
            "commit"
        } else {
            "tree"
        };
        let tagger = tag.tagger().unwrap().to_string();
        let message = tag.message().unwrap_or("").to_string();
        
        // Create Git tag object format
        let mut tag_data = format!("object {}\n", target_id);
        tag_data.push_str(&format!("type {}\n", target_type));
        tag_data.push_str(&format!("tag {}\n", tag_name));
        tag_data.push_str(&format!("tagger {}\n", tagger));
        tag_data.push_str("\n");
        tag_data.push_str(&message);
        
        // Compute SHA1 hash
        let swhid = swhid::hash::hash_git_object("tag", tag_data.as_bytes());
        Ok(format!("swh:1:rel:{}", hex::encode(swhid)))
    }

    pub fn compute_git_snapshot_swhid(repo_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let repo = Repository::open(repo_path)?;
        let mut refs_data = String::new();
        
        // Get all references
        let refs = repo.references()?;
        for reference in refs {
            let reference = reference?;
            let name = reference.name().unwrap_or("").to_string();
            let target_id = reference.target().unwrap().to_string();
            
            // Check if it's a branch or tag reference
            if name.starts_with("refs/heads/") || name.starts_with("refs/tags/") {
                refs_data.push_str(&format!("{} {}\n", name, target_id));
            }
        }
        
        // Compute SHA1 hash
        let swhid = swhid::hash::hash_git_object("snapshot", refs_data.as_bytes());
        Ok(format!("swh:1:snp:{}", hex::encode(swhid)))
    }
}

fn identify_object(
    obj_type: &str,
    follow_symlinks: bool,
    exclude_patterns: &[String],
    obj: &str,
    #[cfg(feature = "git")] revision: Option<&str>,
    #[cfg(feature = "git")] release: Option<&str>,
    #[cfg(feature = "git")] snapshot: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let computer = SwhidComputer::new()
        .with_follow_symlinks(follow_symlinks)
        .with_exclude_patterns(exclude_patterns);

    let obj_type = if obj_type == "auto" {
        if obj == "-" {
            "content"
        } else if Path::new(obj).is_file() {
            "content"
        } else if Path::new(obj).is_dir() {
            // Check if it's a Git repository
            let git_path = Path::new(obj).join(".git");
            if git_path.exists() && git_path.is_dir() {
                #[cfg(feature = "git")]
                if snapshot {
                    return git_support::compute_git_snapshot_swhid(obj);
                }
                "directory"
            } else {
                "directory"
            }
        } else {
            return Err("cannot detect object type".into());
        }
    } else {
        obj_type
    };

    match obj_type {
        "content" => {
            if obj == "-" {
                let mut content = Vec::new();
                std::io::stdin().read_to_end(&mut content)?;
                let swhid = computer.compute_content_swhid(&content)?;
                Ok(swhid.to_string())
            } else {
                let swhid = computer.compute_swhid(obj)?;
                Ok(swhid.to_string())
            }
        }
        "directory" => {
            let swhid = computer.compute_directory_swhid(obj)?;
            Ok(swhid.to_string())
        }
        #[cfg(feature = "git")]
        "revision" => {
            if let Some(rev) = revision {
                git_support::compute_git_revision_swhid(obj, rev)
            } else {
                Err("revision specified but no revision provided".into())
            }
        }
        #[cfg(feature = "git")]
        "release" => {
            if let Some(rel) = release {
                git_support::compute_git_release_swhid(obj, rel)
            } else {
                Err("release specified but no release provided".into())
            }
        }
        #[cfg(feature = "git")]
        "snapshot" => {
            git_support::compute_git_snapshot_swhid(obj)
        }
        _ => Err("invalid object type".into()),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let follow_symlinks = cli.dereference;

    for obj in &cli.objects {
        let result = identify_object(
            &cli.obj_type, 
            follow_symlinks, 
            &cli.exclude, 
            obj,
            #[cfg(feature = "git")]
            cli.revision.as_deref(),
            #[cfg(feature = "git")]
            cli.release.as_deref(),
            #[cfg(feature = "git")]
            cli.snapshot,
        )?;

        if let Some(verify_swhid) = &cli.verify {
            let computer = SwhidComputer::new();
            let is_valid = computer.verify_swhid(obj, verify_swhid)?;
            if is_valid {
                println!("✓ SWHID verification successful");
            } else {
                println!("✗ SWHID verification failed");
                std::process::exit(1);
            }
        } else {
            let display_path = if cli.filename && obj != "-" {
                format!("\t{}", obj)
            } else {
                "".to_string()
            };
            println!("{}{}", result, display_path);
        }
    }

    Ok(())
}
