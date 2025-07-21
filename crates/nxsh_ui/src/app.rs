use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph}};

#[derive(Default)]
pub struct AppState {
    pub input: String,
}

impl AppState {
    pub fn render<B: Backend>(&self, f: &mut Frame<B>) {
        let block = Block::default().title("NexusShell").borders(Borders::ALL);
        let paragraph = Paragraph::new(self.input.as_str()).block(block);
        f.render_widget(paragraph, f.size());
    }
} 