//! Advanced input handling for NexusShell CUI
//! Provides sophisticated key binding, input processing, and interactive features

use crossterm::event::{KeyCode, KeyEvent as CrosstermKeyEvent, KeyModifiers};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Key event wrapper for consistent handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl From<CrosstermKeyEvent> for KeyEvent {
    fn from(event: CrosstermKeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

/// Action that can be triggered by key bindings
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    // Basic editing
    InsertChar(char),
    Backspace,
    Delete,

    // Movement
    MoveLeft,
    MoveRight,
    MoveWordLeft,
    MoveWordRight,
    MoveToStart,
    MoveToEnd,

    // History
    HistoryPrevious,
    HistoryNext,
    HistorySearch,

    // Completion
    Complete,
    CompleteNext,
    CompletePrevious,

    // Line editing
    DeleteWord,
    DeleteToEnd,
    DeleteToStart,
    DeleteLine,

    // Clipboard
    Copy,
    Cut,
    Paste,

    // Control
    Submit,
    Cancel,
    Interrupt,
    Suspend,

    // Special
    ClearScreen,
    Refresh,
    ShowHelp,

    // Custom
    Custom(String),
}

/// Input mode for different editing behaviors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Insert,
    Normal,  // Vi normal mode
    Visual,  // Vi visual mode
    Command, // Command line mode
    Search,  // Search mode
}

/// Configuration for input handling
#[derive(Debug, Clone)]
pub struct InputConfig {
    pub vi_mode: bool,
    pub emacs_bindings: bool,
    pub custom_bindings: HashMap<KeyEvent, InputAction>,
    pub timeout_ms: u64,
    pub enable_mouse: bool,
    pub enable_bracketed_paste: bool,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            vi_mode: false,
            emacs_bindings: true,
            custom_bindings: HashMap::new(),
            timeout_ms: 500,
            enable_mouse: false,
            enable_bracketed_paste: true,
        }
    }
}

