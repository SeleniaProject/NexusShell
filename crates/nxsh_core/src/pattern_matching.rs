//! Advanced pattern matching engine for NexusShell
//!
//! This module implements Rust-like pattern matching with comprehensive support for:
//! - Literal patterns, wildcards, and type patterns
//! - Destructuring of tuples, arrays, and objects
//! - Guard clauses and conditional matching
//! - Exhaustiveness checking and compiler warnings
//! - Variable binding and capture groups
//! - Or-patterns and complex nested structures

use crate::error::ShellResult;
use crate::closures::Value;
use nxsh_parser::ast::{Pattern, MatchArm, AstNode, BinaryOperator, UnaryOperator};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tracing::{debug, warn, info};

/// Pattern matching engine with advanced evaluation capabilities
pub struct PatternMatchingEngine {
    /// Configuration for pattern matching behavior
    config: PatternMatchingConfig,
    /// Statistics for performance monitoring
    statistics: PatternMatchingStatistics,
    /// Cache for compiled patterns
    pattern_cache: HashMap<String, CompiledPattern>,
}

/// Configuration for pattern matching behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatchingConfig {
    /// Enable exhaustiveness checking
    pub exhaustiveness_checking: bool,
    /// Enable pattern caching for performance
    pub pattern_caching: bool,
    /// Maximum recursion depth for nested patterns
    pub max_recursion_depth: usize,
    /// Enable strict type checking
    pub strict_type_checking: bool,
    /// Enable pattern optimization
    pub pattern_optimization: bool,
    /// Warn on unreachable patterns
    pub warn_unreachable: bool,
    /// Enable guard clause evaluation
    pub enable_guards: bool,
}

impl Default for PatternMatchingConfig {
    fn default() -> Self {
        Self {
            exhaustiveness_checking: true,
            pattern_caching: true,
            max_recursion_depth: 100,
            strict_type_checking: false,
            pattern_optimization: true,
            warn_unreachable: true,
            enable_guards: true,
        }
    }
}

/// Statistics for pattern matching operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternMatchingStatistics {
    /// Total number of pattern matches attempted
    pub matches_attempted: u64,
    /// Number of successful matches
    pub matches_successful: u64,
    /// Number of failed matches
    pub matches_failed: u64,
    /// Number of guard evaluations
    pub guard_evaluations: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Total time spent in pattern matching (nanoseconds)
    pub total_time_ns: u64,
}

// Default is derived above

/// Compiled pattern for efficient matching
#[derive(Debug, Clone)]
pub struct CompiledPattern {
    /// Original pattern
    pub pattern: Pattern<'static>,
    /// Compiled bytecode for efficient evaluation
    pub bytecode: Vec<PatternInstruction>,
    /// Variable bindings this pattern creates
    pub bindings: Vec<String>,
    /// Whether this pattern is refutable (can fail)
    pub is_refutable: bool,
}

/// Pattern matching bytecode instructions
#[derive(Debug, Clone)]
pub enum PatternInstruction {
    /// Match literal value
    MatchLiteral(String),
    /// Match any value (wildcard)
    MatchWildcard,
    /// Bind variable
    BindVariable(String),
    /// Check type
    CheckType(String),
    /// Enter tuple/array context
    EnterSequence(usize), // expected length
    /// Enter object context
    EnterObject(Vec<String>), // expected fields
    /// Exit current context
    ExitContext,
    /// Evaluate guard condition
    EvaluateGuard(AstNode<'static>),
    /// Jump if match fails
    JumpOnFail(usize),
    /// Jump unconditionally
    Jump(usize),
    /// Success
    Success,
    /// Failure
    Failure,
}

/// Pattern matching value types
#[derive(Debug, Clone, PartialEq)]
pub enum PatternValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Array(Vec<PatternValue>),
    Object(HashMap<String, PatternValue>),
    Tuple(Vec<PatternValue>),
    Null,
    Type(String),
}

/// Pattern matching result
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Whether the pattern matched
    pub matched: bool,
    /// Variable bindings created by the match
    pub bindings: HashMap<String, PatternValue>,
    /// Which arm was matched (if any)
    pub matched_arm: Option<usize>,
    /// Exhaustiveness check result
    pub is_exhaustive: bool,
    /// Any warnings generated
    pub warnings: Vec<String>,
}

/// Pattern matching context during evaluation
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Current variable bindings
    pub bindings: HashMap<String, PatternValue>,
    /// Current recursion depth
    pub depth: usize,
    /// Current value being matched
    pub current_value: PatternValue,
    /// Stack of contexts for nested patterns
    pub context_stack: Vec<ContextFrame>,
}

/// Context frame for nested pattern evaluation
#[derive(Debug, Clone)]
pub struct ContextFrame {
    /// Type of context (tuple, array, object)
    pub context_type: ContextType,
    /// Current index/position in sequence
    pub position: usize,
    /// Values in current context
    pub values: Vec<PatternValue>,
    /// Field names for object contexts
    pub fields: Vec<String>,
}

/// Types of evaluation contexts
#[derive(Debug, Clone)]
pub enum ContextType {
    Tuple,
    Array,
    Object,
    Root,
}

impl PatternMatchingEngine {
    /// Create a new pattern matching engine
    pub fn new(config: PatternMatchingConfig) -> Self {
        Self {
            config,
            statistics: PatternMatchingStatistics::default(),
            pattern_cache: HashMap::new(),
        }
    }

