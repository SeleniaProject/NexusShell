use crate::executor::{Builtin, ExecutionResult};
use crate::context::ShellContext;
use crate::error::ShellResult;

pub struct ArgDumpBuiltin;

impl Builtin for ArgDumpBuiltin {
    fn execute(&self, _context: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        // Format: first line count, then each arg on its own line (verbatim)
        let mut out = format!("count={}\n", args.len());
        for a in args {
            out.push_str(a);
            out.push('\n');
        }
        Ok(ExecutionResult { exit_code: 0, stdout: out, stderr: String::new(), execution_time: 0, strategy: crate::executor::ExecutionStrategy::DirectInterpreter, metrics: Default::default() })
    }
    fn name(&self) -> &'static str { "__argdump" }
    fn help(&self) -> &'static str { "Test helper: dumps argument count and values" }
    fn synopsis(&self) -> &'static str { "__argdump [args...]" }
    fn description(&self) -> &'static str { "Internal test builtin for verifying argument splitting behavior." }
    fn usage(&self) -> &'static str { "__argdump" }
    fn affects_shell_state(&self) -> bool { false }
}
