#![doc = "Command-line parser turning raw input into an AST."]

pub mod lexer;
pub mod ast;

// Re-export the Parser for external use
pub use ShellCommandParser as Parser;

use anyhow::Result;

use pest::Parser as PestParser;
use pest::error::{Error as PestError, LineColLocation};
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
        // For now, return a simple command node as placeholder
        // TODO: Implement full parsing logic using the PEST grammar
        Ok(ast::AstNode::Command {
            name: Box::new(ast::AstNode::Word("echo")),
            args: vec![ast::AstNode::Word(Box::leak(input.to_string().into_boxed_str()))],
            redirections: Vec::new(),
            background: false,
        })
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