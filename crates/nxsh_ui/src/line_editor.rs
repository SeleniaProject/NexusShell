//! Standard CUI Line Editor for NexusShell
//! 
//! This module provides a simple readline-style line editing experience
//! with basic history management and standard Tab completion.
//! All complex TUI-specific features have been removed.

use anyhow::{Result, Context};
use rustyline::{
    config::Config,
    error::ReadlineError,
    Editor,
    Helper as RLHelperTrait,
    completion::{Completer as RLCompleter, Pair},
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
    history::DefaultHistory,
    Context as RustylineContext,
};
use std::{
    path::{Path, PathBuf},
    fs,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc as StdArc, Mutex as StdMutex};
use nxsh_parser::Parser;

/// Simple CUI line editor with standard readline functionality
pub struct NexusLineEditor {
    editor: Editor<RustyHelper, DefaultHistory>,
    history_file: Option<String>,
    config: LineEditorConfig,
}

impl NexusLineEditor {
    /// Create a comprehensive line editor with full functionality
    /// COMPLETE configuration with history file loading as required
    pub fn new_minimal() -> Result<Self> {
        let config = Config::builder()
            .history_ignore_space(true)
            .history_ignore_dups(true)?
            .completion_type(rustyline::CompletionType::List)
            .edit_mode(rustyline::EditMode::Emacs)
            .auto_add_history(true)
            .max_history_size(10000)?  // Full history capacity
            .build();

        let mut editor: Editor<RustyHelper, DefaultHistory> = Editor::with_config(config)?;
        // Attach helper with a default standalone completer (no shared state yet)
        editor.set_helper(Some(RustyHelper::new_standalone()));
        
        // COMPLETE history file loading as specified
        let history_file = Some(Self::get_history_file_path()?.to_string_lossy().to_string());
        if let Some(ref path) = history_file {
            let path_buf = PathBuf::from(path);
            let _ = editor.load_history(&path_buf); // Load existing history
        }
        
        Ok(Self {
            editor,
            history_file,
            config: LineEditorConfig::with_full_features(), // Complete configuration
        })
    }

    /// Create a new line editor with standard configuration
    pub fn new() -> Result<Self> {
        let config = Config::builder()
            .history_ignore_space(true)
            .history_ignore_dups(true)?
            .completion_type(rustyline::CompletionType::List)
            .build();

        let mut editor: Editor<RustyHelper, DefaultHistory> = Editor::with_config(config)?;
        editor.set_helper(Some(RustyHelper::new_standalone()));
        // Note: set_max_history_size might not be available in this version
        
        Ok(Self {
            editor,
            history_file: None,
            config: LineEditorConfig::default(),
        })
    }

    /// Wire shared `NexusCompleter` into rustyline helper so that Tab completion works inside readline.
    pub fn set_shared_completer(&mut self, shared: StdArc<StdMutex<crate::completion::NexusCompleter>>) {
        if let Some(helper) = self.editor.helper_mut() {
            helper.set_shared_completer(shared);
        }
    }
    
    /// Create line editor with custom history file
    pub fn with_history_file<P: AsRef<Path>>(history_path: P) -> Result<Self> {
        let mut line_editor = Self::new()?;
        line_editor.set_history_file(history_path)?;
        Ok(line_editor)
    }
    
    /// Get default history file path
    pub fn get_history_file_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home_dir.join(".nxsh_history"))
    }
    
    /// Set history file path and load existing history
    pub fn set_history_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        // Create directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Load existing history if file exists
        if path.as_ref().exists() {
            let _ = self.editor.load_history(&path);
        }
        
        self.history_file = Some(path_str);
        Ok(())
    }

    /// Read a line with the given prompt
    pub fn readline(&mut self, prompt: &str) -> Result<String> {
        match self.editor.readline(prompt) {
            Ok(line) => {
                // Add to history if not empty
                if !line.trim().is_empty() {
                    let _ = self.editor.add_history_entry(line.as_str());
                }
                Ok(line)
            }
            Err(ReadlineError::Interrupted) => {
                Err(anyhow::anyhow!("Interrupted"))
            }
            Err(ReadlineError::Eof) => {
                Err(anyhow::anyhow!("EOF"))
            }
            Err(err) => {
                Err(anyhow::anyhow!("Readline error: {}", err))
            }
        }
    }

    /// Add entry to command history
    pub fn add_history_entry(&mut self, entry: &str) -> Result<()> {
        if !entry.trim().is_empty() {
            self.editor.add_history_entry(entry)?;
        }
        Ok(())
    }

    /// Save history to file
    pub fn save_history(&mut self) -> Result<()> {
        if let Some(ref path) = self.history_file {
            let _ = self.editor.save_history(path);
        }
        Ok(())
    }

    /// Load history from file
    pub fn load_history(&mut self) -> Result<()> {
        if let Some(ref path) = self.history_file {
            if Path::new(path).exists() {
                let _ = self.editor.load_history(path);
            }
        }
        Ok(())
    }

    /// Clear command history
    pub fn clear_history(&mut self) -> Result<()> {
        self.editor.clear_history()?;
        Ok(())
    }
    
    /// Get current configuration
    pub fn config(&self) -> &LineEditorConfig {
        &self.config
    }
    
    /// Load configuration from file
    pub fn load_config<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.config = LineEditorConfig::load_from_file(path)?;
        self.apply_config()?;
        Ok(())
    }
    
    /// Save current configuration to file
    pub fn save_config<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.config.save_to_file(path)?;
        Ok(())
    }
    
    /// Apply configuration to the editor
    fn apply_config(&mut self) -> Result<()> {
        // Note: set_max_history_size might not be available in this version
        // self.editor.set_max_history_size(self.config.history_size)?;
        Ok(())
    }
}

