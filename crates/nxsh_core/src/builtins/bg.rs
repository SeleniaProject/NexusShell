//! bg built-in command implementation
//!
//! The bg command resumes a stopped job in the background.

use crate::executor::{Builtin, ExecutionResult};
use crate::context::ShellContext;
use crate::error::ShellResult;

pub struct BgBuiltin;

impl Builtin for BgBuiltin {
    fn execute(&self, context: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        let job_manager = context.job_manager();
        let mut job_manager_guard = job_manager.lock()
            .map_err(|_| crate::error::ShellError::new(
                crate::error::ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                "Job manager lock poisoned".to_string()
            ))?;

        // Parse job specification
        let job_id = if args.is_empty() {
            // Use most recent stopped job
            let stopped_jobs = job_manager_guard.get_stopped_jobs();
            if stopped_jobs.is_empty() {
                return Err(crate::error::ShellError::new(
                    crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                    "bg: no stopped job".to_string()
                ));
            }
            stopped_jobs.last()
                .ok_or_else(|| crate::error::ShellError::new(
                    crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                    "bg: no stopped job available".to_string()
                ))?
                .id
        } else {
            let job_spec = &args[0];
            if let Some(job_num_str) = job_spec.strip_prefix('%') {
                // Parse job number
                job_num_str.parse::<u32>()
                    .map_err(|_| crate::error::ShellError::new(
                        crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                        format!("bg: invalid job specification: {job_spec}")
                    ))?
            } else {
                // Assume it's a job number without %
                job_spec.parse::<u32>()
                    .map_err(|_| crate::error::ShellError::new(
                        crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                        format!("bg: invalid job specification: {job_spec}")
                    ))?
            }
        };

        // Check if job exists
        let job = job_manager_guard.get_job(job_id)?
            .ok_or_else(|| crate::error::ShellError::new(
                crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                format!("bg: job {job_id} not found")
            ))?;

        // Check if job is stopped
        if !job.is_stopped() {
            return Err(crate::error::ShellError::new(
                crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                format!("bg: job {job_id} is not stopped")
            ));
        }

        // Move job to background
        job_manager_guard.move_job_to_background(job_id)?;
        
        // Get job description for output
        let output = format!("[{}] {}", job_id, job.description);
        
        Ok(ExecutionResult::success(0).with_output(output.as_bytes().to_vec()))
    }

    fn name(&self) -> &'static str {
        "bg"
    }

    fn help(&self) -> &'static str {
        "Resume job in background"
    }

    fn synopsis(&self) -> &'static str {
        "bg [job_spec]"
    }

    fn description(&self) -> &'static str {
        "Resume the specified stopped job in the background.\n\
        If no job_spec is given, use the most recent stopped job."
    }

    fn usage(&self) -> &'static str {
        "bg [%n]\n\n\
        Resume job n in the background. If no job number is specified,\n\
        resume the most recent stopped job in the background.\n\n\
        Examples:\n\
        bg       # Resume most recent stopped job in background\n\
        bg %1    # Resume job 1 in background\n\
        bg 2     # Resume job 2 in background"
    }
}
