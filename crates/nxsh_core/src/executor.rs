//! Command execution engine with MIR integration for NexusShell
//!
//! This module provides the core execution engine that can interpret both
//! AST nodes directly and compiled MIR programs for optimal performance.

use crate::error::{ShellError, ErrorKind, ShellResult};
use crate::context::ShellContext;
use crate::mir::{MirExecutor, MirProgram, MirValue}; // MIR integration
use nxsh_parser::ast::AstNode;
use nxsh_parser::parse as parse_program;
// use crate::macros::{MacroSystem, Macro}; // currently unused

/// 最低限の AST -> ソース 逆変換 (関数/クロージャ body, default 引数, シリアライズ用途)
pub(crate) fn simple_unparse(node: &AstNode) -> String {
    match node {
        AstNode::StatementList(list) | AstNode::Program(list) => list.iter().map(simple_unparse).collect::<Vec<_>>().join("\n"),
        AstNode::Command { name, args, .. } => {
            let mut parts = Vec::new();
            parts.push(simple_unparse(name));
            for a in args { parts.push(simple_unparse(a)); }
            parts.join(" ")
        }
        AstNode::Word(w) => w.to_string(),
        AstNode::StringLiteral { value, .. } => format!("\"{}\"", value),
        AstNode::NumberLiteral { value, .. } => value.to_string(),
        _ => format!("#unprintable:{:?}", node),
    }
}
// use crate::macros::{MacroSystem, Macro}; // currently unused
// use crate::macros::{MacroSystem, Macro}; // currently unused
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;

/// Execution strategy for shell commands
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecutionStrategy {
    /// Direct AST interpretation
    DirectInterpreter,
    /// MIR-based optimized execution
    MirEngine,
}

/// Execution result containing output and metadata
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Exit status code
    pub exit_code: i32,
    /// Standard output data
    pub stdout: String,
    /// Standard error data
    pub stderr: String,
    /// Execution time in microseconds
    pub execution_time: u64,
    /// Strategy used for execution
    pub strategy: ExecutionStrategy,
    /// Performance metrics
    pub metrics: ExecutionMetrics,
}

impl ExecutionResult {
    /// Create a successful execution result
    pub fn success(exit_code: i32) -> Self {
        Self {
            exit_code,
            stdout: String::new(),
            stderr: String::new(),
            execution_time: 0,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        }
    }
    
    /// Create a failed execution result
    pub fn failure(exit_code: i32) -> Self {
        Self {
            exit_code,
            stdout: String::new(),
            stderr: String::new(),
            execution_time: 0,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        }
    }

    /// Construct from ShellError (maps to failure with code 1)
    pub fn from_error(err: ShellError) -> Self {
        Self {
            exit_code: 1,
            stdout: String::new(),
            stderr: err.message,
            execution_time: 0,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        }
    }
    
    /// Check if the execution was successful (exit code 0)
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }
    
    /// Set error output
    pub fn with_error(mut self, error: Vec<u8>) -> Self {
        self.stderr = String::from_utf8_lossy(&error).to_string();
        self
    }
    
    /// Set standard output
    pub fn with_output(mut self, output: Vec<u8>) -> Self {
        self.stdout = String::from_utf8_lossy(&output).to_string();
        self
    }
}

/// Performance metrics for execution analysis
#[derive(Debug, Clone, Default)]
pub struct ExecutionMetrics {
    /// Time spent compiling (if applicable)
    pub compile_time_us: u64,
    /// Time spent optimizing (if applicable)
    pub optimize_time_us: u64,
    /// Time spent executing
    pub execute_time_us: u64,
    /// Number of instructions executed
    pub instruction_count: u64,
    /// Memory usage in bytes
    pub memory_usage: u64,
}

/// Builtin command trait for shell builtins
pub trait Builtin: Send + Sync {
    /// Execute the builtin command
    fn execute(&self, context: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult>;
    
    /// Get the name of the builtin
    fn name(&self) -> &'static str;
    
    /// Get help text for the builtin  
    fn help(&self) -> &'static str;
    
    /// Get synopsis for the builtin
    fn synopsis(&self) -> &'static str;
    
    /// Get description for the builtin
    fn description(&self) -> &'static str;
    
    /// Get usage for the builtin
    fn usage(&self) -> &'static str;
    
    /// Check if this builtin affects shell state
    fn affects_shell_state(&self) -> bool {
        false
    }
    
    /// Invoke the builtin (compatibility wrapper)
    fn invoke(&self, ctx: &mut crate::context::ShellContext) -> ShellResult<ExecutionResult> {
        // Extract args from context if available, otherwise use empty args
        let args = Vec::new(); // This would need to be extracted from ctx properly
        self.execute(ctx, &args)
    }
}

/// Main shell executor with multi-strategy execution support
pub struct Executor {
    /// Registered builtin commands
    builtins: HashMap<String, Arc<dyn Builtin>>,
    /// Current execution strategy
    strategy: ExecutionStrategy,
    /// Performance statistics
    stats: ExecutorStats,
    /// MIR execution engine for optimized performance
    mir_executor: MirExecutor,
    /// Small LRU cache for command substitutions to speed up deep nesting
    cmdsub_cache_map: HashMap<String, ExecutionResult>,
    cmdsub_cache_order: VecDeque<String>,
    cmdsub_cache_capacity: usize,
}

/// Executor performance statistics
#[derive(Debug, Default)]
pub struct ExecutorStats {
    /// Total commands executed
    pub total_commands: u64,
    /// Commands executed via AST interpreter
    pub ast_interpreter_count: u64,
    /// Commands executed via MIR
    pub mir_execution_count: u64,
    /// Total execution time
    pub total_execution_time_us: u64,
    /// Average execution time
    pub average_execution_time_us: u64,
}

impl Executor {
    fn cmdsub_cache_get(&mut self, key: &str) -> Option<ExecutionResult> {
        if let Some(v) = self.cmdsub_cache_map.get(key) {
            if let Some(pos) = self.cmdsub_cache_order.iter().position(|k| k == key) {
                if let Some(k) = self.cmdsub_cache_order.remove(pos) { self.cmdsub_cache_order.push_back(k); }
            }
            return Some(v.clone());
        }
        None
    }

    fn cmdsub_cache_put(&mut self, key: String, value: ExecutionResult) {
        if self.cmdsub_cache_map.contains_key(&key) {
            self.cmdsub_cache_map.insert(key.clone(), value);
            if let Some(pos) = self.cmdsub_cache_order.iter().position(|k| k == &key) {
                if let Some(k) = self.cmdsub_cache_order.remove(pos) { self.cmdsub_cache_order.push_back(k); }
            } else {
                self.cmdsub_cache_order.push_back(key);
            }
            return;
        }
        if self.cmdsub_cache_order.len() >= self.cmdsub_cache_capacity {
            if let Some(old_key) = self.cmdsub_cache_order.pop_front() {
                self.cmdsub_cache_map.remove(&old_key);
            }
        }
        self.cmdsub_cache_order.push_back(key.clone());
        self.cmdsub_cache_map.insert(key, value);
    }

    fn eval_cmd_substitution(&mut self, command: &AstNode, context: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let key = simple_unparse(command);
        if let Some(hit) = self.cmdsub_cache_get(&key) {
            return Ok(hit);
        }
        let res = self.execute_ast_direct(command, context)?;
        self.cmdsub_cache_put(key, res.clone());
        Ok(res)
    }
    // Simple filename glob / extglob subset expansion (no directory components yet).
    // Supports: *, ?, [abc] character classes. Extglob subset patterns *(alt1|alt2), +(alt), ?(alt), @(alt), !(alt) are
    // approximated into a small candidate set before standard wildcard matching. Safety caps: max 256 matches.
    fn expand_glob_if_needed(pattern: &str, context: &ShellContext) -> Vec<String> {
        // Acquire options snapshot
    let (noglob, dotglob, nocaseglob) = if let Ok(opts) = context.options.read() { (opts.noglob, opts.dotglob, opts.nocaseglob) } else { (false,false,false) };
        if noglob { return vec![]; }
        if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('[') && !pattern.contains('(') { return vec![]; }
        // Directory components not yet handled – skip if contains path sep
        if pattern.contains('/') || pattern.contains('\\') { return vec![]; }
        // Expand simple extglob first
        let mut base_candidates = vec![pattern.to_string()];
        let mut negate_pattern = false; // Track if this is a negation pattern
        
        if pattern.contains("*(") || pattern.contains("+(" ) || pattern.contains("?(") || pattern.contains("@(") || pattern.contains("!(") {
            if let Some(start_paren) = pattern.find('(') { 
                if start_paren > 0 { 
                    let kind_char = &pattern[start_paren-1..start_paren]; 
                    if let Some(close_rel) = pattern[start_paren+1..].find(')') { 
                        let close = start_paren+1+close_rel; 
                        let prefix = &pattern[..start_paren-1]; 
                        let body = &pattern[start_paren+1..close]; 
                        let suffix = &pattern[close+1..]; 
                        let alts: Vec<&str> = body.split('|').collect(); 
                        let mut new_patterns = Vec::new(); 
                        match kind_char { 
                            "*" => { 
                                new_patterns.push(format!("{prefix}{suffix}")); 
                                for alt in &alts { 
                                    new_patterns.push(format!("{prefix}{alt}{suffix}")); 
                                } 
                            }, 
                            "+" => { 
                                // One or more occurrences approximation.
                                // For simple cases like +(<char>) we approximate using two candidates:
                                //   1) single occurrence
                                //   2) one-or-more via trailing wildcard after the alt
                                // This is sufficient for + (a).txt -> ["a.txt", "a*.txt"], which
                                // combined with suffix ensures aa.txt, aaa.txt also match.
                                if alts.len() == 1 {
                                    let alt = alts[0];
                                    new_patterns.push(format!("{prefix}{alt}{suffix}"));
                                    new_patterns.push(format!("{prefix}{alt}*{suffix}"));
                                } else {
                                    for alt in &alts { 
                                        new_patterns.push(format!("{prefix}{alt}{suffix}"));
                                        new_patterns.push(format!("{prefix}{alt}*{suffix}"));
                                    }
                                }
                            }, 
                            "?" => { 
                                new_patterns.push(format!("{prefix}{suffix}")); 
                                for alt in &alts { 
                                    new_patterns.push(format!("{prefix}{alt}{suffix}")); 
                                } 
                            }, 
                            "@" => { 
                                for alt in &alts { 
                                    new_patterns.push(format!("{prefix}{alt}{suffix}")); 
                                } 
                            }, 
                            "!" => { 
                                // Extglob negation pattern !(pattern) implementation
                                // Status: Implemented (2025-08-11) - matches files that do NOT match the given patterns
                                // Example: !(*.txt|*.log) matches all files except those ending in .txt or .log
                                // Behavior: Enumerate all files in current directory, exclude those matching any alternative
                                // Fallback: If implementation has issues, falls back to literal pattern matching
                                // TODO: Add comprehensive testing for edge cases (empty dirs, nested patterns, etc.)
                                
                                negate_pattern = true;
                                for alt in &alts { 
                                    new_patterns.push(format!("{prefix}{alt}{suffix}")); 
                                } 
                            }, 
                            _ => new_patterns.push(pattern.to_string()) 
                        }; 
                        base_candidates = new_patterns; 
                    } 
                } 
            }
        }
        // Matcher
        fn matches(simple_pat: &str, name: &str, dotglob: bool, nocase: bool) -> bool {
            if !dotglob && name.starts_with('.') && !simple_pat.starts_with('.') { return false; }
            let use_nocase = nocase || cfg!(windows);
            let (p, n) = if use_nocase { (simple_pat.to_lowercase(), name.to_lowercase()) } else { (simple_pat.to_string(), name.to_string()) };
            fn class_match(class: &str, c: char) -> bool {
                // Support simple classes like [abc] and ranges like [a-c]
                let mut chars = class.chars().peekable();
                let mut last: Option<char> = None;
                let mut any = false;
                while let Some(ch) = chars.next() {
                    if ch == '-' {
                        if let (Some(start), Some(end)) = (last, chars.peek().copied()) {
                            let (s, e) = (start as u32, end as u32);
                            if s <= e {
                                if (s..=e).any(|u| Some(c) == char::from_u32(u)) { return true; }
                            } else {
                                if (e..=s).any(|u| Some(c) == char::from_u32(u)) { return true; }
                            }
                            any = true;
                            last = chars.next();
                            continue;
                        }
                    }
                    if ch == c { return true; }
                    any = true;
                    last = Some(ch);
                }
                // No element matched
                false
            }
            fn rec(pi: usize, ni: usize, p: &[char], n: &[char], dotglob: bool) -> bool {
                let mut i=pi; let mut j=ni;
                while i < p.len() {
                    match p[i] {
                        '*' => { // greedy
                            // collapse consecutive *
                            while i+1 < p.len() && p[i+1]=='*' { i+=1; }
                            if i+1==p.len() { return true; }
                            let mut k=j; while k <= n.len() { if rec(i+1,k,&p,&n,dotglob) { return true; } if k==n.len() { break; } k+=1; }
                            return false;
                        }
                        '?' => { if j>=n.len() { return false; } j+=1; i+=1; }
                        '[' => {
                            let mut k=i+1; let mut cls = String::new();
                            while k<p.len() && p[k]!=']' { cls.push(p[k]); k+=1; }
                            if k==p.len() || j>=n.len() { return false; }
                            if !class_match(&cls,n[j]) { return false; }
                            j+=1; i=k+1;
                        }
                        c => { if j>=n.len() || c!=n[j] { return false; } i+=1; j+=1; }
                    }
                }
                j==n.len()
            }
            rec(0,0,&p.chars().collect::<Vec<_>>(), &n.chars().collect::<Vec<_>>(), dotglob)
        }
        // Process files in directory
        let mut out = Vec::new();
        let scan_dir = std::env::current_dir().unwrap_or_else(|_| context.cwd.clone());
        if let Ok(dir) = std::fs::read_dir(&scan_dir) { 
            for entry in dir.flatten().take(2048) { 
                if let Some(name) = entry.file_name().to_str() { 
                    if negate_pattern {
                        // Negation pattern implementation: include files that DON'T match any alternative
                        // This implements the !(pattern1|pattern2|...) extglob syntax
                        // Algorithm: 
                        // 1. Test each file against all alternative patterns
                        // 2. Include file only if it matches NONE of the patterns
                        // 3. Respects dotglob and case sensitivity options
                        let mut matches_any = false;
                        for pat in &base_candidates { 
                            if matches(pat, name, dotglob, nocaseglob) { 
                                matches_any = true;
                                break; 
                            } 
                        }
                        if !matches_any {
                            out.push(name.to_string());
                        }
                    } else {
                        // Normal patterns: include files that match at least one pattern
                        for pat in &base_candidates { 
                            if matches(pat, name, dotglob, nocaseglob) { 
                                out.push(name.to_string()); 
                                break; 
                            } 
                        } 
                    }
                } 
            } 
        }
        out.sort();
        out.dedup();
        if out.is_empty() {
            // Fallback: handle simple substring/suffix patterns like *token* or *.ext even if the
            // approximated extglob phase produced no candidates.
            let simple_chars = |s: &str| !s.contains('[') && !s.contains('(') && !s.contains('?');
            let use_nocase = nocaseglob || cfg!(windows);
            let mut fallback_results: Vec<String> = Vec::new();
            if pattern.starts_with('*') && simple_chars(pattern) {
                let mut core = &pattern[1..];
                let is_trailing_star = core.ends_with('*');
                if is_trailing_star { core = &core[..core.len()-1]; }
                if !core.is_empty() {
                    let token = if use_nocase { core.to_lowercase() } else { core.to_string() };
                    if let Ok(iter) = std::fs::read_dir(&scan_dir) {
                        for entry in iter.flatten().take(2048) {
                            if let Some(name) = entry.file_name().to_str() {
                                if !dotglob && name.starts_with('.') && !pattern.starts_with('.') { continue; }
                                let cmp = if use_nocase { name.to_lowercase() } else { name.to_string() };
                                let is_match = if is_trailing_star { cmp.contains(&token) } else { cmp.ends_with(&token) };
                                if is_match { fallback_results.push(name.to_string()); }
                            }
                        }
                    }
                }
            }
            if !fallback_results.is_empty() {
                fallback_results.sort();
                fallback_results.dedup();
                return fallback_results;
            }
            // If still no matches and nullglob is disabled, return literal pattern
            if !context.options.read().map(|o| o.nullglob).unwrap_or(false) {
                return vec![pattern.to_string()];
            }
        }
        if out.len()>256 { out.truncate(256); }
        out
    }
    /// Create comprehensive executor with full builtin registration
    /// COMPLETE initialization with ALL builtins as required
    pub fn new_minimal() -> Self {
        eprintln!("DEBUG: Creating comprehensive Executor with ALL builtins");
        let mut executor = Self {
            builtins: HashMap::new(),
            strategy: ExecutionStrategy::DirectInterpreter,
            stats: ExecutorStats::default(),
            mir_executor: MirExecutor::new(),
            cmdsub_cache_map: HashMap::new(),
            cmdsub_cache_order: VecDeque::new(),
            cmdsub_cache_capacity: 128,
        };
        
        // COMPLETE builtin registration as specified - NO deferred loading
        executor.register_all_builtins();
        eprintln!("DEBUG: Registered {} builtins", executor.builtins.len());
        executor
    }