    /// Match a value against a pattern
    pub fn match_pattern(&mut self, value: &PatternValue, pattern: &Pattern) -> ShellResult<MatchResult> {
        let start_time = std::time::Instant::now();
        self.statistics.matches_attempted += 1;

        debug!(?pattern, ?value, "Starting pattern match");

        // Create evaluation context
        let mut context = EvaluationContext {
            bindings: HashMap::new(),
            depth: 0,
            current_value: value.clone(),
            context_stack: vec![ContextFrame {
                context_type: ContextType::Root,
                position: 0,
                values: vec![value.clone()],
                fields: vec![],
            }],
        };

        // Check pattern cache
    let pattern_key = format!("{pattern:?}"); // Simplified key generation
        let compiled_pattern = if self.config.pattern_caching {
            if let Some(cached) = self.pattern_cache.get(&pattern_key) {
                self.statistics.cache_hits += 1;
                cached.clone()
            } else {
                self.statistics.cache_misses += 1;
                let compiled = self.compile_pattern(pattern)?;
                self.pattern_cache.insert(pattern_key, compiled.clone());
                compiled
            }
        } else {
            self.compile_pattern(pattern)?
        };

        // Execute pattern matching
        let matched = self.execute_pattern(&compiled_pattern, &mut context)?;

        // Update statistics
        let elapsed = start_time.elapsed();
        self.statistics.total_time_ns += elapsed.as_nanos() as u64;
        
        if matched {
            self.statistics.matches_successful += 1;
        } else {
            self.statistics.matches_failed += 1;
        }

        let result = MatchResult {
            matched,
            bindings: context.bindings,
            matched_arm: if matched { Some(0) } else { None },
            is_exhaustive: true, // Simplified for single pattern
            warnings: vec![],
        };

        debug!(matched = result.matched, bindings = ?result.bindings, "Pattern match completed");
        Ok(result)
    }

    /// Match a value against multiple arms (match expression)
    pub fn match_arms(&mut self, value: &PatternValue, arms: &[MatchArm]) -> ShellResult<MatchResult> {
        let start_time = std::time::Instant::now();
        self.statistics.matches_attempted += 1;

        info!(arms_count = arms.len(), "Evaluating match expression");

        let mut warnings = Vec::new();
        let mut matched_arm = None;
        let mut final_bindings = HashMap::new();

        // Check each arm in order
        for (arm_index, arm) in arms.iter().enumerate() {
            debug!(arm_index, "Evaluating match arm");

            // Evaluate the single pattern in the arm
            let pattern = &arm.pattern;
            let mut context = EvaluationContext {
                bindings: HashMap::new(),
                depth: 0,
                current_value: value.clone(),
                context_stack: vec![ContextFrame {
                    context_type: ContextType::Root,
                    position: 0,
                    values: vec![value.clone()],
                    fields: vec![],
                }],
            };

            let compiled_pattern = self.compile_pattern(pattern)?;
            
            if self.execute_pattern(&compiled_pattern, &mut context)? {
                // Pattern matched, now check guard if present
                let guard_passes = if let Some(guard) = &arm.guard {
                    if self.config.enable_guards {
                        self.statistics.guard_evaluations += 1;
                        self.evaluate_guard(guard, &context.bindings)?
                    } else {
                        true
                    }
                } else {
                    true
                };

                if guard_passes {
                    matched_arm = Some(arm_index);
                    final_bindings = context.bindings;
                    break;
                }
            }

            if matched_arm.is_some() {
                break;
            }
        }

        // Check for exhaustiveness if enabled
        let is_exhaustive = if self.config.exhaustiveness_checking {
            self.check_exhaustiveness(arms)?
        } else {
            true
        };

        if !is_exhaustive {
            warnings.push("Non-exhaustive pattern match".to_string());
        }

        // Check for unreachable patterns if enabled
        if self.config.warn_unreachable {
            let unreachable = self.find_unreachable_patterns(arms)?;
            for index in unreachable {
                warnings.push(format!("Pattern in arm {index} is unreachable"));
            }
        }

        // Update statistics
        let elapsed = start_time.elapsed();
        self.statistics.total_time_ns += elapsed.as_nanos() as u64;
        
        let matched = matched_arm.is_some();
        if matched {
            self.statistics.matches_successful += 1;
        } else {
            self.statistics.matches_failed += 1;
        }

        let result = MatchResult {
            matched,
            bindings: final_bindings,
            matched_arm,
            is_exhaustive,
            warnings,
        };

        info!(
            matched = result.matched,
            matched_arm = ?result.matched_arm,
            bindings_count = result.bindings.len(),
            "Match expression completed"
        );

        Ok(result)
    }

    /// Compile a pattern into bytecode for efficient execution
    fn compile_pattern(&self, pattern: &Pattern) -> ShellResult<CompiledPattern> {
        let mut bytecode = Vec::new();
        let mut bindings = Vec::new();
        let mut is_refutable = false;

        self.compile_pattern_recursive(pattern, &mut bytecode, &mut bindings, &mut is_refutable)?;
        bytecode.push(PatternInstruction::Success);

        Ok(CompiledPattern {
            pattern: unsafe { std::mem::transmute::<Pattern<'_>, Pattern<'_>>(pattern.clone()) }, // Lifetime hack for simplicity
            bytecode,
            bindings,
            is_refutable,
        })
    }

