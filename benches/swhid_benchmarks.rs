use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::Path;
use swhid::{
    ByteRange, Content, DiskDirectoryBuilder, LineRange, QualifiedSwhid, Swhid, WalkOptions,
};
use tempfile::TempDir;

fn bench_content_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("content_creation");

    // Test different data sizes
    let sizes = [16, 1024, 100_000];

    for size in sizes.iter() {
        let data = vec![0u8; *size];
        group.bench_with_input(BenchmarkId::new("from_bytes", size), size, |b, _| {
            b.iter(|| Content::from_bytes(black_box(&data)))
        });
    }

    group.finish();
}

fn bench_content_swhid(c: &mut Criterion) {
    let mut group = c.benchmark_group("content_swhid");

    let sizes = [16, 1024, 100_000];

    for size in sizes.iter() {
        let data = vec![0u8; *size];
        let content = Content::from_bytes(data);
        group.bench_with_input(BenchmarkId::new("swhid", size), size, |b, _| {
            b.iter(|| content.swhid())
        });
    }

    group.finish();
}

fn bench_hash_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_functions");

    let data = vec![0u8; 1024];

    group.bench_function("hash_blob", |b| {
        b.iter(|| swhid::hash::hash_blob(black_box(&data)))
    });

    group.bench_function("hash_object", |b| {
        b.iter(|| swhid::hash::hash_object(black_box("blob"), black_box(&data)))
    });

    group.finish();
}

fn bench_swhid_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("swhid_parsing");

    let swhid_str = "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
    let qualified_str = "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391;origin=https://example.org/repo.git;path=/src/lib.rs;lines=10-20";

    group.bench_function("parse_basic", |b| {
        b.iter(|| swhid_str.parse::<Swhid>().unwrap())
    });

    group.bench_function("parse_qualified", |b| {
        b.iter(|| qualified_str.parse::<QualifiedSwhid>().unwrap())
    });

    group.finish();
}

fn bench_swhid_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("swhid_computation");

    // Test content SWHID computation
    let data = vec![0u8; 1024];
    let content = Content::from_bytes(data);

    group.bench_function("content_swhid", |b| b.iter(|| content.swhid()));

    // Test directory SWHID computation
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, "test content").unwrap();

    let dir = DiskDirectoryBuilder::new(temp_dir.path());
    group.bench_function("directory_swhid", |b| b.iter(|| dir.swhid().unwrap()));

    group.finish();
}

fn bench_directory_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory_processing");

    // Create test directory structure
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("subdir");
    std::fs::create_dir(&subdir).unwrap();

    // Create multiple files
    for i in 0..10 {
        let file_path = temp_dir.path().join(format!("file{}.txt", i));
        std::fs::write(&file_path, format!("content {}", i)).unwrap();
    }

    for i in 0..5 {
        let file_path = subdir.join(format!("subfile{}.txt", i));
        std::fs::write(&file_path, format!("subcontent {}", i)).unwrap();
    }

    let dir = DiskDirectoryBuilder::new(temp_dir.path());
    group.bench_function("multi_file_structure", |b| b.iter(|| dir.swhid().unwrap()));

    // Test with exclude patterns
    let mut opts = WalkOptions::default();
    opts.exclude_suffixes.push(".txt".to_string());
    let dir_with_excludes = DiskDirectoryBuilder::new(temp_dir.path()).with_options(opts);

    group.bench_function("with_exclude_patterns", |b| {
        b.iter(|| dir_with_excludes.swhid().unwrap())
    });

    group.finish();
}

fn bench_symlink_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("symlink_handling");

    let temp_dir = TempDir::new().unwrap();
    let target_file = temp_dir.path().join("target.txt");
    std::fs::write(&target_file, "target content").unwrap();

    let symlink_file = temp_dir.path().join("link.txt");
    std::os::unix::fs::symlink(&target_file, &symlink_file).unwrap();

    // Test default behavior (no follow symlinks)
    let dir_default = DiskDirectoryBuilder::new(temp_dir.path());
    group.bench_function("default_symlinks", |b| {
        b.iter(|| dir_default.swhid().unwrap())
    });

    // Test with follow symlinks
    let mut opts = WalkOptions::default();
    opts.follow_symlinks = true;
    let dir_follow = DiskDirectoryBuilder::new(temp_dir.path()).with_options(opts);

    group.bench_function("follow_symlinks", |b| {
        b.iter(|| dir_follow.swhid().unwrap())
    });

    group.finish();
}

fn bench_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("verification");

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, "test content").unwrap();

    let content = Content::from_bytes(std::fs::read(&test_file).unwrap());
    let expected_swhid = content.swhid();

    group.bench_function("content_verification", |b| {
        b.iter(|| {
            let actual = Content::from_bytes(std::fs::read(&test_file).unwrap()).swhid();
            black_box(actual == expected_swhid)
        })
    });

    let dir = DiskDirectoryBuilder::new(temp_dir.path());
    let expected_dir_swhid = dir.swhid().unwrap();

    group.bench_function("directory_verification", |b| {
        b.iter(|| {
            let actual = DiskDirectoryBuilder::new(temp_dir.path()).swhid().unwrap();
            black_box(actual == expected_dir_swhid)
        })
    });

    group.finish();
}

fn bench_qualified_swhid(c: &mut Criterion) {
    let mut group = c.benchmark_group("qualified_swhid");

    let core: Swhid = "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        .parse()
        .unwrap();

    group.bench_function("create_qualified", |b| {
        b.iter(|| {
            QualifiedSwhid::new(black_box(core.clone()))
                .with_origin("https://example.org/repo.git")
                .with_path("/src/lib.rs")
                .with_lines(LineRange {
                    start: 10,
                    end: Some(20),
                })
                .with_bytes(ByteRange {
                    start: 100,
                    end: Some(200),
                })
        })
    });

    let qualified = QualifiedSwhid::new(core)
        .with_origin("https://example.org/repo.git")
        .with_path("/src/lib.rs")
        .with_lines(LineRange {
            start: 10,
            end: Some(20),
        });

    group.bench_function("to_string", |b| {
        b.iter(|| black_box(&qualified).to_string())
    });

    let qualified_str = qualified.to_string();
    group.bench_function("parse_qualified", |b| {
        b.iter(|| qualified_str.parse::<QualifiedSwhid>().unwrap())
    });

    group.finish();
}

fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    let invalid_swhids = [
        "invalid",
        "swh:2:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391",
        "swh:1:invalid:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391",
        "swh:1:cnt:invalid",
    ];

    for (i, invalid) in invalid_swhids.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("parse_invalid", i), invalid, |b, s| {
            b.iter(|| s.parse::<Swhid>().is_err())
        });
    }

    group.finish();
}

fn bench_large_data(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_data");

    let large_data = vec![0u8; 1_000_000]; // 1MB

    group.bench_function("large_content_creation", |b| {
        b.iter(|| Content::from_bytes(black_box(&large_data)))
    });

    let content = Content::from_bytes(large_data);
    group.bench_function("large_content_swhid", |b| b.iter(|| content.swhid()));

    group.finish();
}

criterion_group!(
    benches,
    bench_content_creation,
    bench_content_swhid,
    bench_hash_functions,
    bench_swhid_parsing,
    bench_swhid_computation,
    bench_directory_processing,
    bench_symlink_handling,
    bench_verification,
    bench_qualified_swhid,
    bench_error_handling,
    bench_large_data
);

criterion_main!(benches);
