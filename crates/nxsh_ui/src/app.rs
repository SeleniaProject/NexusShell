use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph, List, ListItem}};
use crate::widgets::toast::{Toast,render as render_toast};

#[derive(Default)]
pub struct AppState {
    pub input: String,
    pub side_panel_visible: bool,
    pub suggestions: Vec<String>,
    pub toasts: Vec<Toast>,
}

impl AppState {
    pub fn toggle_side_panel(&mut self) {
        self.side_panel_visible = !self.side_panel_visible;
    }

    pub fn render(&self, f: &mut Frame) {
        if self.side_panel_visible {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(70),
                    Constraint::Percentage(30),
                ])
                .split(f.size());

            // Main area
            let block = Block::default().title("NexusShell").borders(Borders::ALL);
            let paragraph = Paragraph::new(self.input.as_str()).block(block);
            f.render_widget(paragraph, chunks[0]);

            // Side panel suggestions
            let items: Vec<ListItem> = self
                .suggestions
                .iter()
                .map(|s| ListItem::new(s.as_str()))
                .collect();
            let list = List::new(items)
                .block(Block::default().title("Suggestions").borders(Borders::ALL));
            f.render_widget(list, chunks[1]);
        } else {
            let block = Block::default().title("NexusShell").borders(Borders::ALL);
            let paragraph = Paragraph::new(self.input.as_str()).block(block);
            f.render_widget(paragraph, f.size());
        }
        // Toast overlay (show newest)
        if let Some(t) = self.toasts.last() {
            render_toast(f, t);
        }
    }
}

/// Maximum render FPS to reduce CPU usage.
pub const MAX_FPS: u64 = 60; 