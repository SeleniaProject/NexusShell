pub mod app;
pub mod widgets;
pub mod scroll_buffer;
pub mod tui;
pub mod highlighting;
pub mod line_editor;
pub mod themes;
pub mod completion;
pub mod config;

pub use tui::run as run_tui; 