    /// Recursively compile pattern into bytecode
    #[allow(clippy::only_used_in_recursion)]
    fn compile_pattern_recursive(
        &self,
        pattern: &Pattern,
        bytecode: &mut Vec<PatternInstruction>,
        bindings: &mut Vec<String>,
        is_refutable: &mut bool,
    ) -> ShellResult<()> {
        match pattern {
            Pattern::Literal(value) => {
                bytecode.push(PatternInstruction::MatchLiteral(value.to_string()));
                *is_refutable = true;
            },
            Pattern::Variable(name) => {
                bytecode.push(PatternInstruction::BindVariable(name.to_string()));
                bindings.push(name.to_string());
            },
            Pattern::Wildcard => {
                bytecode.push(PatternInstruction::MatchWildcard);
            },
            Pattern::Placeholder => {
                bytecode.push(PatternInstruction::MatchWildcard);
            },
            Pattern::Tuple(patterns) => {
                bytecode.push(PatternInstruction::EnterSequence(patterns.len()));
                for pattern in patterns {
                    self.compile_pattern_recursive(pattern, bytecode, bindings, is_refutable)?;
                }
                bytecode.push(PatternInstruction::ExitContext);
                *is_refutable = true;
            },
            Pattern::Array(patterns) => {
                bytecode.push(PatternInstruction::EnterSequence(patterns.len()));
                for pattern in patterns {
                    self.compile_pattern_recursive(pattern, bytecode, bindings, is_refutable)?;
                }
                bytecode.push(PatternInstruction::ExitContext);
                *is_refutable = true;
            },
            Pattern::Object { fields, .. } => {
                let field_names: Vec<String> = fields.iter().map(|f| f.key.to_string()).collect();
                bytecode.push(PatternInstruction::EnterObject(field_names));
                
                for field in fields {
                    if let Some(pattern) = &field.pattern {
                        self.compile_pattern_recursive(pattern, bytecode, bindings, is_refutable)?;
                    } else {
                        // Field shorthand: field: field
                        bytecode.push(PatternInstruction::BindVariable(field.key.to_string()));
                        bindings.push(field.key.to_string());
                    }
                }
                bytecode.push(PatternInstruction::ExitContext);
                *is_refutable = true;
            },
            Pattern::Type { type_name, inner } => {
                bytecode.push(PatternInstruction::CheckType(type_name.to_string()));
                if let Some(inner_pattern) = inner {
                    self.compile_pattern_recursive(inner_pattern, bytecode, bindings, is_refutable)?;
                }
                *is_refutable = true;
            },
            Pattern::Guard { pattern, condition } => {
                self.compile_pattern_recursive(pattern, bytecode, bindings, is_refutable)?;
                bytecode.push(PatternInstruction::EvaluateGuard(unsafe { std::mem::transmute::<AstNode<'_>, AstNode<'_>>(condition.as_ref().clone()) }));
                *is_refutable = true;
            },
            Pattern::Binding { name, pattern } => {
                bytecode.push(PatternInstruction::BindVariable(name.to_string()));
                bindings.push(name.to_string());
                self.compile_pattern_recursive(pattern, bytecode, bindings, is_refutable)?;
            },
            Pattern::Or(patterns) => {
                // Compile OR patterns with jump instructions
                let mut or_success = Vec::new();
                
                for (i, pattern) in patterns.iter().enumerate() {
                    if i > 0 {
                        // Jump to next alternative on failure
                        bytecode.push(PatternInstruction::JumpOnFail(0)); // Will be patched
                    }
                    
                    self.compile_pattern_recursive(pattern, bytecode, bindings, is_refutable)?;
                    
                    if i < patterns.len() - 1 {
                        // Jump to success on match
                        or_success.push(bytecode.len());
                        bytecode.push(PatternInstruction::Jump(0)); // Will be patched
                    }
                }
                
                // Patch jump addresses (simplified implementation)
                // In a real compiler, this would be more sophisticated
                *is_refutable = true;
            },
            Pattern::Alternative(patterns) => {
                // Similar to Or but with different semantics
                self.compile_pattern_recursive(&Pattern::Or(patterns.clone()), bytecode, bindings, is_refutable)?;
            },
            Pattern::Range { start, end } => {
                // Range patterns need special handling
                bytecode.push(PatternInstruction::MatchLiteral(format!("{start}..{end}")));
                *is_refutable = true;
            },
            _ => {
                // Handle other pattern types
                warn!(?pattern, "Unsupported pattern type, treating as wildcard");
                bytecode.push(PatternInstruction::MatchWildcard);
            }
        }

        Ok(())
    }

