#![doc = "Command-line parser turning raw input into an AST."]

pub mod lexer;
pub mod ast;

#[cfg(test)]
mod tests;

// Re-export the Parser for external use
pub use ShellCommandParser as Parser;

use anyhow::{Result, Context};

use pest::Parser as PestParser;
use pest::error::{Error as PestError, LineColLocation};
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar/shell.pest"]
struct ShellParser;

/// Parse states for if statement processing
#[derive(Debug, Clone, PartialEq)]
enum IfParseState {
    Condition,
    ThenBranch,
    ElifCondition,
    ElifBranch,
    ElseBranch,
}

/// Parse states for for statement processing
#[derive(Debug, Clone, PartialEq)]
enum ForParseState {
    Variable,
    In,
    Arguments,
    Body,
}

/// Parse states for while statement processing
#[derive(Debug, Clone, PartialEq)]
enum WhileParseState {
    Condition,
    Body,
}

/// Parse states for case statement processing
#[derive(Debug, Clone, PartialEq)]
enum CaseParseState {
    Expression,
    Items,
}

/// Parse states for function definition processing
#[derive(Debug, Clone, PartialEq)]
enum FunctionParseState {
    Name,
    Parameters,
    Body,
}

/// Parse states for match statement processing
#[derive(Debug, Clone, PartialEq)]
enum MatchParseState {
    Expression,
    Arms,
}

/// Public parser interface for shell commands
pub struct ShellCommandParser {
    _private: (),
}

impl Default for ShellCommandParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellCommandParser {
    /// Create a new parser instance
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Parse shell command text into an AST
    pub fn parse(&self, input: &str) -> Result<ast::AstNode<'static>> {
        let pairs = ShellParser::parse(Rule::program, input)
            .with_context(|| format!("Failed to parse input: {input}"))?;
        
