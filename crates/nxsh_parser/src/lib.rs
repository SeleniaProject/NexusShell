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

/// Public parser interface for shell commands
pub struct ShellCommandParser {
    _private: (),
}

impl ShellCommandParser {
    /// Create a new parser instance
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Parse shell command text into an AST
    pub fn parse(&self, input: &str) -> Result<ast::AstNode<'static>> {
        let pairs = ShellParser::parse(Rule::program, input)
            .with_context(|| format!("Failed to parse input: {}", input))?;
        
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
                        }
                    }
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
            Ok(statements.into_iter().next().unwrap())
        } else {
            Ok(ast::AstNode::Program(statements))
        }
    }

    /// Parse a line (statement with optional operators)
    fn parse_line(&self, pair: Pair<Rule>, input: &str) -> Result<Option<ast::AstNode<'static>>> {
        let mut statements = Vec::new();
        let mut background = false;
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::statement => {
                    let stmt = self.parse_statement(inner_pair, input)?;
                    statements.push(stmt);
                }
                Rule::and_op => {
                    // TODO: Handle && operator
                }
                Rule::or_op => {
                    // TODO: Handle || operator
                }
                Rule::semicolon => {
                    // TODO: Handle ; operator
                }
                Rule::background => {
                    background = true;
                }
                _ => {}
            }
        }
        
        if statements.is_empty() {
            return Ok(None);
        }
        
        if statements.len() == 1 && !background {
            Ok(Some(statements.into_iter().next().unwrap()))
        } else {
            // For now, just return the first statement
            // TODO: Implement proper compound statement handling
            let mut stmt = statements.into_iter().next().unwrap();
            if background {
                stmt = self.mark_background(stmt);
            }
            Ok(Some(stmt))
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
                Rule::match_statement => {
                    return self.parse_match_statement(inner_pair, input);
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
                    commands.push(cmd);
                }
                Rule::pipe => {
                    operators.push(ast::PipeOperator::Pipe);
                }
                _ => {}
            }
        }
        
        if commands.len() == 1 && operators.is_empty() {
            // Single command, not a pipeline
            Ok(commands.into_iter().next().unwrap())
        } else {
            // Actual pipeline
            Ok(ast::AstNode::Pipeline {
                elements: commands,
                operators,
            })
        }
    }

    /// Parse a simple command
    fn parse_simple_command(&self, pair: Pair<Rule>, input: &str) -> Result<ast::AstNode<'static>> {
        let mut name: Option<Box<ast::AstNode<'static>>> = None;
        let mut args = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::word => {
                    let word_value = self.leak_string(inner_pair.as_str());
                    let word_node = ast::AstNode::Word(word_value);
                    
                    if name.is_none() {
                        name = Some(Box::new(word_node));
                    } else {
                        args.push(word_node);
                    }
                }
                Rule::argument => {
                    let arg = self.parse_argument(inner_pair, input)?;
                    args.push(arg);
                }
                _ => {}
            }
        }
        
        let name = name.ok_or_else(|| anyhow::anyhow!("Command must have a name"))?;
        
        Ok(ast::AstNode::Command {
            name,
            args,
            redirections: Vec::new(), // TODO: Parse redirections
            background: false,
        })
    }

    /// Parse an argument
    fn parse_argument(&self, pair: Pair<Rule>, _input: &str) -> Result<ast::AstNode<'static>> {
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
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
                    // For now, create a dummy command
                    // TODO: Parse the inner command properly
                    let dummy_command = ast::AstNode::Word(self.leak_string("placeholder"));
                    return Ok(ast::AstNode::CommandSubstitution {
                        command: Box::new(dummy_command),
                        is_legacy: sub_text.starts_with("`"),
                    });
                }
                _ => {}
            }
        }
        
        Err(anyhow::anyhow!("Unable to parse argument"))
    }

    /// Helper functions for control structures (stubs for now)
    fn parse_if_statement(&self, _pair: Pair<Rule>, _input: &str) -> Result<ast::AstNode<'static>> {
        // TODO: Implement if statement parsing
        Ok(ast::AstNode::Word(self.leak_string("if_placeholder")))
    }

    fn parse_for_statement(&self, _pair: Pair<Rule>, _input: &str) -> Result<ast::AstNode<'static>> {
        // TODO: Implement for statement parsing
        Ok(ast::AstNode::Word(self.leak_string("for_placeholder")))
    }

    fn parse_while_statement(&self, _pair: Pair<Rule>, _input: &str) -> Result<ast::AstNode<'static>> {
        // TODO: Implement while statement parsing
        Ok(ast::AstNode::Word(self.leak_string("while_placeholder")))
    }

    fn parse_case_statement(&self, _pair: Pair<Rule>, _input: &str) -> Result<ast::AstNode<'static>> {
        // TODO: Implement case statement parsing
        Ok(ast::AstNode::Word(self.leak_string("case_placeholder")))
    }

    fn parse_function_def(&self, _pair: Pair<Rule>, _input: &str) -> Result<ast::AstNode<'static>> {
        // TODO: Implement function definition parsing
        Ok(ast::AstNode::Word(self.leak_string("function_placeholder")))
    }

    fn parse_match_statement(&self, _pair: Pair<Rule>, _input: &str) -> Result<ast::AstNode<'static>> {
        // TODO: Implement match statement parsing
        Ok(ast::AstNode::Word(self.leak_string("match_placeholder")))
    }

    /// Mark an AST node as background
    fn mark_background(&self, mut node: ast::AstNode<'static>) -> ast::AstNode<'static> {
        match &mut node {
            ast::AstNode::Command { background, .. } => {
                *background = true;
            }
            _ => {
                // For other types, wrap in a command-like structure
                // This is a simplification - real implementation would be more sophisticated
            }
        }
        node
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
        Ok(_pairs) => {
            // For now, create a simple AST node
            // TODO: Implement proper AST construction from pest pairs
            Ok(ast::AstNode::Program(vec![ast::AstNode::Word(
                input.trim(),
            )]))
        },
        Err(e) => Err(anyhow::anyhow!(highlight_error(input, e))),
    }
}

pub use lexer::TokenKind; 