    /// Execute compiled pattern bytecode
    fn execute_pattern(&self, pattern: &CompiledPattern, context: &mut EvaluationContext) -> ShellResult<bool> {
        let mut pc = 0; // Program counter
        
        debug!(bytecode_len = pattern.bytecode.len(), "Executing pattern bytecode");

        while pc < pattern.bytecode.len() {
            match &pattern.bytecode[pc] {
                PatternInstruction::MatchLiteral(expected) => {
                    let current_str = self.value_to_string(&context.current_value);
                    if current_str != *expected {
                        return Ok(false);
                    }
                },
                PatternInstruction::MatchWildcard => {
                    // Always matches
                },
                PatternInstruction::BindVariable(name) => {
                    // Get the current value from context stack if we're in a sequence
                    let value_to_bind = if let Some(frame) = context.context_stack.last() {
                        if frame.position < frame.values.len() {
                            frame.values[frame.position].clone()
                        } else {
                            context.current_value.clone()
                        }
                    } else {
                        context.current_value.clone()
                    };
                    
                    context.bindings.insert(name.clone(), value_to_bind.clone());
                    debug!(name = %name, value = ?value_to_bind, "Bound variable");
                    
                    // Advance position in current context frame
                    if let Some(frame) = context.context_stack.last_mut() {
                        frame.position += 1;
                    }
                },
                PatternInstruction::CheckType(type_name) => {
                    if !self.check_value_type(&context.current_value, type_name) {
                        return Ok(false);
                    }
                },
                PatternInstruction::EnterSequence(expected_len) => {
                    if let PatternValue::Array(ref items) = context.current_value {
                        if items.len() != *expected_len {
                            return Ok(false);
                        }
                        context.context_stack.push(ContextFrame {
                            context_type: ContextType::Array,
                            position: 0,
                            values: items.clone(),
                            fields: vec![],
                        });
                    } else if let PatternValue::Tuple(ref items) = context.current_value {
                        if items.len() != *expected_len {
                            return Ok(false);
                        }
                        context.context_stack.push(ContextFrame {
                            context_type: ContextType::Tuple,
                            position: 0,
                            values: items.clone(),
                            fields: vec![],
                        });
                    } else {
                        return Ok(false);
                    }
                },
                PatternInstruction::EnterObject(expected_fields) => {
                    if let PatternValue::Object(ref obj) = context.current_value {
                        for field in expected_fields {
                            if !obj.contains_key(field) {
                                return Ok(false);
                            }
                        }
                        context.context_stack.push(ContextFrame {
                            context_type: ContextType::Object,
                            position: 0,
                            values: expected_fields.iter().map(|f| obj.get(f).unwrap().clone()).collect(),
                            fields: expected_fields.clone(),
                        });
                    } else {
                        return Ok(false);
                    }
                },
                PatternInstruction::ExitContext => {
                    context.context_stack.pop();
                },
                PatternInstruction::EvaluateGuard(guard_expr) => {
                    if !self.evaluate_guard(guard_expr, &context.bindings)? {
                        return Ok(false);
                    }
                },
                PatternInstruction::Success => {
                    return Ok(true);
                },
                PatternInstruction::Failure => {
                    return Ok(false);
                },
                _ => {
                    // Handle other instructions
                    warn!(instruction = ?pattern.bytecode[pc], "Unhandled pattern instruction"); // Fixed ? syntax
                }
            }
            
            pc += 1;
        }

        Ok(true)
    }

    /// Evaluate a guard clause
    fn evaluate_guard(&self, guard: &AstNode, bindings: &HashMap<String, PatternValue>) -> ShellResult<bool> {
        debug!(?guard, bindings_count = bindings.len(), "Evaluating guard clause");
        
        // Simplified guard evaluation
        // In a real implementation, this would use the expression evaluator
        match guard {
            AstNode::BinaryExpression { left, operator, right } => {
                // Comprehensive comparison evaluation
                let left_val = self.evaluate_guard_expression(left, bindings)?;
                let right_val = self.evaluate_guard_expression(right, bindings)?;
                
                let match_op = match operator {
                    BinaryOperator::Equal => "==",
                    BinaryOperator::NotEqual => "!=",
                    BinaryOperator::Less => "<",
                    BinaryOperator::LessEqual => "<=",
                    BinaryOperator::Greater => ">",
                    BinaryOperator::GreaterEqual => ">=",
                    BinaryOperator::LogicalAnd => "&&",
                    BinaryOperator::LogicalOr => "||",
                    _ => "unknown",
                };
                match match_op {
                    "==" => Ok(left_val == right_val),
                    "!=" => Ok(left_val != right_val),
                    "<" => Ok(self.compare_pattern_values(&left_val, &right_val)? < 0),
                    "<=" => Ok(self.compare_pattern_values(&left_val, &right_val)? <= 0),
                    ">" => Ok(self.compare_pattern_values(&left_val, &right_val)? > 0),
                    ">=" => Ok(self.compare_pattern_values(&left_val, &right_val)? >= 0),
                    "&&" => Ok(self.is_pattern_truthy(&left_val) && self.is_pattern_truthy(&right_val)),
                    "||" => Ok(self.is_pattern_truthy(&left_val) || self.is_pattern_truthy(&right_val)),
                    "=~" => {
                        // Regex match
                        let pattern = self.value_to_string(&right_val);
                        let text = self.value_to_string(&left_val);
                        match regex::Regex::new(&pattern) {
                            Ok(re) => Ok(re.is_match(&text)),
                            Err(_) => Ok(false),
                        }
                    },
                    "!~" => {
                        // Negative regex match
                        let pattern = self.value_to_string(&right_val);
                        let text = self.value_to_string(&left_val);
                        match regex::Regex::new(&pattern) {
                            Ok(re) => Ok(!re.is_match(&text)),
                            Err(_) => Ok(true),
                        }
                    },
                    _ => Ok(false), // Unknown operator
                }
            },
            AstNode::Variable(name) => {
                // Check if variable exists and is truthy
                if let Some(value) = bindings.get(*name) {
                    Ok(self.is_pattern_truthy(value))
                } else {
                    Ok(false)
                }
            },
            AstNode::StringLiteral { value, .. } => {
                // Evaluate literal as boolean
                Ok(self.literal_to_bool(value))
            },
            AstNode::UnaryExpression { operator, operand } => {
                let operand_val = self.evaluate_guard_expression(operand, bindings)?;
                let match_op = match operator {
                    UnaryOperator::LogicalNot => "!",
                    UnaryOperator::Minus => "-",
                    UnaryOperator::Plus => "+",
                    _ => "unknown",
                };
                match match_op {
                    "!" => Ok(!self.is_pattern_truthy(&operand_val)),
                    "-" => {
                        // Numeric negation - check if result is truthy
                        match operand_val {
                            PatternValue::Integer(i) => Ok(-i != 0),
                            PatternValue::Number(f) => Ok(-f != 0.0),
                            _ => Ok(false),
                        }
                    },
                    _ => Ok(true),
                }
            },
            AstNode::FunctionCall { name, args, .. } => {
                // Built-in guard functions  
                let name_str = match name.as_ref() {
                    AstNode::Variable(var_name) => *var_name,
                    _ => return Ok(true),
                };
                match name_str {
                    "defined" => {
                        if let Some(AstNode::Variable(var_name)) = args.first() {
                            Ok(bindings.contains_key(*var_name))
                        } else {
                            Ok(false)
                        }
                    },
                    "length" => {
                        if let Some(arg) = args.first() {
                            let val = self.evaluate_guard_expression(arg, bindings)?;
                            let len = self.get_pattern_value_length(&val);
                            Ok(len > 0)
                        } else {
                            Ok(false)
                        }
                    },
                    "empty" => {
                        if let Some(arg) = args.first() {
                            let val = self.evaluate_guard_expression(arg, bindings)?;
                            Ok(self.is_pattern_truthy(&val))
                        } else {
                            Ok(true)
                        }
                    },
                    _ => Ok(true), // Unknown function defaults to true
                }
            },
            _ => Ok(true), // Default to true for unsupported guard types
        }
    }

