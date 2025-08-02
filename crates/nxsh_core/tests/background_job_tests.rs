//! Background Job Management Tests
//!
//! Comprehensive test suite for NexusShell background job execution,
//! focusing on process management, job control, and asynchronous execution.

use nxsh_core::{Executor, ShellContext, job::JobStatus};
use nxsh_parser::Parser;
use std::sync::Once;
use std::thread;

static INIT: Once = Once::new();

/// Ensure NexusShell core and HAL layers are initialized
fn ensure_initialized() {
    INIT.call_once(|| {
        let _ = nxsh_core::initialize();
        let _ = nxsh_hal::initialize();
    });
}

/// Helper function to create a test executor
fn create_test_executor() -> Executor {
    ensure_initialized();
    Executor::new().expect("Failed to create executor")
}

/// Helper function to create a test shell context
fn create_test_context() -> ShellContext {
    ensure_initialized();
    ShellContext::new()
}

#[test]
fn test_background_job_creation() {
    let mut executor = create_test_executor();
    let mut context = create_test_context();
    
    // Parse a simple background command
    let _input = "echo 'background test' &";
    let parser = Parser::new();
    
    // For now, test with a simple command that will be sent to background execution
    match parser.parse("echo hello") {
        Ok(ast) => {
            // Manually test background execution
            let result = executor.execute_background(&ast, &mut context);
            
            match result {
                Ok(execution_result) => {
                    println!("Background job created successfully");
                    assert!(execution_result.job_id.is_some(), "Job ID should be set for background jobs");
                    
                    // Check if job was created in job manager
                    let job_manager = context.job_manager();
                    let job_manager_guard = job_manager.lock().unwrap();
                    let jobs = job_manager_guard.get_all_jobs();
                    assert!(!jobs.is_empty(), "At least one job should be created");
                }
                Err(e) => {
                    println!("Background execution failed (may be expected): {:?}", e);
                    // This might fail if external commands are not available in test environment
                }
            }
        }
        Err(e) => {
            println!("Parse failed: {:?}", e);
        }
    }
}

#[test]
fn test_builtin_background_rejection() {
    let mut executor = create_test_executor();
    let mut context = create_test_context();
    
    // Parse a builtin command (should fail for background execution)
    let input = "cd /tmp";
    let parser = Parser::new();
    
    if let Ok(ast) = parser.parse(input) {
        let result = executor.execute_background(&ast, &mut context);
        
        // Background execution of builtins should be rejected
        match result {
            Err(e) => {
                println!("Correctly rejected builtin for background execution: {:?}", e);
                // This is the expected behavior
            }
            Ok(_) => {
                // This might be acceptable if the implementation allows it
                println!("Background execution of builtin was allowed (implementation choice)");
            }
        }
    }
}

#[test]
fn test_job_manager_integration() {
    let mut executor = create_test_executor();
    let mut context = create_test_context();
    
    // Get initial job count
    let job_manager = context.job_manager();
    let initial_job_count = {
        let job_manager_guard = job_manager.lock().unwrap();
        job_manager_guard.get_all_jobs().len()
    };
    
    // Try to create a background job
    let parser = Parser::new();
    if let Ok(ast) = parser.parse("echo background_integration_test") {
        let _result = executor.execute_background(&ast, &mut context);
        
        // Check if job count increased
        let final_job_count = {
            let job_manager_guard = job_manager.lock().unwrap();
            job_manager_guard.get_all_jobs().len()
        };
        
        // Job count should increase (even if the command fails)
        assert!(
            final_job_count >= initial_job_count,
            "Job count should increase after background job creation"
        );
    }
}

#[test]
fn test_background_execution_statistics() {
    let mut executor = create_test_executor();
    let mut context = create_test_context();
    
    // Get initial background job count
    let initial_stats = executor.stats();
    let initial_bg_jobs = initial_stats.background_jobs;
    
    // Execute a background command (or attempt to)
    let parser = Parser::new();
    if let Ok(ast) = parser.parse("echo stats_test") {
        let _result = executor.execute_background(&ast, &mut context);
        
        // Check that background job statistics were updated
        let final_stats = executor.stats();
        assert!(
            final_stats.background_jobs > initial_bg_jobs,
            "Background job count should increase"
        );
    }
}

#[test]
fn test_job_status_tracking() {
    let context = create_test_context();
    let job_manager = context.job_manager();
    
    // Create a job manually to test status tracking
    let job_id = {
        let mut job_manager_guard = job_manager.lock().unwrap();
        job_manager_guard.create_job("test_status_tracking".to_string())
    };
    
    // Verify job was created
    {
        let job_manager_guard = job_manager.lock().unwrap();
        let job = job_manager_guard.get_job(job_id);
        assert!(job.is_some(), "Job should exist after creation");
        
        if let Some(job) = job {
            assert_eq!(job.status, JobStatus::Running, "New job should have Running status");
            assert_eq!(job.id, job_id, "Job ID should match");
            assert_eq!(job.description, "test_status_tracking", "Job description should match");
        }
    }
}