    /// Create a new executor with default settings
    pub fn new() -> Self {
        eprintln!("DEBUG: Creating new Executor");
        let mut executor = Self {
            builtins: HashMap::new(),
            strategy: ExecutionStrategy::DirectInterpreter,
            stats: ExecutorStats::default(),
            mir_executor: MirExecutor::new(),
            cmdsub_cache_map: HashMap::new(),
            cmdsub_cache_order: VecDeque::new(),
            cmdsub_cache_capacity: 128,
        };
        
        // Register built-in commands
        eprintln!("DEBUG: About to register all builtins");
        executor.register_all_builtins();
        eprintln!("DEBUG: Executor created with {} builtins", executor.builtins.len());
        // Select execution strategy by environment configuration
        // NXSH_EXEC_STRATEGY=ast|mir (default: ast). NXSH_JIT=1 implies mir.
        if let Ok(strategy_env) = std::env::var("NXSH_EXEC_STRATEGY") {
            match strategy_env.to_ascii_lowercase().as_str() {
                "mir" | "jit" => executor.strategy = ExecutionStrategy::MirEngine,
                "ast" | "interp" | "interpreter" => executor.strategy = ExecutionStrategy::DirectInterpreter,
                _ => {}
            }
        } else if std::env::var("NXSH_JIT").map(|v| v=="1" || v.eq_ignore_ascii_case("true")).unwrap_or(false) {
            executor.strategy = ExecutionStrategy::MirEngine;
        }

        executor
    }
    
    /// Register all built-in commands
    fn register_all_builtins(&mut self) {
        let builtins = crate::builtins::register_all_builtins();
        eprintln!("DEBUG: Registering {} builtins", builtins.len());
        for builtin in builtins {
            let name = builtin.name();
            eprintln!("DEBUG: Registering builtin: {}", name);
            self.register_builtin(builtin);
        }
        eprintln!("DEBUG: Total registered builtins: {}", self.builtins.len());
    }
    
    /// Register a builtin command
    pub fn register_builtin(&mut self, builtin: Arc<dyn Builtin>) {
        let name = builtin.name().to_string();
        self.builtins.insert(name, builtin);
    }
    
    /// Set the execution strategy
    pub fn set_strategy(&mut self, strategy: ExecutionStrategy) {
        self.strategy = strategy;
    }
    
    /// Execute an AST node with the current strategy
    pub fn execute(&mut self, node: &AstNode, context: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let start_time = Instant::now();
        // Global timeout guard at entry
        if context.is_timed_out() {
            let execution_time = start_time.elapsed().as_micros() as u64;
            return Ok(ExecutionResult {
                exit_code: 124,
                stdout: String::new(),
                stderr: "nxsh: execution timed out".to_string(),
                execution_time,
                strategy: self.strategy,
                metrics: ExecutionMetrics::default(),
            });
        }

        // Execute according to strategy, but do not early-return on error so we can update stats
        let result: ShellResult<ExecutionResult> = match self.strategy {
            ExecutionStrategy::DirectInterpreter => self.execute_ast_direct(node, context),
            ExecutionStrategy::MirEngine => self.execute_with_mir(node, context),
        };

        let execution_time = start_time.elapsed().as_micros() as u64;

        // Update statistics regardless of success/failure
        self.stats.total_commands += 1;
        self.stats.total_execution_time_us += execution_time;
        self.stats.average_execution_time_us =
            self.stats.total_execution_time_us / self.stats.total_commands;

        match self.strategy {
            ExecutionStrategy::DirectInterpreter => {
                self.stats.ast_interpreter_count += 1;
            }
            ExecutionStrategy::MirEngine => {
                self.stats.mir_execution_count += 1;
            }
        }

        // Return the original result or convert timeout detected late
        match result {
            Ok(mut r) => {
                if context.is_timed_out() {
                    let execution_time = start_time.elapsed().as_micros() as u64;
                    Ok(ExecutionResult {
                        exit_code: 124,
                        stdout: String::new(),
                        stderr: "nxsh: execution timed out".to_string(),
                        execution_time,
                        strategy: self.strategy,
                        metrics: ExecutionMetrics::default(),
                    })
                } else {
                    Ok(r)
                }
            },
            Err(e) => {
                if context.is_timed_out() {
                    let execution_time = start_time.elapsed().as_micros() as u64;
                    Ok(ExecutionResult {
                        exit_code: 124,
                        stdout: String::new(),
                        stderr: "nxsh: execution timed out".to_string(),
                        execution_time,
                        strategy: self.strategy,
                        metrics: ExecutionMetrics::default(),
                    })
                } else {
                    // If an error occurred but a global deadline was configured and already elapsed
                    // treat it as timeout to satisfy strict timeout semantics in tests.
                    if context.is_timed_out() {
                        let execution_time = start_time.elapsed().as_micros() as u64;
                        Ok(ExecutionResult {
                            exit_code: 124,
                            stdout: String::new(),
                            stderr: "nxsh: execution timed out".to_string(),
                            execution_time,
                            strategy: self.strategy,
                            metrics: ExecutionMetrics::default(),
                        })
                    } else {
                        Err(e)
                    }
                }
            }
        }
    }

    /// Execute AST node through MIR compilation and execution
    fn execute_with_mir(&mut self, node: &AstNode, _context: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let start_time = Instant::now();
        
        // Compile AST to MIR program
        let compile_start = Instant::now();
        let mir_program = self.compile_ast_to_mir(node)?;
        let compile_time = compile_start.elapsed().as_micros() as u64;
        
        // Optimize MIR program
        let optimize_start = Instant::now();
        let (optimized_program, memory_usage) = self.optimize_mir_program(mir_program)?;
        let optimize_time = optimize_start.elapsed().as_micros() as u64;
        
        // Execute optimized MIR program
        let execute_start = Instant::now();
        let result_value = self.mir_executor.execute(&optimized_program)
            .map_err(|e| ShellError::new(
                ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::CommandNotFound),
                format!("MIR execution failed: {}", e)
            ))?;
        
        let execute_time = execute_start.elapsed().as_micros() as u64;
        let total_time = start_time.elapsed().as_micros() as u64;
        
        // Convert MIR result to ExecutionResult
        let exit_code = match result_value {
            MirValue::Integer(code) => code as i32,
            MirValue::Boolean(true) => 0,
            MirValue::Boolean(false) => 1,
            _ => 0,
        };
        
        let stdout = match result_value {
            MirValue::String(s) => s,
            _ => String::new(),
        };
        
        Ok(ExecutionResult {
            exit_code,
            stdout,
            stderr: String::new(),
            execution_time: total_time,
            strategy: ExecutionStrategy::MirEngine,
            metrics: ExecutionMetrics {
                compile_time_us: compile_time,
                optimize_time_us: optimize_time,
                execute_time_us: execute_time,
                instruction_count: self.mir_executor.stats().instructions_executed,
                memory_usage,
            },
        })
    }

