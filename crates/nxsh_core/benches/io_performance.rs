use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use nxsh_core::io_optimization::IoManager;
use std::{fs, path::PathBuf};
use tempfile::tempdir;

fn bench_file_operations(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("benchmark_file.txt");
    
    // Create test content of various sizes
    let small_content = "x".repeat(1024);      // 1KB
    let medium_content = "x".repeat(10240);    // 10KB
    
    let mut group = c.benchmark_group("file_operations");
    
    // Benchmark standard file operations
    group.bench_with_input(
        BenchmarkId::new("std_write_small", 1024),
        &small_content,
        |b, content| {
            b.iter(|| {
                fs::write(&file_path, content).unwrap();
            });
        },
    );
    
    group.bench_with_input(
        BenchmarkId::new("std_read_small", 1024),
        &small_content,
        |b, content| {
            fs::write(&file_path, content).unwrap();
            b.iter(|| {
                let _: String = fs::read_to_string(&file_path).unwrap();
            });
        },
    );
    
    // Benchmark optimized I/O operations
    let io_manager = IoManager::new(8192);
    
    group.bench_with_input(
        BenchmarkId::new("optimized_write_small", 1024),
        &small_content,
        |b, content| {
            b.iter(|| {
                io_manager.write_file_buffered(&file_path, content).unwrap();
            });
        },
    );
    
    group.bench_with_input(
        BenchmarkId::new("optimized_read_small", 1024),
        &small_content,
        |b, content| {
            io_manager.write_file_buffered(&file_path, content).unwrap();
            b.iter(|| {
                let _: String = io_manager.read_file_buffered(&file_path).unwrap();
            });
        },
    );
    
    // Medium size benchmarks
    group.bench_with_input(
        BenchmarkId::new("std_write_medium", 10240),
        &medium_content,
        |b, content| {
            b.iter(|| {
                fs::write(&file_path, content).unwrap();
            });
        },
    );
    
    group.bench_with_input(
        BenchmarkId::new("optimized_write_medium", 10240),
        &medium_content,
        |b, content| {
            b.iter(|| {
                io_manager.write_file_buffered(&file_path, content).unwrap();
            });
        },
    );
    
    group.finish();
}

fn bench_multiple_file_operations(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let content = "test content for multiple file operations";
    
    let mut group = c.benchmark_group("multiple_file_operations");
    
    // Create multiple files for testing
    let file_count = 10;
    let file_paths: Vec<PathBuf> = (0..file_count)
        .map(|i| dir.path().join(format!("test_{}.txt", i)))
        .collect();
    
    // Benchmark sequential file operations
    group.bench_function("sequential_write", |b| {
        b.iter(|| {
            for path in &file_paths {
                fs::write(path, content).unwrap();
            }
        });
    });
    
    group.bench_function("sequential_read", |b| {
        // Pre-write files
        for path in &file_paths {
            fs::write(path, content).unwrap();
        }
        
        b.iter(|| {
            for path in &file_paths {
                let _: String = fs::read_to_string(path).unwrap();
            }
        });
    });
    
    // Benchmark optimized file operations
    let io_manager = IoManager::new(4096);
    
    group.bench_function("optimized_sequential_write", |b| {
        b.iter(|| {
            for path in &file_paths {
                io_manager.write_file_buffered(path, content).unwrap();
            }
        });
    });
    
    group.bench_function("optimized_sequential_read", |b| {
        // Pre-write files
        for path in &file_paths {
            io_manager.write_file_buffered(path, content).unwrap();
        }
        
        b.iter(|| {
            for path in &file_paths {
                let _: String = io_manager.read_file_buffered(path).unwrap();
            }
        });
    });
    
    group.finish();
}

fn bench_line_reading(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("lines_test.txt");
    
    // Create file with many lines
    let line_count = 1000;
    let content: String = (0..line_count)
        .map(|i| format!("This is line number {}\n", i))
        .collect();
    
    fs::write(&file_path, &content).unwrap();
    
    let mut group = c.benchmark_group("line_reading");
    
    group.bench_function("std_read_lines", |b| {
        b.iter(|| {
            use std::io::{BufRead, BufReader};
            let file = std::fs::File::open(&file_path).unwrap();
            let reader = BufReader::new(file);
            let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(lines.len(), line_count);
        });
    });
    
    let io_manager = IoManager::new(8192);
    group.bench_function("optimized_read_lines", |b| {
        b.iter(|| {
            let lines = io_manager.read_file_lines(&file_path).unwrap();
            assert_eq!(lines.len(), line_count);
        });
    });
    
    group.finish();
}

fn bench_buffer_sizes(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("buffer_test.txt");
    
    let content = "x".repeat(50000); // 50KB file
    
    let mut group = c.benchmark_group("buffer_sizes");
    
    let buffer_sizes = [1024, 4096, 8192, 16384, 32768];
    
    for &buffer_size in &buffer_sizes {
        let io_manager = IoManager::new(buffer_size);
        
        group.bench_with_input(
            BenchmarkId::new("write", buffer_size),
            &buffer_size,
            |b, _| {
                b.iter(|| {
                    io_manager.write_file_buffered(&file_path, &content).unwrap();
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("read", buffer_size),
            &buffer_size,
            |b, _| {
                io_manager.write_file_buffered(&file_path, &content).unwrap();
                b.iter(|| {
                    let _: String = io_manager.read_file_buffered(&file_path).unwrap();
                });
            },
        );
    }
    
    group.finish();
}

#[cfg(feature = "async")]
fn bench_async_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let dir = tempdir().unwrap();
    let content = "test content for async operations";
    
    let file_count = 20;
    let file_paths: Vec<PathBuf> = (0..file_count)
        .map(|i| dir.path().join(format!("async_test_{}.txt", i)))
        .collect();
    
    let mut group = c.benchmark_group("async_operations");
    
    // Benchmark async multiple file operations
    let async_io = AsyncIoManager::new(8192, 5); // 5 concurrent operations
    
    group.bench_function("async_concurrent_write", |b| {
        b.to_async(&rt).iter(|| async {
            let data: Vec<_> = file_paths.iter()
                .map(|path| (path.as_path(), content))
                .collect();
            async_io.write_multiple_files(data).await.unwrap();
        });
    });
    
    group.bench_function("async_concurrent_read", |b| {
        // Pre-write files
        rt.block_on(async {
            let data: Vec<_> = file_paths.iter()
                .map(|path| (path.as_path(), content))
                .collect();
            async_io.write_multiple_files(data).await.unwrap();
        });
        
        b.to_async(&rt).iter(|| async {
            let paths: Vec<_> = file_paths.iter().map(|p| p.as_path()).collect();
            let _results = async_io.read_multiple_files(paths).await.unwrap();
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_file_operations,
    bench_multiple_file_operations,
    bench_line_reading,
    bench_buffer_sizes
);

criterion_main!(benches);
