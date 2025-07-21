#![doc = "Command-line parser turning raw input into an AST."]

pub mod lexer;
pub mod ast;

use anyhow::Result;

/// Parse raw shell input into an AST node.
pub fn parse(input: &str) -> Result<ast::AstNode> {
    // Extremely naive implementation for initial compile-time sanity.
    Ok(ast::AstNode::Command(input.trim().to_string()))
} 