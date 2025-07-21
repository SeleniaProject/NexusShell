use ratatui::{prelude::*, widgets::{Paragraph, Block, Borders}};
use std::time::{Duration, Instant};

/// Toast notification entry.
#[derive(Clone)]
pub struct Toast {
    pub message: String,
    pub created: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(msg: impl Into<String>, duration_ms: u64) -> Self {
        Self { message: msg.into(), created: Instant::now(), duration: Duration::from_millis(duration_ms) }
    }

    pub fn expired(&self) -> bool {
        self.created.elapsed() >= self.duration
    }
}

/// Render active toast overlay in bottom-right corner.
pub fn render<B: Backend>(f: &mut Frame<B>, toast: &Toast) {
    let area = f.size();
    let width = toast.message.len() as u16 + 4;
    let height = 3;
    let x = area.width.saturating_sub(width) - 2;
    let y = area.height.saturating_sub(height) - 1;
    let rect = Rect::new(x, y, width, height);
    let block = Block::default().borders(Borders::ALL).title("Toast");
    let paragraph = Paragraph::new(toast.message.as_str()).block(block);
    f.render_widget(paragraph, rect);
} 