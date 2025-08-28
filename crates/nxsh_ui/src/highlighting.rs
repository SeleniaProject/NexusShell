use anyhow::Result;

#[derive(Default)]
pub struct SyntaxHighlighter;

impl SyntaxHighlighter {
    pub fn new() -> Result<Self> { Ok(Self::default()) }
    pub fn set_theme<T>(&mut self, _theme: &T) -> Result<()> { Ok(()) }
    
    /// Apply syntax highlighting to command line input
    pub fn highlight_line(&self, line: &str) -> String {
        if line.trim().is_empty() {
            return line.to_string();
        }
        
        let mut result = String::new();
        let tokens = self.tokenize(line);
        
        for token in tokens {
            let highlighted = match token.token_type {
                TokenType::Command => format!("\x1b[96m{}\x1b[0m", token.text), // Cyan
                TokenType::Flag => format!("\x1b[93m{}\x1b[0m", token.text),    // Yellow
                TokenType::String => format!("\x1b[92m{}\x1b[0m", token.text),  // Green
                TokenType::Number => format!("\x1b[91m{}\x1b[0m", token.text),  // Red
                TokenType::Path => format!("\x1b[94m{}\x1b[0m", token.text),    // Blue
                TokenType::Operator => format!("\x1b[95m{}\x1b[0m", token.text), // Magenta
                TokenType::Variable => format!("\x1b[97m{}\x1b[0m", token.text), // White
                TokenType::Comment => format!("\x1b[90m{}\x1b[0m", token.text),  // Gray
                TokenType::Normal => token.text,
            };
            result.push_str(&highlighted);
        }
        
        result
    }
    
    /// Tokenize command line for syntax highlighting
    fn tokenize(&self, line: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = line.char_indices().peekable();
        let mut current_token = String::new();
        let mut current_start = 0;
        let mut in_string = false;
        let mut string_char = '"';
        
        while let Some((i, ch)) = chars.next() {
            match ch {
                ' ' | '\t' if !in_string => {
                    if !current_token.is_empty() {
                        let token_type = self.classify_token(&current_token, tokens.is_empty());
                        tokens.push(Token {
                            text: current_token.clone(),
                            start: current_start,
                            end: i,
                            token_type,
                        });
                        current_token.clear();
                    }
                    // Add whitespace as normal token
                    tokens.push(Token {
                        text: ch.to_string(),
                        start: i,
                        end: i + 1,
                        token_type: TokenType::Normal,
                    });
                    current_start = i + 1;
                }
                '"' | '\'' if !in_string => {
                    if !current_token.is_empty() {
                        let token_type = self.classify_token(&current_token, tokens.is_empty());
                        tokens.push(Token {
                            text: current_token.clone(),
                            start: current_start,
                            end: i,
                            token_type,
                        });
                        current_token.clear();
                    }
                    in_string = true;
                    string_char = ch;
                    current_token.push(ch);
                    current_start = i;
                }
                c if c == string_char && in_string => {
                    current_token.push(ch);
                    tokens.push(Token {
                        text: current_token.clone(),
                        start: current_start,
                        end: i + 1,
                        token_type: TokenType::String,
                    });
                    current_token.clear();
                    in_string = false;
                    current_start = i + 1;
                }
                '#' if !in_string => {
                    if !current_token.is_empty() {
                        let token_type = self.classify_token(&current_token, tokens.is_empty());
                        tokens.push(Token {
                            text: current_token.clone(),
                            start: current_start,
                            end: i,
                            token_type,
                        });
                        current_token.clear();
                    }
                    // Rest of line is comment
                    let comment = line[i..].to_string();
                    tokens.push(Token {
                        text: comment,
                        start: i,
                        end: line.len(),
                        token_type: TokenType::Comment,
                    });
                    break;
                }
                '|' | '>' | '<' | '&' | ';' if !in_string => {
                    if !current_token.is_empty() {
                        let token_type = self.classify_token(&current_token, tokens.is_empty());
                        tokens.push(Token {
                            text: current_token.clone(),
                            start: current_start,
                            end: i,
                            token_type,
                        });
                        current_token.clear();
                    }
                    tokens.push(Token {
                        text: ch.to_string(),
                        start: i,
                        end: i + 1,
                        token_type: TokenType::Operator,
                    });
                    current_start = i + 1;
                }
                _ => {
                    if current_token.is_empty() {
                        current_start = i;
                    }
                    current_token.push(ch);
                }
            }
        }
        
        // Handle remaining token
        if !current_token.is_empty() {
            let token_type = if in_string {
                TokenType::String
            } else {
                self.classify_token(&current_token, tokens.is_empty())
            };
            tokens.push(Token {
                text: current_token,
                start: current_start,
                end: line.len(),
                token_type,
            });
        }
        
        tokens
    }
    
    /// Classify token type based on content and position
    fn classify_token(&self, token: &str, is_first: bool) -> TokenType {
        if is_first {
            return TokenType::Command;
        }
        
        if token.starts_with('-') {
            return TokenType::Flag;
        }
        
        if token.starts_with('$') {
            return TokenType::Variable;
        }
        
        if token.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return TokenType::Number;
        }
        
        if token.contains('/') || token.contains('\\') || token.contains('.') {
            return TokenType::Path;
        }
        
        TokenType::Normal
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub token_type: TokenType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Command,
    Flag,
    String,
    Number,
    Path,
    Operator,
    Variable,
    Comment,
    Normal,
}