        let ast = self.build_ast_from_pairs(pairs, input)?;
        Ok(ast)
    }

    /// Build AST from parsed PEST pairs
    fn build_ast_from_pairs(&self, pairs: Pairs<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        let mut statements = Vec::new();
        
        for pair in pairs {
            match pair.as_rule() {
                Rule::program => {
                    // Process the program's inner pairs
                    for inner_pair in pair.into_inner() {
                        if inner_pair.as_rule() == Rule::line {
                            let statement = self.parse_line(inner_pair, input)?;
                            if let Some(stmt) = statement {
                                statements.push(stmt);
                            }
                        } else if inner_pair.as_rule() == Rule::inner_program {
                            // Handle inner_program
                            let inner_ast = self.build_ast_from_pairs(inner_pair.into_inner(), input)?;
                            statements.push(inner_ast);
                        }
                    }
                }
                Rule::inner_program => {
                    // Process inner program directly
                    for inner_pair in pair.into_inner() {
                        if inner_pair.as_rule() == Rule::line {
                            let statement = self.parse_line(inner_pair, input)?;
                            if let Some(stmt) = statement {
                                statements.push(stmt);
                            }
                        }
                    }
                }
                Rule::line => {
                    let statement = self.parse_line(pair, input)?;
                    if let Some(stmt) = statement {
                        statements.push(stmt);
                    }
                }
                Rule::statement => {
                    // Handle statement rule directly
                    let statement = self.parse_statement(pair, input)?;
                    statements.push(statement);
                }
                Rule::EOI => {
                    // End of input - ignore
                }
                _ => {
                    return Err(anyhow::anyhow!("Unexpected top-level rule: {:?}", pair.as_rule()));
                }
            }
        }
        
        if statements.len() == 1 {
            statements.into_iter().next().ok_or_else(|| {
                anyhow::anyhow!("No statements found after parsing")
            })
        } else {
            Ok(ast::AstNode::Program(statements))
        }
    }

    /// Parse a line (statement with optional operators)
    fn parse_line(&self, pair: Pair<Rule>, input: &str) -> Result<Option<ast::AstNode<'static>>> {
        let mut current_node: Option<ast::AstNode<'static>> = None;
        let mut background = false;
        
        let inner_pairs: Vec<_> = pair.into_inner().collect();
        let mut i = 0;
        
        while i < inner_pairs.len() {
            let inner_pair = &inner_pairs[i];
            
            match inner_pair.as_rule() {
                Rule::statement => {
                    #[cfg(debug_assertions)]
                    #[cfg(feature = "debug_parse")]
                    eprintln!("PARSE_DEBUG: saw statement: {}", inner_pair.as_str());
                    let stmt = self.parse_statement(inner_pair.clone(), input)?;
                    
                    if current_node.is_none() {
                        current_node = Some(stmt);
                    } else {
                        // This should not happen with proper grammar parsing
                        return Err(anyhow::anyhow!("Unexpected statement sequence"));
                    }
                }
                Rule::and_op => {
                    #[cfg(debug_assertions)]
                    #[cfg(feature = "debug_parse")]
                    eprintln!("PARSE_DEBUG: saw && operator");
                    // Handle && operator: execute next command only if current succeeds
                    if let Some(left) = current_node.take() {
                        i += 1; // Move to next statement
                        if i < inner_pairs.len() && inner_pairs[i].as_rule() == Rule::statement {
                            let right = self.parse_statement(inner_pairs[i].clone(), input)?;
                            current_node = Some(ast::AstNode::LogicalAnd {
                                left: Box::new(left),
                                right: Box::new(right),
                            });
                        } else {
                            return Err(anyhow::anyhow!("Expected statement after && operator"));
                        }
                    } else {
                        return Err(anyhow::anyhow!("No left operand for && operator"));
                    }
                }
                Rule::or_op => {
                    #[cfg(debug_assertions)]
                    #[cfg(feature = "debug_parse")]
                    eprintln!("PARSE_DEBUG: saw || operator");
                    // Handle || operator: execute next command only if current fails
                    if let Some(left) = current_node.take() {
                        i += 1; // Move to next statement
                        if i < inner_pairs.len() && inner_pairs[i].as_rule() == Rule::statement {
                            let right = self.parse_statement(inner_pairs[i].clone(), input)?;
                            current_node = Some(ast::AstNode::LogicalOr {
                                left: Box::new(left),
                                right: Box::new(right),
                            });
                        } else {
                            return Err(anyhow::anyhow!("Expected statement after || operator"));
                        }
                    } else {
                        return Err(anyhow::anyhow!("No left operand for || operator"));
                    }
                }
                Rule::semicolon => {
                    #[cfg(debug_assertions)]
                    #[cfg(feature = "debug_parse")]
                    eprintln!("PARSE_DEBUG: saw ; operator");
                    // Handle ; operator: execute commands sequentially regardless of exit status
                    if let Some(left) = current_node.take() {
                        i += 1; // Move to next statement
                        if i < inner_pairs.len() && inner_pairs[i].as_rule() == Rule::statement {
                            let right = self.parse_statement(inner_pairs[i].clone(), input)?;
                            current_node = Some(ast::AstNode::Sequence {
                                left: Box::new(left),
                                right: Box::new(right),
                            });
                        } else {
                            return Err(anyhow::anyhow!("Expected statement after ; operator"));
                        }
                    } else {
                        return Err(anyhow::anyhow!("No left operand for ; operator"));
                    }
                }
                Rule::background => {
                    #[cfg(debug_assertions)]
                    #[cfg(feature = "debug_parse")]
                    eprintln!("PARSE_DEBUG: saw background token '&'");
                    background = true;
                }
                _ => {
                    // Ignore other rules that might be present
                }
            }
            
            i += 1;
        }
        
        // Apply background flag if present
        if let Some(mut node) = current_node.take() {
            if background {
                node = self.mark_background(node);
            }
            Ok(Some(node))
        } else {
            Ok(None)
        }
    }

    /// Parse a statement (command, control structure, etc.)
    fn parse_statement(&self, pair: Pair<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::command => {
                    return self.parse_command(inner_pair, input);
                }
                Rule::if_statement => {
                    return self.parse_if_statement(inner_pair, input);
                }
                Rule::for_statement => {
                    return self.parse_for_statement(inner_pair, input);
                }
                Rule::while_statement => {
                    return self.parse_while_statement(inner_pair, input);
                }
                Rule::case_statement => {
                    return self.parse_case_statement(inner_pair, input);
                }
                Rule::function_def => {
                    return self.parse_function_def(inner_pair, input);
                }
                Rule::macro_declaration => {
                    return self.parse_macro_declaration(inner_pair, input);
                }
                Rule::macro_invocation => {
                    return self.parse_macro_invocation(inner_pair, input);
                }
                Rule::match_statement => {
                    return self.parse_match_statement(inner_pair, input);
                }
                Rule::closure_expr => {
                    return self.parse_closure_expr(inner_pair, input);
                }
                _ => {}
            }
        }
        
        Err(anyhow::anyhow!("Unable to parse statement"))
    }

    /// Parse a command (simple command or pipeline)
    fn parse_command(&self, pair: Pair<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::pipeline {
                return self.parse_pipeline(inner_pair, input);
            }
        }
        
        Err(anyhow::anyhow!("Unable to parse command"))
    }

    /// Parse a pipeline
    fn parse_pipeline(&self, pair: Pair<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        let mut commands = Vec::new();
        let mut operators = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::simple_command => {
                    let cmd = self.parse_simple_command(inner_pair, input)?;
                    #[cfg(debug_assertions)]
                    #[cfg(feature = "debug_parse")]
                    eprintln!("PARSE_DEBUG: pipeline push simple_command");
                    commands.push(cmd);
                }
                Rule::command_element => {
                    // Unwrap command_element to its inner content
                    let mut found = false;
                    for ce_inner in inner_pair.into_inner() {
                        match ce_inner.as_rule() {
                            Rule::simple_command => {
                                let cmd = self.parse_simple_command(ce_inner, input)?;
                                #[cfg(debug_assertions)]
                                #[cfg(feature = "debug_parse")]
                                eprintln!("PARSE_DEBUG: pipeline push command_element->simple_command");
                                commands.push(cmd);
                                found = true;
                            }
                            Rule::subshell => {
                                // For now treat subshell as its own node
                                let text = ce_inner.as_str();
                                let node = ast::AstNode::Subshell(Box::new(ast::AstNode::Word(self.leak_string(text))));
                                commands.push(node);
                                found = true;
                            }
                            _ => {}
                        }
                    }
                    if !found {
                        #[cfg(debug_assertions)]
                        #[cfg(feature = "debug_parse")]
                        eprintln!("PARSE_DEBUG: empty command_element encountered");
                    }
                }
                Rule::pipe => {
                    operators.push(ast::PipeOperator::Pipe);
            #[cfg(debug_assertions)]
            #[cfg(feature = "debug_parse")]
            eprintln!("PARSE_DEBUG: pipeline saw pipe operator");
                }
                _ => {}
            }
        }
        
        if commands.len() == 1 && operators.is_empty() {
            // Single command, not a pipeline
        #[cfg(debug_assertions)]
    #[cfg(feature = "debug_parse")]
    eprintln!("PARSE_DEBUG: collapsing single-element pipeline to Command");
            Ok(commands.into_iter().next().unwrap())
        } else {
            // Actual pipeline
        #[cfg(debug_assertions)]
    #[cfg(feature = "debug_parse")]
    eprintln!("PARSE_DEBUG: building real pipeline: elements={}, operators={}", commands.len(), operators.len());
            Ok(ast::AstNode::Pipeline {
                elements: commands,
                operators,
            })
        }
    }

    /// Parse a simple command
    fn parse_simple_command(&self, pair: Pair<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        let mut opt_name: Option<Box<ast::AstNode<'static>>> = None;
        let mut args = Vec::new();
        let mut redirections = Vec::new();
    let mut call_generics: Vec<&str> = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::word => {
                    let word_value = self.leak_string(inner_pair.as_str());
                    let word_node = ast::AstNode::Word(word_value);
                    if opt_name.is_none() { opt_name = Some(Box::new(word_node)); } else { args.push(word_node); }
                }
                Rule::call_generic_args => {
                    for g in inner_pair.into_inner() {
                        if g.as_rule() == Rule::identifier { call_generics.push(self.leak_string(g.as_str())); }
                    }
                }
                Rule::argument => {
                    let arg = self.parse_argument(inner_pair, input)?;
                    args.push(arg);
                }
                Rule::redirection => {
                    let redirect = self.parse_redirection(inner_pair, input)?;
                    redirections.push(redirect);
                }
                _ => {}
            }
        }
        let name_box = opt_name.ok_or_else(|| anyhow::anyhow!("Command must have a name"))?;
        if !call_generics.is_empty() {
            return Ok(ast::AstNode::FunctionCall { name: name_box, args, is_async: false, generics: call_generics });
        }
        Ok(ast::AstNode::Command { name: name_box, args, redirections, background: false })
    }

    /// Parse an argument
    fn parse_argument(&self, pair: Pair<Rule>, _input: &str) -> Result<ast::AstNode<'static>> {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::assignment => {
                    // identifier '=' assignment_value
                    let mut name: Option<&str> = None; let mut value: Option<&str> = None;
                    for a in inner_pair.clone().into_inner() { match a.as_rule() { Rule::identifier => if name.is_none() { name = Some(self.leak_string(a.as_str())); } , _ => {} } }
                    // Fallback: raw text split
                    if name.is_none() {
                        let text = inner_pair.as_str();
                        if let Some(pos) = text.find('=') { name = Some(self.leak_string(&text[..pos])); value = Some(self.leak_string(&text[pos+1..])); }
                    } else {
                        let text = inner_pair.as_str(); if let Some(pos) = text.find('=') { value = Some(self.leak_string(&text[pos+1..])); }
                    }
                    let name = name.ok_or_else(|| anyhow::anyhow!("Invalid assignment"))?;
                    let val_node = ast::AstNode::Word(value.unwrap_or(""));
                    return Ok(ast::AstNode::VariableAssignment { name, operator: ast::AssignmentOperator::Assign, value: Box::new(val_node), is_local: false, is_export: false, is_readonly: false });
                }
                Rule::closure_expr => { return self.parse_closure_expr(inner_pair, _input); }
                Rule::word => {
                    return Ok(ast::AstNode::Word(self.leak_string(inner_pair.as_str())));
                }
                Rule::variable => {
                    let var_text = inner_pair.as_str();
                    // Remove $ prefix
                    let var_name = if var_text.starts_with("${") && var_text.ends_with("}") {
                        &var_text[2..var_text.len()-1]
                    } else if var_text.starts_with("$") {
                        &var_text[1..]
                    } else {
                        var_text
                    };
                    return Ok(ast::AstNode::VariableExpansion {
                        name: self.leak_string(var_name),
                        modifier: None,
                    });
                }
                Rule::command_substitution => {
                    let sub_text = inner_pair.as_str();
                    let is_legacy = sub_text.starts_with("`");
                    
                    // Extract the command part
                    let command_str = if is_legacy {
                        // Legacy backtick syntax: `command`
                        &sub_text[1..sub_text.len()-1]
                    } else {
                        // Modern syntax: $(command)
                        &sub_text[2..sub_text.len()-1]
                    };
                    
                    // Parse the inner command (recursively parse for proper semantics)
                    let inner_command = if command_str.trim().is_empty() {
                        ast::AstNode::Word(self.leak_string(""))
                    } else {
                        match self.parse(command_str) {
                            Ok(node) => node,
                            Err(_) => {
                                // Fallback to raw word if nested parse fails
                                ast::AstNode::Word(self.leak_string(command_str))
                            }
                        }
                    };
                    
                    return Ok(ast::AstNode::CommandSubstitution {
                        command: Box::new(inner_command),
                        is_legacy,
                    });
                }
                _ => {}
            }
        }
        
        Err(anyhow::anyhow!("Unable to parse argument"))
    }

    /// Parse a redirection
    fn parse_redirection(&self, pair: Pair<Rule>, _input: &str) -> Result<ast::Redirection<'static>> {
        let mut operator = None;
        let mut redir_type = None;
        let mut target = None;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::redirect_in => {
                    operator = Some(ast::RedirectionOperator::Input);
                    redir_type = Some(ast::RedirectionType::Input);
                }
                Rule::redirect_out => {
                    operator = Some(ast::RedirectionOperator::Output);
                    redir_type = Some(ast::RedirectionType::Output);
                }
                Rule::redirect_append => {
                    operator = Some(ast::RedirectionOperator::OutputAppend);
                    redir_type = Some(ast::RedirectionType::Append);
                }
                Rule::redirect_err => {
                    operator = Some(ast::RedirectionOperator::Output);
                    redir_type = Some(ast::RedirectionType::Error);
                }
                Rule::redirect_both => {
                    operator = Some(ast::RedirectionOperator::OutputBoth);
                    redir_type = Some(ast::RedirectionType::Both);
                }
                Rule::word => {
                    let word_node = ast::AstNode::Word(self.leak_string(inner_pair.as_str()));
                    target = Some(ast::RedirectionTarget::File(Box::new(word_node)));
                }
                _ => {}
            }
        }
        
        let operator = operator.ok_or_else(|| anyhow::anyhow!("Redirection must have an operator"))?;
        let redir_type = redir_type.ok_or_else(|| anyhow::anyhow!("Redirection must have a type"))?;
        let target = target.ok_or_else(|| anyhow::anyhow!("Redirection must have a target"))?;
        
        Ok(ast::Redirection {
            fd: None,
            operator,
            target,
            redir_type,
        })
    }

    /// Parse if statement with complete condition and branch handling
    fn parse_if_statement(&self, pair: Pair<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        let mut condition: Option<ast::AstNode<'static>> = None;
        let mut then_branch: Option<ast::AstNode<'static>> = None;
        let mut elif_branches = Vec::new();
        let mut else_branch: Option<ast::AstNode<'static>> = None;
        let mut current_state = IfParseState::Condition;
        // Keep pending elif condition until we see its body to avoid placeholder push/pop
        let mut pending_elif_condition: Option<ast::AstNode<'static>> = None;
        
        let inner_pairs: Vec<_> = pair.into_inner().collect();
        let mut i = 0;
        
        while i < inner_pairs.len() {
            let inner_pair = &inner_pairs[i];
            
            match inner_pair.as_rule() {
                Rule::if_kw => {
                    current_state = IfParseState::Condition;
                }
                Rule::test_command => {
                    match current_state {
                        IfParseState::Condition => {
                            condition = Some(self.parse_test_command(inner_pair.clone(), input)?);
                        }
                        IfParseState::ElifCondition => {
                            // Capture condition for elif branch; body will be consumed later
                            let elif_condition = self.parse_test_command(inner_pair.clone(), input)?;
                            pending_elif_condition = Some(elif_condition);
                        }
                        _ => {
                            return Err(anyhow::anyhow!("Unexpected test_command in if statement"));
                        }
                    }
                }
                Rule::command => {
                    match current_state {
                        IfParseState::Condition => {
                            condition = Some(self.parse_command(inner_pair.clone(), input)?);
                        }
                        IfParseState::ElifCondition => {
                            // Capture condition for elif branch; body will be consumed later
                            let elif_condition = self.parse_command(inner_pair.clone(), input)?;
                            pending_elif_condition = Some(elif_condition);
                        }
                        _ => {
                            return Err(anyhow::anyhow!("Unexpected command in if statement"));
                        }
                    }
                }
                Rule::then_kw => {
                    current_state = IfParseState::ThenBranch;
                }
                Rule::command_list => {
                    let body = self.parse_command_list(inner_pair.clone(), input)?;
                    match current_state {
                        IfParseState::ThenBranch => {
                            then_branch = Some(body);
                        }
                        IfParseState::ElifBranch => {
                            // Finalize elif branch with the pending condition
                            let cond = pending_elif_condition.take().ok_or_else(|| anyhow::anyhow!("elif branch missing condition"))?;
                            elif_branches.push((cond, body));
                        }
                        IfParseState::ElseBranch => {
                            else_branch = Some(body);
                        }
                        _ => {
                            return Err(anyhow::anyhow!("Unexpected command_list in if statement"));
                        }
                    }
                }
                Rule::program => {
                    let body = self.build_ast_from_pairs(inner_pair.clone().into_inner(), input)?;
                    match current_state {
                        IfParseState::ThenBranch => {
                            then_branch = Some(body);
                        }
                        IfParseState::ElifBranch => {
                            let cond = pending_elif_condition.take().ok_or_else(|| anyhow::anyhow!("elif branch missing condition"))?;
                            elif_branches.push((cond, body));
                        }
                        IfParseState::ElseBranch => {
                            else_branch = Some(body);
                        }
                        _ => {
                            return Err(anyhow::anyhow!("Unexpected program block in if statement"));
                        }
                    }
                }
                Rule::inner_program => {
                    // Handle inner_program rule as well for nested structures
                    let body = self.build_ast_from_pairs(inner_pair.clone().into_inner(), input)?;
                    match current_state {
                        IfParseState::ThenBranch => {
                            then_branch = Some(body);
                        }
                        IfParseState::ElifBranch => {
                            let cond = pending_elif_condition.take().ok_or_else(|| anyhow::anyhow!("elif branch missing condition"))?;
                            elif_branches.push((cond, body));
                        }
                        IfParseState::ElseBranch => {
                            else_branch = Some(body);
                        }
                        _ => {
                            return Err(anyhow::anyhow!("Unexpected inner_program block in if statement"));
                        }
                    }
                }
                Rule::elif_kw => {
                    current_state = IfParseState::ElifCondition;
                    pending_elif_condition = None;
                }
                Rule::else_kw => {
                    current_state = IfParseState::ElseBranch;
                }
                Rule::fi_kw => {
                    // End of if statement
                    break;
                }
                _ => {
                    // Ignore other tokens
                }
            }
            
            // Update state transitions
            if current_state == IfParseState::ElifCondition && inner_pair.as_rule() == Rule::then_kw {
                current_state = IfParseState::ElifBranch;
            }
            
            i += 1;
        }
        
        // Validate required components
        let condition = condition.ok_or_else(|| anyhow::anyhow!("If statement missing condition"))?;
        let then_branch = then_branch.ok_or_else(|| anyhow::anyhow!("If statement missing then branch"))?;
        
        Ok(ast::AstNode::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            elif_branches,
            else_branch: else_branch.map(Box::new),
        })
    }

    /// Parse for statement with variable, iterable, and body
    fn parse_for_statement(&self, pair: Pair<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        let mut variable: Option<&str> = None;
        let mut iterable_args = Vec::new();
        let mut body: Option<ast::AstNode<'static>> = None;
        let mut current_state = ForParseState::Variable;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::for_kw => {
                    current_state = ForParseState::Variable;
                }
                Rule::identifier => {
                    match current_state {
                        ForParseState::Variable => {
                            variable = Some(self.leak_string(inner_pair.as_str()));
                            current_state = ForParseState::In;
                        }
                        _ => {
                            return Err(anyhow::anyhow!("Unexpected identifier in for statement"));
                        }
                    }
                }
                Rule::in_kw => {
                    current_state = ForParseState::Arguments;
                }
                Rule::argument => {
                    if current_state == ForParseState::Arguments {
                        // Parse the argument as part of the iterable list
                        let arg_text = inner_pair.as_str();
                        iterable_args.push(ast::AstNode::Word(self.leak_string(arg_text)));
                    }
                }
                Rule::do_kw => {
                    current_state = ForParseState::Body;
                }
                Rule::command_list => {
                    if current_state == ForParseState::Body {
                        body = Some(self.parse_command_list(inner_pair, input)?);
                    }
                }
                Rule::program | Rule::inner_program => {
                    if current_state == ForParseState::Body {
                        body = Some(self.build_ast_from_pairs(inner_pair.into_inner(), input)?);
                    }
                }
                Rule::done_kw => {
                    // End of for statement
                    break;
                }
                _ => {
                    // Ignore other rules
                }
            }
        }
        
        // Validate required components
        let variable = variable.ok_or_else(|| anyhow::anyhow!("For statement missing variable"))?;
        let body = body.ok_or_else(|| anyhow::anyhow!("For statement missing body"))?;
        
        // Create iterable from arguments
        let iterable = if iterable_args.is_empty() {
            // Default to $@ (all positional parameters) if no explicit iterable
            ast::AstNode::Variable("@")
        } else if iterable_args.len() == 1 {
            iterable_args.into_iter().next().unwrap()
        } else {
            ast::AstNode::ArgumentList(iterable_args)
        };
        
        Ok(ast::AstNode::For {
            variable,
            iterable: Box::new(iterable),
            body: Box::new(body),
            is_async: false, // Standard for loops are synchronous
        })
    }

    /// Parse while statement with condition and body
    fn parse_while_statement(&self, pair: Pair<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        let mut condition: Option<ast::AstNode<'static>> = None;
        let mut body: Option<ast::AstNode<'static>> = None;
        let mut current_state = WhileParseState::Condition;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::while_kw => {
                    current_state = WhileParseState::Condition;
                }
                Rule::test_command => {
                    if current_state == WhileParseState::Condition {
                        condition = Some(self.parse_test_command(inner_pair, input)?);
                    } else {
                        return Err(anyhow::anyhow!("Unexpected test_command in while statement"));
                    }
                }
                Rule::command => {
                    if current_state == WhileParseState::Condition {
                        condition = Some(self.parse_command(inner_pair, input)?);
                    } else {
                        return Err(anyhow::anyhow!("Unexpected command in while statement"));
                    }
                }
                Rule::do_kw => {
                    current_state = WhileParseState::Body;
                }
                Rule::command_list => {
                    if current_state == WhileParseState::Body {
                        body = Some(self.parse_command_list(inner_pair, input)?);
                    }
                }
                Rule::program | Rule::inner_program => {
                    if current_state == WhileParseState::Body {
                        body = Some(self.build_ast_from_pairs(inner_pair.into_inner(), input)?);
                    }
                }
                Rule::done_kw => {
                    // End of while statement
                    break;
                }
                _ => {
                    // Ignore other tokens
                }
            }
        }
        
        // Validate required components
        let condition = condition.ok_or_else(|| anyhow::anyhow!("While statement missing condition"))?;
        let body = body.ok_or_else(|| anyhow::anyhow!("While statement missing body"))?;
        
        Ok(ast::AstNode::While {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }

    /// Parse case statement with expression, patterns, and bodies
    fn parse_case_statement(&self, pair: Pair<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        let mut expr: Option<ast::AstNode<'static>> = None;
        let mut arms = Vec::new();
        let mut current_state = CaseParseState::Expression;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::case_kw => {
                    current_state = CaseParseState::Expression;
                }
                Rule::word => {
                    if current_state == CaseParseState::Expression {
                        expr = Some(ast::AstNode::Word(self.leak_string(inner_pair.as_str())));
                        current_state = CaseParseState::Items;
                    }
                }
                Rule::case_item => {
                    if current_state == CaseParseState::Items {
                        let arm = self.parse_case_item(inner_pair, input)?;
                        arms.push(arm);
                    }
                }
                Rule::esac_kw => {
                    // End of case statement
                    break;
                }
                Rule::in_kw => {
                    // Transition to case items
                    current_state = CaseParseState::Items;
                }
                _ => {
                    // Ignore other tokens
                }
            }
        }
        
        // Validate required components
        let expr = expr.ok_or_else(|| anyhow::anyhow!("Case statement missing expression"))?;
        
        Ok(ast::AstNode::Case {
            expr: Box::new(expr),
            arms,
        })
    }
    
    /// Parse a single case item (pattern => body)
    fn parse_case_item(&self, pair: Pair<Rule>, input: &str) -> Result<ast::CaseArm<'static>> {
        let mut patterns = Vec::new();
        let mut body: Option<ast::AstNode<'static>> = None;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::pattern => {
                    let pattern = self.parse_pattern(inner_pair)?;
                    patterns.push(pattern);
                }
                Rule::program | Rule::inner_program => {
                    body = Some(self.build_ast_from_pairs(inner_pair.into_inner(), input)?);
                }
                _ => {
                    // Ignore other tokens like ")" and ";;"
                }
            }
        }
        
        let body = body.ok_or_else(|| anyhow::anyhow!("Case item missing body"))?;
        
        Ok(ast::CaseArm { patterns, body })
    }
    
    /// Parse a pattern for case statements
    fn parse_pattern(&self, pair: Pair<Rule>) -> Result<ast::Pattern<'static>> {
        let mut alternatives: Vec<ast::Pattern<'static>> = Vec::new();

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::word => {
                    let word = inner_pair.as_str();
                    // Map single underscore to placeholder pattern
                    if word == "_" {
                        alternatives.push(ast::Pattern::Placeholder);
                        continue;
                    }

                    // Treat common glob tokens as glob-based patterns
                    if word.contains('*') || word.contains('?') || word.contains('[') {
                        let element = if word.contains('*') {
                            ast::GlobElement::Wildcard
                        } else if word.contains('?') {
                            ast::GlobElement::SingleChar
                        } else {
                            ast::GlobElement::Literal(self.leak_string(word))
                        };
                        let glob_pattern = ast::GlobPattern { elements: vec![element] };
                        alternatives.push(ast::Pattern::Glob(glob_pattern));
                    } else {
                        alternatives.push(ast::Pattern::Literal(self.leak_string(word)));
                    }
                }
                _ => { /* ignore */ }
            }
        }

        if alternatives.len() == 1 {
            Ok(alternatives.into_iter().next().unwrap())
        } else {
            Ok(ast::Pattern::Alternative(alternatives))
        }
    }

    /// Parse function definition with name, parameters, and body
    fn parse_function_def(&self, pair: Pair<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        let mut name: Option<&str> = None;
        let mut params = Vec::new();
        let mut body: Option<ast::AstNode<'static>> = None;
        let mut current_state = FunctionParseState::Name;
    let mut generics: Vec<&str> = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::function_kw => {
                    current_state = FunctionParseState::Name;
                }
                Rule::identifier => {
                    match current_state {
                        FunctionParseState::Name => {
                            name = Some(self.leak_string(inner_pair.as_str()));
                            current_state = FunctionParseState::Body;  // 直接Body状態に移行
                        }
                        FunctionParseState::Parameters => {
                            // Parameter names inside parentheses
                            let param = ast::Parameter {
                                name: self.leak_string(inner_pair.as_str()),
                                default: None,
                                is_variadic: false,
                            };
                            params.push(param);
                        }
                        _ => {
                            return Err(anyhow::anyhow!("Unexpected identifier in function definition"));
                        }
                    }
                }
                Rule::generic_params => {
                    // Collect identifiers inside generic_params
                    for gp in inner_pair.clone().into_inner() {
                        if gp.as_rule() == Rule::identifier {
                            generics.push(self.leak_string(gp.as_str()));
                        }
                    }
                }
                Rule::program | Rule::inner_program | Rule::command_list => {
                    if current_state == FunctionParseState::Body {
                        // command_listの中身をstatementとして解析
                        let mut statements = Vec::new();
                        for cmd_pair in inner_pair.into_inner() {
                            if cmd_pair.as_rule() == Rule::statement {
                                let stmt = self.parse_statement(cmd_pair, input)?;
                                statements.push(stmt);
                            }
                        }
                        body = Some(ast::AstNode::Program(statements));
                    }
                }
                _ => {
                    // Handle transitions based on literal characters
                    match inner_pair.as_str() {
                        "(" => {
                            // Start of parameters
                            current_state = FunctionParseState::Parameters;
                        }
                        ")" => {
                            // End of parameters
                            current_state = FunctionParseState::Body;
                        }
                        "{" => {
                            // Start of body
                            current_state = FunctionParseState::Body;
                        }
                        "}" => {
                            // End of function
                            break;
                        }
                        _ => {
                            // Ignore other tokens
                        }
                    }
                }
            }
        }
        
        // Validate required components
        let name = name.ok_or_else(|| anyhow::anyhow!("Function definition missing name"))?;
        let body = body.ok_or_else(|| anyhow::anyhow!("Function definition missing body"))?;
        
        Ok(ast::AstNode::Function {
            name,
            params,
            body: Box::new(body),
            is_async: false, // Standard functions are synchronous
            generics,
        })
    }

    /// Parse closure expression: (param1,param2){ body }
    fn parse_closure_expr(&self, pair: Pair<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        let mut params: Vec<ast::Parameter<'static>> = Vec::new();
        let mut body_opt: Option<ast::AstNode<'static>> = None;
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::closure_param_list => {
                    for p in inner.into_inner() {
                        if p.as_rule() == Rule::identifier {
                            params.push(ast::Parameter { name: self.leak_string(p.as_str()), default: None, is_variadic: false });
                        }
                    }
                }
                Rule::brace_group => {
                    // brace_group -> statement_list | (nested statements)
                    let mut statements = Vec::new();
                    for bg in inner.into_inner() {
                        match bg.as_rule() {
                            Rule::statement_list => {
                                for st in bg.into_inner() {
                                    if st.as_rule() == Rule::statement { statements.push(self.parse_statement(st, input)?); }
                                }
                            }
                            Rule::statement => {
                                statements.push(self.parse_statement(bg, input)?);
                            }
                            Rule::program | Rule::inner_program => {
                                statements.push(self.build_ast_from_pairs(bg.into_inner(), input)?);
                            }
                            _ => {}
                        }
                    }
                    body_opt = Some(if statements.len() == 1 { statements.remove(0) } else { ast::AstNode::StatementList(statements) });
                }
                _ => {}
            }
        }
        let body = body_opt.unwrap_or_else(|| ast::AstNode::StatementList(Vec::new()));
        Ok(ast::AstNode::Closure { params, body: Box::new(body), captures: Vec::new(), is_async: false })
    }

    fn parse_macro_declaration(&self, pair: Pair<Rule>, _input: &str) -> Result<ast::AstNode<'static>> {
        let mut name: Option<&str> = None;
        let mut params: Vec<&str> = Vec::new();
        let mut body: Option<ast::AstNode<'static>> = None;
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::identifier => {
                    if name.is_none() { name = Some(self.leak_string(inner.as_str())); }
                    else { params.push(self.leak_string(inner.as_str())); }
                }
                Rule::brace_group => {
                    // Treat group as body program
                    let mut statements = Vec::new();
                    for s in inner.into_inner() {
                        if s.as_rule() == Rule::statement_list {
                            for st in s.into_inner() {
                                if st.as_rule() == Rule::statement { statements.push(self.parse_statement(st, _input)?); }
                            }
                        }
                    }
                    body = Some(ast::AstNode::Program(statements));
                }
                _ => {}
            }
        }
        let name = name.ok_or_else(|| anyhow::anyhow!("Macro missing name"))?;
        let body = body.unwrap_or(ast::AstNode::Empty);
        Ok(ast::AstNode::MacroDeclaration { name, params, body: Box::new(body) })
    }

    fn parse_macro_invocation(&self, pair: Pair<Rule>, _input: &str) -> Result<ast::AstNode<'static>> {
        let mut name: Option<&str> = None;
        let mut args: Vec<ast::AstNode<'static>> = Vec::new();
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::identifier => {
                    if name.is_none() { name = Some(self.leak_string(inner.as_str())); }
                    else { args.push(ast::AstNode::Word(self.leak_string(inner.as_str()))); }
                }
                Rule::word => {
                    args.push(ast::AstNode::Word(self.leak_string(inner.as_str())));
                }
                _ => {}
            }
        }
        let name = name.ok_or_else(|| anyhow::anyhow!("Macro invocation missing name"))?;
        Ok(ast::AstNode::MacroInvocation { name, args })
    }

    /// Parse modern match statement with expression and arms
    fn parse_match_statement(&self, pair: Pair<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        let mut expr: Option<ast::AstNode<'static>> = None;
        let mut arms = Vec::new();
        let mut current_state = MatchParseState::Expression;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::match_kw => {
                    current_state = MatchParseState::Expression;
                }
                Rule::argument => {
                    if current_state == MatchParseState::Expression {
                        // Parse the match expression
                        let arg_text = inner_pair.as_str();
                        expr = Some(ast::AstNode::Word(self.leak_string(arg_text)));
                        current_state = MatchParseState::Arms;
                    }
                }
                Rule::with_kw => {
                    current_state = MatchParseState::Arms;
                }
                Rule::match_arm => {
                    if current_state == MatchParseState::Arms {
                        let arm = self.parse_match_arm(inner_pair, input)?;
                        arms.push(arm);
                    }
                }
                _ => {
                    // Ignore other tokens
                }
            }
        }
        
        // Validate required components
        let expr = expr.ok_or_else(|| anyhow::anyhow!("Match statement missing expression"))?;

        // Minimal exhaustiveness check: treat as exhaustive if any arm is a catch-all
        fn pattern_is_catch_all(p: &ast::Pattern<'_>) -> bool {
            use ast::Pattern::*;
            match p {
                Placeholder | Wildcard => true,
                Literal(s) if *s == "_" => true,
                Or(list) | Alternative(list) => list.iter().any(pattern_is_catch_all),
                Guard { pattern, .. } => pattern_is_catch_all(pattern),
                Binding { pattern, .. } => pattern_is_catch_all(pattern),
                Reference(inner) => pattern_is_catch_all(inner),
                ArraySlice { before, rest, after } => {
                    before.iter().any(pattern_is_catch_all)
                        || rest.as_deref().map(pattern_is_catch_all).unwrap_or(false)
                        || after.iter().any(pattern_is_catch_all)
                }
                Object { rest, .. } => *rest,
                _ => false,
            }
        }

        let is_exhaustive = arms.iter().any(|arm| pattern_is_catch_all(&arm.pattern));

        Ok(ast::AstNode::Match {
            expr: Box::new(expr),
            arms,
            is_exhaustive,
        })
    }
    
    /// Parse a single match arm (pattern => body)
    fn parse_match_arm(&self, pair: Pair<Rule>, input: &str) -> Result<ast::MatchArm<'static>> {
        let mut pattern: Option<ast::Pattern<'static>> = None;
        let guard: Option<ast::AstNode<'static>> = None;
        let mut body: Option<ast::AstNode<'static>> = None;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::pattern => {
                    pattern = Some(self.parse_pattern(inner_pair)?);
                }
                Rule::program | Rule::inner_program => {
                    body = Some(self.build_ast_from_pairs(inner_pair.into_inner(), input)?);
                }
                _ => {
                    // Handle "=>" separator and potential guard clauses
                }
            }
        }
        
        let pattern = pattern.ok_or_else(|| anyhow::anyhow!("Match arm missing pattern"))?;
        let body = body.ok_or_else(|| anyhow::anyhow!("Match arm missing body"))?;
        
        Ok(ast::MatchArm {
            pattern,
            guard,
            body,
        })
    }

    /// Mark an AST node as background
    fn mark_background(&self, mut node: ast::AstNode<'static>) -> ast::AstNode<'static> {
        match &mut node {
            ast::AstNode::Command { background, .. } => {
                *background = true;
            }
            ast::AstNode::Pipeline { elements, .. } => {
                if elements.len() == 1 {
                    // Unwrap single-element pipeline into a command with background
                    let mut only = elements.remove(0);
                    if let ast::AstNode::Command { background, .. } = &mut only {
                        *background = true;
                    }
                    return only;
                } else if let Some(last) = elements.last_mut() {
                    if let ast::AstNode::Command { background, .. } = last {
                        *background = true;
                    }
                }
            }
            _ => {
                // For other types, wrap in a command-like structure
                // This is a simplification - real implementation would be more sophisticated
            }
        }
        node
    }

    /// Parse a test command (command with optional semicolon)
    fn parse_test_command(&self, pair: Pair<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::command => {
                    return self.parse_command(inner_pair, input);
                }
                Rule::semicolon => {
                    // Ignore semicolon
                }
                _ => {}
            }
        }
        Err(anyhow::anyhow!("Unable to parse test command"))
    }

    /// Parse a command list
    fn parse_command_list(&self, pair: Pair<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        let mut statements = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::statement => {
                    let stmt = self.parse_statement(inner_pair, input)?;
                    statements.push(stmt);
                }
                Rule::line_terminator => {
                    // Ignore line terminators
                }
                _ => {}
            }
        }
        
        if statements.len() == 1 {
            Ok(statements.into_iter().next().unwrap())
        } else {
            Ok(ast::AstNode::Program(statements))
        }
    }

    /// Helper to leak strings for 'static lifetime
    fn leak_string(&self, s: &str) -> &'static str {
        Box::leak(s.to_string().into_boxed_str())
    }
}

/// Highlight parsing error with line and column.
pub fn highlight_error(input: &str, err: PestError<Rule>) -> String {
    let (line_no, col_no) = match err.line_col {
        LineColLocation::Pos((line, col)) => (line, col),
        LineColLocation::Span((line, col), _) => (line, col),
    };
    let line_str = input.lines().nth(line_no - 1).unwrap_or("");
    format!(
        "Parse error: {} at line {}, column {}\n{}\n{}^",
        err.variant.message(),
        line_no,
        col_no,
        line_str,
        " ".repeat(col_no.saturating_sub(1))
    )
}

/// Parse raw input into AST using PEG grammar.
pub fn parse(input: &str) -> Result<ast::AstNode> {
    match ShellParser::parse(Rule::program, input) {
        Ok(pairs) => {
            let parser = ShellCommandParser::new();
            parser.build_ast_from_pairs(pairs, input)
        },
        Err(e) => Err(anyhow::anyhow!(highlight_error(input, e))),
    }
}

pub use lexer::TokenKind; 