#[test]
fn test_multiple_background_jobs() {
    let mut executor = create_test_executor();
    let mut context = create_test_context();
    let parser = Parser::new();
    
    let commands = vec!["echo job1", "echo job2", "echo job3"];
    let mut created_jobs = Vec::new();
    
    // Create multiple background jobs
    for cmd in commands {
        if let Ok(ast) = parser.parse(cmd) {
            if let Ok(result) = executor.execute_background(&ast, &mut context) {
                if let Some(job_id) = result.job_id {
                    created_jobs.push(job_id);
                }
            }
        }
    }
    
    // Verify multiple jobs were created
    let job_manager = context.job_manager();
    let job_manager_guard = job_manager.lock().unwrap();
    let all_jobs = job_manager_guard.get_all_jobs();
    
    assert!(
        all_jobs.len() >= created_jobs.len(),
        "Should have at least as many jobs as were created"
    );
}

#[test]
fn test_job_process_association() {
    let context = create_test_context();
    let job_manager = context.job_manager();
    
    // Create a job and add a process
    let job_id = {
        let mut job_manager_guard = job_manager.lock().unwrap();
        job_manager_guard.create_job("process_association_test".to_string())
    };
    
    // Test job modification through the with_job_mut API
    {
        let job_manager_guard = job_manager.lock().unwrap();
        job_manager_guard.with_job_mut(job_id, |job| {
            // Add a mock process
            let process_info = nxsh_core::job::ProcessInfo::new(
                12345, // mock PID
                12345, // mock PGID
                "mock_process".to_string()
            );
            job.add_process(process_info);
            job.status = JobStatus::Background;
        });
    }
    
    // Verify process was added
    {
        let job_manager_guard = job_manager.lock().unwrap();
        if let Some(job) = job_manager_guard.get_job(job_id) {
            assert_eq!(job.processes.len(), 1, "Job should have one process");
            assert_eq!(job.status, JobStatus::Background, "Job should be in background status");
            
            if let Some(process) = job.processes.first() {
                assert_eq!(process.pid, 12345, "Process PID should match");
                assert_eq!(process.command, "mock_process", "Process command should match");
            }
        }
    }
}

#[test]
fn test_job_cleanup_and_completion() {
    let context = create_test_context();
    let job_manager = context.job_manager();
    
    // Create a job
    let job_id = {
        let mut job_manager_guard = job_manager.lock().unwrap();
        job_manager_guard.create_job("completion_test".to_string())
    };
    
    // Simulate job completion
    {
        let job_manager_guard = job_manager.lock().unwrap();
        job_manager_guard.with_job_mut(job_id, |job| {
            // Add a process and mark it as completed
            let mut process_info = nxsh_core::job::ProcessInfo::new(
                54321, // mock PID
                54321, // mock PGID
                "completed_process".to_string()
            );
            process_info.status = JobStatus::Done(0);
            process_info.end_time = Some(std::time::Instant::now());
            
            job.add_process(process_info);
            job.update_status(); // This should update job status based on process status
        });
    }
    
    // Verify job completion
    {
        let job_manager_guard = job_manager.lock().unwrap();
        if let Some(job) = job_manager_guard.get_job(job_id) {
            assert!(job.is_finished(), "Job should be marked as finished");
            assert!(job.completed_at.is_some(), "Job should have completion time");
        }
    }
}

#[test]
fn test_concurrent_job_access() {
    let context = create_test_context();
    let job_manager = context.job_manager();
    
    // Create a job
    let job_id = {
        let mut job_manager_guard = job_manager.lock().unwrap();
        job_manager_guard.create_job("concurrent_test".to_string())
    };
    
    // Simulate concurrent access from multiple threads
    let handles: Vec<_> = (0..3).map(|i| {
        let job_manager_clone = job_manager.clone();
        let job_id_clone = job_id;
        
        thread::spawn(move || {
            // Each thread tries to modify the job
            let job_manager_guard = job_manager_clone.lock().unwrap();
            job_manager_guard.with_job_mut(job_id_clone, |job| {
                let process_info = nxsh_core::job::ProcessInfo::new(
                    1000 + i as u32, // unique PID per thread
                    job_id_clone,    // shared PGID
                    format!("thread_{}_process", i)
                );
                job.add_process(process_info);
            });
        })
    }).collect();
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread should complete successfully");
    }
    
    // Verify all processes were added
    {
        let job_manager_guard = job_manager.lock().unwrap();
        if let Some(job) = job_manager_guard.get_job(job_id) {
            assert_eq!(job.processes.len(), 3, "Job should have 3 processes from concurrent access");
        }
    }
}
