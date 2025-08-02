//! High-performance zero-copy tokenizer for NexusShell using logos
//!
//! This module provides a comprehensive tokenizer that recognizes all shell
//! constructs with maximum performance and zero-copy string handling.

use logos::{Logos, Lexer, Span};
use std::fmt;

/// Token types for shell parsing with logos integration
#[derive(Logos, Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals and identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Word(String),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_owned() // Remove quotes
    })]
    #[regex(r#"'([^'\\]|\\.)*'"#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_owned() // Remove quotes
    })]
    String(String),

    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().unwrap_or(0))]
    Number(i64),

    #[regex(r"-?[0-9]+\.[0-9]+", |lex| lex.slice().to_owned())]
    Float(String),

    // Variables
    #[regex(r"\$[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice()[1..].to_owned())]
    Variable(String),

    #[regex(r"\$\{[^}]+\}", |lex| {
        let s = lex.slice();
        s[2..s.len()-1].to_owned() // Remove ${ and }
    })]
    VariableBrace(String),

    #[regex(r"\$\([^)]+\)", |lex| {
        let s = lex.slice();
        s[2..s.len()-1].to_owned() // Remove $( and )
    })]
    CommandSubstitution(String),

    // Operators
    #[token("|")]
    Pipe,

    #[token("||")]
    Or,

    #[token("|>")]
    ObjectPipe,

    #[token("||>")]
    ObjectPipeParallel,

    #[token("&")]
    Background,

    #[token("&&")]
    And,

    #[token(";")]
    Semicolon,

    #[token(";;")]
    DoubleSemicolon,

    // Redirections (higher priority than comparison operators)
    #[token(">", priority = 3)]
    RedirectOut,

    #[token(">>")]
    RedirectAppend,

    #[token("<", priority = 3)]
    RedirectIn,

    #[token("<<")]
    HeredocStart,

    #[token("<<<")]
    Herestring,

    #[token("2>")]
    RedirectStderr,

    #[token("2>>")]
    RedirectStderrAppend,

    #[token("&>")]
    RedirectBoth,

    #[token("&>>")]
    RedirectBothAppend,

    #[regex(r"[0-9]+>", |lex| {
        let s = lex.slice();
        s[..s.len()-1].parse::<u32>().unwrap_or(1)
    })]
    RedirectFd(u32),

    #[regex(r"[0-9]+>>", |lex| {
        let s = lex.slice();
        s[..s.len()-2].parse::<u32>().unwrap_or(1)
    })]
    RedirectFdAppend(u32),

    // Brackets and parentheses
    #[token("(")]
    OpenParen,

    #[token(")")]
    CloseParen,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    #[token("[")]
    OpenBracket,

    #[token("]")]
    CloseBracket,

    // Glob patterns (higher priority than arithmetic operators)
    #[token("*", priority = 3)]
    Glob,

    #[token("?")]
    GlobSingle,

    #[regex(r"\[[^\]]*\]", |lex| lex.slice().to_owned())]
    GlobClass(String),

    #[regex(r"\{[^}]*\}", |lex| lex.slice().to_owned())]
    BraceExpansion(String),

    // Assignment operators
    #[token("=")]
    Assign,

    #[token("+=")]
    AssignAdd,

    #[token("-=")]
    AssignSub,

    #[token("*=")]
    AssignMul,

    #[token("/=")]
    AssignDiv,

    #[token("%=")]
    AssignMod,

    // Comparison operators
    #[token("==")]
    Equal,

    #[token("!=")]
    NotEqual,

    #[token("<", priority = 2)]
    Less,

    #[token("<=")]
    LessEqual,

    #[token(">", priority = 2)]
    Greater,

    #[token(">=")]
    GreaterEqual,

    #[token("=~")]
    Match,

    #[token("!~")]
    NotMatch,

    // Arithmetic operators
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*", priority = 2)]
    Multiply,

    #[token("/", priority = 2)]
    Divide,

    #[token("%")]
    Modulo,

    #[token("**")]
    Power,

    #[token("++")]
    Increment,

    #[token("--")]
    Decrement,

    // Logical operators
    #[token("!")]
    Not,

    // Keywords
    #[token("if")]
    If,

    #[token("then")]
    Then,

    #[token("else")]
    Else,

    #[token("elif")]
    Elif,

    #[token("fi")]
    Fi,

    #[token("for")]
    For,

    #[token("while")]
    While,

    #[token("until")]
    Until,

    #[token("do")]
    Do,

    #[token("done")]
    Done,

    #[token("case")]
    Case,

    #[token("esac")]
    Esac,

    #[token("in")]
    In,

    #[token("select")]
    Select,

    #[token("function")]
    Function,

    #[token("return")]
    Return,

    #[token("break")]
    Break,

    #[token("continue")]
    Continue,

    #[token("local")]
    Local,

    #[token("export")]
    Export,

    #[token("readonly")]
    Readonly,

    #[token("declare")]
    Declare,

    #[token("typeset")]
    Typeset,

    #[token("let")]
    Let,

    #[token("eval")]
    Eval,

    #[token("exec")]
    Exec,

    #[token("source")]
    Source,

    #[token(".")]
    Dot,

    #[token("alias")]
    Alias,

    #[token("unalias")]
    Unalias,

    #[token("set")]
    Set,

    #[token("unset")]
    Unset,

    #[token("shift")]
    Shift,

    #[token("getopts")]
    Getopts,

    #[token("read")]
    Read,

    #[token("echo")]
    Echo,

    #[token("printf")]
    Printf,

    #[token("test")]
    Test,

    #[token("trap")]
    Trap,

    #[token("kill")]
    Kill,

    #[token("jobs")]
    Jobs,

    #[token("bg")]
    Bg,

    #[token("fg")]
    Fg,

    #[token("wait")]
    Wait,

    #[token("suspend")]
    Suspend,

    #[token("times")]
    Times,

    #[token("type")]
    Type,

    #[token("which")]
    Which,

    #[token("command")]
    Command,

    #[token("builtin")]
    Builtin,

    #[token("enable")]
    Enable,

    #[token("help")]
    Help,

    #[token("history")]
    History,

    #[token("fc")]
    Fc,

    #[token("dirs")]
    Dirs,

    #[token("pushd")]
    Pushd,

    #[token("popd")]
    Popd,

    #[token("cd")]
    Cd,

    #[token("pwd")]
    Pwd,

    #[token("exit")]
    Exit,

    #[token("logout")]
    Logout,

    // Modern shell keywords
    #[token("match")]
    MatchKeyword,

    #[token("with")]
    With,

    #[token("try")]
    Try,

    #[token("catch")]
    Catch,

    #[token("finally")]
    Finally,

    #[token("throw")]
    Throw,

    #[token("async")]
    Async,

    #[token("await")]
    Await,

    #[token("yield")]
    Yield,

    #[token("import")]
    Import,

    #[token("from")]
    From,

    #[token("as")]
    As,

    #[token("use")]
    Use,

    #[token("mod")]
    Mod,

    #[token("pub")]
    Pub,

    #[token("struct")]
    Struct,

    #[token("enum")]
    Enum,

    #[token("trait")]
    Trait,

    #[token("impl")]
    Impl,

    #[token("where")]
    Where,

    #[token("const")]
    Const,

    #[token("static")]
    Static,

    #[token("mut")]
    Mut,

    #[token("ref")]
    Ref,

    #[token("move")]
    Move,

    #[token("self")]
    SelfKeyword,

    #[token("Self")]
    SelfType,

    #[token("super")]
    Super,

    #[token("crate")]
    Crate,

    // Comments
    #[regex(r"#[^\n]*", |lex| lex.slice().to_owned())]
    Comment(String),

    // Whitespace and newlines
    #[regex(r"[ \t]+", logos::skip)]
    Whitespace,

    #[token("\n")]
    Newline,

    #[token("\r\n")]
    WindowsNewline,

    // Line continuation
    #[token("\\\n")]
    LineContinuation,

    // Path and file patterns
    #[regex(r"~[a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    TildeExpansion(String),

    #[regex(r"\./[^\s]*", |lex| lex.slice().to_owned())]
    RelativePath(String),

    #[regex(r"/[^\s/][^\s]*", priority = 3, callback = |lex| lex.slice().to_owned())]
    AbsolutePath(String),

    // Process substitution
    #[regex(r"<\([^)]+\)", |lex| {
        let s = lex.slice();
        s[2..s.len()-1].to_owned()
    })]
    ProcessSubstitutionIn(String),

    #[regex(r">\([^)]+\)", |lex| {
        let s = lex.slice();
        s[2..s.len()-1].to_owned()
    })]
    ProcessSubstitutionOut(String),

    // Special parameters
    #[token("$0")]
    ParamZero,

    #[regex(r"\$[1-9][0-9]*", |lex| {
        lex.slice()[1..].parse::<u32>().unwrap_or(0)
    })]
    ParamPositional(u32),

    #[token("$$")]
    ParamPid,

    #[token("$?")]
    ParamExitStatus,

    #[token("$!")]
    ParamLastBgPid,

    #[token("$#")]
    ParamArgCount,

    #[token("$@")]
    ParamAllArgs,

    #[token("$*")]
    ParamAllArgsString,

    #[token("$-")]
    ParamFlags,

    #[token("$_")]
    ParamLastArg,

    // Heredoc delimiter (dynamic)
    HeredocDelimiter(String),

    // Heredoc content (dynamic)
    HeredocContent(String),

    // End of file
    Eof,

    // Error token for invalid input
    Error,
}

