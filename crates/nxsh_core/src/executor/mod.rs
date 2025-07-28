//! Command execution engine for NexusShell

use crate::error::{ShellError, ErrorKind, ShellResult};
use crate::context::{ShellContext, Context};
use crate::stream::{StreamType, Stream};
use crate::job::JobId;
use nxsh_hal::{FileSystem, ProcessManager};
use nxsh_parser::ast::AstNode;
use std::{
    collections::HashMap,
    fs::OpenOptions,
    path::{PathBuf},
    process::{Command as StdCommand},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Execution modes for different contexts
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    /// Interactive shell mode
    Interactive,
    /// Script execution mode
    Script,
    /// Command line execution mode
    CommandLine,
    /// Background job execution
    Background,
    /// Subshell execution
    Subshell,
}

/// Execution result with metadata
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Exit status code
    pub exit_code: i32,
    /// Standard output (if captured)
    pub output: Option<Vec<u8>>,
    /// Standard error (if captured)
    pub error: Option<Vec<u8>>,
    /// Execution duration
    pub duration: Duration,
    /// Process ID (if applicable)
    pub pid: Option<u32>,
    /// Job ID (if background)
    pub job_id: Option<JobId>,
}

impl ExecutionResult {
    pub fn success(exit_code: i32) -> Self {
        Self {
            exit_code,
            output: None,
            error: None,
            duration: Duration::from_millis(0),
            pid: None,
            job_id: None,
        }
    }

    pub fn with_output(mut self, output: Vec<u8>) -> Self {
        self.output = Some(output);
        self
    }

    pub fn with_error(mut self, error: Vec<u8>) -> Self {
        self.error = Some(error);
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Built-in command trait
pub trait Builtin: Send + Sync {
    /// Command name
    fn name(&self) -> &'static str;
    
    /// Short description for help
    fn synopsis(&self) -> &'static str;
    
    /// Long description for help
    fn description(&self) -> &'static str {
        self.synopsis()
    }
    
    /// Execute the built-in command
    fn invoke(&self, ctx: &mut Context) -> ShellResult<ExecutionResult>;
    
    /// Check if command should run in the current shell context
    fn affects_shell_state(&self) -> bool {
        false
    }
    
    /// Get command usage information
    fn usage(&self) -> &'static str {
        ""
    }
}

/// Main execution engine
pub struct Executor {
    /// Built-in commands registry
    builtins: HashMap<String, Box<dyn Builtin>>,
    /// Execution mode
    mode: ExecutionMode,
    /// File system abstraction
    filesystem: Arc<FileSystem>,
    /// Process manager
    process_manager: Arc<ProcessManager>,
    /// Command cache for faster lookup
    command_cache: Arc<Mutex<HashMap<String, PathBuf>>>,
    /// Execution statistics
    stats: ExecutionStats,
}

