//! fg built-in command implementation
//!
//! The fg command brings a background job to the foreground.

use crate::context::ShellContext;
use crate::error::ShellResult;
use crate::executor::{Builtin, ExecutionResult};

pub struct FgBuiltin;

impl Builtin for FgBuiltin {
    fn execute(&self, context: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        let job_manager = context.job_manager();
        let mut job_manager_guard = job_manager.lock().map_err(|_| {
            crate::error::ShellError::new(
                crate::error::ErrorKind::InternalError(
                    crate::error::InternalErrorKind::InvalidState,
                ),
                "Job manager lock poisoned".to_string(),
            )
        })?;

        // Parse job specification
        let job_id = if args.is_empty() {
            // Use most recent job
            let jobs = job_manager_guard.get_all_jobs();
            if jobs.is_empty() {
                return Err(crate::error::ShellError::new(
                    crate::error::ErrorKind::RuntimeError(
                        crate::error::RuntimeErrorKind::InvalidArgument,
                    ),
                    "fg: no current job".to_string(),
                ));
            }
            jobs.last()
                .ok_or_else(|| {
                    crate::error::ShellError::new(
                        crate::error::ErrorKind::RuntimeError(
                            crate::error::RuntimeErrorKind::InvalidArgument,
                        ),
                        "fg: no current job available".to_string(),
                    )
                })?
                .id
        } else {
            let job_spec = &args[0];
            if let Some(job_num_str) = job_spec.strip_prefix('%') {
                // Parse job number
                job_num_str.parse::<u32>().map_err(|_| {
                    crate::error::ShellError::new(
                        crate::error::ErrorKind::RuntimeError(
                            crate::error::RuntimeErrorKind::InvalidArgument,
                        ),
                        format!("fg: invalid job specification: {job_spec}"),
                    )
                })?
            } else {
                // Assume it's a job number without %
                job_spec.parse::<u32>().map_err(|_| {
                    crate::error::ShellError::new(
                        crate::error::ErrorKind::RuntimeError(
                            crate::error::RuntimeErrorKind::InvalidArgument,
                        ),
                        format!("fg: invalid job specification: {job_spec}"),
                    )
                })?
            }
        };

        // Check if job exists
        if job_manager_guard.get_job(job_id)?.is_none() {
            return Err(crate::error::ShellError::new(
                crate::error::ErrorKind::RuntimeError(
                    crate::error::RuntimeErrorKind::InvalidArgument,
                ),
                format!("fg: job {job_id} not found"),
            ));
        }

        // Move job to foreground
        job_manager_guard.move_job_to_foreground(job_id)?;

        // Get job description for output
        let job = job_manager_guard.get_job(job_id)?.ok_or_else(|| {
            crate::error::ShellError::new(
                crate::error::ErrorKind::RuntimeError(
                    crate::error::RuntimeErrorKind::InvalidArgument,
                ),
                format!("fg: job {job_id} not found after move"),
            )
        })?;
        let output = job.description.to_string();

        // Wait for job completion
        drop(job_manager_guard); // Release lock before waiting
        let job_manager_for_wait = context.job_manager();
        let job_manager_wait_guard = job_manager_for_wait.lock().map_err(|_| {
            crate::error::ShellError::new(
                crate::error::ErrorKind::InternalError(
                    crate::error::InternalErrorKind::InvalidState,
                ),
                "Job manager lock poisoned".to_string(),
            )
        })?;

        let final_status = job_manager_wait_guard.wait_for_job(job_id)?;

        // Return with appropriate exit code
        let exit_code = match final_status {
            crate::job::JobStatus::Done(code) => code,
            crate::job::JobStatus::Terminated(_) => 128 + 15, // 128 + SIGTERM
            _ => 0,
        };

        Ok(ExecutionResult::success(exit_code).with_output(output.as_bytes().to_vec()))
    }

    fn name(&self) -> &'static str {
        "fg"
    }

    fn help(&self) -> &'static str {
        "Bring job to foreground"
    }

    fn synopsis(&self) -> &'static str {
        "fg [job_spec]"
    }

    fn description(&self) -> &'static str {
        "Bring the specified job to the foreground and make it the current job.\n\
        If no job_spec is given, use the most recent job."
    }

    fn usage(&self) -> &'static str {
        "fg [%n]\n\n\
        Bring job n to the foreground. If no job number is specified,\n\
        bring the most recent job to the foreground.\n\n\
        Examples:\n\
        fg       # Bring most recent job to foreground\n\
        fg %1    # Bring job 1 to foreground\n\
        fg 2     # Bring job 2 to foreground"
    }
}