/// Token with position information
#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub span: Span,
    pub slice: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, span: Span, slice: &'a str) -> Self {
        Self { kind, span, slice }
    }

    /// Get the line and column of this token
    pub fn line_col(&self, input: &str) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        
        for (i, ch) in input.char_indices() {
            if i >= self.span.start {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        
        (line, col)
    }

    /// Check if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(self.kind,
            TokenKind::If | TokenKind::Then | TokenKind::Else | TokenKind::Elif | TokenKind::Fi |
            TokenKind::For | TokenKind::While | TokenKind::Until | TokenKind::Do | TokenKind::Done |
            TokenKind::Case | TokenKind::Esac | TokenKind::In | TokenKind::Select |
            TokenKind::Function | TokenKind::Return | TokenKind::Break | TokenKind::Continue |
            TokenKind::Local | TokenKind::Export | TokenKind::Readonly | TokenKind::Declare |
            TokenKind::Typeset | TokenKind::Let | TokenKind::Eval | TokenKind::Exec |
            TokenKind::Source | TokenKind::Dot | TokenKind::Alias | TokenKind::Unalias |
            TokenKind::Set | TokenKind::Unset | TokenKind::Shift | TokenKind::Getopts |
            TokenKind::Read | TokenKind::Echo | TokenKind::Printf | TokenKind::Test |
            TokenKind::Trap | TokenKind::Kill | TokenKind::Jobs | TokenKind::Bg | TokenKind::Fg |
            TokenKind::Wait | TokenKind::Suspend | TokenKind::Times | TokenKind::Type |
            TokenKind::Which | TokenKind::Command | TokenKind::Builtin | TokenKind::Enable |
            TokenKind::Help | TokenKind::History | TokenKind::Fc | TokenKind::Dirs |
            TokenKind::Pushd | TokenKind::Popd | TokenKind::Cd | TokenKind::Pwd |
            TokenKind::Exit | TokenKind::Logout
        )
    }

    /// Check if this token is an operator
    pub fn is_operator(&self) -> bool {
        matches!(self.kind,
            TokenKind::Pipe | TokenKind::Or | TokenKind::ObjectPipe | TokenKind::ObjectPipeParallel |
            TokenKind::Background | TokenKind::And | TokenKind::Semicolon | TokenKind::DoubleSemicolon |
            TokenKind::RedirectOut | TokenKind::RedirectAppend | TokenKind::RedirectIn |
            TokenKind::HeredocStart | TokenKind::Herestring | TokenKind::RedirectStderr |
            TokenKind::RedirectStderrAppend | TokenKind::RedirectBoth | TokenKind::RedirectBothAppend |
            TokenKind::RedirectFd(_) | TokenKind::RedirectFdAppend(_) |
            TokenKind::Assign | TokenKind::AssignAdd | TokenKind::AssignSub | TokenKind::AssignMul |
            TokenKind::AssignDiv | TokenKind::AssignMod | TokenKind::Equal | TokenKind::NotEqual |
            TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual |
            TokenKind::Match | TokenKind::NotMatch | TokenKind::Plus | TokenKind::Minus |
            TokenKind::Multiply | TokenKind::Divide | TokenKind::Modulo | TokenKind::Power |
            TokenKind::Increment | TokenKind::Decrement | TokenKind::Not
        )
    }

    /// Check if this token is a literal
    pub fn is_literal(&self) -> bool {
        matches!(self.kind,
            TokenKind::Word(_) | TokenKind::String(_) | TokenKind::Number(_) | TokenKind::Float(_)
        )
    }

    /// Check if this token can start a command
    pub fn can_start_command(&self) -> bool {
        matches!(self.kind,
            TokenKind::Word(_) | TokenKind::String(_) | TokenKind::Variable(_) |
            TokenKind::VariableBrace(_) | TokenKind::CommandSubstitution(_) |
            TokenKind::TildeExpansion(_) | TokenKind::RelativePath(_) | TokenKind::AbsolutePath(_)
        ) || self.is_keyword()
    }

    /// Get the precedence of this operator token
    pub fn precedence(&self) -> Option<u8> {
        match self.kind {
            TokenKind::Or | TokenKind::And => Some(1),
            TokenKind::Pipe | TokenKind::ObjectPipe | TokenKind::ObjectPipeParallel => Some(2),
            TokenKind::Semicolon | TokenKind::DoubleSemicolon | TokenKind::Background => Some(3),
            TokenKind::Equal | TokenKind::NotEqual | TokenKind::Less | TokenKind::LessEqual |
            TokenKind::Greater | TokenKind::GreaterEqual | TokenKind::Match | TokenKind::NotMatch => Some(4),
            TokenKind::Plus | TokenKind::Minus => Some(5),
            TokenKind::Multiply | TokenKind::Divide | TokenKind::Modulo => Some(6),
            TokenKind::Power => Some(7),
            TokenKind::Not => Some(8),
            _ => None,
        }
    }
}