/// Execution statistics
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    pub commands_executed: u64,
    pub builtins_executed: u64,
    pub external_commands: u64,
    pub pipelines_executed: u64,
    pub background_jobs: u64,
    pub total_execution_time: Duration,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl Executor {
    /// Create a new executor
    pub fn new() -> ShellResult<Self> {
        let filesystem = Arc::new(FileSystem::new()?);
        let process_manager = Arc::new(ProcessManager::new()?);
        
        let mut executor = Self {
            builtins: HashMap::new(),
            mode: ExecutionMode::Interactive,
            filesystem,
            process_manager,
            command_cache: Arc::new(Mutex::new(HashMap::new())),
            stats: ExecutionStats::default(),
        };
        
        // Register built-in commands
        executor.register_builtins()?;
        
        Ok(executor)
    }

    /// Set execution mode
    pub fn set_mode(&mut self, mode: ExecutionMode) {
        self.mode = mode;
    }

    /// Get execution mode
    pub fn mode(&self) -> &ExecutionMode {
        &self.mode
    }

    /// Register a built-in command
    pub fn register_builtin(&mut self, builtin: Box<dyn Builtin>) {
        self.builtins.insert(builtin.name().to_string(), builtin);
    }

    /// Check if a command is a built-in
    pub fn is_builtin(&self, name: &str) -> bool {
        self.builtins.contains_key(name)
    }

    /// Get built-in command
    pub fn get_builtin(&self, name: &str) -> Option<&dyn Builtin> {
        self.builtins.get(name).map(|b| b.as_ref())
    }

    /// List all built-in commands
    pub fn list_builtins(&self) -> Vec<&str> {
        self.builtins.keys().map(|s| s.as_str()).collect()
    }

    /// Execute an AST node
    pub fn execute(&mut self, node: &AstNode, ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let start_time = Instant::now();
        
        // Check for timeout
        if ctx.is_timed_out() {
            return Err(ShellError::new(
                ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::Timeout),
                "Command execution timed out"
            ));
        }

        let result = match node {
            AstNode::Program(statements) => self.execute_program(statements, ctx),
            AstNode::Pipeline { elements, operators } => {
                let pipeline = nxsh_parser::ast::Pipeline { elements: elements.clone(), operators: operators.clone() };
                self.execute_pipeline(&pipeline, ctx)
            },
            AstNode::Command { name, args, redirections, background } => {
                let command = nxsh_parser::ast::Command { 
                    name: name.clone(), 
                    args: args.iter().map(|arg| match arg {
                        nxsh_parser::ast::AstNode::Word(w) => nxsh_parser::ast::Argument::Word(w),
                        nxsh_parser::ast::AstNode::StringLiteral { value, .. } => nxsh_parser::ast::Argument::String(value),
                        nxsh_parser::ast::AstNode::NumberLiteral { value, .. } => {
                            if let Ok(num) = value.parse::<i64>() {
                                nxsh_parser::ast::Argument::Number(num)
                            } else {
                                nxsh_parser::ast::Argument::Word(value)
                            }
                        },
                        _ => nxsh_parser::ast::Argument::Word(""),
                    }).collect(),
                    redirections: redirections.clone(), 
                    background: *background 
                };
                self.execute_command(&command, ctx)
            },
            AstNode::If { condition, then_branch, elif_branches, else_branch } => {
                self.execute_if_statement(condition, then_branch, elif_branches, else_branch, ctx)
            },
            AstNode::For { variable, iterable, body, is_async } => {
                self.execute_for_statement(variable, iterable, body, *is_async, ctx)
            },
            AstNode::While { condition, body } => {
                self.execute_while_statement(condition, body, ctx)
            },
            AstNode::Function { name, params, body, is_async } => {
                self.execute_function_declaration(name, params, body, *is_async, ctx)
            },
            AstNode::Assignment { name, operator, value, is_local, is_export, is_readonly } => {
                self.execute_assignment_statement(name, operator, value, *is_local, *is_export, *is_readonly, ctx)
            },
            AstNode::Subshell(subshell) => {
                // Convert single node to slice
                let nodes = vec![subshell.as_ref()];
                self.execute_subshell(&nodes[..], ctx)
            },
            AstNode::Background(bg_cmd) => self.execute_background(bg_cmd, ctx),
            _ => {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                    format!("Unsupported AST node type: {:?}", node)
                ));
            }
        };

        // Update statistics
        self.stats.commands_executed += 1;
        self.stats.total_execution_time += start_time.elapsed();

        result
    }

    /// Execute a program (list of statements)
    fn execute_program(&mut self, statements: &[AstNode], ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let mut last_result = ExecutionResult::success(0);
        
        for statement in statements {
            // Check errexit option
            if ctx.get_option("errexit")? && last_result.exit_code != 0 {
                break;
            }
            
            last_result = self.execute(statement, ctx)?;
            ctx.set_exit_status(last_result.exit_code);
        }
        
        Ok(last_result)
    }

    /// Execute a pipeline
    fn execute_pipeline(&mut self, pipeline: &nxsh_parser::ast::Pipeline, ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        self.stats.pipelines_executed += 1;
        
        if pipeline.elements.is_empty() {
            return Ok(ExecutionResult::success(0));
        }
        
        if pipeline.elements.len() == 1 {
            // Single command, no pipe
            // TODO: Convert AstNode to Command
            return Ok(ExecutionResult::success(0));
        }
        
        // Multi-command pipeline
        let mut _processes: Vec<std::process::Child> = Vec::new();
        let mut _last_stdout: Option<std::process::Child> = None;
        
        for (i, _element) in pipeline.elements.iter().enumerate() {
            let _is_first = i == 0;
            let _is_last = i == pipeline.elements.len() - 1;
            // TODO: Convert AstNode to Command and execute
            
            // TODO: Convert element to command and build
            // let mut cmd = self.build_std_command(command, ctx)?;
            
            // TODO: Implement pipeline execution
            /*
            // Set up stdin
            if !_is_first {
                if let Some(mut prev_child) = _last_stdout.take() {
                    if let Some(stdout) = prev_child.stdout.take() {
                        cmd.stdin(stdout);
                    }
                }
            }
            
            // Set up stdout
            if !is_last {
                cmd.stdout(Stdio::piped());
            }
            
            let child = cmd.spawn()
                .map_err(|e| ShellError::new(ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError), format!("Failed to spawn command: {}", e)))?;
            
            if !is_last {
                last_stdout = Some(child);
            } else {
                processes.push(child);
            }
            */
        }
        
        // TODO: Implement pipeline process waiting
        Ok(ExecutionResult::success(0))
    }

    /// Execute a single command
    fn execute_command(&mut self, command: &nxsh_parser::ast::Command, ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let command_name = self.resolve_command_name(command, ctx)?;
        
        // Check if it's a built-in command
        if let Some(builtin) = self.builtins.get(&command_name) {
            self.stats.builtins_executed += 1;
            
            // Create execution context for built-in
            let args = self.resolve_command_args(command, ctx)?;
            let mut builtin_ctx = Context::new(
                args,
                ctx,
                Stream::new(StreamType::Byte),
                Stream::new(StreamType::Byte),
                Stream::new(StreamType::Byte),
            )?;
            
            return builtin.invoke(&mut builtin_ctx);
        }
        
        // External command
        self.stats.external_commands += 1;
        self.execute_external_command(command, ctx)
    }

    /// Execute an external command
    fn execute_external_command(&mut self, command: &nxsh_parser::ast::Command, ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let command_path = self.find_command_in_path(&self.resolve_command_name(command, ctx)?)?;
        let args = self.resolve_command_args(command, ctx)?;
        
        let mut cmd = StdCommand::new(&command_path);
        cmd.args(&args[1..]);  // Skip command name
        
        // Set up environment
        for entry in ctx.env.iter() {
            cmd.env(entry.key(), entry.value());
        }
        
        // Set working directory
        cmd.current_dir(&ctx.cwd);
        
        // Handle redirections
        self.apply_redirections(&mut cmd, &command.redirections, ctx)?;
        
        let start_time = Instant::now();
        
        let output = cmd.output()
            .map_err(|e| ShellError::new(ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError), format!("Failed to execute command: {}", e)))?;
        
        let duration = start_time.elapsed();
        let exit_code = output.status.code().unwrap_or(-1);
        
        Ok(ExecutionResult::success(exit_code)
            .with_output(output.stdout)
            .with_error(output.stderr)
            .with_duration(duration))
    }

    /// Execute background command
    fn execute_background(&mut self, command: &AstNode, ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        self.stats.background_jobs += 1;
        
        // Create a new job
        let _job_id = {
            let job_manager_arc = ctx.job_manager();
            let mut job_manager = job_manager_arc.lock()
                .map_err(|_| ShellError::new(ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState), "Job manager lock poisoned"))?;
            job_manager.create_job(format!("Background job"))
        };
        
        // Execute command in background (simplified for now)
        // TODO: Implement proper background execution with job control
        let result = self.execute(command, ctx)?;
        
        Ok(result)
    }

    /// Execute subshell
    fn execute_subshell(&mut self, commands: &[&AstNode], ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        // Convert &[&AstNode] to &[AstNode]
        let commands_owned: Vec<AstNode> = commands.iter().map(|&node| node.clone()).collect();
        
        // Create a new context for the subshell
        // TODO: Implement proper subshell isolation
        self.execute_program(&commands_owned, ctx)
    }

    /// Register built-in commands
    fn register_builtins(&mut self) -> ShellResult<()> {
        // Register core built-in commands
        // TODO: Register builtins without circular dependency
        // self.register_builtin(Box::new(nxsh_builtins::cd::CdCommand::new()));
        // self.register_builtin(Box::new(nxsh_builtins::echo::EchoCommand::new()));
        // self.register_builtin(Box::new(nxsh_builtins::export::ExportCommand::new()));
        // self.register_builtin(Box::new(nxsh_builtins::alias::AliasCommand::new()));
        // self.register_builtin(Box::new(nxsh_builtins::history::HistoryCommand::new()));
        
        Ok(())
    }

    /// Resolve command name (handle aliases, functions, etc.)
    fn resolve_command_name(&self, command: &nxsh_parser::ast::Command, ctx: &ShellContext) -> ShellResult<String> {
        if command.args.is_empty() {
            return Err(ShellError::new(ErrorKind::ParseError(crate::error::ParseErrorKind::SyntaxError), "Empty command"));
        }
        
        let name = match &command.args[0] {
            nxsh_parser::ast::Argument::Word(w) => w.to_string(),
            nxsh_parser::ast::Argument::String(s) => s.to_string(),
            _ => return Err(ShellError::new(ErrorKind::ParseError(crate::error::ParseErrorKind::SyntaxError), "Invalid command name")),
        };
        
        // Check for alias
        if let Some(alias_value) = ctx.get_alias(&name) {
            return Ok(alias_value);
        }
        
        Ok(name)
    }

    /// Resolve command arguments
    fn resolve_command_args(&self, command: &nxsh_parser::ast::Command, ctx: &ShellContext) -> ShellResult<Vec<String>> {
        let mut args = Vec::new();
        
        for arg in &command.args {
            let resolved = match arg {
                nxsh_parser::ast::Argument::Word(w) => w.to_string(),
                nxsh_parser::ast::Argument::String(s) => s.to_string(),
                nxsh_parser::ast::Argument::Variable(v) => {
                    ctx.get_var(v).unwrap_or_default()
                }
                nxsh_parser::ast::Argument::Number(n) => n.to_string(),
                _ => return Err(ShellError::new(ErrorKind::ParseError(crate::error::ParseErrorKind::SyntaxError), "Unsupported argument type")),
            };
            args.push(resolved);
        }
        
        Ok(args)
    }

    /// Find command in PATH
    fn find_command_in_path(&self, command: &str) -> ShellResult<PathBuf> {
        // Check cache first
        if let Ok(cache) = self.command_cache.lock() {
            if let Some(path) = cache.get(command) {
                // TODO: Update cache hits statistics
                return Ok(path.clone());
            }
        }
        
        // TODO: Update cache misses statistics
        
        // Search in PATH
        if let Some(path_var) = std::env::var_os("PATH") {
            for path_dir in std::env::split_paths(&path_var) {
                let full_path = path_dir.join(command);
                if full_path.is_file() {
                    // Cache the result
                    if let Ok(mut cache) = self.command_cache.lock() {
                        cache.insert(command.to_string(), full_path.clone());
                    }
                    return Ok(full_path);
                }
            }
        }
        
        Err(ShellError::new(
            ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::CommandNotFound),
            format!("Command not found: {}", command)
        ))
    }

    /// Build standard command for external execution
    fn build_std_command(&self, command: &nxsh_parser::ast::Command, ctx: &ShellContext) -> ShellResult<StdCommand> {
        let command_name = self.resolve_command_name(command, ctx)?;
        let command_path = self.find_command_in_path(&command_name).unwrap_or_else(|_| PathBuf::from(&command_name));
        let args = self.resolve_command_args(command, ctx)?;
        
        let mut cmd = StdCommand::new(command_path);
        cmd.args(&args[1..]);
        
        // Set environment
        for entry in ctx.env.iter() {
            cmd.env(entry.key(), entry.value());
        }
        
        cmd.current_dir(&ctx.cwd);
        
        Ok(cmd)
    }

    /// Apply redirections to a command
    fn apply_redirections(&self, cmd: &mut StdCommand, redirections: &[nxsh_parser::ast::Redirection], _ctx: &ShellContext) -> ShellResult<()> {
        for redir in redirections {
            match redir.redir_type {
                nxsh_parser::ast::RedirectionType::Output => {
                    let file = std::fs::File::create(&redir.target)
                        .map_err(|e| ShellError::new(ErrorKind::IoError(crate::error::IoErrorKind::FileCreateError), format!("Failed to create file: {}", e)))?;
                    cmd.stdout(file);
                }
                nxsh_parser::ast::RedirectionType::Append => {
                    let file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&redir.target)
                        .map_err(|e| ShellError::new(ErrorKind::IoError(crate::error::IoErrorKind::FileWriteError), format!("Failed to open file: {}", e)))?;
                    cmd.stdout(file);
                }
                nxsh_parser::ast::RedirectionType::Input => {
                    let file = std::fs::File::open(&redir.target)
                        .map_err(|e| ShellError::new(ErrorKind::IoError(crate::error::IoErrorKind::FileReadError), format!("Failed to open file: {}", e)))?;
                    cmd.stdin(file);
                }
                nxsh_parser::ast::RedirectionType::Error => {
                    let file = std::fs::File::create(&redir.target)
                        .map_err(|e| ShellError::new(ErrorKind::IoError(crate::error::IoErrorKind::FileCreateError), format!("Failed to create file: {}", e)))?;
                    cmd.stderr(file);
                }
                _ => {
                    // TODO: Implement other redirection types
                }
            }
        }
        
        Ok(())
    }

    /// Resolve items for for-loop iteration
    fn resolve_for_items(&self, _items: &nxsh_parser::ast::AstNode, _ctx: &ShellContext) -> ShellResult<Vec<String>> {
        // TODO: Implement proper item resolution (glob expansion, variable expansion, etc.)
        Ok(vec!["item1".to_string(), "item2".to_string(), "item3".to_string()])
    }

    /// Resolve assignment value
    fn resolve_assignment_value(&self, _value: &nxsh_parser::ast::AstNode, _ctx: &ShellContext) -> ShellResult<String> {
        // TODO: Implement proper value resolution
        Ok("value".to_string())
    }

    /// Get execution statistics
    pub fn stats(&self) -> &ExecutionStats {
        &self.stats
    }

    /// Reset execution statistics
    pub fn reset_stats(&mut self) {
        self.stats = ExecutionStats::default();
    }

    /// Execute MIR program (for optimized execution)
    pub fn execute_mir(&mut self, _program: &str, _ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        // TODO: Implement MIR execution
        // This would be used for optimized execution of compiled shell scripts
        
        Ok(ExecutionResult::success(0))
    }

    /// Execute if statement
    fn execute_if_statement(&self, _condition: &Box<nxsh_parser::ast::AstNode>, _then_branch: &Box<nxsh_parser::ast::AstNode>, _elif_branches: &Vec<(nxsh_parser::ast::AstNode, nxsh_parser::ast::AstNode)>, _else_branch: &Option<Box<nxsh_parser::ast::AstNode>>, _ctx: &ShellContext) -> ShellResult<ExecutionResult> {
        // TODO: Implement if statement execution
        Ok(ExecutionResult::success(0))
    }

    /// Execute for statement
    fn execute_for_statement(&self, _variable: &str, _iterable: &Box<nxsh_parser::ast::AstNode>, _body: &Box<nxsh_parser::ast::AstNode>, _is_async: bool, _ctx: &ShellContext) -> ShellResult<ExecutionResult> {
        // TODO: Implement for statement execution
        Ok(ExecutionResult::success(0))
    }

    /// Execute while statement
    fn execute_while_statement(&self, _condition: &Box<nxsh_parser::ast::AstNode>, _body: &Box<nxsh_parser::ast::AstNode>, _ctx: &ShellContext) -> ShellResult<ExecutionResult> {
        // TODO: Implement while statement execution
        Ok(ExecutionResult::success(0))
    }

    /// Execute function declaration
    fn execute_function_declaration(&self, _name: &str, _params: &Vec<nxsh_parser::ast::Parameter>, _body: &Box<nxsh_parser::ast::AstNode>, _is_async: bool, _ctx: &ShellContext) -> ShellResult<ExecutionResult> {
        // TODO: Implement function declaration execution
        Ok(ExecutionResult::success(0))
    }

    /// Execute assignment statement
    fn execute_assignment_statement(&self, _name: &str, _operator: &nxsh_parser::ast::AssignmentOperator, _value: &Box<nxsh_parser::ast::AstNode>, _is_local: bool, _is_export: bool, _is_readonly: bool, _ctx: &ShellContext) -> ShellResult<ExecutionResult> {
        // TODO: Implement assignment statement execution
        Ok(ExecutionResult::success(0))
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new().expect("Failed to create default executor")
    }
} 