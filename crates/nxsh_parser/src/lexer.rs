//! Simple but full-featured tokenizer for NexusShell.
//! Currently generates up to 15 different token kinds and recognizes heredoc start via `<<`.

use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Word(String),       // command or argument
    String(String),     // quoted string
    Number(i64),        // numeric literal
    Operator(String),   // generic operator like `=`, `+=`
    Pipe,               // `|`
    RedirectOut,        // `>`
    RedirectIn,         // `<`
    Semicolon,          // `;`
    And,                // `&&`
    Or,                 // `||`
    OpenParen,          // `(`
    CloseParen,         // `)`
    OpenBrace,          // `{`
    CloseBrace,         // `}`
    Newline,            // `\n`
    HeredocStart(String), // `<<DELIM` (captures delimiter)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: std::ops::Range<usize>,
}

impl Token {
    fn new(kind: TokenKind, start: usize, end: usize) -> Self {
        Self {
            kind,
            span: start..end,
        }
    }
}

/// Tokenize input and return vector of tokens.
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut chars = input.chars().peekable();
    let mut idx = 0;
    let mut tokens = Vec::new();

    while let Some(c) = chars.peek().copied() {
        let start_idx = idx;
        match c {
            ' ' | '\t' => {
                chars.next();
                idx += 1;
            }
            '\n' => {
                chars.next();
                idx += 1;
                tokens.push(Token::new(TokenKind::Newline, start_idx, idx));
            }
            '|' => {
                chars.next();
                idx += 1;
                tokens.push(Token::new(TokenKind::Pipe, start_idx, idx));
            }
            ';' => {
                chars.next();
                idx += 1;
                tokens.push(Token::new(TokenKind::Semicolon, start_idx, idx));
            }
            '(' => {
                chars.next();
                idx += 1;
                tokens.push(Token::new(TokenKind::OpenParen, start_idx, idx));
            }
            ')' => {
                chars.next();
                idx += 1;
                tokens.push(Token::new(TokenKind::CloseParen, start_idx, idx));
            }
            '{' => {
                chars.next();
                idx += 1;
                tokens.push(Token::new(TokenKind::OpenBrace, start_idx, idx));
            }
            '}' => {
                chars.next();
                idx += 1;
                tokens.push(Token::new(TokenKind::CloseBrace, start_idx, idx));
            }
            '>' => {
                chars.next();
                idx += 1;
                tokens.push(Token::new(TokenKind::RedirectOut, start_idx, idx));
            }
            '<' => {
                chars.next();
                idx += 1;
                if chars.peek() == Some(&'<') {
                    // heredoc start
                    chars.next();
                    idx += 1;
                    // read delimiter until whitespace/newline
                    let mut delim = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_whitespace() {
                            break;
                        }
                        delim.push(ch);
                        chars.next();
                        idx += 1;
                    }
                    tokens.push(Token::new(TokenKind::HeredocStart(delim), start_idx, idx));
                } else {
                    tokens.push(Token::new(TokenKind::RedirectIn, start_idx, idx));
                }
            }
            '&' => {
                chars.next();
                idx += 1;
                if chars.peek() == Some(&'&') {
                    chars.next();
                    idx += 1;
                    tokens.push(Token::new(TokenKind::And, start_idx, idx));
                } else {
                    tokens.push(Token::new(TokenKind::Operator("&".into()), start_idx, idx));
                }
            }
            '|' => unreachable!(), // handled earlier
            '"' | '\'' => {
                // quoted string
                let quote = chars.next().unwrap();
                idx += 1;
                let mut content = String::new();
                while let Some(&ch) = chars.peek() {
                    idx += 1;
                    chars.next();
                    if ch == quote {
                        break;
                    }
                    content.push(ch);
                }
                tokens.push(Token::new(TokenKind::String(content), start_idx, idx));
            }
            ch if ch.is_ascii_digit() => {
                let mut num = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        num.push(d);
                        chars.next();
                        idx += 1;
                    } else {
                        break;
                    }
                }
                let val = num.parse::<i64>().unwrap_or_default();
                tokens.push(Token::new(TokenKind::Number(val), start_idx, idx));
            }
            _ => {
                // Word or operator sequence
                let mut buf = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() || "|;&(){}<>\"'".contains(ch) {
                        break;
                    }
                    buf.push(ch);
                    chars.next();
                    idx += 1;
                }
                tokens.push(Token::new(TokenKind::Word(buf), start_idx, idx));
            }
        }
    }
    tokens
} 