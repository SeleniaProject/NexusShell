#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Word(String),
}

/// Tokenize the input by whitespace. This is a placeholder that will be replaced by a full tokenizer.
pub fn tokenize(input: &str) -> Vec<Token> {
    input
        .split_whitespace()
        .map(|w| Token::Word(w.to_string()))
        .collect()
} 