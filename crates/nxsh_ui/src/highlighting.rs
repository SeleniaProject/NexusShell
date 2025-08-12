use anyhow::Result;

#[derive(Default)]
pub struct SyntaxHighlighter;

impl SyntaxHighlighter {
    pub fn new() -> Result<Self> { Ok(Self::default()) }
    pub fn set_theme<T>(&mut self, _theme: &T) -> Result<()> { Ok(()) }
    pub fn highlight_line(&self, line: &str) -> String { line.to_string() }
}
