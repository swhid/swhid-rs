use criterion::{criterion_group, criterion_main, Criterion};
use swhid::{
    Content, SwhidComputer, Swhid,
    hash::{sha1_git_hash, sha1_hash, hash_git_object}
};
use std::{fs, hint::black_box};
use tempfile::TempDir;

fn bench_content_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("content_creation");
    
    // Small content
    group.bench_function("small_content_16b", |b| {
        let data = b"Hello, World! 16";
        b.iter(|| Content::from_data(black_box(data.to_vec())))
    });
    
    // Medium content
    group.bench_function("medium_content_1kb", |b| {
        let data = vec![b'a'; 1024];
        b.iter(|| Content::from_data(black_box(data.clone())))
    });
    
    // Large content
    group.bench_function("large_content_100kb", |b| {
        let data = vec![b'b'; 100 * 1024];
        b.iter(|| Content::from_data(black_box(data.clone())))
    });
    
    group.finish();
}

fn bench_hash_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_functions");
    
    let test_data = vec![b'x'; 1024];
    
    group.bench_function("sha1_git_hash_1kb", |b| {
        b.iter(|| sha1_git_hash(black_box(&test_data)))
    });
    
    group.bench_function("sha1_hash_1kb", |b| {
        b.iter(|| sha1_hash(black_box(&test_data)))
    });
    
    group.bench_function("hash_git_object_blob_1kb", |b| {
        b.iter(|| hash_git_object(black_box("blob"), black_box(&test_data)))
    });
    
    group.bench_function("hash_git_object_tree_1kb", |b| {
        b.iter(|| hash_git_object(black_box("tree"), black_box(&test_data)))
    });
    
    group.finish();
}

fn bench_swhid_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("swhid_parsing");
    
    let valid_swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684";
    let qualified_swhid = "swh:1:cnt:b45ef6fec89518d314f546fd6c3025367b721684;origin=https://github.com/user/repo;path=src/main.rs;lines=1-10";
    
    group.bench_function("parse_basic_swhid", |b| {
        b.iter(|| Swhid::from_string(black_box(valid_swhid)))
    });
    
    group.bench_function("parse_qualified_swhid", |b| {
        b.iter(|| swhid::QualifiedSwhid::from_string(black_box(qualified_swhid)))
    });
    
    group.finish();
}

fn bench_swhid_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("swhid_computation");
    
    // Create temporary test data
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("benchmark.txt");
    let content = vec![b'z'; 1024];
    fs::write(&file_path, &content).unwrap();
    
    let computer = SwhidComputer::new();
    
    group.bench_function("compute_content_swhid_1kb", |b| {
        b.iter(|| computer.compute_content_swhid(black_box(&content)))
    });
    
    group.bench_function("compute_file_swhid_1kb", |b| {
        b.iter(|| computer.compute_file_swhid(black_box(&file_path)))
    });
    
    group.finish();
}

fn bench_directory_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory_processing");
    
    // Create a test directory structure
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_structure");
    fs::create_dir(&test_dir).unwrap();
    
    // Create some files
    for i in 0..10 {
        let file_path = test_dir.join(format!("file_{}.txt", i));
        fs::write(&file_path, format!("Content for file {}", i)).unwrap();
    }
    
    // Create a subdirectory
    let subdir = test_dir.join("subdir");
    fs::create_dir(&subdir).unwrap();
    for i in 0..5 {
        let file_path = subdir.join(format!("subfile_{}.txt", i));
        fs::write(&file_path, format!("Sub content {}", i)).unwrap();
    }
    
    let computer = SwhidComputer::new();
    
    group.bench_function("process_directory_15_files", |b| {
        b.iter(|| computer.compute_directory_swhid(black_box(&test_dir)))
    });
    
    group.finish();
}

fn bench_symlink_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("symlink_handling");
    
    let temp_dir = TempDir::new().unwrap();
    let target_file = temp_dir.path().join("target.txt");
    fs::write(&target_file, "target content").unwrap();
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let symlink_path = temp_dir.path().join("symlink.txt");
        symlink("target.txt", &symlink_path).unwrap();
        
        let computer = SwhidComputer::new();
        
        group.bench_function("compute_symlink_default", |b| {
            b.iter(|| computer.compute_swhid(black_box(&symlink_path)))
        });
        
        let computer_deref = SwhidComputer::new().with_follow_symlinks(true);
        group.bench_function("compute_symlink_dereference", |b| {
            b.iter(|| computer_deref.compute_swhid(black_box(&symlink_path)))
        });
    }
    
    group.finish();
}

fn bench_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("verification");
    
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("verify.txt");
    let content = "test content for verification";
    fs::write(&file_path, content).unwrap();
    
    let computer = SwhidComputer::new();
    let swhid = computer.compute_file_swhid(&file_path).unwrap();
    let swhid_str = swhid.to_string();
    
    group.bench_function("verify_swhid_match", |b| {
        b.iter(|| computer.verify_swhid(black_box(&file_path), black_box(&swhid_str)))
    });
    
    let wrong_swhid = "swh:1:cnt:0000000000000000000000000000000000000000";
    group.bench_function("verify_swhid_mismatch", |b| {
        b.iter(|| computer.verify_swhid(black_box(&file_path), black_box(wrong_swhid)))
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_content_creation,
    bench_hash_functions,
    bench_swhid_parsing,
    bench_swhid_computation,
    bench_directory_processing,
    bench_symlink_handling,
    bench_verification
);
criterion_main!(benches);