/// High-performance tokenizer with zero-copy string handling
pub struct Tokenizer<'a> {
    lexer: Lexer<'a, TokenKind>,
    input: &'a str,
    current_token: Option<Token<'a>>,
    peeked_token: Option<Token<'a>>,
    heredoc_stack: Vec<String>,
    in_heredoc: bool,
}

impl<'a> Tokenizer<'a> {
    /// Create a new tokenizer for the given input
    pub fn new(input: &'a str) -> Self {
        let mut lexer = TokenKind::lexer(input);
        let current_token = Self::next_token(&mut lexer, input, &mut Vec::new(), &mut false);
        
        Self {
            lexer,
            input,
            current_token,
            peeked_token: None,
            heredoc_stack: Vec::new(),
            in_heredoc: false,
        }
    }

    /// Get the next token from the lexer
    fn next_token(
        lexer: &mut Lexer<'a, TokenKind>,
        input: &'a str,
        heredoc_stack: &mut Vec<String>,
        in_heredoc: &mut bool,
    ) -> Option<Token<'a>> {
        // Handle heredoc processing
        if *in_heredoc && !heredoc_stack.is_empty() {
            return Self::process_heredoc(lexer, input, heredoc_stack, in_heredoc);
        }

        match lexer.next() {
            Some(Ok(kind)) => {
                let span = lexer.span();
                let slice = lexer.slice();

                // Handle heredoc start
                if matches!(kind, TokenKind::HeredocStart) {
                    // Look for delimiter after <<
                    if let Some(Ok(TokenKind::Word(delimiter))) = lexer.next() {
                        heredoc_stack.push(delimiter.clone());
                        *in_heredoc = true;
                        return Some(Token::new(TokenKind::HeredocDelimiter(delimiter), span, slice));
                    }
                }

                Some(Token::new(kind, span, slice))
            }
            Some(Err(_)) => {
                let span = lexer.span();
                let slice = lexer.slice();
                Some(Token::new(TokenKind::Error, span, slice))
            }
            None => Some(Token::new(TokenKind::Eof, lexer.span(), "")),
        }
    }

    /// Process heredoc content
    fn process_heredoc(
        lexer: &mut Lexer<'a, TokenKind>,
        input: &'a str,
        heredoc_stack: &mut Vec<String>,
        in_heredoc: &mut bool,
    ) -> Option<Token<'a>> {
        if let Some(delimiter) = heredoc_stack.last() {
            let start = lexer.span().end;
            let mut content = String::new();
            let mut current_pos = start;

            // Read lines until we find the delimiter
            for line in input[start..].lines() {
                if line.trim() == delimiter {
                    heredoc_stack.pop();
                    if heredoc_stack.is_empty() {
                        *in_heredoc = false;
                    }
                    let span = start..current_pos;
                    let span_slice = &input[span.clone()];
                    return Some(Token::new(TokenKind::HeredocContent(content), span, span_slice));
                }
                content.push_str(line);
                content.push('\n');
                current_pos += line.len() + 1;
            }

            // If we reach here, heredoc is unterminated
            *in_heredoc = false;
            heredoc_stack.clear();
            let span = start..input.len();
            let span_slice = &input[span.clone()];
            Some(Token::new(TokenKind::HeredocContent(content), span, span_slice))
        } else {
            *in_heredoc = false;
            None
        }
    }

    /// Get the current token without consuming it
    pub fn current(&self) -> Option<&Token<'a>> {
        self.current_token.as_ref()
    }

    /// Peek at the next token without consuming it
    pub fn peek(&mut self) -> Option<&Token<'a>> {
        if self.peeked_token.is_none() {
            self.peeked_token = Self::next_token(
                &mut self.lexer,
                self.input,
                &mut self.heredoc_stack,
                &mut self.in_heredoc,
            );
        }
        self.peeked_token.as_ref()
    }

    /// Advance to the next token
    pub fn advance(&mut self) -> Option<Token<'a>> {
        let current = self.current_token.take();
        
        if let Some(peeked) = self.peeked_token.take() {
            self.current_token = Some(peeked);
        } else {
            self.current_token = Self::next_token(
                &mut self.lexer,
                self.input,
                &mut self.heredoc_stack,
                &mut self.in_heredoc,
            );
        }
        
        current
    }

    /// Check if the current token matches the given kind
    pub fn matches(&self, kind: &TokenKind) -> bool {
        self.current_token.as_ref().map_or(false, |t| std::mem::discriminant(&t.kind) == std::mem::discriminant(kind))
    }

    /// Consume the current token if it matches the given kind
    pub fn consume(&mut self, kind: &TokenKind) -> Option<Token<'a>> {
        if self.matches(kind) {
            self.advance()
        } else {
            None
        }
    }

    /// Check if we're at the end of input
    pub fn is_eof(&self) -> bool {
        matches!(self.current_token, Some(Token { kind: TokenKind::Eof, .. }))
    }

    /// Get all remaining tokens as a vector
    pub fn collect_all(mut self) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        
        while let Some(token) = self.advance() {
            if matches!(token.kind, TokenKind::Eof) {
                break;
            }
            tokens.push(token);
        }
        
        tokens
    }

    /// Skip whitespace and comments
    pub fn skip_trivia(&mut self) {
        while let Some(token) = &self.current_token {
            match token.kind {
                TokenKind::Comment(_) | TokenKind::Whitespace => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    /// Get the current position in the input
    pub fn position(&self) -> usize {
        self.current_token.as_ref().map_or(self.input.len(), |t| t.span.start)
    }

    /// Get the remaining input from current position
    pub fn remaining_input(&self) -> &'a str {
        let pos = self.position();
        &self.input[pos..]
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Word(s) => write!(f, "Word({})", s),
            TokenKind::String(s) => write!(f, "String(\"{}\")", s),
            TokenKind::Number(n) => write!(f, "Number({})", n),
            TokenKind::Float(s) => write!(f, "Float({})", s),
            TokenKind::Variable(s) => write!(f, "Variable({})", s),
            TokenKind::VariableBrace(s) => write!(f, "VariableBrace({})", s),
            TokenKind::CommandSubstitution(s) => write!(f, "CommandSubstitution({})", s),
            TokenKind::Comment(s) => write!(f, "Comment({})", s),
            TokenKind::HeredocDelimiter(s) => write!(f, "HeredocDelimiter({})", s),
            TokenKind::HeredocContent(s) => write!(f, "HeredocContent({})", s),
            TokenKind::GlobClass(s) => write!(f, "GlobClass({})", s),
            TokenKind::BraceExpansion(s) => write!(f, "BraceExpansion({})", s),
            TokenKind::TildeExpansion(s) => write!(f, "TildeExpansion({})", s),
            TokenKind::RelativePath(s) => write!(f, "RelativePath({})", s),
            TokenKind::AbsolutePath(s) => write!(f, "AbsolutePath({})", s),
            TokenKind::ProcessSubstitutionIn(s) => write!(f, "ProcessSubstitutionIn({})", s),
            TokenKind::ProcessSubstitutionOut(s) => write!(f, "ProcessSubstitutionOut({})", s),
            TokenKind::RedirectFd(n) => write!(f, "RedirectFd({})", n),
            TokenKind::RedirectFdAppend(n) => write!(f, "RedirectFdAppend({})", n),
            TokenKind::ParamPositional(n) => write!(f, "ParamPositional({})", n),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Convenience function to tokenize input into a vector
pub fn tokenize(input: &str) -> Vec<Token> {
    Tokenizer::new(input).collect_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let input = "echo hello world";
        let tokens = tokenize(input);
        
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0].kind, TokenKind::Echo));
        assert!(matches!(tokens[1].kind, TokenKind::Word(ref s) if s == "hello"));
        assert!(matches!(tokens[2].kind, TokenKind::Word(ref s) if s == "world"));
    }

    #[test]
    fn test_string_tokenization() {
        let input = r#"echo "hello world" 'single quotes'"#;
        let tokens = tokenize(input);
        
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0].kind, TokenKind::Echo));
        assert!(matches!(tokens[1].kind, TokenKind::String(ref s) if s == "hello world"));
        assert!(matches!(tokens[2].kind, TokenKind::String(ref s) if s == "single quotes"));
    }

    #[test]
    fn test_variable_tokenization() {
        let input = "$var ${complex_var} $(command)";
        let tokens = tokenize(input);
        
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0].kind, TokenKind::Variable(ref s) if s == "var"));
        assert!(matches!(tokens[1].kind, TokenKind::VariableBrace(ref s) if s == "complex_var"));
        assert!(matches!(tokens[2].kind, TokenKind::CommandSubstitution(ref s) if s == "command"));
    }

    #[test]
    fn test_pipeline_tokenization() {
        let input = "ls | grep test || echo failed";
        let tokens = tokenize(input);
        
        assert!(matches!(tokens[0].kind, TokenKind::Word(ref s) if s == "ls"));
        assert!(matches!(tokens[1].kind, TokenKind::Pipe));
        assert!(matches!(tokens[2].kind, TokenKind::Word(ref s) if s == "grep"));
        assert!(matches!(tokens[3].kind, TokenKind::Test));
        assert!(matches!(tokens[4].kind, TokenKind::Or));
        assert!(matches!(tokens[5].kind, TokenKind::Echo));
        assert!(matches!(tokens[6].kind, TokenKind::Word(ref s) if s == "failed"));
    }

    #[test]
    fn test_redirection_tokenization() {
        let input = "command > output.txt 2>> error.log";
        let tokens = tokenize(input);
        
        assert!(matches!(tokens[0].kind, TokenKind::Command));
        assert!(matches!(tokens[1].kind, TokenKind::RedirectOut));
        assert!(matches!(tokens[2].kind, TokenKind::Word(ref s) if s == "output"));
        assert!(matches!(tokens[3].kind, TokenKind::Dot));
        assert!(matches!(tokens[4].kind, TokenKind::Word(ref s) if s == "txt"));
        assert!(matches!(tokens[5].kind, TokenKind::RedirectStderrAppend));
        assert!(matches!(tokens[6].kind, TokenKind::Word(ref s) if s == "error"));
        assert!(matches!(tokens[7].kind, TokenKind::Dot));
        assert!(matches!(tokens[8].kind, TokenKind::Word(ref s) if s == "log"));
    }

    #[test]
    fn test_object_pipe_tokenization() {
        let input = "data |> map |> filter ||> parallel_process";
        let tokens = tokenize(input);
        
        assert!(matches!(tokens[0].kind, TokenKind::Word(ref s) if s == "data"));
        assert!(matches!(tokens[1].kind, TokenKind::ObjectPipe));
        assert!(matches!(tokens[2].kind, TokenKind::Word(ref s) if s == "map"));
        assert!(matches!(tokens[3].kind, TokenKind::ObjectPipe));
        assert!(matches!(tokens[4].kind, TokenKind::Word(ref s) if s == "filter"));
        assert!(matches!(tokens[5].kind, TokenKind::ObjectPipeParallel));
        assert!(matches!(tokens[6].kind, TokenKind::Word(ref s) if s == "parallel_process"));
    }
} 