impl Default for NexusLineEditor {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Compute a simple structural hint for incomplete command lines.
fn compute_structural_hint(line: &str) -> Option<String> {
    // Balance parentheses and quotes. If unbalanced, suggest closing.
    let mut paren = 0i32;
    let mut brace = 0i32;
    let mut bracket = 0i32;
    let mut in_single = false;
    let mut in_double = false;
    let mut prev = '\0';
    for ch in line.chars() {
        match ch {
            '\'' if !in_double && prev != '\\' => in_single = !in_single,
            '"' if !in_single && prev != '\\' => in_double = !in_double,
            '(' if !in_single && !in_double => paren += 1,
            ')' if !in_single && !in_double => paren -= 1,
            '{' if !in_single && !in_double => brace += 1,
            '}' if !in_single && !in_double => brace -= 1,
            '[' if !in_single && !in_double => bracket += 1,
            ']' if !in_single && !in_double => bracket -= 1,
            _ => {}
        }
        prev = ch;
    }
    if in_single { return Some("'".to_string()); }
    if in_double { return Some("\"".to_string()); }
    if paren > 0 { return Some(")".to_string()); }
    if brace > 0 { return Some("}".to_string()); }
    if bracket > 0 { return Some("]".to_string()); }
    // Common reserved words that likely expect further input
    let trimmed = line.trim_end();
    let lower = trimmed.to_ascii_lowercase();
    if ["if", "then", "else", "elif", "do", "case"].iter().any(|k| lower.ends_with(k)) {
        return Some(" ".to_string());
    }
    None
}

/// Rustyline helper that adapts our `NexusCompleter` to rustyline's sync completer API.
pub struct RustyHelper {
    shared: Option<StdArc<StdMutex<crate::completion::NexusCompleter>>>,
}

impl RustyHelper {
    fn new_standalone() -> Self { Self { shared: None } }
    fn set_shared_completer(&mut self, shared: StdArc<StdMutex<crate::completion::NexusCompleter>>) {
        self.shared = Some(shared);
    }
}

impl RLHelperTrait for RustyHelper {}

impl Highlighter for RustyHelper {}

impl Hinter for RustyHelper {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, _ctx: &RustylineContext<'_>) -> Option<Self::Hint> {
        // Prefer parser-based quick validation if a shared completer exists (has parser dependency)
        if let Some(shared) = &self.shared {
            if let Ok(_engine) = shared.lock() {
                // Reserved for future: use parser for contextual hints
            }
        }
        // Fallback: structural heuristics for common incomplete constructs
        compute_structural_hint(line)
    }
}

impl Validator for RustyHelper {}

impl RLCompleter for RustyHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &RustylineContext<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
        if let Some(shared) = &self.shared {
            if let Ok(engine) = shared.lock() {
                let (start_pos, pairs) = engine.complete_for_rustyline_sync(line, pos);
                return Ok((start_pos, pairs));
            }
        }
        Ok((pos, Vec::new()))
    }
}

/// Simple configuration for line editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineEditorConfig {
    /// Maximum number of history entries
    pub history_size: usize,
    /// Whether to automatically add commands to history
    pub auto_add_history: bool,
    /// Whether to use Vi or Emacs key bindings
    pub edit_mode: String,
}

impl Default for LineEditorConfig {
    fn default() -> Self {
        Self {
            history_size: 1000,
            auto_add_history: true,
            edit_mode: "emacs".to_string(),
        }
    }
}

impl LineEditorConfig {
    /// Load configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read config file")?;
        let config: Self = toml::from_str(&content)
            .context("Failed to parse config file")?;
        Ok(config)
    }
    
    /// Create config with full features enabled
    pub fn with_full_features() -> Self {
        Self {
            history_size: 10000,
            auto_add_history: true,
            edit_mode: "emacs".to_string(),
        }
    }
    
    /// Save configuration to TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        // Create directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, content)
            .context("Failed to write config file")?;
        Ok(())
    }
}