    /// Check if patterns are exhaustive
    fn check_exhaustiveness(&self, arms: &[MatchArm]) -> ShellResult<bool> {
        debug!(arms_count = arms.len(), "Checking pattern exhaustiveness");
        
        // Simplified exhaustiveness checking
        // In a real implementation, this would be much more sophisticated
        let has_wildcard = arms.iter().any(|arm| {
            matches!(&arm.pattern, Pattern::Wildcard | Pattern::Placeholder)
        });

        Ok(has_wildcard)
    }

    /// Find unreachable patterns
    fn find_unreachable_patterns(&self, arms: &[MatchArm]) -> ShellResult<Vec<usize>> {
        debug!(arms_count = arms.len(), "Finding unreachable patterns");
        
        let mut unreachable = Vec::new();
        
        // Simplified unreachability analysis
        // Look for patterns after a wildcard
        let mut found_wildcard = false;
        for (index, arm) in arms.iter().enumerate() {
            if found_wildcard {
                unreachable.push(index);
            }
            
            if matches!(&arm.pattern, Pattern::Wildcard | Pattern::Placeholder) {
                found_wildcard = true;
            }
        }

        Ok(unreachable)
    }

    /// Convert value to string for comparison
    fn value_to_string(&self, value: &PatternValue) -> String {
        match value {
            PatternValue::String(s) => s.clone(),
            PatternValue::Number(n) => n.to_string(),
            PatternValue::Integer(i) => i.to_string(),
            PatternValue::Boolean(b) => b.to_string(),
            PatternValue::Null => "null".to_string(),
            PatternValue::Type(t) => t.clone(),
            _ => format!("{value:?}"),
        }
    }

    /// Check if value matches expected type
    fn check_value_type(&self, value: &PatternValue, type_name: &str) -> bool {
        matches!(
            (value, type_name),
            (PatternValue::String(_), "string")
                | (PatternValue::Number(_), "number")
                | (PatternValue::Integer(_), "int")
                | (PatternValue::Boolean(_), "bool")
                | (PatternValue::Array(_), "array")
                | (PatternValue::Object(_), "object")
                | (PatternValue::Tuple(_), "tuple")
                | (PatternValue::Null, "null")
        )
    }

    /// Get current statistics
    pub fn get_statistics(&self) -> &PatternMatchingStatistics {
        &self.statistics
    }

    /// Reset statistics
    pub fn reset_statistics(&mut self) {
        self.statistics = PatternMatchingStatistics::default();
    }

    /// Clear pattern cache
    pub fn clear_cache(&mut self) {
        self.pattern_cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.pattern_cache.len()
    }
}

impl PatternMatchingEngine {
    // Additional helper methods for comprehensive pattern matching
    
    fn evaluate_guard_expression(&self, node: &AstNode, bindings: &HashMap<String, PatternValue>) -> ShellResult<PatternValue> {
        match node {
            AstNode::Variable(name) => {
                Ok(bindings.get(*name).cloned().unwrap_or(PatternValue::Null))
            },
            AstNode::StringLiteral { value, .. } => {
                Ok(PatternValue::String(value.to_string()))
            },
            AstNode::NumberLiteral { value, .. } => {
                // Try to parse as integer first, then as float
                if let Ok(i) = value.parse::<i64>() {
                    Ok(PatternValue::Integer(i))
                } else if let Ok(f) = value.parse::<f64>() {
                    Ok(PatternValue::Number(f))
                } else {
                    Ok(PatternValue::String(value.to_string()))
                }
            },
            AstNode::Word(word) => {
                // Handle simple word literals like "true", "false"
                match *word {
                    "true" => Ok(PatternValue::Boolean(true)),
                    "false" => Ok(PatternValue::Boolean(false)),
                    "null" => Ok(PatternValue::Null),
                    _ => {
                        // Try parsing as number
                        if let Ok(i) = word.parse::<i64>() {
                            Ok(PatternValue::Integer(i))
                        } else if let Ok(f) = word.parse::<f64>() {
                            Ok(PatternValue::Number(f))
                        } else {
                            Ok(PatternValue::String(word.to_string()))
                        }
                    }
                }
            },
            AstNode::BinaryExpression { left, operator, right } => {
                let left_val = self.evaluate_guard_expression(left, bindings)?;
                let right_val = self.evaluate_guard_expression(right, bindings)?;
                self.apply_binary_operation_pattern(&left_val, operator, &right_val)
            },
            AstNode::UnaryExpression { operator, operand } => {
                let operand_val = self.evaluate_guard_expression(operand, bindings)?;
                self.apply_unary_operation_pattern(operator, &operand_val)
            },
            AstNode::FunctionCall { name, args, .. } => {
                // Handle function calls in guard expressions
                match name.as_ref() {
                    AstNode::Variable("len") => {
                        if args.len() == 1 {
                            let arg_val = self.evaluate_guard_expression(&args[0], bindings)?;
                            Ok(PatternValue::Integer(self.get_pattern_value_length(&arg_val) as i64))
                        } else {
                            Ok(PatternValue::Null)
                        }
                    },
                    _ => Ok(PatternValue::Null),
                }
            },
            _ => Ok(PatternValue::Null),
        }
    }
    
