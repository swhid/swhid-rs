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

    /// Follow symlinks (default: follow)
    #[arg(long, default_value = "true")]
    dereference: bool,

    /// Don't follow symlinks
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

    /// Objects to identify
    objects: Vec<String>,
}

fn identify_object(
    obj_type: &str,
    follow_symlinks: bool,
    exclude_patterns: &[String],
    obj: &str,
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
            "directory"
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
                let swhid = computer.compute_file_swhid(obj)?;
                Ok(swhid.to_string())
            }
        }
        "directory" => {
            let swhid = computer.compute_directory_swhid(obj)?;
            Ok(swhid.to_string())
        }
        _ => Err("invalid object type".into()),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let follow_symlinks = !cli.no_dereference;

    for obj in &cli.objects {
        let result = identify_object(&cli.obj_type, follow_symlinks, &cli.exclude, obj)?;

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
