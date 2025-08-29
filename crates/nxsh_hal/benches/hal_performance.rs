use criterion::{criterion_group, criterion_main, Criterion};
use nxsh_hal::{
    completion::{CompletionContext, CompletionEngine, CompletionType},
    fast_completion::FastCompletionEngine,
    fs_enhanced::{DiskUsageAnalyzer, FileSystemMonitor},
    process_enhanced::{CommandExecutor, ProcessMonitor},
    time_enhanced::{PerformanceMonitor, TimeManager},
};
use std::time::Duration;
use tempfile::tempdir;

fn bench_completion_engine(c: &mut Criterion) {
    let mut group = c.benchmark_group("completion_engine");

    let original_engine = CompletionEngine::new();
    let fast_engine = FastCompletionEngine::new();

    let context = CompletionContext {
        completion_type: CompletionType::Command,
        working_dir: std::env::current_dir().unwrap(),
        command_line: "l".to_string(),
        cursor_position: 1,
    };

    // Benchmark original engine
    group.bench_function("original_completion_cached", |b| {
        // Pre-warm cache
        let _ = original_engine.get_completions("l", &context);

        b.iter(|| {
            let completions = original_engine.get_completions("l", &context).unwrap();
            assert!(!completions.is_empty());
        });
    });

    group.bench_function("original_completion_cold", |b| {
        b.iter(|| {
            original_engine.clear_cache();
            let completions = original_engine.get_completions("l", &context).unwrap();
            assert!(!completions.is_empty());
        });
    });

    // Benchmark FAST engine (target: <1ms)
    group.bench_function("fast_completion", |b| {
        b.iter(|| {
            let completions = fast_engine.get_completions_fast("l").unwrap();
            assert!(!completions.is_empty());
        });
    });

    // Test various inputs
    for input in &["ls", "git", "c", "echo"] {
        group.bench_function(format!("fast_completion_{input}"), |b| {
            b.iter(|| {
                let completions = fast_engine.get_completions_fast(input).unwrap();
                assert!(!completions.is_empty());
            });
        });
    }

    // Benchmark file completion using original engine
    let file_context = CompletionContext {
        completion_type: CompletionType::File,
        working_dir: std::env::current_dir().unwrap(),
        command_line: "".to_string(),
        cursor_position: 0,
    };

    group.bench_function("original_file_completion", |b| {
        b.iter(|| {
            let _completions = original_engine.get_completions("", &file_context).unwrap();
        });
    });

    // Test completion performance target (<1ms)
    group.bench_function("fast_performance_target", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();
            let _completions = fast_engine.get_completions_fast("ls").unwrap();
            let duration = start.elapsed();
            assert!(
                duration < Duration::from_millis(1),
                "Fast completion took {duration:?}, should be < 1ms"
            );
        });
    });

    group.finish();
}

fn bench_time_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_management");

    let time_manager = TimeManager::new();

    group.bench_function("time_measurement", |b| {
        b.iter(|| {
            let (result, duration) = time_manager.measure(|| {
                std::thread::sleep(Duration::from_micros(100));
                42
            });
            assert_eq!(result, 42);
            assert!(duration >= Duration::from_micros(100));
        });
    });

    let monitor = PerformanceMonitor::new();
    group.bench_function("counter_operations", |b| {
        b.iter(|| {
            monitor.increment_counter("test_counter");
            monitor.add_to_counter("test_counter", 5);
            let value = monitor.get_counter("test_counter");
            assert!(value > 0);
        });
    });

    group.bench_function("timing_records", |b| {
        b.iter(|| {
            monitor.record_timing("test_operation", Duration::from_millis(10));
            let timing = monitor.get_timing("test_operation");
            assert!(timing > Duration::ZERO);
        });
    });

    group.finish();
}