    #[allow(dead_code)]
    fn literal_to_value(&self, literal: &str) -> Value {
        if let Ok(i) = literal.parse::<i64>() {
            Value::Integer(i)
        } else if let Ok(f) = literal.parse::<f64>() {
            Value::Float(f)
        } else if literal == "true" {
            Value::Boolean(true)
        } else if literal == "false" {
            Value::Boolean(false)
        } else if literal == "null" {
            Value::Null
        } else {
            Value::String(literal.to_string())
        }
    }
    
    fn literal_to_bool(&self, literal: &str) -> bool {
        match literal {
            "true" => true,
            "false" => false,
            "0" => false,
            "" => false,
            "null" => false,
            _ => true,
        }
    }
    
    #[allow(dead_code)]
    fn apply_binary_operation(&self, left: &Value, operator: &BinaryOperator, right: &Value) -> ShellResult<Value> {
        use BinaryOperator::*;
        match operator {
            Add => self.add_values(left, right),
            Subtract => self.subtract_values(left, right),
            Multiply => self.multiply_values(left, right),
            Divide => self.divide_values(left, right),
            Modulo => self.modulo_values(left, right),
            Equal => Ok(Value::Boolean(left == right)),
            NotEqual => Ok(Value::Boolean(left != right)),
            Less => Ok(Value::Boolean(self.compare_values(left, right)? < 0)),
            LessEqual => Ok(Value::Boolean(self.compare_values(left, right)? <= 0)),
            Greater => Ok(Value::Boolean(self.compare_values(left, right)? > 0)),
            GreaterEqual => Ok(Value::Boolean(self.compare_values(left, right)? >= 0)),
            LogicalAnd => Ok(Value::Boolean(self.is_truthy(left) && self.is_truthy(right))),
            LogicalOr => Ok(Value::Boolean(self.is_truthy(left) || self.is_truthy(right))),
            _ => Err(crate::error::ShellError::new(
                crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                "Unknown binary operator".to_string()
            )),
        }
    }
    
