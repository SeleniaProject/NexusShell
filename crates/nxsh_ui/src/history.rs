//! Advanced history management for NexusShell CUI
//! Provides persistent history with search, filtering, and deduplication

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

/// A single history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub working_directory: Option<String>,
    pub session_id: Option<String>,
}

impl HistoryEntry {
    pub fn new(command: String) -> Self {
        Self {
            command,
            timestamp: Utc::now(),
            exit_code: None,
            working_directory: std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            session_id: None,
        }
    }

    pub fn with_exit_code(mut self, exit_code: i32) -> Self {
        self.exit_code = Some(exit_code);
        self
    }

    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

/// History configuration
#[derive(Debug, Clone)]
pub struct HistoryConfig {
    pub max_entries: usize,
    pub persist_to_file: bool,
    pub file_path: Option<PathBuf>,
    pub deduplicate: bool,
    pub ignore_duplicates: bool,
    pub ignore_space_prefixed: bool,
    pub save_exit_codes: bool,
    pub save_working_directory: bool,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        let mut history_file = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        history_file.push(".nxsh_history");

        Self {
            max_entries: 10000,
            persist_to_file: true,
            file_path: Some(history_file),
            deduplicate: true,
            ignore_duplicates: true,
            ignore_space_prefixed: true,
            save_exit_codes: true,
            save_working_directory: true,
        }
    }
}

/// Advanced history manager
pub struct History {
    entries: VecDeque<HistoryEntry>,
    config: HistoryConfig,
    current_index: Option<usize>,
    search_results: Vec<usize>,
    session_id: String,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    pub fn new() -> Self {
        Self::with_config(HistoryConfig::default())
    }

    pub fn with_config(config: HistoryConfig) -> Self {
        let mut history = Self {
            entries: VecDeque::new(),
            config,
            current_index: None,
            search_results: Vec::new(),
            session_id: uuid::Uuid::new_v4().to_string(),
        };

        if history.config.persist_to_file {
            if let Err(e) = history.load_from_file() {
                eprintln!("Warning: Failed to load history: {}", e);
            }
        }

        history
    }

    /// Add a new entry to history
    pub fn add_entry(&mut self, command: String) {
        // Skip empty commands
        if command.trim().is_empty() {
            return;
        }

        // Skip space-prefixed commands if configured
        if self.config.ignore_space_prefixed && command.starts_with(' ') {
            return;
        }

        // Check for duplicates
        if self.config.ignore_duplicates {
            if let Some(last_entry) = self.entries.back() {
                if last_entry.command == command {
                    return;
                }
            }
        }

        let entry =
            HistoryEntry::new(command.trim().to_string()).with_session_id(self.session_id.clone());

        self.entries.push_back(entry);

        // Maintain size limit
        while self.entries.len() > self.config.max_entries {
            self.entries.pop_front();
        }

        // Deduplicate if configured
        if self.config.deduplicate {
            self.deduplicate();
        }

        // Save to file if configured
        if self.config.persist_to_file {
            if let Err(e) = self.save_to_file() {
                eprintln!("Warning: Failed to save history: {}", e);
            }
        }

        self.current_index = None;
    }