/// Advanced input handler with key binding support
pub struct InputHandler {
    config: InputConfig,
    key_bindings: HashMap<KeyEvent, InputAction>,
    mode: InputMode,
    last_key_time: Option<Instant>,
    key_sequence: Vec<KeyEvent>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self::with_config(InputConfig::default())
    }

    pub fn with_config(config: InputConfig) -> Self {
        let mut handler = Self {
            key_bindings: HashMap::new(),
            mode: InputMode::Insert,
            last_key_time: None,
            key_sequence: Vec::new(),
            config,
        };

        handler.setup_default_bindings();
        handler
    }

    /// Process a key event and return the corresponding action
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<InputAction> {
        let now = Instant::now();

        // Check for timeout in key sequences
        if let Some(last_time) = self.last_key_time {
            if now.duration_since(last_time) > Duration::from_millis(self.config.timeout_ms) {
                self.key_sequence.clear();
            }
        }

        self.last_key_time = Some(now);
        self.key_sequence.push(key);

        // Try to match key sequence
        if let Some(action) = self.match_key_sequence() {
            self.key_sequence.clear();
            return Some(action);
        }

        // If no sequence match, try single key
        self.key_sequence.clear();
        self.key_sequence.push(key);

        if let Some(action) = self.match_key_sequence() {
            self.key_sequence.clear();
            return Some(action);
        }

        // Default handling
        match key.code {
            KeyCode::Char(c) if key.modifiers.is_empty() => Some(InputAction::InsertChar(c)),
            _ => None,
        }
    }

    /// Set input mode
    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
        self.key_sequence.clear();
    }

    /// Get current input mode
    pub fn mode(&self) -> InputMode {
        self.mode
    }

    /// Add custom key binding
    pub fn bind_key(&mut self, key: KeyEvent, action: InputAction) {
        self.key_bindings.insert(key, action);
    }

    /// Remove key binding
    pub fn unbind_key(&mut self, key: &KeyEvent) {
        self.key_bindings.remove(key);
    }

    /// Get all current bindings
    pub fn bindings(&self) -> &HashMap<KeyEvent, InputAction> {
        &self.key_bindings
    }

    fn setup_default_bindings(&mut self) {
        if self.config.emacs_bindings {
            self.setup_emacs_bindings();
        }

        if self.config.vi_mode {
            self.setup_vi_bindings();
        }

        // Add custom bindings from config
        for (key, action) in &self.config.custom_bindings {
            self.key_bindings.insert(*key, action.clone());
        }
    }

    fn setup_emacs_bindings(&mut self) {
        use KeyCode::*;

        // Basic editing
        self.bind_key(
            KeyEvent {
                code: Backspace,
                modifiers: KeyModifiers::NONE,
            },
            InputAction::Backspace,
        );
        self.bind_key(
            KeyEvent {
                code: Delete,
                modifiers: KeyModifiers::NONE,
            },
            InputAction::Delete,
        );
        self.bind_key(
            KeyEvent {
                code: Enter,
                modifiers: KeyModifiers::NONE,
            },
            InputAction::Submit,
        );
        self.bind_key(
            KeyEvent {
                code: Tab,
                modifiers: KeyModifiers::NONE,
            },
            InputAction::Complete,
        );
        self.bind_key(
            KeyEvent {
                code: BackTab,
                modifiers: KeyModifiers::SHIFT,
            },
            InputAction::CompletePrevious,
        );

        // Movement
        self.bind_key(
            KeyEvent {
                code: Left,
                modifiers: KeyModifiers::NONE,
            },
            InputAction::MoveLeft,
        );
        self.bind_key(
            KeyEvent {
                code: Right,
                modifiers: KeyModifiers::NONE,
            },
            InputAction::MoveRight,
        );
        self.bind_key(
            KeyEvent {
                code: Left,
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::MoveWordLeft,
        );
        self.bind_key(
            KeyEvent {
                code: Right,
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::MoveWordRight,
        );
        self.bind_key(
            KeyEvent {
                code: Home,
                modifiers: KeyModifiers::NONE,
            },
            InputAction::MoveToStart,
        );
        self.bind_key(
            KeyEvent {
                code: End,
                modifiers: KeyModifiers::NONE,
            },
            InputAction::MoveToEnd,
        );

        // History
        self.bind_key(
            KeyEvent {
                code: Up,
                modifiers: KeyModifiers::NONE,
            },
            InputAction::HistoryPrevious,
        );
        self.bind_key(
            KeyEvent {
                code: Down,
                modifiers: KeyModifiers::NONE,
            },
            InputAction::HistoryNext,
        );

        // Emacs-style control bindings
        self.bind_key(
            KeyEvent {
                code: Char('a'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::MoveToStart,
        );
        self.bind_key(
            KeyEvent {
                code: Char('e'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::MoveToEnd,
        );
        self.bind_key(
            KeyEvent {
                code: Char('f'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::MoveRight,
        );
        self.bind_key(
            KeyEvent {
                code: Char('b'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::MoveLeft,
        );
        self.bind_key(
            KeyEvent {
                code: Char('p'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::HistoryPrevious,
        );
        self.bind_key(
            KeyEvent {
                code: Char('n'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::HistoryNext,
        );
        self.bind_key(
            KeyEvent {
                code: Char('d'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::Delete,
        );
        self.bind_key(
            KeyEvent {
                code: Char('h'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::Backspace,
        );
        self.bind_key(
            KeyEvent {
                code: Char('k'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::DeleteToEnd,
        );
        self.bind_key(
            KeyEvent {
                code: Char('u'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::DeleteToStart,
        );
        self.bind_key(
            KeyEvent {
                code: Char('w'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::DeleteWord,
        );
        self.bind_key(
            KeyEvent {
                code: Char('l'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::ClearScreen,
        );
        self.bind_key(
            KeyEvent {
                code: Char('r'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::HistorySearch,
        );
        self.bind_key(
            KeyEvent {
                code: Char('c'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::Interrupt,
        );
        self.bind_key(
            KeyEvent {
                code: Char('z'),
                modifiers: KeyModifiers::CONTROL,
            },
            InputAction::Suspend,
        );

        // Meta/Alt bindings
        self.bind_key(
            KeyEvent {
                code: Char('f'),
                modifiers: KeyModifiers::ALT,
            },
            InputAction::MoveWordRight,
        );
        self.bind_key(
            KeyEvent {
                code: Char('b'),
                modifiers: KeyModifiers::ALT,
            },
            InputAction::MoveWordLeft,
        );
        self.bind_key(
            KeyEvent {
                code: Char('d'),
                modifiers: KeyModifiers::ALT,
            },
            InputAction::DeleteWord,
        );
        self.bind_key(
            KeyEvent {
                code: Backspace,
                modifiers: KeyModifiers::ALT,
            },
            InputAction::DeleteWord,
        );
    }

    fn setup_vi_bindings(&mut self) {
        // Vi bindings would be mode-dependent
        // This is a simplified version
        use KeyCode::*;

        // Insert mode bindings (similar to emacs for basic editing)
        self.bind_key(
            KeyEvent {
                code: Backspace,
                modifiers: KeyModifiers::NONE,
            },
            InputAction::Backspace,
        );
        self.bind_key(
            KeyEvent {
                code: Enter,
                modifiers: KeyModifiers::NONE,
            },
            InputAction::Submit,
        );
        self.bind_key(
            KeyEvent {
                code: Tab,
                modifiers: KeyModifiers::NONE,
            },
            InputAction::Complete,
        );

        // Escape to normal mode would be handled specially
        self.bind_key(
            KeyEvent {
                code: Esc,
                modifiers: KeyModifiers::NONE,
            },
            InputAction::Custom("vi_normal_mode".to_string()),
        );
    }

    fn match_key_sequence(&self) -> Option<InputAction> {
        // Simple single-key matching for now
        if self.key_sequence.len() == 1 {
            let key = self.key_sequence[0];
            return self.key_bindings.get(&key).cloned();
        }

        // Could implement multi-key sequences here
        None
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for creating key events
pub mod keys {
    use super::*;

    pub fn ctrl(c: char) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::CONTROL,
        }
    }

    pub fn alt(c: char) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::ALT,
        }
    }

    pub fn shift(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::SHIFT,
        }
    }

    pub fn char(c: char) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_key_handling() {
        let mut handler = InputHandler::new();

        // Test character insertion
        let action = handler.handle_key(keys::char('a'));
        assert_eq!(action, Some(InputAction::InsertChar('a')));

        // Test control-a (move to start)
        let action = handler.handle_key(keys::ctrl('a'));
        assert_eq!(action, Some(InputAction::MoveToStart));
    }

    #[test]
    fn test_custom_bindings() {
        let mut handler = InputHandler::new();

        // Add custom binding
        handler.bind_key(keys::ctrl('x'), InputAction::Custom("test".to_string()));

        let action = handler.handle_key(keys::ctrl('x'));
        assert_eq!(action, Some(InputAction::Custom("test".to_string())));
    }
}