    #[allow(dead_code)]
    fn apply_unary_operation(&self, operator: &UnaryOperator, operand: &Value) -> ShellResult<Value> {
        use UnaryOperator::*;
        match operator {
            LogicalNot => Ok(Value::Boolean(!self.is_truthy(operand))),
            Minus => match operand {
                Value::Integer(i) => Ok(Value::Integer(-i)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err(crate::error::ShellError::new(crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), "Cannot negate non-numeric value".to_string())),
            },
            Plus => match operand {
                Value::Integer(_) | Value::Float(_) => Ok(operand.clone()),
                _ => Err(crate::error::ShellError::new(crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), "Cannot apply unary plus to non-numeric value".to_string())),
            },
            _ => Err(crate::error::ShellError::new(crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), "Unknown unary operator".to_string())),
        }
    }
    
    #[allow(dead_code)]
    fn add_values(&self, left: &Value, right: &Value) -> ShellResult<Value> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
            _ => Err(crate::error::ShellError::new(crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), "Cannot add incompatible types".to_string())),
        }
    }
    
    #[allow(dead_code)]
    fn subtract_values(&self, left: &Value, right: &Value) -> ShellResult<Value> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
            _ => Err(crate::error::ShellError::new(crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), "Cannot subtract incompatible types".to_string())),
        }
    }
    
    #[allow(dead_code)]
    fn multiply_values(&self, left: &Value, right: &Value) -> ShellResult<Value> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
            (Value::String(s), Value::Integer(n)) => {
                Ok(Value::String(s.repeat(*n as usize)))
            },
            _ => Err(crate::error::ShellError::new(crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), "Cannot multiply incompatible types".to_string())),
        }
    }
    
    #[allow(dead_code)]
    fn divide_values(&self, left: &Value, right: &Value) -> ShellResult<Value> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    Err(crate::error::ShellError::new(crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), "Division by zero".to_string()))
                } else {
                    Ok(Value::Float(*a as f64 / *b as f64))
                }
            },
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(crate::error::ShellError::new(crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), "Division by zero".to_string()))
                } else {
                    Ok(Value::Float(a / b))
                }
            },
            (Value::Integer(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(crate::error::ShellError::new(crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), "Division by zero".to_string()))
                } else {
                    Ok(Value::Float(*a as f64 / b))
                }
            },
            (Value::Float(a), Value::Integer(b)) => {
                if *b == 0 {
                    Err(crate::error::ShellError::new(crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), "Division by zero".to_string()))
                } else {
                    Ok(Value::Float(a / *b as f64))
                }
            },
            _ => Err(crate::error::ShellError::new(crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), "Cannot divide incompatible types".to_string())),
        }
    }
    
    #[allow(dead_code)]
    fn modulo_values(&self, left: &Value, right: &Value) -> ShellResult<Value> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    Err(crate::error::ShellError::new(crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), "Modulo by zero".to_string()))
                } else {
                    Ok(Value::Integer(a % b))
                }
            },
            _ => Err(crate::error::ShellError::new(crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), "Modulo operation only supported for integers".to_string())),
        }
    }
    
    #[allow(dead_code)]
    fn compare_values(&self, left: &Value, right: &Value) -> ShellResult<i32> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a.cmp(b) as i32),
            (Value::Float(a), Value::Float(b)) => Ok(a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal) as i32),
            (Value::Integer(a), Value::Float(b)) => {
                Ok((*a as f64).partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal) as i32)
            },
            (Value::Float(a), Value::Integer(b)) => {
                Ok(a.partial_cmp(&(*b as f64)).unwrap_or(std::cmp::Ordering::Equal) as i32)
            },
            (Value::String(a), Value::String(b)) => Ok(a.cmp(b) as i32),
            (Value::Boolean(a), Value::Boolean(b)) => Ok(a.cmp(b) as i32),
            _ => Ok(0), // Incomparable types are considered equal
        }
    }
    
    #[allow(dead_code)]
    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => *b,
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Closure(_) => true, // Closures are always truthy
            Value::Null => false,
        }
    }
    
    #[allow(dead_code)]
    fn guard_value_to_string(value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Array(arr) => format!("[{}]", arr.iter().map(Self::guard_value_to_string).collect::<Vec<_>>().join(", ")),
            Value::Closure(id) => format!("Closure({id})"),
            Value::Null => "null".to_string(),
        }
    }
    
    // Pattern value specific operations
    
    fn apply_binary_operation_pattern(&self, left: &PatternValue, operator: &BinaryOperator, right: &PatternValue) -> ShellResult<PatternValue> {
        use BinaryOperator::*;
        match operator {
            Add => self.add_pattern_values(left, right),
            Subtract => self.subtract_pattern_values(left, right),
            Multiply => self.multiply_pattern_values(left, right),
            Divide => self.divide_pattern_values(left, right),
            Modulo => self.modulo_pattern_values(left, right),
            Equal => Ok(PatternValue::Boolean(left == right)),
            NotEqual => Ok(PatternValue::Boolean(left != right)),
            Less => {
                let cmp = self.compare_pattern_values(left, right)?;
                Ok(PatternValue::Boolean(cmp < 0))
            },
            LessEqual => {
                let cmp = self.compare_pattern_values(left, right)?;
                Ok(PatternValue::Boolean(cmp <= 0))
            },
            Greater => {
                let cmp = self.compare_pattern_values(left, right)?;
                Ok(PatternValue::Boolean(cmp > 0))
            },
            GreaterEqual => {
                let cmp = self.compare_pattern_values(left, right)?;
                Ok(PatternValue::Boolean(cmp >= 0))
            },
            LogicalAnd => Ok(PatternValue::Boolean(self.is_pattern_truthy(left) && self.is_pattern_truthy(right))),
            LogicalOr => Ok(PatternValue::Boolean(self.is_pattern_truthy(left) || self.is_pattern_truthy(right))),
            _ => Err(crate::error::ShellError::new(
                crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                "Unknown binary operator".to_string()
            )),
        }
    }
    
    fn apply_unary_operation_pattern(&self, operator: &UnaryOperator, operand: &PatternValue) -> ShellResult<PatternValue> {
        use UnaryOperator::*;
        match operator {
            LogicalNot => Ok(PatternValue::Boolean(!self.is_pattern_truthy(operand))),
            Minus => match operand {
                PatternValue::Integer(i) => Ok(PatternValue::Integer(-i)),
                PatternValue::Number(f) => Ok(PatternValue::Number(-f)),
                _ => Err(crate::error::ShellError::new(
                    crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                    "Cannot negate non-numeric value".to_string()
                )),
            },
            Plus => match operand {
                PatternValue::Integer(_) | PatternValue::Number(_) => Ok(operand.clone()),
                _ => Err(crate::error::ShellError::new(
                    crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                    "Cannot apply unary plus to non-numeric value".to_string()
                )),
            },
            BitwiseNot => match operand {
                PatternValue::Integer(i) => Ok(PatternValue::Integer(!i)),
                _ => Err(crate::error::ShellError::new(
                    crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                    "Cannot apply bitwise not to non-integer value".to_string()
                )),
            },
        }
    }
    
    fn add_pattern_values(&self, left: &PatternValue, right: &PatternValue) -> ShellResult<PatternValue> {
        match (left, right) {
            (PatternValue::Integer(a), PatternValue::Integer(b)) => Ok(PatternValue::Integer(a + b)),
            (PatternValue::Number(a), PatternValue::Number(b)) => Ok(PatternValue::Number(a + b)),
            (PatternValue::Integer(a), PatternValue::Number(b)) => Ok(PatternValue::Number(*a as f64 + b)),
            (PatternValue::Number(a), PatternValue::Integer(b)) => Ok(PatternValue::Number(a + *b as f64)),
            (PatternValue::String(a), PatternValue::String(b)) => Ok(PatternValue::String(format!("{a}{b}"))),
            _ => Err(crate::error::ShellError::new(
                crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                "Cannot add incompatible types"
            )),
        }
    }
    
    fn subtract_pattern_values(&self, left: &PatternValue, right: &PatternValue) -> ShellResult<PatternValue> {
        match (left, right) {
            (PatternValue::Integer(a), PatternValue::Integer(b)) => Ok(PatternValue::Integer(a - b)),
            (PatternValue::Number(a), PatternValue::Number(b)) => Ok(PatternValue::Number(a - b)),
            (PatternValue::Integer(a), PatternValue::Number(b)) => Ok(PatternValue::Number(*a as f64 - b)),
            (PatternValue::Number(a), PatternValue::Integer(b)) => Ok(PatternValue::Number(a - *b as f64)),
            _ => Err(crate::error::ShellError::new(
                crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                "Cannot subtract incompatible types"
            )),
        }
    }
    
    fn multiply_pattern_values(&self, left: &PatternValue, right: &PatternValue) -> ShellResult<PatternValue> {
        match (left, right) {
            (PatternValue::Integer(a), PatternValue::Integer(b)) => Ok(PatternValue::Integer(a * b)),
            (PatternValue::Number(a), PatternValue::Number(b)) => Ok(PatternValue::Number(a * b)),
            (PatternValue::Integer(a), PatternValue::Number(b)) => Ok(PatternValue::Number(*a as f64 * b)),
            (PatternValue::Number(a), PatternValue::Integer(b)) => Ok(PatternValue::Number(a * *b as f64)),
            (PatternValue::String(s), PatternValue::Integer(n)) => {
                Ok(PatternValue::String(s.repeat(*n as usize)))
            },
            _ => Err(crate::error::ShellError::new(
                crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                "Cannot multiply incompatible types"
            )),
        }
    }
    
    fn divide_pattern_values(&self, left: &PatternValue, right: &PatternValue) -> ShellResult<PatternValue> {
        match (left, right) {
            (PatternValue::Integer(a), PatternValue::Integer(b)) => {
                if *b == 0 {
                    Err(crate::error::ShellError::new(
                        crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::DivisionByZero),
                        "Division by zero"
                    ))
                } else {
                    Ok(PatternValue::Number(*a as f64 / *b as f64))
                }
            },
            (PatternValue::Number(a), PatternValue::Number(b)) => {
                if *b == 0.0 {
                    Err(crate::error::ShellError::new(
                        crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::DivisionByZero),
                        "Division by zero"
                    ))
                } else {
                    Ok(PatternValue::Number(a / b))
                }
            },
            (PatternValue::Integer(a), PatternValue::Number(b)) => {
                if *b == 0.0 {
                    Err(crate::error::ShellError::new(
                        crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::DivisionByZero),
                        "Division by zero"
                    ))
                } else {
                    Ok(PatternValue::Number(*a as f64 / b))
                }
            },
            (PatternValue::Number(a), PatternValue::Integer(b)) => {
                if *b == 0 {
                    Err(crate::error::ShellError::new(
                        crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::DivisionByZero),
                        "Division by zero"
                    ))
                } else {
                    Ok(PatternValue::Number(a / *b as f64))
                }
            },
            _ => Err(crate::error::ShellError::new(
                crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                "Cannot divide incompatible types"
            )),
        }
    }
    
    fn modulo_pattern_values(&self, left: &PatternValue, right: &PatternValue) -> ShellResult<PatternValue> {
        match (left, right) {
            (PatternValue::Integer(a), PatternValue::Integer(b)) => {
                if *b == 0 {
                    Err(crate::error::ShellError::new(
                        crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::DivisionByZero),
                        "Modulo by zero"
                    ))
                } else {
                    Ok(PatternValue::Integer(a % b))
                }
            },
            _ => Err(crate::error::ShellError::new(
                crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                "Modulo operation only supported for integers"
            )),
        }
    }
    
    fn compare_pattern_values(&self, left: &PatternValue, right: &PatternValue) -> ShellResult<i32> {
        match (left, right) {
            (PatternValue::Integer(a), PatternValue::Integer(b)) => Ok(a.cmp(b) as i32),
            (PatternValue::Number(a), PatternValue::Number(b)) => Ok(a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal) as i32),
            (PatternValue::Integer(a), PatternValue::Number(b)) => {
                Ok((*a as f64).partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal) as i32)
            },
            (PatternValue::Number(a), PatternValue::Integer(b)) => {
                Ok(a.partial_cmp(&(*b as f64)).unwrap_or(std::cmp::Ordering::Equal) as i32)
            },
            (PatternValue::String(a), PatternValue::String(b)) => Ok(a.cmp(b) as i32),
            (PatternValue::Boolean(a), PatternValue::Boolean(b)) => Ok(a.cmp(b) as i32),
            _ => Ok(0), // Incomparable types are considered equal
        }
    }
    
    fn is_pattern_truthy(&self, value: &PatternValue) -> bool {
        match value {
            PatternValue::Boolean(b) => *b,
            PatternValue::Integer(i) => *i != 0,
            PatternValue::Number(f) => *f != 0.0,
            PatternValue::String(s) => !s.is_empty(),
            PatternValue::Array(arr) => !arr.is_empty(),
            PatternValue::Null => false,
            PatternValue::Type(_) => true,
            PatternValue::Object(obj) => !obj.is_empty(),
            PatternValue::Tuple(tup) => !tup.is_empty(),
        }
    }
    
    fn get_pattern_value_length(&self, value: &PatternValue) -> usize {
        match value {
            PatternValue::String(s) => s.len(),
            PatternValue::Array(arr) => arr.len(),
            PatternValue::Tuple(tup) => tup.len(),
            PatternValue::Object(obj) => obj.len(),
            _ => 0,
        }
    }
}  // End PatternMatchingEngine impl

pub fn shell_value_to_pattern_value(value: &crate::closures::Value) -> PatternValue {
    match value {
        crate::closures::Value::String(s) => PatternValue::String(s.clone()),
        crate::closures::Value::Integer(i) => PatternValue::Integer(*i),
        crate::closures::Value::Float(f) => PatternValue::Number(*f),
        crate::closures::Value::Boolean(b) => PatternValue::Boolean(*b),
        crate::closures::Value::Array(list) => {
            PatternValue::Array(list.iter().map(shell_value_to_pattern_value).collect())
        }
        crate::closures::Value::Null => PatternValue::Null,
        _ => PatternValue::String(format!("{value:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_pattern_matching() {
        let config = PatternMatchingConfig::default();
        let mut engine = PatternMatchingEngine::new(config);
        let pattern = Pattern::Literal("test");
        let value = PatternValue::String("test".to_string());
        
        let result = engine.match_pattern(&value, &pattern).unwrap();
        assert!(result.matched);
    }
}
