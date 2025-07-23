#![doc = "Command-line parser turning raw input into an AST."]

pub mod lexer;
pub mod ast;

use anyhow::Result;

use pest::Parser;
use pest::error::{Error as PestError, LineColLocation};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct ShellParser;

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
        Ok(_) => Ok(ast::AstNode::Program(vec![ast::AstNode::Word(
            input.trim().to_string(),
        )])),
        Err(e) => Err(anyhow::anyhow!(highlight_error(input, e))),
    }
}

pub use lexer::TokenKind; 