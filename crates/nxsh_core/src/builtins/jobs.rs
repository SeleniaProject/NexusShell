//! jobs built-in command implementation
//!
//! The jobs command lists active jobs in the shell.

use crate::context::ShellContext;
use crate::error::ShellResult;
use crate::executor::{Builtin, ExecutionResult};
use crate::job::JobStatus;

pub struct JobsBuiltin;

impl Builtin for JobsBuiltin {
    fn execute(&self, context: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        let job_manager = context.job_manager();
        let job_manager_guard = job_manager.lock().map_err(|_| {
            crate::error::ShellError::new(
                crate::error::ErrorKind::InternalError(
                    crate::error::InternalErrorKind::InvalidState,
                ),
                "Job manager lock poisoned".to_string(),
            )
        })?;

        let jobs = job_manager_guard.get_all_jobs();

        let mut output = String::new();

        // Parse options
        let show_pids = args.contains(&"-p".to_string());
        let show_long = args.contains(&"-l".to_string());

        if jobs.is_empty() {
            // No jobs to display
            return Ok(ExecutionResult::success(0));
        }

        for job in jobs {
            let status_str = match &job.status {
                JobStatus::Running => "Running",
                JobStatus::Background => "Running",
                JobStatus::Foreground => "Running",
                JobStatus::Stopped => "Stopped",
                JobStatus::Waiting => "Waiting",
                JobStatus::Done(code) => {
                    if *code == 0 {
                        "Done"
                    } else {
                        "Exit"
                    }
                }
                JobStatus::Failed(_) => "Failed",
                JobStatus::Terminated(_) => "Terminated",
            };

            if show_pids {
                // Show process IDs
                for process in &job.processes {
                    output.push_str(&format!("{}\n", process.pid));
                }
            } else if show_long {
                // Show detailed information
                for process in &job.processes {
                    output.push_str(&format!(
                        "[{}] {} {} {} {}\n",
                        job.id,
                        process.pid,
                        status_str,
                        if job.foreground { "+" } else { " " },
                        job.description
                    ));
                }
            } else {
                // Standard format
                let foreground_indicator = if job.foreground { "+" } else { " " };
                output.push_str(&format!(
                    "[{}]{} {} {}\n",
                    job.id, foreground_indicator, status_str, job.description
                ));
            }
        }

        Ok(ExecutionResult::success(0).with_output(output.trim().as_bytes().to_vec()))
    }

    fn name(&self) -> &'static str {
        "jobs"
    }

    fn help(&self) -> &'static str {
        "List active jobs"
    }

    fn synopsis(&self) -> &'static str {
        "jobs [-lp]"
    }

    fn description(&self) -> &'static str {
        "Display status of jobs in the current shell session.\n\n\
        Options:\n\
        -l  Display detailed information including process IDs\n\
        -p  Display only process IDs of job leaders"
    }

    fn usage(&self) -> &'static str {
        "jobs [-lp]\n\n\
        List all active jobs with their status.\n\
        Use 'fg %n' to bring job n to foreground.\n\
        Use 'bg %n' to resume job n in background."
    }
}
