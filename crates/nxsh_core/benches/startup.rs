use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{process::Command, time::Instant};

fn benchmark_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup");
    
    group.bench_function("cold_start", |b| {
        b.iter(|| {
            let start = Instant::now();
            let output = Command::new("../../target/release/nxsh.exe")
                .args(&["-c", "exit 0"])
                .output()
                .expect("Failed to execute nxsh");
            
            assert!(output.status.success());
            black_box(start.elapsed())
        })
    });
    
    group.bench_function("warm_start", |b| {
        // Pre-warm by running once
        let _ = Command::new("../../target/release/nxsh.exe")
            .args(&["-c", "exit 0"])
            .output();
        
        b.iter(|| {
            let start = Instant::now();
            let output = Command::new("../../target/release/nxsh.exe")
                .args(&["-c", "exit 0"])
                .output()
                .expect("Failed to execute nxsh");
            
            assert!(output.status.success());
            black_box(start.elapsed())
        })
    });
    
    group.bench_function("interactive_prompt_time", |b| {
        use std::process::{Stdio};
        use std::io::Write;
        
        b.iter(|| {
            let start = Instant::now();
            let mut child = Command::new("../../target/release/nxsh.exe")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped()) 
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to spawn nxsh");
            
            // Send exit command
            if let Some(ref mut stdin) = child.stdin.take() {
                writeln!(stdin, "exit").unwrap();
            }
            
            let _ = child.wait();
            black_box(start.elapsed())
        })
    });

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");
    
    group.bench_function("resident_memory", |b| {
        b.iter(|| {
            use sysinfo::{System, Pid};
            
            let mut child = Command::new("../../target/release/nxsh.exe")
                .args(&["-c", "sleep 0.1"])
                .spawn()
                .expect("Failed to spawn nxsh");
            
            let pid = child.id();
            let mut sys = System::new_all();
            sys.refresh_all();
            
            let memory_usage = if let Some(process) = sys.process(Pid::from_u32(pid)) {
                process.memory() * 1024 // Convert to bytes
            } else {
                0
            };
            
            let _ = child.wait();
            black_box(memory_usage)
        })
    });

    group.finish();
}

fn benchmark_command_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("command_execution");
    
    // Test basic commands
    let commands = [
        ("echo", vec!["-c", "echo hello"]),
        ("pwd", vec!["-c", "pwd"]),
        ("ls", vec!["-c", "ls"]),
        ("env", vec!["-c", "env | head -5"]),
    ];
    
    for (name, args) in &commands {
        group.bench_function(*name, |b| {
            b.iter(|| {
                let start = Instant::now();
                let output = Command::new("../../target/release/nxsh.exe")
                    .args(args)
                    .output()
                    .expect("Failed to execute nxsh");
                
                assert!(output.status.success());
                black_box(start.elapsed())
            })
        });
    }
    
    group.finish();
}

fn benchmark_comparison_with_bash(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison");
    
    let test_commands = [
        ("echo_hello", "echo hello"),
        ("pwd", "pwd"),
        ("ls_simple", "ls"),
    ];
    
    for (name, cmd) in &test_commands {
        group.bench_function(&format!("nxsh_{}", name), |b| {
            b.iter(|| {
                let start = Instant::now();
                let output = Command::new("../../target/release/nxsh.exe")
                    .args(&["-c", cmd])
                    .output()
                    .expect("Failed to execute nxsh");
                
                assert!(output.status.success());
                black_box(start.elapsed())
            })
        });
        
        group.bench_function(&format!("bash_{}", name), |b| {
            b.iter(|| {
                let start = Instant::now();
                let output = Command::new("bash")
                    .args(&["-c", cmd])
                    .output()
                    .expect("Failed to execute bash");
                
                assert!(output.status.success());
                black_box(start.elapsed())
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches, 
    benchmark_startup,
    benchmark_memory_usage,
    benchmark_command_execution,
    benchmark_comparison_with_bash
);
criterion_main!(benches);