fn bench_filesystem_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("filesystem_operations");

    let monitor = FileSystemMonitor::new();
    let temp_dir = tempdir().unwrap();

    group.bench_function("directory_analysis", |b| {
        // Create some test files
        std::fs::write(temp_dir.path().join("test1.txt"), "content1").unwrap();
        std::fs::write(temp_dir.path().join("test2.rs"), "fn main() {}").unwrap();
        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        b.iter(|| {
            let analysis = monitor.analyze_directory(temp_dir.path()).unwrap();
            assert!(analysis.files >= 2);
            assert!(analysis.directories >= 1);
        });
    });

    group.bench_function("disk_usage_analysis", |b| {
        b.iter(|| {
            let usage = DiskUsageAnalyzer::analyze(temp_dir.path()).unwrap();
            assert!(usage.total_size > 0);
        });
    });

    // Test filesystem operation recording
    group.bench_function("operation_recording", |b| {
        use nxsh_hal::fs_enhanced::FileOperation;

        b.iter(|| {
            monitor.record_operation(FileOperation::Read, Duration::from_micros(500), 1024);
            let stats = monitor.stats();
            assert!(stats.reads > 0);
        });
    });

    group.finish();
}

fn bench_process_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_operations");

    let monitor = ProcessMonitor::new();
    let _executor = CommandExecutor::new();

    group.bench_function("process_monitoring", |b| {
        use nxsh_hal::process_enhanced::ProcessInfo;

        b.iter(|| {
            let info = ProcessInfo::new(12345, "test".to_string(), "test command".to_string());
            monitor.register_process(12345, info);
            let retrieved = monitor.get_process(12345);
            assert!(retrieved.is_some());
            monitor.unregister_process(12345);
        });
    });

    // Only run command execution test on Unix systems for consistency
    #[cfg(unix)]
    group.bench_function("command_execution", |b| {
        b.iter(|| {
            let result = executor.execute("echo", &["benchmark"]).unwrap();
            assert!(result.success());
            assert_eq!(result.stdout.trim(), "benchmark");
        });
    });

    group.bench_function("statistics_recording", |b| {
        b.iter(|| {
            monitor.record_execution(Duration::from_millis(100), 0);
            let stats = monitor.stats();
            assert!(stats.executions > 0);
        });
    });

    group.finish();
}

fn bench_hal_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("hal_integration");

    // Test full HAL initialization and cleanup
    group.bench_function("hal_initialization", |b| {
        b.iter(|| {
            let result = nxsh_hal::initialize();
            assert!(result.is_ok());

            let cleanup_result = nxsh_hal::shutdown();
            assert!(cleanup_result.is_ok());
        });
    });

    // Benchmark combined operations
    group.bench_function("combined_hal_operations", |b| {
        let completion_engine = CompletionEngine::new();
        let time_manager = TimeManager::new();
        let fs_monitor = FileSystemMonitor::new();

        let context = CompletionContext {
            completion_type: CompletionType::Command,
            working_dir: std::env::current_dir().unwrap(),
            command_line: "l".to_string(),
            cursor_position: 1,
        };

        b.iter(|| {
            // Time a completion operation
            let (completions, duration) =
                time_manager.measure(|| completion_engine.get_completions("l", &context).unwrap());

            // Record filesystem stats
            fs_monitor.record_operation(
                nxsh_hal::fs_enhanced::FileOperation::Read,
                duration,
                completions.len() as u64,
            );

            assert!(!completions.is_empty());
        });
    });

    // Test performance targets
    group.bench_function("performance_targets", |b| {
        let completion_engine = CompletionEngine::new();
        let context = CompletionContext {
            completion_type: CompletionType::Command,
            working_dir: std::env::current_dir().unwrap(),
            command_line: "ls".to_string(),
            cursor_position: 2,
        };

        b.iter(|| {
            let start = std::time::Instant::now();
            let completions = completion_engine.get_completions("ls", &context).unwrap();
            let duration = start.elapsed();

            // Assert SPEC.md performance target: completion < 1ms
            assert!(
                duration < Duration::from_millis(1),
                "Completion performance target failed: {duration:?} >= 1ms"
            );

            assert!(!completions.is_empty());
        });
    });

    group.finish();
}

criterion_group!(
    hal_benches,
    bench_completion_engine,
    bench_time_management,
    bench_filesystem_operations,
    bench_process_operations,
    bench_hal_integration
);

criterion_main!(hal_benches);