    /// Get the previous entry in history
    pub fn previous(&mut self) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }

        let new_index = match self.current_index {
            None => self.entries.len() - 1,
            Some(index) => {
                if index > 0 {
                    index - 1
                } else {
                    return None;
                }
            }
        };

        self.current_index = Some(new_index);
        self.entries
            .get(new_index)
            .map(|entry| entry.command.clone())
    }

    /// Get the next entry in history
    pub fn next_entry(&mut self) -> Option<String> {
        match self.current_index {
            None => None,
            Some(index) => {
                if index + 1 < self.entries.len() {
                    self.current_index = Some(index + 1);
                    self.entries
                        .get(index + 1)
                        .map(|entry| entry.command.clone())
                } else {
                    self.current_index = None;
                    None
                }
            }
        }
    }

    /// Search history entries
    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|entry| entry.command.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Reverse search (like Ctrl+R)
    pub fn reverse_search(&mut self, query: &str) -> Option<String> {
        let query_lower = query.to_lowercase();

        // Start from current position or end
        let start_index = self.current_index.unwrap_or(self.entries.len());

        for i in (0..start_index).rev() {
            if let Some(entry) = self.entries.get(i) {
                if entry.command.to_lowercase().contains(&query_lower) {
                    self.current_index = Some(i);
                    return Some(entry.command.clone());
                }
            }
        }

        None
    }

    /// Get all entries
    pub fn entries(&self) -> impl Iterator<Item = &HistoryEntry> {
        self.entries.iter()
    }

    /// Get recent entries
    pub fn recent(&self, count: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().rev().take(count).collect()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_index = None;
        self.search_results.clear();

        if self.config.persist_to_file {
            if let Err(e) = self.save_to_file() {
                eprintln!("Warning: Failed to save cleared history: {}", e);
            }
        }
    }

    /// Get statistics
    pub fn stats(&self) -> HistoryStats {
        let total_entries = self.entries.len();
        let unique_commands = self
            .entries
            .iter()
            .map(|entry| &entry.command)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let commands_by_frequency = {
            let mut freq_map = std::collections::HashMap::new();
            for entry in &self.entries {
                *freq_map.entry(&entry.command).or_insert(0) += 1;
            }
            let mut freq_vec: Vec<_> = freq_map.into_iter().collect();
            freq_vec.sort_by(|a, b| b.1.cmp(&a.1));
            freq_vec
                .into_iter()
                .take(10)
                .map(|(k, v)| (k.clone(), v))
                .collect()
        };

        HistoryStats {
            total_entries,
            unique_commands,
            commands_by_frequency,
        }
    }

    fn deduplicate(&mut self) {
        let mut seen = std::collections::HashSet::new();
        let mut unique_entries = VecDeque::new();

        for entry in self.entries.drain(..) {
            if seen.insert(entry.command.clone()) {
                unique_entries.push_back(entry);
            }
        }

        self.entries = unique_entries;
    }

    fn load_from_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(file_path) = &self.config.file_path {
            if file_path.exists() {
                let file = File::open(file_path)?;
                let reader = BufReader::new(file);

                for line in reader.lines() {
                    let line = line?;
                    if let Ok(entry) = serde_json::from_str::<HistoryEntry>(&line) {
                        self.entries.push_back(entry);
                    } else {
                        // Fallback for plain text format
                        if !line.trim().is_empty() {
                            self.entries.push_back(HistoryEntry::new(line));
                        }
                    }
                }

                // Maintain size limit
                while self.entries.len() > self.config.max_entries {
                    self.entries.pop_front();
                }
            }
        }

        Ok(())
    }

    fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(file_path) = &self.config.file_path {
            // Create parent directories if they don't exist
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(file_path)?;

            let mut writer = BufWriter::new(file);

            for entry in &self.entries {
                let json = serde_json::to_string(entry)?;
                writeln!(writer, "{}", json)?;
            }

            writer.flush()?;
        }

        Ok(())
    }
}

/// History statistics
#[derive(Debug)]
pub struct HistoryStats {
    pub total_entries: usize,
    pub unique_commands: usize,
    pub commands_by_frequency: Vec<(String, usize)>,
}

impl Drop for History {
    fn drop(&mut self) {
        if self.config.persist_to_file {
            if let Err(e) = self.save_to_file() {
                eprintln!("Warning: Failed to save history on exit: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_basic_operations() {
        let mut history = History::with_config(HistoryConfig {
            persist_to_file: false,
            ..Default::default()
        });

        history.add_entry("ls -la".to_string());
        history.add_entry("cd /tmp".to_string());
        history.add_entry("pwd".to_string());

        assert_eq!(history.entries.len(), 3);
        assert_eq!(history.previous(), Some("pwd".to_string()));
        assert_eq!(history.previous(), Some("cd /tmp".to_string()));
        assert_eq!(history.next_entry(), Some("pwd".to_string()));
    }

    #[test]
    fn test_history_search() {
        let mut history = History::with_config(HistoryConfig {
            persist_to_file: false,
            ..Default::default()
        });

        history.add_entry("ls -la".to_string());
        history.add_entry("find . -name '*.rs'".to_string());
        history.add_entry("grep -r pattern .".to_string());

        let results = history.search("ls");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].command, "ls -la");
    }
}