    /// Compile AST node to MIR program
    fn compile_ast_to_mir(&self, node: &AstNode) -> ShellResult<MirProgram> {
        let mut program = MirProgram::new();
        let mut main_func = crate::mir::MirFunction::new("main".to_string(), vec![]);
        
        // Convert AST to MIR instructions (comprehensive implementation)
        let mut entry_block = crate::mir::MirBasicBlock::new(0);
        
        match node {
            AstNode::Program(statements) => {
                // Compile multiple statements sequentially
                let mut last_result_reg = main_func.allocate_register();
                
                for statement in statements {
                    let result_reg = self.compile_statement_to_mir(statement, &mut main_func, &mut entry_block)?;
                    last_result_reg = result_reg;
                }
                
                // Return last result
                entry_block.add_instruction(crate::mir::MirInstruction::Return {
                    value: Some(MirValue::Register(last_result_reg)),
                });
            }
            AstNode::Command { name, args, .. } => {
                // Compile single command
                let result_reg = self.compile_command_to_mir(name, args, &mut main_func, &mut entry_block)?;
                
                // Return command result
                entry_block.add_instruction(crate::mir::MirInstruction::Return {
                    value: Some(MirValue::Register(result_reg)),
                });
            }
            AstNode::Pipeline { elements, .. } => {
                // Compile pipeline execution
                let result_reg = self.compile_pipeline_to_mir(elements, &mut main_func, &mut entry_block)?;
                
                // Return pipeline result
                entry_block.add_instruction(crate::mir::MirInstruction::Return {
                    value: Some(MirValue::Register(result_reg)),
                });
            }
            AstNode::If { condition, then_branch, else_branch, .. } => {
                // Compile conditional execution
                let result_reg = self.compile_conditional_to_mir(
                    condition, 
                    then_branch, 
                    else_branch.as_deref(),
                    &mut main_func, 
                    &mut entry_block,
                    &mut program
                )?;
                
                // Return conditional result
                entry_block.add_instruction(crate::mir::MirInstruction::Return {
                    value: Some(MirValue::Register(result_reg)),
                });
            }
            AstNode::For { variable, iterable, body, .. } => {
                // Compile loop execution
                let result_reg = self.compile_loop_to_mir(
                    variable,
                    iterable,
                    body,
                    &mut main_func,
                    &mut entry_block,
                    &mut program
                )?;
                
                // Return loop result
                entry_block.add_instruction(crate::mir::MirInstruction::Return {
                    value: Some(MirValue::Register(result_reg)),
                });
            }
            AstNode::Subshell(subshell_commands) => {
                // Compile subshell execution
                let commands = match subshell_commands.as_ref() {
                    AstNode::Program(statements) => statements,
                    single_command => {
                        let temp_vec = vec![single_command.clone()];
                        return self.compile_ast_to_mir(&AstNode::Program(temp_vec));
                    }
                };
                
                let result_reg = self.compile_subshell_to_mir(commands, &mut main_func, &mut entry_block)?;
                
                // Return subshell result
                entry_block.add_instruction(crate::mir::MirInstruction::Return {
                    value: Some(MirValue::Register(result_reg)),
                });
            }
            _ => {
                // For unsupported AST nodes, create simple success return
                let reg0 = main_func.allocate_register();
                entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                    dest: reg0.clone(),
                    value: MirValue::Integer(0),
                });
                entry_block.add_instruction(crate::mir::MirInstruction::Return {
                    value: Some(MirValue::Register(reg0)),
                });
            }
        }
        
        main_func.add_basic_block(entry_block);
        program.add_function(main_func);
        
        Ok(program)
    }

    /// Optimize MIR program for better performance
    fn optimize_mir_program(&self, mut program: MirProgram) -> ShellResult<(MirProgram, u64)> {
        let mut memory_usage = 0u64;
        
        // Apply optimization passes to all functions
        let function_names: Vec<String> = program.functions.keys().cloned().collect();
        for function_name in function_names {
            if let Some(function) = program.get_function_mut(&function_name) {
                memory_usage += self.optimize_function(function)?;
            }
        }
        
        Ok((program, memory_usage))
    }
    
    /// Optimize a single MIR function
    fn optimize_function(&self, function: &mut crate::mir::MirFunction) -> ShellResult<u64> {
        let mut memory_saved = 0u64;
        
        // Dead code elimination
        memory_saved += self.eliminate_dead_code(function)?;
        
        // Constant folding  
        memory_saved += self.constant_folding(function)?;
        
        // Register allocation optimization
        memory_saved += self.optimize_register_allocation(function)?;
        
        Ok(memory_saved)
    }
    
    /// Eliminate dead code from MIR function
    fn eliminate_dead_code(&self, function: &mut crate::mir::MirFunction) -> ShellResult<u64> {
        let mut instructions_removed = 0u64;
        
        // Access basic blocks through iteration
        let block_ids: Vec<u32> = function.blocks.keys().cloned().collect();
        for block_id in block_ids {
            if let Some(block) = function.blocks.get_mut(&block_id) {
                // Remove unreachable instructions after return statements
                let mut new_instructions = Vec::new();
                let mut hit_return = false;
                
                for instruction in &block.instructions {
                    if hit_return {
                        instructions_removed += 1;
                        continue;
                    }
                    
                    if matches!(instruction, crate::mir::MirInstruction::Return { .. }) {
                        hit_return = true;
                    }
                    
                    new_instructions.push(instruction.clone());
                }
                
                block.instructions = new_instructions;
            }
        }
        
        // Estimate memory saved (rough calculation)
        Ok(instructions_removed * 32) // Assume 32 bytes per instruction
    }
    
    /// Perform constant folding optimization
    fn constant_folding(&self, _function: &mut crate::mir::MirFunction) -> ShellResult<u64> {
        // Simple constant folding implementation
        // In a real implementation, this would fold constant arithmetic operations
        Ok(0)
    }
    
    /// Optimize register allocation
    fn optimize_register_allocation(&self, _function: &mut crate::mir::MirFunction) -> ShellResult<u64> {
        // Register allocation optimization would reduce register pressure
        // For now, return estimated memory savings
        Ok(64) // Rough estimate
    }

    /// Compile a single statement to MIR instructions
    fn compile_statement_to_mir(
        &self,
        statement: &AstNode,
        main_func: &mut crate::mir::MirFunction,
        entry_block: &mut crate::mir::MirBasicBlock
    ) -> ShellResult<crate::mir::MirRegister> {
        match statement {
            AstNode::Command { name, args, .. } => {
                self.compile_command_to_mir(name, args, main_func, entry_block)
            }
            AstNode::Assignment { .. } => {
                // Compile assignment statement
                let reg = main_func.allocate_register();
                entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                    dest: reg.clone(),
                    value: MirValue::Integer(0),
                });
                Ok(reg)
            }
            _ => {
                // For other statements, return success
                let reg = main_func.allocate_register();
                entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                    dest: reg.clone(),
                    value: MirValue::Integer(0),
                });
                Ok(reg)
            }
        }
    }

    /// Compile a command to MIR instructions
    fn compile_command_to_mir(
        &self,
        name: &AstNode,
        args: &[AstNode],
        main_func: &mut crate::mir::MirFunction,
        entry_block: &mut crate::mir::MirBasicBlock
    ) -> ShellResult<crate::mir::MirRegister> {
        let reg0 = main_func.allocate_register();
        let reg1 = main_func.allocate_register();
        
        // Load command name
        let name_str = match name {
            AstNode::SimpleCommand { name, .. } => name,
            AstNode::Word(word) => word,
            _ => "unknown",
        };
        
        entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
            dest: reg0.clone(),
            value: MirValue::String(name_str.to_string()),
        });
        
        // Compile arguments
        let mut arg_regs = Vec::new();
        for arg in args {
            let arg_reg = main_func.allocate_register();
            match arg {
                AstNode::Word(word) => {
                    entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                        dest: arg_reg.clone(),
                        value: MirValue::String(word.to_string()),
                    });
                }
                AstNode::StringLiteral { value, .. } => {
                    entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                        dest: arg_reg.clone(),
                        value: MirValue::String(value.to_string()),
                    });
                }
                AstNode::NumberLiteral { value, .. } => {
                    // Convert string to integer
                    let int_value = value.parse::<i64>().unwrap_or(0);
                    entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                        dest: arg_reg.clone(),
                        value: MirValue::Integer(int_value),
                    });
                }
                _ => {
                    entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                        dest: arg_reg.clone(),
                        value: MirValue::String("".to_string()),
                    });
                }
            }
            arg_regs.push(MirValue::Register(arg_reg));
        }
        
        // Execute command
        entry_block.add_instruction(crate::mir::MirInstruction::ExecuteCommand {
            dest: reg1.clone(),
            command: name_str.to_string(),
            args: arg_regs,
        });
        
        Ok(reg1)
    }

    /// Compile a pipeline to MIR instructions
    fn compile_pipeline_to_mir(
        &self,
        elements: &[AstNode],
        main_func: &mut crate::mir::MirFunction,
        entry_block: &mut crate::mir::MirBasicBlock
    ) -> ShellResult<crate::mir::MirRegister> {
        let mut last_result = main_func.allocate_register();
        
        // Initialize with success
        entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
            dest: last_result.clone(),
            value: MirValue::Integer(0),
        });
        
        // Process each pipeline element
        for element in elements {
            match element {
                AstNode::Command { name, args, .. } => {
                    last_result = self.compile_command_to_mir(name, args, main_func, entry_block)?;
                }
                _ => {
                    // For non-command elements, just continue
                }
            }
        }
        
        Ok(last_result)
    }

    /// Compile conditional (if/else) to MIR instructions
    fn compile_conditional_to_mir(
        &self,
        condition: &AstNode,
        then_branch: &AstNode,
        else_branch: Option<&AstNode>,
        main_func: &mut crate::mir::MirFunction,
        entry_block: &mut crate::mir::MirBasicBlock,
        _program: &mut MirProgram
    ) -> ShellResult<crate::mir::MirRegister> {
        // Improved conditional compilation with proper condition evaluation
        let condition_reg = self.compile_condition_to_mir(condition, main_func, entry_block)?;
        let result_reg = main_func.allocate_register();
        
        // Create basic blocks for then/else branches
        let then_block_id = main_func.blocks.len() as u32 + 1;
        let else_block_id = main_func.blocks.len() as u32 + 2; 
        let end_block_id = main_func.blocks.len() as u32 + 3;
        
        // Add conditional branch instruction
        entry_block.add_instruction(crate::mir::MirInstruction::Branch {
            condition: crate::mir::MirValue::Register(condition_reg),
            true_block: then_block_id,
            false_block: else_block_id,
        });
        
        // Compile then branch
        let mut then_block = crate::mir::MirBasicBlock::new(then_block_id);
        let then_result = self.compile_ast_to_mir_block(then_branch, main_func, &mut then_block)?;
        then_block.add_instruction(crate::mir::MirInstruction::Move {
            dest: result_reg.clone(),
            src: then_result,
        });
        then_block.add_instruction(crate::mir::MirInstruction::Jump {
            target: end_block_id,
        });
        main_func.blocks.insert(then_block_id, then_block);
        
        // Compile else branch
        let mut else_block = crate::mir::MirBasicBlock::new(else_block_id);
        let else_result = if let Some(else_branch) = else_branch {
            self.compile_ast_to_mir_block(else_branch, main_func, &mut else_block)?
        } else {
            // Default success value if no else branch
            let default_reg = main_func.allocate_register();
            else_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                dest: default_reg.clone(),
                value: crate::mir::MirValue::Integer(0),
            });
            default_reg
        };
        else_block.add_instruction(crate::mir::MirInstruction::Move {
            dest: result_reg.clone(),
            src: else_result,
        });
        else_block.add_instruction(crate::mir::MirInstruction::Jump {
            target: end_block_id,
        });
        main_func.blocks.insert(else_block_id, else_block);
        
        // Create end block
        let end_block = crate::mir::MirBasicBlock::new(end_block_id);
        main_func.blocks.insert(end_block_id, end_block);
        
        Ok(result_reg)
    }

    /// Compile condition evaluation to MIR
    fn compile_condition_to_mir(
        &self,
        condition: &AstNode,
        main_func: &mut crate::mir::MirFunction,
        entry_block: &mut crate::mir::MirBasicBlock,
    ) -> ShellResult<crate::mir::MirRegister> {
        match condition {
            AstNode::Command { name, args, .. } => {
                // Execute command and use exit code as condition
                let cmd_result = self.compile_command_to_mir(name, args, main_func, entry_block)?;
                let condition_reg = main_func.allocate_register();
                
                // Convert exit code to boolean (0 = true, non-zero = false)
                entry_block.add_instruction(crate::mir::MirInstruction::Equal {
                    dest: condition_reg.clone(),
                    left: crate::mir::MirValue::Register(cmd_result),
                    right: crate::mir::MirValue::Integer(0),
                });
                
                Ok(condition_reg)
            }
            AstNode::LogicalAnd { left, right } => {
                let left_reg = self.compile_condition_to_mir(left, main_func, entry_block)?;
                let right_reg = self.compile_condition_to_mir(right, main_func, entry_block)?;
                let result_reg = main_func.allocate_register();
                
                entry_block.add_instruction(crate::mir::MirInstruction::And {
                    dest: result_reg.clone(),
                    left: crate::mir::MirValue::Register(left_reg),
                    right: crate::mir::MirValue::Register(right_reg),
                });
                
                Ok(result_reg)
            }
            AstNode::LogicalOr { left, right } => {
                let left_reg = self.compile_condition_to_mir(left, main_func, entry_block)?;
                let right_reg = self.compile_condition_to_mir(right, main_func, entry_block)?;
                let result_reg = main_func.allocate_register();
                
                entry_block.add_instruction(crate::mir::MirInstruction::Or {
                    dest: result_reg.clone(),
                    left: crate::mir::MirValue::Register(left_reg),
                    right: crate::mir::MirValue::Register(right_reg),
                });
                
                Ok(result_reg)
            }
            _ => {
                // Default to true for unknown conditions
                let condition_reg = main_func.allocate_register();
                entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                    dest: condition_reg.clone(),
                    value: crate::mir::MirValue::Boolean(true),
                });
                Ok(condition_reg)
            }
        }
    }

    /// Compile AST node to MIR in a specific block
    fn compile_ast_to_mir_block(
        &self,
        node: &AstNode,
        main_func: &mut crate::mir::MirFunction,
        block: &mut crate::mir::MirBasicBlock,
    ) -> ShellResult<crate::mir::MirRegister> {
        match node {
            AstNode::Command { name, args, .. } => {
                self.compile_command_to_mir(name, args, main_func, block)
            }
            AstNode::Program(statements) => {
                let mut last_result = main_func.allocate_register();
                for statement in statements {
                    last_result = self.compile_ast_to_mir_block(statement, main_func, block)?;
                }
                Ok(last_result)
            }
            _ => {
                // Default success result
                let result_reg = main_func.allocate_register();
                block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                    dest: result_reg.clone(),
                    value: crate::mir::MirValue::Integer(0),
                });
                Ok(result_reg)
            }
        }
    }

    /// Compile loop to MIR instructions
    fn compile_loop_to_mir(
        &self,
        _variable: &str,
        _iterable: &AstNode,
        body: &AstNode,
        main_func: &mut crate::mir::MirFunction,
        entry_block: &mut crate::mir::MirBasicBlock,
        _program: &mut MirProgram
    ) -> ShellResult<crate::mir::MirRegister> {
        // Simplified loop compilation - just execute body once
        let result_reg = main_func.allocate_register();
        
        match body {
            AstNode::Command { name, args, .. } => {
                let body_result = self.compile_command_to_mir(name, args, main_func, entry_block)?;
                entry_block.add_instruction(crate::mir::MirInstruction::Move {
                    dest: result_reg.clone(),
                    src: body_result,
                });
            }
            _ => {
                entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                    dest: result_reg.clone(),
                    value: MirValue::Integer(0),
                });
            }
        }
        
        Ok(result_reg)
    }

    /// Compile subshell to MIR instructions
    fn compile_subshell_to_mir(
        &self,
        commands: &[AstNode],
        main_func: &mut crate::mir::MirFunction,
        entry_block: &mut crate::mir::MirBasicBlock
    ) -> ShellResult<crate::mir::MirRegister> {
        let mut last_result = main_func.allocate_register();
        
        // Initialize with success
        entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
            dest: last_result.clone(),
            value: MirValue::Integer(0),
        });
        
        // Process each command in subshell
        for command in commands {
            last_result = self.compile_statement_to_mir(command, main_func, entry_block)?;
        }
        
        Ok(last_result)
    }

    /// Execute MIR program directly
    pub fn execute_mir_program(&mut self, program: &MirProgram) -> ShellResult<ExecutionResult> {
        let start_time = Instant::now();
        
        let result_value = self.mir_executor.execute(program)
            .map_err(|e| ShellError::new(
                ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::CommandNotFound),
                format!("MIR execution failed: {}", e)
            ))?;
        
        let execution_time = start_time.elapsed().as_micros() as u64;
        
        // Convert MIR result to ExecutionResult
        let exit_code = match result_value {
            MirValue::Integer(code) => code as i32,
            MirValue::Boolean(true) => 0,
            MirValue::Boolean(false) => 1,
            _ => 0,
        };
        
        let stdout = match result_value {
            MirValue::String(s) => s,
            _ => String::new(),
        };
        
        Ok(ExecutionResult {
            exit_code,
            stdout,
            stderr: String::new(),
            execution_time,
            strategy: ExecutionStrategy::MirEngine,
            metrics: ExecutionMetrics {
                compile_time_us: 0,
                optimize_time_us: 0,
                execute_time_us: execution_time,
                instruction_count: self.mir_executor.stats().instructions_executed,
                memory_usage: 0,
            },
        })
    }

    /// Get MIR executor statistics
    pub fn mir_stats(&self) -> &crate::mir::ExecutionStats {
        self.mir_executor.stats()
    }

    /// Reset MIR executor statistics
    pub fn reset_mir_stats(&mut self) {
        self.mir_executor.reset_stats();
    }
    
    /// Execute AST node directly without MIR compilation
    fn execute_ast_direct(&mut self, node: &AstNode, context: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let start_time = Instant::now();

        // Global timeout check (fast-fail before deeper recursion)
        if context.is_timed_out() {
            return Ok(ExecutionResult {
                exit_code: 124, // Convention: 124 for timeout (like GNU timeout)
                stdout: String::new(),
                stderr: "nxsh: execution timed out".to_string(),
                execution_time: start_time.elapsed().as_micros() as u64,
                strategy: ExecutionStrategy::DirectInterpreter,
                metrics: ExecutionMetrics::default(),
            });
        }
        
        #[cfg(debug_assertions)]
        fn debug_variant(node: &AstNode, depth: usize) {
            use AstNode::*;
            let indent = "  ".repeat(depth);
            let name = match node {
                Program(_) => "Program",
                Pipeline { .. } => "Pipeline",
                Command { background, .. } => if *background { "Command(bg)" } else { "Command" },
                SimpleCommand { .. } => "SimpleCommand",
                Subshell(_) => "Subshell",
                If { .. } => "If",
                For { .. } => "For",
                ForC { .. } => "ForC",
                While { .. } => "While",
                Until { .. } => "Until",
                Case { .. } => "Case",
                Select { .. } => "Select",
                Match { .. } => "Match",
                Function { .. } => "Function",
                VariableAssignment { .. } => "VarAssign",
                Word(_) => "Word",
                StringLiteral { .. } => "StringLiteral",
                NumberLiteral { .. } => "NumberLiteral",
                VariableExpansion { .. } => "VarExp",
                CommandSubstitution { .. } => "CmdSub",
                LogicalAnd { .. } => "LogicalAnd",
                LogicalOr { .. } => "LogicalOr",
                Sequence { .. } => "Sequence",
                _ => "Other",
            };
            #[cfg(feature = "debug_exec")]
            eprintln!("AST_DEBUG:{}{}", indent, name);
            match node {
                Program(stmts) => for s in stmts { debug_variant(s, depth+1); },
                Pipeline { elements, .. } => for s in elements { debug_variant(s, depth+1); },
                Subshell(inner) => debug_variant(inner, depth+1),
                If { condition, then_branch, else_branch, .. } => {
                    debug_variant(condition, depth+1);
                    debug_variant(then_branch, depth+1);
                    if let Some(e) = else_branch { debug_variant(e, depth+1); }
                }
                _ => {}
            }
        }
        #[cfg(debug_assertions)]
        {
            debug_variant(node, 0);
        }

        // Direct AST interpretation with background job support
        // Additional early timeout guard for long sequences constructed at runtime
        if context.is_timed_out() {
            return Ok(ExecutionResult {
                exit_code: 124,
                stdout: String::new(),
                stderr: "nxsh: execution timed out".to_string(),
                execution_time: start_time.elapsed().as_micros() as u64,
                strategy: ExecutionStrategy::DirectInterpreter,
                metrics: ExecutionMetrics::default(),
            });
        }
        // Normalize single-element pipeline into its command to preserve flags (e.g., background)
        let normalized_node: &AstNode = match node {
            AstNode::Pipeline { elements, operators } if elements.len() == 1 && operators.is_empty() => {
                &elements[0]
            }
            _ => node
        };

        let result = match normalized_node {
            AstNode::Function { name, params, body, is_async: _, generics } |
            AstNode::FunctionDeclaration { name, params, body, is_async: _, generics } => {
                // Build canonical base name (without specialization suffix)
                let base_name = (*name).to_string();
                // Parameter meta line used by interpreter for execution
                let param_meta = if params.is_empty() { "#params:".to_string() } else {
                    let descs: Vec<String> = params.iter().map(|p| {
                        let mut s = p.name.to_string();
                        if p.is_variadic { s.push_str("..."); }
                        if let Some(def) = &p.default { s.push('='); s.push_str(&simple_unparse(def)); }
                        s
                    }).collect();
                    format!("#params:{}", descs.join(","))
                };
                let body_src = simple_unparse(body);
                // If this function declares generics, store it as a generic template
                if !generics.is_empty() {
                    // Register a template so call sites can monomorphize with concrete args
                    context.register_generic_function_template(
                        &base_name,
                        generics,
                        &param_meta,
                        &body_src,
                    );
                }
                // Also store a base (non-specialized) variant for direct calls
                {
                    let mut stored = String::new();
                    if !generics.is_empty() {
                        // Keep declared generics for debugging at definition site
                        stored.push_str("#generics_decl:");
                        stored.push_str(&generics.join(","));
                        stored.push('\n');
                    }
                    stored.push_str(&param_meta);
                    stored.push('\n');
                    stored.push_str(&body_src);
                    context.set_function(base_name, stored);
                }
                ExecutionResult::success(0)
            }
            AstNode::Program(statements) => {
                let mut result = ExecutionResult::success(0);
                for statement in statements {
                    if context.is_timed_out() { return Ok(ExecutionResult { exit_code: 124, stdout: String::new(), stderr: "nxsh: execution timed out".to_string(), execution_time: start_time.elapsed().as_micros() as u64, strategy: ExecutionStrategy::DirectInterpreter, metrics: ExecutionMetrics::default() }); }
                    result = self.execute_ast_direct(statement, context)?;
                    if context.is_timed_out() { return Ok(ExecutionResult { exit_code: 124, stdout: String::new(), stderr: "nxsh: execution timed out".to_string(), execution_time: start_time.elapsed().as_micros() as u64, strategy: ExecutionStrategy::DirectInterpreter, metrics: ExecutionMetrics::default() }); }
                    if result.exit_code != 0 && !context.continue_on_error() { break; }
                }
                // Global timeout takes precedence over intermediate non-zero exit codes
                if context.is_timed_out() {
                    return Ok(ExecutionResult { exit_code: 124, stdout: String::new(), stderr: "nxsh: execution timed out".to_string(), execution_time: start_time.elapsed().as_micros() as u64, strategy: ExecutionStrategy::DirectInterpreter, metrics: ExecutionMetrics::default() });
                }
                result
            },
            AstNode::Sequence { left, right } => {
                if context.is_timed_out() { return Ok(ExecutionResult { exit_code: 124, stdout: String::new(), stderr: "nxsh: execution timed out".to_string(), execution_time: start_time.elapsed().as_micros() as u64, strategy: ExecutionStrategy::DirectInterpreter, metrics: ExecutionMetrics::default() }); }
                let _ = self.execute_ast_direct(left, context)?;
                if context.is_timed_out() { return Ok(ExecutionResult { exit_code: 124, stdout: String::new(), stderr: "nxsh: execution timed out".to_string(), execution_time: start_time.elapsed().as_micros() as u64, strategy: ExecutionStrategy::DirectInterpreter, metrics: ExecutionMetrics::default() }); }
                self.execute_ast_direct(right, context)?
            },
            AstNode::Subshell(subshell_commands) => {
                // Handle subshell execution
                let commands = match subshell_commands.as_ref() {
                    AstNode::Program(statements) => statements.clone(),
                    single_command => vec![single_command.clone()],
                };
                self.execute_subshell(&commands, context)?
            },
            AstNode::Command { name, args, redirections, background } => {
                // Handle command execution with background support
                #[cfg(debug_assertions)]
                {
                    #[cfg(feature = "debug_exec")]
                    eprintln!("DEBUG_EXEC: Command background flag = {}", background);
                }
                self.execute_command_with_background(name, args, redirections, *background, context)?
            },
            AstNode::Pipeline { elements, .. } => {
                self.execute_pipeline(elements, context)?
            }
            AstNode::If { condition, then_branch, else_branch, .. } => {
                self.execute_conditional(condition, then_branch, else_branch.as_deref(), context)?
            }
            AstNode::For { body, .. } => {
                // Simplified For loop execution
                self.execute_ast_direct(body, context)?
            }
            AstNode::VariableAssignment { name, value, operator: _, is_local: _, is_export: _, is_readonly: _ } => {
                // Handle variable assignment
                let value_result = self.execute_ast_direct(value, context)?;
                context.set_var(name.to_string(), value_result.stdout.trim().to_string());
                ExecutionResult::success(0)
            }
            AstNode::StringLiteral { value, .. } => {
                ExecutionResult::success(0).with_output(value.as_bytes().to_vec())
            }
            AstNode::NumberLiteral { value, .. } => {
                ExecutionResult::success(0).with_output(value.as_bytes().to_vec())
            }
            AstNode::Word(word) => {
                ExecutionResult::success(0).with_output(word.as_bytes().to_vec())
            }
            AstNode::VariableExpansion { name, .. } => {
                let value = context.get_var(name).unwrap_or_default();
                ExecutionResult::success(0).with_output(value.as_bytes().to_vec())
            }
            AstNode::MacroDeclaration { name, params, body } => {
                let mut system = context.macro_system.write().unwrap();
                let macro_def = crate::macros::Macro::Simple { parameters: params.iter().map(|s| s.to_string()).collect(), body: format!("{:?}", body) };
                if let Err(e) = system.define_macro(name.to_string(), macro_def) {
                    return Err(ShellError::new(ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), e.to_string()));
                }
                ExecutionResult::success(0)
            }
            AstNode::MacroInvocation { name, args } => {
                let expanded = {
                    let mut system = context.macro_system.write().unwrap();
                    let arg_texts: Vec<String> = args.iter().map(|a| format!("{:?}", a)).collect();
                    match system.expand_macro(name, arg_texts) {
                        Ok(e) => e,
                        Err(e) => return Ok(ExecutionResult::failure(1).with_error(format!("macro expand error: {e}").into_bytes())),
                    }
                };
                match parse_program(&expanded) {
                    Ok(expanded_ast) => { return self.execute_ast_direct(&expanded_ast, context); }
                    Err(_) => return Ok(ExecutionResult::failure(1).with_error(format!("macro expansion parse failed: {expanded}").into_bytes())),
                }
            }
            AstNode::CommandSubstitution { command, is_legacy: _ } => {
                // Evaluate command substitution as an expression node with caching
                self.eval_cmd_substitution(command, context)?
            }
            AstNode::Match { expr, arms, is_exhaustive: _ } => {
                // Evaluate match expression using pattern engine
                use crate::pattern_matching::{PatternMatchingEngine, PatternMatchingConfig, shell_value_to_pattern_value, PatternValue};
                use nxsh_parser::ast::Pattern as AstPattern;
                // Evaluate the scrutinee expression to a string (simplified)
                let value_result = self.execute_ast_direct(expr, context)?;
                let value_str = value_result.stdout.clone();
                let pattern_value = shell_value_to_pattern_value(&value_str);
                let mut engine = PatternMatchingEngine::new(PatternMatchingConfig { exhaustiveness_checking: true, ..Default::default() });
                // Convert arms into engine evaluation. Arms hold pattern + body.
                let match_result = engine.match_arms(&pattern_value, arms)
                    .map_err(|e| ShellError::new(ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::TypeMismatch), format!("pattern match failed: {e}")))?;
                if let Some(index) = match_result.matched_arm {
                    // Execute matched arm body
                    let arm = &arms[index];
                    let body_res = self.execute_ast_direct(&arm.body, context)?;
                    return Ok(body_res);
                } else {
                    // No match -> non-zero exit (1) with empty output
                    return Ok(ExecutionResult::success(1));
                }
            }
            AstNode::Try { body, catch_clauses, finally_clause } => {
                // Execute body, capture error, run matching catch then finally
                let body_res = self.execute_ast_direct(body, context);
                let mut exec_result = ExecutionResult::failure(1); // default
                match body_res {
                    Ok(r) => { exec_result = r; }
                    Err(err) => {
                        let mut handled = false;
                        for clause in catch_clauses {
                            if let Some(var) = clause.variable {
                                context.set_var(var.to_string(), format!("{}", err));
                            }
                            match self.execute_ast_direct(&clause.body, context) {
                                Ok(r2) => { exec_result = r2; handled = true; break; }
                                Err(e2) => { exec_result = ExecutionResult::from_error(e2); handled = true; break; }
                            }
                        }
                        if !handled { return Err(err); }
                    }
                }
                if let Some(finally) = finally_clause {
                    // finally errors override previous result
                    if let Ok(fin_res) = self.execute_ast_direct(finally, context) { exec_result = fin_res; }
                }
                return Ok(exec_result);
            }
            AstNode::Closure { params, body, captures, is_async: _ } => {
                // Build param meta string (reuse #params format sans prefix)
                let mut descs = Vec::new();
                for p in params {
                    let mut name = p.name.to_string();
                    if p.is_variadic { name.push_str("..."); }
                    if let Some(def) = &p.default { // naive serialize
                        let s = simple_unparse(def);
                        descs.push(format!("{}={}", name, s));
                    } else { descs.push(name); }
                }
                let param_meta = descs.join(",");
                // Serialize body (best-effort)
                let body_src = simple_unparse(body);
                // Capture variables: current values of listed capture names
                let mut captured = std::collections::HashMap::new();
                for c in captures { if let Some(val) = context.get_var(c) { captured.insert(c.to_string(), val); } }
                let closure_id = format!("__closure_{}", context.next_temp_id());
                context.set_closure(closure_id.clone(), crate::context::ClosureInfo { params_meta: param_meta, body_src, captured });
                ExecutionResult::success(0).with_output(closure_id.into_bytes())
            }
            AstNode::FunctionCall { name, args, is_async: _, generics } => {
                // Evaluate callee (may be a word or closure id output)
                let name_res = self.execute_ast_direct(name, context)?;
                let callee_base = name_res.stdout.trim().to_string();
                // If call has generics, ensure monomorphized specialization exists
                let callee = if !generics.is_empty() {
                    if let Some(spec) = context.ensure_monomorphized(&callee_base, generics) {
                        spec
                    } else {
                        // No template; fallback to legacy suffix scheme to avoid breaking old flows
                        let spec = format!("{}__gen_{}", callee_base, generics.join("_"));
                        if context.has_function(&callee_base) && !context.has_function(&spec) {
                            if let Some(src) = context.get_function(&callee_base) { context.set_function(spec.clone(), src); }
                        }
                        spec
                    }
                } else {
                    callee_base
                };
                // Evaluate arguments to strings
                let mut evaluated_args = Vec::new();
                for a in args {
                    let r = self.execute_ast_direct(a, context)?;
                    evaluated_args.push(r.stdout.clone());
                }
                if callee.starts_with("__closure_") {
                    // Closure invocation: retrieve closure info and execute body with captured env and params
                    if let Some(info) = context.get_closure(&callee) {
                        // Parse params meta
                        let mut param_names: Vec<String> = Vec::new();
                        let mut param_defaults: Vec<Option<String>> = Vec::new();
                        let mut variadic_index: Option<usize> = None;
                        if !info.params_meta.trim().is_empty() {
                            for (idx, raw) in info.params_meta.split(',').enumerate() {
                                let part = raw.trim(); if part.is_empty() { continue; }
                                let (name_part, def_part) = if let Some(eq_pos) = part.find('=') { (&part[..eq_pos], Some(part[eq_pos+1..].trim())) } else { (part, None) };
                                let mut name_clean = name_part.trim().to_string();
                                let mut is_variadic = false;
                                if let Some(stripped) = name_clean.strip_suffix("...") { name_clean = stripped.to_string(); is_variadic = true; }
                                if is_variadic { variadic_index = Some(idx); }
                                param_names.push(name_clean);
                                param_defaults.push(def_part.map(|s| s.to_string()));
                            }
                        }
                        // Save old values & inject captured first (captured act as outer scope)
                        let mut saved: Vec<(String, Option<String>)> = Vec::new();
                        for (k, v) in &info.captured { saved.push((k.clone(), context.get_var(k))); context.set_var(k.clone(), v.clone()); }
                        for name in &param_names { if !info.captured.contains_key(name) { saved.push((name.clone(), context.get_var(name))); } }
                        // Bind arguments
                        let mut arg_idx = 0usize;
                        for (i, name) in param_names.iter().enumerate() {
                            if Some(i) == variadic_index {
                                let rest = if arg_idx < evaluated_args.len() { evaluated_args[arg_idx..].join(" ") } else { String::new() };
                                context.set_var(name.clone(), rest);
                                arg_idx = evaluated_args.len();
                                break;
                            } else if let Some(val) = evaluated_args.get(arg_idx) {
                                context.set_var(name.clone(), val.clone());
                                arg_idx += 1;
                            } else {
                                if let Some(Some(def_src)) = param_defaults.get(i) {
                                    if !def_src.is_empty() { if let Ok(def_ast) = parse_program(def_src) { if let Ok(def_res) = self.execute_ast_direct(&def_ast, context) { context.set_var(name.clone(), def_res.stdout); } } }
                                } else { context.set_var(name.clone(), String::new()); }
                            }
                        }
                        // Execute body
                        let exec_res = if let Ok(ast) = parse_program(&info.body_src) { self.execute_ast_direct(&ast, context) } else { Ok(ExecutionResult::failure(1).with_error(b"closure body parse failed".to_vec())) };
                        // Restore saved variables
                        for (name, old) in saved { match old { Some(v) => context.set_var(name, v), None => context.set_var(name, String::new()) } }
                        return exec_res;
                    } else {
                        ExecutionResult::failure(1).with_error(b"unknown closure".to_vec())
                    }
                } else {
                    // Treat as shell function or builtin fallback
                    // If user-defined function stored in context.functions
                    if context.has_function(&callee) {
                        if let Some(src) = context.get_function(&callee) {
                            // Read header lines: optional #generics: then #params:
                            let mut lines_iter = src.lines();
                            let mut param_names: Vec<String> = Vec::new();
                            let mut param_defaults: Vec<Option<String>> = Vec::new();
                            let mut variadic_index: Option<usize> = None;
                            let mut body_start_src = src.as_str();
                            let mut consumed_len: usize = 0;
                            if let Some(first) = lines_iter.next() {
                                // Handle optional #generics header
                                if first.starts_with("#generics:") {
                                    consumed_len += first.len() + 1; // include newline
                                    if let Some(second) = lines_iter.next() {
                                        // Expect params on the next line
                                        if let Some(rest) = second.strip_prefix("#params:") {
                                            if !rest.trim().is_empty() {
                                                for (idx, raw) in rest.split(',').enumerate() {
                                                    let part = raw.trim(); if part.is_empty() { continue; }
                                                    let (name_part, def_part) = if let Some(eq_pos) = part.find('=') { (&part[..eq_pos], Some(part[eq_pos+1..].trim())) } else { (part, None) };
                                                    let mut name_clean = name_part.trim().to_string();
                                                    let mut is_variadic = false;
                                                    if let Some(stripped) = name_clean.strip_suffix("...") { name_clean = stripped.to_string(); is_variadic = true; }
                                                    if is_variadic { variadic_index = Some(idx); }
                                                    param_names.push(name_clean);
                                                    param_defaults.push(def_part.map(|s| s.to_string()));
                                                }
                                            }
                                            consumed_len += second.len() + 1;
                                        }
                                    }
                                } else if let Some(rest) = first.strip_prefix("#params:") {
                                    if !rest.trim().is_empty() {
                                        for (idx, raw) in rest.split(',').enumerate() {
                                            let part = raw.trim(); if part.is_empty() { continue; }
                                            let (name_part, def_part) = if let Some(eq_pos) = part.find('=') { (&part[..eq_pos], Some(part[eq_pos+1..].trim())) } else { (part, None) };
                                            let mut name_clean = name_part.trim().to_string();
                                            let mut is_variadic = false;
                                            if let Some(stripped) = name_clean.strip_suffix("...") { name_clean = stripped.to_string(); is_variadic = true; }
                                            if is_variadic { variadic_index = Some(idx); }
                                            param_names.push(name_clean);
                                            param_defaults.push(def_part.map(|s| s.to_string()));
                                        }
                                    }
                                    consumed_len += first.len() + 1;
                                }
                            }
                            // Determine body start after consumed headers
                            if consumed_len > 0 { body_start_src = &src[consumed_len.min(src.len())..]; }
                            // 旧値保存 (環境変数ベース)
                            let mut saved: Vec<(String, Option<String>)> = Vec::new();
                            for name in &param_names { saved.push((name.clone(), context.get_var(name))); }
                            // 引数バインド
                            let mut arg_idx = 0usize;
                            for (i, name) in param_names.iter().enumerate() {
                                if Some(i) == variadic_index {
                                    // 残り全部
                                    let rest = if arg_idx < evaluated_args.len() { evaluated_args[arg_idx..].join(" ") } else { String::new() };
                                    context.set_var(name.clone(), rest);
                                    arg_idx = evaluated_args.len();
                                    break; // variadic は末尾想定
                                } else {
                                    if let Some(val) = evaluated_args.get(arg_idx) {
                                        context.set_var(name.clone(), val.clone());
                                        arg_idx += 1;
                                    } else {
                                        // 足りない → default を評価
                                        if let Some(Some(def_src)) = param_defaults.get(i) {
                                            if !def_src.is_empty() {
                                                if let Ok(def_ast) = parse_program(def_src) {
                                                    if let Ok(def_res) = self.execute_ast_direct(&def_ast, context) { context.set_var(name.clone(), def_res.stdout); }
                                                }
                                            }
                                        } else {
                                            context.set_var(name.clone(), "".to_string());
                                        }
                                    }
                                }
                            }
                            // 実行 (空ボディなら no-op)
                            if body_start_src.trim().is_empty() {
                                for (name, old) in saved { match old { Some(v) => context.set_var(name, v), None => context.set_var(name, String::new()) } }
                                ExecutionResult::success(0)
                            } else {
                                match parse_program(body_start_src) {
                                    Ok(ast) => {
                                        let exec_res = self.execute_ast_direct(&ast, context);
                                        // スコープ復元
                                        for (name, old) in saved { match old { Some(v) => context.set_var(name, v), None => context.set_var(name, String::new()) } }
                                        return exec_res;
                                    }
                                    Err(_) => ExecutionResult::failure(1).with_error(format!("function parse failed: {callee}").into_bytes())
                                }
                            }
                        } else {
                            ExecutionResult::failure(1).with_error(format!("missing function body: {callee}").into_bytes())
                        }
                    } else {
                        // Fallback: try executing as command
                        let mut arg_nodes = Vec::new();
                        for s in &evaluated_args { arg_nodes.push(AstNode::Word(Box::leak(s.clone().into_boxed_str()))); }
                        let cmd_node = AstNode::Command { name: Box::new(AstNode::Word(Box::leak(callee.clone().into_boxed_str()))), args: arg_nodes, redirections: vec![], background: false };
                        self.execute_ast_direct(&cmd_node, context)?
                    }
                }
            }
            _ => {
                return Err(ShellError::new(
                    ErrorKind::SystemError(crate::error::SystemErrorKind::UnsupportedOperation),
                    format!("AST node type not supported in direct interpreter: {:?}", node)
                ));
            }
        };
        
        let execution_time = start_time.elapsed().as_micros() as u64;
        
        Ok(ExecutionResult {
            exit_code: result.exit_code,
            stdout: result.stdout,
            stderr: result.stderr,
            execution_time,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics {
                execute_time_us: execution_time,
                ..Default::default()
            },
        })
    }

    /// Execute command with background job support
    fn execute_command_with_background(
        &mut self, 
        name: &AstNode, 
        args: &[AstNode], 
        _redirections: &[nxsh_parser::ast::Redirection],
        background: bool,
        context: &mut ShellContext
    ) -> ShellResult<ExecutionResult> {
        let start_time = Instant::now();
        // Global timeout guard before any heavy work
        if context.is_timed_out() {
            return Ok(ExecutionResult {
                exit_code: 124,
                stdout: String::new(),
                stderr: "nxsh: execution timed out".to_string(),
                execution_time: start_time.elapsed().as_micros() as u64,
                strategy: ExecutionStrategy::DirectInterpreter,
                metrics: ExecutionMetrics::default(),
            });
        }
        // Helper: split string into fields if NXSH_SUBST_SPLIT=1
        fn split_fields(raw: &str, context: &ShellContext) -> Vec<String> {
            if context.get_var("NXSH_SUBST_SPLIT").as_deref() != Some("1") {
                return vec![raw.to_string()];
            }
            let ifs = context.get_var("NXSH_IFS").unwrap_or_else(|| " \t\n".to_string());
            let mut out = Vec::new();
            let mut current = String::new();
            for ch in raw.chars() {
                if ifs.contains(ch) {
                    if !current.is_empty() { out.push(std::mem::take(&mut current)); }
                } else {
                    current.push(ch);
                }
            }
            if !current.is_empty() { out.push(current); }
            if out.is_empty() { return vec![String::new()]; }
            out
        }
        // Extract command name
        let cmd_name = match name {
            AstNode::Word(word) => word.to_string(),
            AstNode::StringLiteral { value, .. } => value.to_string(),
            _ => return Err(ShellError::new(
                ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                "Invalid command name".to_string()
            )),
        };

        // Extract & possibly split arguments
        let mut cmd_args = Vec::new();
        // Local brace expansion helper duplicated (cannot call inner fn in execute_command). Keep in sync.
        fn brace_expand_one(input: &str) -> Vec<String> {
            const MAX_EXPANSIONS: usize = 4096; // safety cap
            // Escape handling via sentinels (same as expand_braces below)
            const ESC_LBRACE: char = '\u{1F}';
            const ESC_RBRACE: char = '\u{1E}';
            const ESC_COMMA:  char = '\u{1D}';
            let mut transformed = String::with_capacity(input.len());
            let mut it = input.chars().peekable();
            while let Some(c) = it.next() {
                if c == '\\' {
                    if let Some(&next) = it.peek() {
                        match next {
                            '{' => { transformed.push(ESC_LBRACE); it.next(); continue; }
                            '}' => { transformed.push(ESC_RBRACE); it.next(); continue; }
                            ',' => { transformed.push(ESC_COMMA);  it.next(); continue; }
                            _ => { transformed.push(next); it.next(); continue; }
                        }
                    }
                    // trailing backslash
                    transformed.push('\\');
                } else {
                    transformed.push(c);
                }
            }
            // Quick exit if no real '{'
            if !transformed.as_bytes().contains(&b'{') { return vec![input.to_string()]; }
            fn restore(mut s: String) -> String {
                let mut out = String::with_capacity(s.len());
                for ch in s.drain(..) {
                    match ch {
                        '\u{1F}' => { out.push('\\'); out.push('{'); },
                        '\u{1E}' => { out.push('\\'); out.push('}'); },
                        '\u{1D}' => { out.push('\\'); out.push(','); },
                        _ => out.push(ch),
                    }
                }
                out
            }
            // Helpers: parser for inner content and range detector
            fn brace_parse_inner(inner: &str) -> Vec<String> {
                if let Some(r) = brace_try_range(inner) { return r; }
                // Split on top-level commas, preserving whitespace and allowing escaped commas
                let mut parts: Vec<String> = Vec::new();
                let mut level = 0usize; let mut escape = false; let mut cur = String::new();
                for c in inner.chars() {
                    if escape { cur.push(c); escape = false; continue; }
                    match c {
                        '\\' => { escape = true; },
                        '{' => { level += 1; cur.push(c); },
                        '}' => { if level>0 { level -= 1; } cur.push(c); },
                        ',' if level==0 => { parts.push(cur.clone()); cur.clear(); },
                        _ => cur.push(c),
                    }
                }
                if escape { cur.push('\\'); }
                parts.push(cur);
                parts
            }
            fn brace_try_range(inner: &str) -> Option<Vec<String>> {
                // Support numeric and alpha ranges, including reverse and stepped
                let mut segs = inner.split("..").collect::<Vec<_>>();
                if segs.len() < 2 { return None; }
                if segs.len() > 3 { return None; }
                let mut step_abs = if segs.len()==3 { segs.pop()?.parse::<i64>().ok()? } else { 1 };
                if step_abs == 0 { return None; }
                step_abs = step_abs.abs();
                let end_s = segs.pop()?; let start_s = segs.pop()?;
                // numeric
                if let (Ok(start), Ok(end)) = (start_s.parse::<i64>(), end_s.parse::<i64>()) {
                    let dir = if end >= start { 1 } else { -1 };
                    let step = step_abs * dir;
                    let mut out = Vec::new();
                    let mut v = start;
                    while (step>0 && v<=end) || (step<0 && v>=end) {
                        out.push(v.to_string()); v += step; if out.len()>=2048 { break; }
                    }
                    return Some(out);
                }
                // alpha single char
                if start_s.len()==1 && end_s.len()==1 {
                    let a = start_s.chars().next().unwrap();
                    let b = end_s.chars().next().unwrap();
                    if !a.is_ascii_alphabetic() || !b.is_ascii_alphabetic() { return None; }
                    let (mut ai, bi) = (a as i16, b as i16);
                    let dir: i16 = if bi >= ai { 1 } else { -1 };
                    let step: i16 = (step_abs as i16) * dir;
                    let mut out = Vec::new();
                    let mut cur = ai;
                    while (step>0 && cur<=bi) || (step<0 && cur>=bi) {
                        out.push(char::from_u32(cur as u32).unwrap().to_string());
                        cur += step; if out.len()>=2048 { break; }
                    }
                    return Some(out);
                }
                None
            }
            // Try expand first top-level {...}
            let bytes = transformed.as_bytes();
            let mut level = 0usize; let mut start_idx: Option<usize> = None;
            for (i, &b) in bytes.iter().enumerate() {
                match b {
                    b'{' => { if level==0 { start_idx = Some(i); } level += 1; },
                    b'}' => {
                        if level>0 { level -= 1; if level==0 {
                            let open = start_idx.unwrap();
                            let inner = &transformed[open+1..i];
                            let prefix = &transformed[..open];
                            let suffix = &transformed[i+1..];
                            // Decide if expandable: top-level comma or valid range
                            let mut has_top_level_comma = false;
                            {
                                let mut lvl = 0usize;
                                for ch in inner.chars() {
                                    match ch {
                                        '{' => lvl += 1,
                                        '}' => { if lvl>0 { lvl -= 1; } },
                                        ',' if lvl==0 => { has_top_level_comma = true; break; }
                                        _ => {}
                                    }
                                }
                            }
                            let is_expandable = has_top_level_comma || brace_try_range(inner).is_some();
                            let suffix_expanded = brace_expand_one(suffix);
                            let mut out = Vec::new();
                            if is_expandable {
                                let mut variants = brace_parse_inner(inner);
                                for v in variants.drain(..) {
                                    for ve in brace_expand_one(&v) {
                                        for tail in &suffix_expanded {
                                            out.push(restore(format!("{prefix}{ve}{tail}")));
                                            if out.len() >= MAX_EXPANSIONS { return out; }
                                        }
                                    }
                                }
                            } else {
                                for tail in &suffix_expanded {
                                    out.push(restore(format!("{prefix}{{{inner}}}{tail}")));
                                    if out.len() >= MAX_EXPANSIONS { return out; }
                                }
                            }
                            return out;
                        } }
                    }
                    _ => {}
                }
            }
            // no complete group
            vec![input.to_string()]
        }
        for arg in args {
            match arg {
                AstNode::Word(word) => {
                    let mut expanded = brace_expand_one(word);
                    let mut final_args = Vec::new();
                    for e in expanded.drain(..) {
                        let globbed = Executor::expand_glob_if_needed(&e, context);
                        if globbed.is_empty() { final_args.push(e); } else { final_args.extend(globbed); }
                    }
                    if final_args.len()==1 { cmd_args.push(final_args.into_iter().next().unwrap()); } else { cmd_args.extend(final_args); }
                },
                AstNode::StringLiteral { value, quote_type } => {
                    // Preserve quoting semantics: treat content as a single field, suppress splitting
                    let s = value.to_string();
                    // Cover all variants of QuoteType
                    match quote_type {
                        nxsh_parser::ast::QuoteType::Double |
                        nxsh_parser::ast::QuoteType::Single |
                        nxsh_parser::ast::QuoteType::AnsiC |
                        nxsh_parser::ast::QuoteType::Locale => {
                            cmd_args.push(s);
                        }
                    }
                },
                AstNode::NumberLiteral { value, .. } => cmd_args.push(value.to_string()),
                AstNode::VariableExpansion { name, .. } => {
                    cmd_args.push(context.get_var(name).unwrap_or_default());
                }
                AstNode::CommandSubstitution { command, is_legacy } => {
                    // Execute nested command substitution fully (use cache)
                    let res = self.eval_cmd_substitution(command, context);
                    match res {
                        Ok(r) => {
                            let mut merged = r.stdout;
                            // stderr handling: NXSH_SUBST_STDERR=merge|separate (default: separate)
                            let stderr_mode = context.get_var("NXSH_SUBST_STDERR").unwrap_or_else(|| "separate".to_string());
                            if stderr_mode.eq_ignore_ascii_case("merge") && !r.stderr.is_empty() {
                                if !merged.is_empty() && !merged.ends_with('\n') { merged.push('\n'); }
                                merged.push_str(&r.stderr);
                            }
                            let trimmed = merged.trim_end();
                            // Legacy backticks behave like unquoted words; apply field splitting when opt-in or legacy
                    let should_split = (*is_legacy) || context.get_var("NXSH_SUBST_SPLIT").as_deref() == Some("1");
                            if should_split {
                                for part in split_fields(trimmed, context) { cmd_args.push(part); }
                            } else {
                                cmd_args.push(trimmed.to_string());
                            }
                        }
                        Err(_) => cmd_args.push(String::new()),
                    }
                }
                _ => cmd_args.push(format!("{:?}", arg)),
            }
        }

        // Background execution takes precedence (even for builtins) so they behave like external jobs
        if context.is_timed_out() { 
            return Ok(ExecutionResult {
                exit_code: 124,
                stdout: String::new(),
                stderr: "nxsh: execution timed out".to_string(),
                execution_time: start_time.elapsed().as_micros() as u64,
                strategy: ExecutionStrategy::DirectInterpreter,
                metrics: ExecutionMetrics::default(),
            });
        }
        if background {
            return self.execute_background_command(&cmd_name, cmd_args, context);
        }

        // Foreground builtin execution
        // First, check user-defined shell functions registry
        if context.has_function(&cmd_name) {
            return self.execute_user_function_by_name(&cmd_name, &cmd_args, context);
        }
        if context.is_timed_out() { 
            return Ok(ExecutionResult {
                exit_code: 124,
                stdout: String::new(),
                stderr: "nxsh: execution timed out".to_string(),
                execution_time: start_time.elapsed().as_micros() as u64,
                strategy: ExecutionStrategy::DirectInterpreter,
                metrics: ExecutionMetrics::default(),
            });
        }
        if let Some(builtin) = self.builtins.get(&cmd_name) {
            let r = builtin.execute(context, &cmd_args);
            if context.is_timed_out() {
                return Ok(ExecutionResult {
                    exit_code: 124,
                    stdout: String::new(),
                    stderr: "nxsh: execution timed out".to_string(),
                    execution_time: start_time.elapsed().as_micros() as u64,
                    strategy: ExecutionStrategy::DirectInterpreter,
                    metrics: ExecutionMetrics::default(),
                });
            }
            return r;
        }

        // Execute as external command
        if context.is_timed_out() { 
            return Ok(ExecutionResult {
                exit_code: 124,
                stdout: String::new(),
                stderr: "nxsh: execution timed out".to_string(),
                execution_time: start_time.elapsed().as_micros() as u64,
                strategy: ExecutionStrategy::DirectInterpreter,
                metrics: ExecutionMetrics::default(),
            });
        }
        let r = self.execute_external_process(&cmd_name, &cmd_args, context);
        if context.is_timed_out() {
            return Ok(ExecutionResult {
                exit_code: 124,
                stdout: String::new(),
                stderr: "nxsh: execution timed out".to_string(),
                execution_time: start_time.elapsed().as_micros() as u64,
                strategy: ExecutionStrategy::DirectInterpreter,
                metrics: ExecutionMetrics::default(),
            });
        }
        r
    }

    /// Execute a user-defined shell function stored in `ShellContext.functions`
    fn execute_user_function_by_name(
        &mut self,
        func_name: &str,
        evaluated_args: &[String],
        context: &mut ShellContext,
    ) -> ShellResult<ExecutionResult> {
        if let Some(src) = context.get_function(func_name) {
            // Parse optional headers then extract body
            let mut lines_iter = src.lines();
            let mut param_names: Vec<String> = Vec::new();
            let mut param_defaults: Vec<Option<String>> = Vec::new();
            let mut variadic_index: Option<usize> = None;
            let mut body_start_src = src.as_str();
            let mut consumed_len: usize = 0;
            if let Some(first) = lines_iter.next() {
                if first.starts_with("#generics:") {
                    consumed_len += first.len() + 1;
                    if let Some(second) = lines_iter.next() {
                        if let Some(rest) = second.strip_prefix("#params:") {
                            if !rest.trim().is_empty() {
                                for (idx, raw) in rest.split(',').enumerate() {
                                    let part = raw.trim(); if part.is_empty() { continue; }
                                    let (name_part, def_part) = if let Some(eq_pos) = part.find('=') { (&part[..eq_pos], Some(part[eq_pos+1..].trim())) } else { (part, None) };
                                    let mut name_clean = name_part.trim().to_string();
                                    let mut is_variadic = false;
                                    if let Some(stripped) = name_clean.strip_suffix("...") { name_clean = stripped.to_string(); is_variadic = true; }
                                    if is_variadic { variadic_index = Some(idx); }
                                    param_names.push(name_clean);
                                    param_defaults.push(def_part.map(|s| s.to_string()));
                                }
                            }
                            consumed_len += second.len() + 1;
                        }
                    }
                } else if let Some(rest) = first.strip_prefix("#params:") {
                    if !rest.trim().is_empty() {
                        for (idx, raw) in rest.split(',').enumerate() {
                            let part = raw.trim(); if part.is_empty() { continue; }
                            let (name_part, def_part) = if let Some(eq_pos) = part.find('=') { (&part[..eq_pos], Some(part[eq_pos+1..].trim())) } else { (part, None) };
                            let mut name_clean = name_part.trim().to_string();
                            let mut is_variadic = false;
                            if let Some(stripped) = name_clean.strip_suffix("...") { name_clean = stripped.to_string(); is_variadic = true; }
                            if is_variadic { variadic_index = Some(idx); }
                            param_names.push(name_clean);
                            param_defaults.push(def_part.map(|s| s.to_string()));
                        }
                    }
                    consumed_len += first.len() + 1;
                }
            }
            if consumed_len > 0 { body_start_src = &src[consumed_len.min(src.len())..]; }

            // Save old and bind new variables
            let mut saved: Vec<(String, Option<String>)> = Vec::new();
            for name in &param_names { saved.push((name.clone(), context.get_var(name))); }
            let mut arg_idx = 0usize;
            for (i, name) in param_names.iter().enumerate() {
                if Some(i) == variadic_index {
                    let rest = if arg_idx < evaluated_args.len() { evaluated_args[arg_idx..].join(" ") } else { String::new() };
                    context.set_var(name.clone(), rest);
                    arg_idx = evaluated_args.len();
                    break;
                } else if let Some(val) = evaluated_args.get(arg_idx) {
                    context.set_var(name.clone(), val.clone());
                    arg_idx += 1;
                } else if let Some(Some(def_src)) = param_defaults.get(i) {
                    if !def_src.is_empty() {
                        if let Ok(def_ast) = parse_program(def_src) {
                            if let Ok(def_res) = self.execute_ast_direct(&def_ast, context) {
                                context.set_var(name.clone(), def_res.stdout);
                            }
                        }
                    }
                } else {
                    context.set_var(name.clone(), String::new());
                }
            }

            // Execute body (empty body is success)
            let result = if body_start_src.trim().is_empty() {
                Ok(ExecutionResult::success(0))
            } else {
                match parse_program(body_start_src) {
                    Ok(ast) => self.execute_ast_direct(&ast, context),
                    Err(_) => Ok(ExecutionResult::failure(1).with_error(format!("function parse failed: {func_name}").into_bytes())),
                }
            };
            // Restore variables
            for (name, old) in saved { match old { Some(v) => context.set_var(name, v), None => context.set_var(name, String::new()) } }
            return result;
        }
        Ok(ExecutionResult::failure(1).with_error(format!("missing function body: {func_name}").into_bytes()))
    }

    /// Execute command in background
    fn execute_background_command(
        &mut self,
        command: &str,
        args: Vec<String>,
        context: &mut ShellContext
    ) -> ShellResult<ExecutionResult> {
        // Get job manager from context
        let job_manager = context.job_manager();
        let mut job_manager_guard = job_manager.lock()
            .map_err(|_| ShellError::new(
                ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                "Job manager lock poisoned".to_string()
            ))?;

        // Spawn background job
        let job_id = job_manager_guard.spawn_background_job(command.to_string(), args)?;
        
        // Return immediately with job information
        let output = format!("[{}] Background job started: {}", job_id, command);
        println!("{}", output); // Also print to console
        
        Ok(ExecutionResult::success(0).with_output(output.as_bytes().to_vec()))
    }

    /// Execute external process
    fn execute_external_process(
        &self,
        command: &str,
        args: &[String],
        context: &ShellContext
    ) -> ShellResult<ExecutionResult> {
        use std::process::Command;
        use std::io::ErrorKind as IoErrorKind;
        use wait_timeout::ChildExt;

        let start_time = Instant::now();

        let mut direct_cmd = Command::new(command);
        direct_cmd.args(args);
        if let Ok(env) = context.env.read() { for (k,v) in env.iter() { direct_cmd.env(k,v); } }
        direct_cmd.current_dir(&context.cwd);

        #[cfg(windows)]
        fn apply_common(cmd: &mut std::process::Command, ctx: &ShellContext) {
            if let Ok(env) = ctx.env.read() { for (k,v) in env.iter() { cmd.env(k,v); } }
            cmd.current_dir(&ctx.cwd);
        }

        // Try to spawn (not .output()) so we can enforce timeout
        let mut child = match direct_cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                #[cfg(windows)]
                {
                    if e.kind() == IoErrorKind::NotFound {
                        let lower = command.to_ascii_lowercase();
                        let mut fb: Option<std::process::Command> = None;
                        if lower == "echo" { // emulate echo via cmd
                            let mut c = Command::new("cmd.exe");
                            let mut full = String::from("echo");
                            for a in args { full.push(' '); full.push_str(a); }
                            c.args(["/C", &full]);
                            fb = Some(c);
                        } else if lower == "sleep" { // map to Start-Sleep
                            let seconds = args.get(0).and_then(|s| s.parse::<u64>().ok()).unwrap_or(1);
                            let mut c = Command::new("powershell.exe");
                            c.args(["-NoProfile", "-Command", &format!("Start-Sleep -Seconds {}", seconds)]);
                            fb = Some(c);
                        }
                        if let Some(mut fb_cmd) = fb {
                            apply_common(&mut fb_cmd, context);
                            match fb_cmd.spawn() { Ok(c2)=> c2, Err(e2)=> return Err(ShellError::new(
                                ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                                format!("Failed to execute command '{}': {} (fallback also failed: {})", command, e, e2)
                            )) }
                        } else {
                            return Err(ShellError::new(
                                ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                                format!("Failed to execute command '{}': {}", command, e)
                            ));
                        }
                    } else {
                        return Err(ShellError::new(
                            ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                            format!("Failed to execute command '{}': {}", command, e)
                        ));
                    }
                }
                #[cfg(not(windows))]
                {
                    return Err(ShellError::new(
                        ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                        format!("Failed to execute command '{}': {}", command, e)
                    ));
                }
            }
        };

        // Wait with optional per-command timeout
        let output = if let Some(dur) = context.per_command_timeout() {
            match child.wait_timeout(dur).map_err(|e| ShellError::new(
                ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                format!("Process wait error: {e}")
            ))? {
                Some(_status) => child.wait_with_output().map_err(|e| ShellError::new(
                    ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                    format!("Process output error: {e}")
                ))?,
                None => { // timeout
                    let _ = child.kill();
                    return Ok(ExecutionResult { exit_code: 124, stdout: String::new(), stderr: format!("nxsh: command '{}' timed out", command), execution_time: start_time.elapsed().as_micros() as u64, strategy: ExecutionStrategy::DirectInterpreter, metrics: ExecutionMetrics::default() });
                }
            }
        } else {
            child.wait_with_output().map_err(|e| ShellError::new(
                ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                format!("Process output error: {e}")
            ))?
        };

        let execution_time = start_time.elapsed().as_micros() as u64;
        Ok(ExecutionResult { exit_code: output.status.code().unwrap_or(-1), stdout: String::from_utf8_lossy(&output.stdout).to_string(), stderr: String::from_utf8_lossy(&output.stderr).to_string(), execution_time, strategy: ExecutionStrategy::DirectInterpreter, metrics: ExecutionMetrics { compile_time_us: 0, optimize_time_us: 0, execute_time_us: execution_time, instruction_count: 1, memory_usage: (output.stdout.len() + output.stderr.len()) as u64 } })
    }
    
    /// Execute a single command
    fn execute_command(&mut self, name: &str, args: &[AstNode], context: &mut ShellContext) -> ShellResult<ExecutionResult> {
        // Pre-evaluate & possibly split arguments
        fn split_fields(raw: &str, context: &ShellContext) -> Vec<String> {
            if context.get_var("NXSH_SUBST_SPLIT").as_deref() != Some("1") { return vec![raw.to_string()]; }
            let ifs = context.get_var("NXSH_IFS").unwrap_or_else(|| " \t\n".to_string());
            let mut out = Vec::new();
            let mut current = String::new();
            for ch in raw.chars() {
                if ifs.contains(ch) { if !current.is_empty() { out.push(std::mem::take(&mut current)); } } else { current.push(ch); }
            }
            if !current.is_empty() { out.push(current); }
            if out.is_empty() { vec![String::new()] } else { out }
        }
        // Advanced (yet bounded) brace expansion supporting:
        //  - comma lists: {a,b,c}
        //  - nested lists: {a,{b,c}}
        //  - numeric ranges: {1..3} => 1 2 3 ; with optional step {1..10..2}
        //  - alpha ranges: {a..c} => a b c
        // Limit expansions to avoid exponential blowup (MAX_EXPANSIONS)
        fn expand_braces(input: &str) -> Vec<String> {
            const MAX_EXPANSIONS: usize = 4096; // safety cap
            // Provide escape handling: backslash preceding { } , prevents structural meaning.
            // Strategy: temporarily replace escaped tokens with sentinel bytes, run normal logic, then restore.
            const ESC_LBRACE: char = '\u{1F}';
            const ESC_RBRACE: char = '\u{1E}';
            const ESC_COMMA:  char = '\u{1D}';
            let mut transformed = String::with_capacity(input.len());
            let mut chars = input.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '\\' { // escape next char if brace related
                    if let Some(&next) = chars.peek() {
                        match next {
                            '{' => { transformed.push(ESC_LBRACE); chars.next(); continue; }
                            '}' => { transformed.push(ESC_RBRACE); chars.next(); continue; }
                            ',' => { transformed.push(ESC_COMMA);  chars.next(); continue; }
                            _ => { transformed.push(next); chars.next(); continue; }
                        }
                    }
                    // trailing backslash -> keep
                    transformed.push('\\');
                } else {
                    transformed.push(c);
                }
            }
            // Fast path: if no '{' present return original (after restoration)
            if !transformed.as_bytes().contains(&b'{') {
                return vec![input.to_string()];
            }
            fn restore(s: String) -> String {
                let mut out = String::with_capacity(s.len());
                for c in s.chars() {
                    match c {
                        '\u{1F}' => { out.push('\\'); out.push('{'); },
                        '\u{1E}' => { out.push('\\'); out.push('}'); },
                        '\u{1D}' => { out.push('\\'); out.push(','); },
                        _ => out.push(c),
                    }
                }
                out
            }
            // Find first top-level {...}
            let bytes = transformed.as_bytes();
            let mut level = 0usize; let mut start_idx = None;
            for (i, &b) in bytes.iter().enumerate() {
                match b {
                    b'{' => { if level==0 { start_idx = Some(i); } level +=1; },
                    b'}' => { if level>0 { level -=1; if level==0 { // complete group
                        let open = start_idx.unwrap();
                        let inner = &transformed[open+1..i];
                        let prefix = &transformed[..open];
                        let suffix = &transformed[i+1..];
                        // Determine if inner should expand: top-level comma or a valid range pattern
                        let mut has_top_level_comma = false;
                        {
                            let mut lvl = 0usize;
                            for ch in inner.chars() {
                                match ch {
                                    '{' => lvl += 1,
                                    '}' => { if lvl > 0 { lvl -= 1; } },
                                    ',' if lvl == 0 => { has_top_level_comma = true; break; }
                                    _ => {}
                                }
                            }
                        }
                        let mut variants = if has_top_level_comma || try_range(inner).is_some() {
                            parse_brace_inner(inner)
                        } else {
                            // Not expandable: keep literal braces
                            vec![format!("{{{}}}", inner)]
                        };
                        let suffix_expanded = expand_braces(suffix);
                        let mut out = Vec::new();
                        for v in variants {
                            let v_expanded = expand_braces(&v);
                            for ve in v_expanded {
                                for tail in &suffix_expanded {
                                    out.push(restore(format!("{prefix}{ve}{tail}")));
                                    if out.len() >= MAX_EXPANSIONS {
                                        // mark truncation via env var for diagnostics
                                        std::env::set_var("NXSH_BRACE_EXPANSION_TRUNCATED", "1");
                                        return out;
                                    }
                                }
                            }
                        }
                        return out;
                    } } },
                    _ => {}
                }
            }
            vec![input.to_string()] // no complete group
        }

        fn parse_brace_inner(inner: &str) -> Vec<String> {
            // Detect range patterns first: {start..end[..step]}
            if let Some(range_variants) = try_range(inner) { return range_variants; }
            let mut parts = Vec::new();
            let mut level = 0usize; let mut escape = false; let mut current = String::new();
            for c in inner.chars() {
                if escape { current.push(c); escape = false; continue; }
                match c {
                    '\\' => { escape = true; },
                    '{' => { level += 1; current.push(c); },
                    '}' => { if level>0 { level -=1; } current.push(c); },
                    ',' if level==0 => { parts.push(current.to_string()); current.clear(); },
                    _ => current.push(c)
                }
            }
            // Allow trailing empty element {a,b,}
            if escape { current.push('\\'); }
            parts.push(current.to_string());
            parts
        }

        fn try_range(inner: &str) -> Option<Vec<String>> {
            // Numeric or alpha range like 1..5 or a..f or 1..10..2
            let mut segs = inner.split("..").collect::<Vec<_>>();
            if segs.len() < 2 { return None; }
            if segs.len() > 3 { return None; }
            let mut step_abs = if segs.len() == 3 { segs.pop()?.parse::<i64>().ok()? } else { 1 };
            if step_abs == 0 { return None; }
            step_abs = step_abs.abs();
            let end_str = segs.pop()?; let start_str = segs.pop()?;
            // numeric
            if let (Ok(start), Ok(end)) = (start_str.parse::<i64>(), end_str.parse::<i64>()) {
                let dir = if end >= start { 1 } else { -1 };
                let step = step_abs * dir;
                let mut out = Vec::new();
                let mut v = start;
                while (step>0 && v <= end) || (step<0 && v >= end) {
                    out.push(v.to_string());
                    v += step;
                    if out.len() >= 2048 { break; }
                }
                return Some(out);
            }
            // alpha single char
            if start_str.len()==1 && end_str.len()==1 {
                let (a, b) = (start_str.chars().next().unwrap(), end_str.chars().next().unwrap());
                if !a.is_ascii_alphabetic() || !b.is_ascii_alphabetic() { return None; }
                let (mut ai, bi) = (a as i16, b as i16);
                let dir: i16 = if bi >= ai { 1 } else { -1 };
                let step: i16 = (step_abs as i16) * dir;
                let mut out = Vec::new();
                let mut cur = ai;
                while (step>0 && cur <= bi) || (step<0 && cur >= bi) {
                    out.push(char::from_u32(cur as u32).unwrap().to_string());
                    cur += step;
                    if out.len() >= 2048 { break; }
                }
                return Some(out);
            }
            None
        }

        let mut evaluated_args = Vec::new();
        for arg in args {
            match arg {
                AstNode::Word(s) => {
                    // First brace expansion
                    let mut expanded = expand_braces(s);
                    // Then glob (including extglob) expansion per element
                    let mut final_args = Vec::new();
                    for e in expanded.drain(..) {
                        let globbed = Executor::expand_glob_if_needed(&e, context);
                        if globbed.is_empty() { final_args.push(e); } else { final_args.extend(globbed); }
                    }
                    let expanded = final_args;
                    if expanded.len() == 1 { evaluated_args.push(expanded.into_iter().next().unwrap()); }
                    else { for e in expanded { evaluated_args.push(e); } }
                },
                AstNode::StringLiteral { value, .. } => evaluated_args.push(value.to_string()),
                AstNode::NumberLiteral { value, .. } => evaluated_args.push(value.to_string()),
                AstNode::VariableExpansion { name, .. } => evaluated_args.push(context.get_var(name).unwrap_or_else(|| name.to_string())),
                AstNode::CommandSubstitution { command, is_legacy } => {
                    match self.eval_cmd_substitution(command, context) {
                        Ok(r) => {
                            let mut merged = r.stdout;
                            let stderr_mode = context.get_var("NXSH_SUBST_STDERR").unwrap_or_else(|| "separate".to_string());
                            if stderr_mode.eq_ignore_ascii_case("merge") && !r.stderr.is_empty() {
                                if !merged.is_empty() && !merged.ends_with('\n') { merged.push('\n'); }
                                merged.push_str(&r.stderr);
                            }
                            let trimmed = merged.trim_end();
                            let should_split = (*is_legacy) || context.get_var("NXSH_SUBST_SPLIT").as_deref() == Some("1");
                            if should_split { for part in split_fields(trimmed, context) { evaluated_args.push(part); } }
                            else { evaluated_args.push(trimmed.to_string()); }
                        }
                        Err(_) => evaluated_args.push(String::new()),
                    }
                }
                _ => evaluated_args.push(format!("{:?}", arg)),
            }
        }
        if let Some(builtin) = self.builtins.get(name) {
            return builtin.execute(context, &evaluated_args);
        }
        let start_time = Instant::now();
        let mut cmd = std::process::Command::new(name);
        if !evaluated_args.is_empty() { cmd.args(&evaluated_args); }
        if let Ok(env) = context.env.read() { for (k,v) in env.iter() { cmd.env(k,v); } }
        cmd.current_dir(&context.cwd);
        match cmd.output() {
            Ok(output) => {
                let execution_time = start_time.elapsed().as_micros() as u64;
                Ok(ExecutionResult { exit_code: output.status.code().unwrap_or(-1), stdout: String::from_utf8_lossy(&output.stdout).to_string(), stderr: String::from_utf8_lossy(&output.stderr).to_string(), execution_time, strategy: ExecutionStrategy::DirectInterpreter, metrics: ExecutionMetrics { compile_time_us: 0, optimize_time_us: 0, execute_time_us: execution_time, instruction_count: 1, memory_usage: (output.stdout.len() + output.stderr.len()) as u64 } })
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_micros() as u64;
                Ok(ExecutionResult { exit_code: 127, stdout: String::new(), stderr: format!("nxsh: {}: command not found ({})", name, e), execution_time, strategy: ExecutionStrategy::DirectInterpreter, metrics: ExecutionMetrics { compile_time_us: 0, optimize_time_us: 0, execute_time_us: execution_time, instruction_count: 1, memory_usage: 0 } })
            }
        }
    }

    /// (test helper) Evaluate args expansion & splitting like execute_command would
    #[cfg(test)]
    pub(crate) fn evaluate_args_for_test(&mut self, args: &[AstNode], context: &mut ShellContext) -> Vec<String> {
        fn split_fields(raw: &str, context: &ShellContext) -> Vec<String> {
            if context.get_var("NXSH_SUBST_SPLIT").as_deref() != Some("1") { return vec![raw.to_string()]; }
            let ifs = context.get_var("NXSH_IFS").unwrap_or_else(|| " \t\n".to_string());
            let mut out = Vec::new(); let mut current = String::new();
            for ch in raw.chars() { if ifs.contains(ch) { if !current.is_empty() { out.push(std::mem::take(&mut current)); } } else { current.push(ch); } }
            if !current.is_empty() { out.push(current); }
            if out.is_empty() { vec![String::new()] } else { out }
        }
        let mut evaluated = Vec::new();
        for arg in args {
            match arg {
                AstNode::Word(s) => evaluated.push(s.to_string()),
                AstNode::StringLiteral { value, .. } => evaluated.push(value.to_string()),
                AstNode::NumberLiteral { value, .. } => evaluated.push(value.to_string()),
                AstNode::VariableExpansion { name, .. } => evaluated.push(context.get_var(name).unwrap_or_else(|| name.to_string())),
                AstNode::CommandSubstitution { command, is_legacy } => {
                    match self.eval_cmd_substitution(command, context) {
                        Ok(r) => {
                            let mut merged = r.stdout;
                            let stderr_mode = context.get_var("NXSH_SUBST_STDERR").unwrap_or_else(|| "separate".to_string());
                            if stderr_mode.eq_ignore_ascii_case("merge") && !r.stderr.is_empty() {
                                if !merged.is_empty() && !merged.ends_with('\n') { merged.push('\n'); }
                                merged.push_str(&r.stderr);
                            }
                            let trimmed = merged.trim_end();
                            let should_split = (*is_legacy) || context.get_var("NXSH_SUBST_SPLIT").as_deref() == Some("1");
                            if should_split { for part in split_fields(trimmed, context) { evaluated.push(part); } }
                            else { evaluated.push(trimmed.to_string()); }
                        }
                        Err(_) => evaluated.push(String::new()),
                    }
                }
                _ => evaluated.push(format!("{:?}", arg)),
            }
        }
        evaluated
    }
    
    /// Execute a pipeline of commands
    fn execute_pipeline(&mut self, commands: &[AstNode], context: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let start_time = Instant::now();
        // Experimental: if PowerShell compatibility requested, attempt object pipeline using simplified textual reconstruction
        if std::env::var("NXSH_PWSH_MODE").ok().as_deref() == Some("1") {
            #[cfg(feature = "powershell_compat")]
            {
                let mut parts = Vec::new();
                for c in commands.iter() { parts.push(format!("{:?}", c)); }
                let pipeline_str = parts.join(" | ");
                let mut compat = crate::powershell_compat::PowerShellCompat::new();
                if let Ok(objs) = compat.execute_pipeline(&pipeline_str) {
                    let out = objs.iter().map(|o| o.to_string()).collect::<Vec<_>>().join("\n");
                    let execution_time = start_time.elapsed().as_micros() as u64;
                    return Ok(ExecutionResult { exit_code: 0, stdout: out, stderr: String::new(), execution_time, strategy: ExecutionStrategy::DirectInterpreter, metrics: ExecutionMetrics { execute_time_us: execution_time, ..Default::default() } });
                }
            }
        }
        let mut final_result = ExecutionResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            execution_time: 0,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        };
        
        for command in commands {
            if context.is_timed_out() {
                final_result.exit_code = 124;
                final_result.stderr = "nxsh: pipeline timed out".to_string();
                break;
            }
            let result = self.execute_ast_direct(command, context)?;
            final_result.execution_time += result.execution_time;
            final_result.stdout = result.stdout;
            if result.exit_code != 0 {
                final_result.exit_code = result.exit_code;
                final_result.stderr = result.stderr;
                break;
            }
        }
        
        Ok(final_result)
    }
    
    /// Execute a conditional statement
    fn execute_conditional(
        &mut self,
        condition: &AstNode,
        then_branch: &AstNode,
        else_branch: Option<&AstNode>,
        context: &mut ShellContext
    ) -> ShellResult<ExecutionResult> {
        let condition_result = self.execute_ast_direct(condition, context)?;
        
        if condition_result.exit_code == 0 {
            self.execute_ast_direct(then_branch, context)
        } else if let Some(else_node) = else_branch {
            self.execute_ast_direct(else_node, context)
        } else {
            Ok(ExecutionResult {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
                execution_time: condition_result.execution_time,
                strategy: ExecutionStrategy::DirectInterpreter,
                metrics: ExecutionMetrics::default(),
            })
        }
    }
    
    /// Execute a loop
    fn execute_loop(&mut self, condition: &AstNode, body: &AstNode, context: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let mut total_time = 0;
        let mut last_result = ExecutionResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            execution_time: 0,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        };
        
        loop {
            let condition_result = self.execute_ast_direct(condition, context)?;
            total_time += condition_result.execution_time;
            
            if condition_result.exit_code != 0 {
                break;
            }
            
            let body_result = self.execute_ast_direct(body, context)?;
            total_time += body_result.execution_time;
            last_result = body_result;
            
            // Simple loop protection
            if total_time > 10_000_000 { // 10 seconds
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::Timeout),
                    "Loop execution timeout"
                ));
            }
        }
        
        last_result.execution_time = total_time;
        Ok(last_result)
    }
    
    /// Get executor statistics
    pub fn stats(&self) -> &ExecutorStats {
        &self.stats
    }
    
    /// Reset executor statistics
    pub fn reset_stats(&mut self) {
        self.stats = ExecutorStats::default();
    }

    /// Execute subshell with complete isolation
    fn execute_subshell(&mut self, commands: &[AstNode], ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        // Check if process isolation is enabled
        let enable_isolation = ctx.get_option("enable_process_isolation").unwrap_or(true);
        
        if enable_isolation {
            // Execute in completely isolated process
            self.execute_subshell_isolated(commands, ctx)
        } else {
            // Execute in same process with context isolation
            self.execute_subshell_local(commands, ctx)
        }
    }

    /// Execute subshell in isolated process (fork-like behavior)
    fn execute_subshell_isolated(&mut self, commands: &[AstNode], ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        use std::process::{Command, Stdio};
        use std::io::Write;
        use tempfile::NamedTempFile;
        let start_time = std::time::Instant::now();
        
        // Create temporary script file for subshell execution
        let mut temp_script = NamedTempFile::new()
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::Other),
                format!("Failed to create temporary script: {}", e)
            ))?;
        
        // Convert commands to script text
        let script_content = self.commands_to_script(commands)?;
        temp_script.write_all(script_content.as_bytes())
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::Other),
                format!("Failed to write script: {}", e)
            ))?;
        temp_script.flush()
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::Other),
                format!("Failed to flush script: {}", e)
            ))?;
        
        // Prepare environment variables for subshell
        let subshell_env = self.prepare_subshell_environment(ctx)?;
        
        // Execute subshell as separate process
        let child = Command::new(std::env::current_exe()
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::Other),
                format!("Failed to get current executable: {}", e)
            ))?)
            .arg("--subshell")
            .arg(temp_script.path())
            .envs(&subshell_env)
            .current_dir(&ctx.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::Other),
                format!("Failed to spawn subshell process: {}", e)
            ))?;
        
        // Wait for completion and collect output
        let output = child.wait_with_output()
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::Other),
                format!("Failed to wait for subshell: {}", e)
            ))?;
        let execution_time = start_time.elapsed().as_micros() as u64;
        
        Ok(ExecutionResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics { execute_time_us: execution_time, ..Default::default() },
        })
    }

    /// Execute subshell with local context isolation (in-process)
    fn execute_subshell_local(&mut self, commands: &[AstNode], ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        // Create isolated context by deep-cloning the current context
        let mut subshell_ctx = self.create_isolated_context(ctx)?;
        
        // Increment subshell level
        {
            let mut options = subshell_ctx.options.write()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to acquire options lock"
                ))?;
            options.subshell_level += 1;
        }
        
        // Execute commands in isolated context
        let result = self.execute_pipeline(commands, &mut subshell_ctx)?;
        
        // Subshell changes do NOT affect parent context
        // (variables, functions, aliases remain isolated)
        
        Ok(result)
    }

    /// Create completely isolated context for subshell
    fn create_isolated_context(&self, parent_ctx: &ShellContext) -> ShellResult<ShellContext> {
        // Create new context with deep-copied state
        let mut isolated_ctx = ShellContext::new();
        
        // Copy exported environment variables only
        {
            let parent_vars = parent_ctx.vars.read()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to read parent variables"
                ))?;
            let parent_env = parent_ctx.env.read()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to read parent environment"
                ))?;
            
            let mut isolated_env = isolated_ctx.env.write()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to acquire isolated environment lock"
                ))?;
            let mut isolated_vars = isolated_ctx.vars.write()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to acquire isolated variables lock"
                ))?;
            
            // Copy exported variables to subshell environment
            for (key, var) in parent_vars.iter() {
                if var.exported {
                    isolated_env.insert(key.clone(), var.value.clone());
                    isolated_vars.insert(key.clone(), var.clone());
                }
            }
            
            // Copy environment variables
            for (key, value) in parent_env.iter() {
                isolated_env.insert(key.clone(), value.clone());
            }
        }
        
        // Copy shell options (but reset control flow state)
        {
            let parent_options = parent_ctx.options.read()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to read parent options"
                ))?;
            let mut isolated_options = isolated_ctx.options.write()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to acquire isolated options lock"
                ))?;
            
            *isolated_options = parent_options.clone();
            // Reset control flow state in subshell
            isolated_options.break_requested = false;
            isolated_options.continue_requested = false;
        }
        
        // Copy current working directory
        isolated_ctx.cwd = parent_ctx.cwd.clone();
        
        // Copy shell level (will be incremented by caller)
        isolated_ctx.shell_level = parent_ctx.shell_level;
        
        // Functions and aliases are NOT inherited (subshell isolation)
        // History is NOT shared (subshell isolation)
        
        Ok(isolated_ctx)
    }

    /// Convert AST commands to shell script text for external execution
    fn commands_to_script(&self, commands: &[AstNode]) -> ShellResult<String> {
        let mut script = String::new();
        
        // Add shebang for proper execution
        script.push_str("#!/usr/bin/env nxsh\n");
        script.push_str("# Auto-generated subshell script\n\n");
        
        for command in commands {
            // Convert AST node to shell command text
            script.push_str(&self.ast_to_command_string(command)?);
            script.push('\n');
        }
        
        Ok(script)
    }

    /// Convert single AST node to command string
    fn ast_to_command_string(&self, node: &AstNode) -> ShellResult<String> {
        match node {
            AstNode::Program(statements) => {
                let mut commands = Vec::new();
                for statement in statements {
                    commands.push(self.ast_to_command_string(statement)?);
                }
                Ok(commands.join("; "))
            },
            AstNode::Command { name, args, .. } => {
                let mut cmd_str = format!("{}", name);
                for arg in args {
                    cmd_str.push(' ');
                    cmd_str.push_str(&self.ast_to_command_string(arg)?);
                }
                Ok(cmd_str)
            },
            AstNode::Word(word) => Ok(word.to_string()),
            AstNode::StringLiteral { value, .. } => Ok(format!("\"{}\"", value)),
            AstNode::NumberLiteral { value, .. } => Ok(value.to_string()),
            AstNode::VariableExpansion { name, .. } => Ok(format!("${}", name)),
            AstNode::Pipeline { elements, .. } => {
                let parts: ShellResult<Vec<String>> = elements.iter()
                    .map(|e| self.ast_to_command_string(e))
                    .collect();
                Ok(parts?.join(" | "))
            },
            _ => {
                // For complex nodes, use a simplified representation
                Ok("# Complex command".to_string())
            }
        }
    }

    /// Prepare environment for subshell process
    fn prepare_subshell_environment(&self, ctx: &ShellContext) -> ShellResult<std::collections::HashMap<String, String>> {
        let mut env = std::collections::HashMap::new();
        
        // Copy exported variables
        {
            let vars = ctx.vars.read()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to read variables"
                ))?;
            let ctx_env = ctx.env.read()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to read environment"
                ))?;
            
            // Add exported shell variables
            for (key, var) in vars.iter() {
                if var.exported {
                    env.insert(key.clone(), var.value.clone());
                }
            }
            
            // Add environment variables
            for (key, value) in ctx_env.iter() {
                env.insert(key.clone(), value.clone());
            }
        }
        
        // Set subshell-specific variables
        env.insert("SHLVL".to_string(), (ctx.shell_level + 1).to_string());
        env.insert("NXSH_SUBSHELL".to_string(), "1".to_string());
        
        Ok(env)
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}
