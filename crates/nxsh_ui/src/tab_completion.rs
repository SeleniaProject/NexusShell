//! Enhanced Tab Completion Handler
//!
//! This module provides advanced tab completion functionality with:
//! - Beautiful visual completion panel
//! - Tab navigation through candidates
//! - Smart filtering and ranking
//! - Contextual completion suggestions

use anyhow::Result;
use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    terminal::size,
};
use std::{collections::VecDeque, time::Instant};

use crate::{
    completion_engine::CompletionEngine,
    // completion_panel::{CompletionPanel, PanelConfig}, // Temporarily disabled
};

/// Enhanced tab completion handler with visual panel
pub struct TabCompletionHandler {
    /// Advanced completion engine
    completion_engine: CompletionEngine,
    /// Visual completion panel
    // completion_panel: CompletionPanel, // Temporarily disabled
    /// Current completion state
    completion_state: CompletionState,
    /// Input history for smart suggestions
    input_history: VecDeque<String>,
    /// Performance metrics
    metrics: CompletionMetrics,
}

/// Current state of tab completion
#[derive(Debug, Clone, Default)]
pub struct CompletionState {
    /// Whether completion panel is visible
    pub is_visible: bool,
    /// Original input text before completion
    pub original_input: String,
    /// Current cursor position
    pub cursor_position: usize,
    /// Last completion request time
    pub last_request_time: Option<Instant>,
    /// Number of tab presses in sequence
    pub tab_sequence_count: u32,
    /// Whether we're in multi-tab navigation mode
    pub navigation_mode: bool,
}

/// Performance metrics for completion
#[derive(Debug, Clone, Default)]
pub struct CompletionMetrics {
    /// Total completion requests
    pub requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Last update time
    pub last_update: Option<Instant>,
}

impl TabCompletionHandler {
    /// Create a new tab completion handler
    pub fn new() -> Result<Self> {
        let completion_engine = CompletionEngine::new();

        Ok(Self {
            completion_engine,
            // completion_panel, // Temporarily disabled
            completion_state: CompletionState::default(),
            input_history: VecDeque::with_capacity(100),
            metrics: CompletionMetrics::default(),
        })
    }

    /// Handle tab key press with visual completion
    pub async fn handle_tab_key(
        &mut self,
        input: &str,
        cursor_pos: usize,
    ) -> Result<TabCompletionResult> {
        let start_time = Instant::now();

        // Update completion state
        self.update_completion_state(input, cursor_pos);

        // Get completion suggestions
        let completion_result = self.completion_engine.get_completions(input);

        // Update performance metrics
        self.update_metrics(start_time);

        if completion_result.items.is_empty() {
            // No candidates available
            self.hide_panel()?;
            return Ok(TabCompletionResult::NoSuggestions);
        }

        // Handle different tab completion scenarios
        match self.completion_state.tab_sequence_count {
            1 => {
                // First tab - try auto-completion or show panel
                self.handle_first_tab(completion_result).await
            }
            2..=3 => {
                // Second/third tab - navigate through suggestions
                self.handle_navigation_tab().await
            }
            _ => {
                // Multiple tabs - cycle through all candidates
                self.handle_cycling_tab().await
            }
        }
    }

    /// Handle key input during completion navigation
    pub async fn handle_key_during_completion(
        &mut self,
        key_event: KeyEvent,
    ) -> Result<Option<TabCompletionResult>> {
        if !self.completion_state.is_visible {
            return Ok(None);
        }

        match (key_event.code, key_event.modifiers) {
            // Tab or Down arrow - next candidate
            (KeyCode::Tab, KeyModifiers::NONE) | (KeyCode::Down, _) => {
                // self.completion_panel.select_next()?;
                self.render_panel().await?;
                Ok(Some(TabCompletionResult::NavigationUpdate))
            }

            // Shift+Tab or Up arrow - previous candidate
            (KeyCode::BackTab, _) | (KeyCode::Up, _) => {
                // self.completion_panel.select_previous()?;
                self.render_panel().await?;
                Ok(Some(TabCompletionResult::NavigationUpdate))
            }

            // Enter - accept selected candidate
            (KeyCode::Enter, _) => {
                let result = self.accept_selected_candidate().await?;
                Ok(Some(result))
            }

            // Escape - cancel completion
            (KeyCode::Esc, _) => {
                self.hide_panel()?;
                Ok(Some(TabCompletionResult::Cancelled))
            }

            // Character input - update completion
            (KeyCode::Char(_ch), _) => {
                // Let the caller handle character input
                Ok(None)
            }

            // Other keys - hide panel
            _ => {
                self.hide_panel()?;
                Ok(None)
            }
        }
    }

    /// Handle first tab press
    async fn handle_first_tab(
        &mut self,
        completion_result: crate::completion_engine::CompletionResult,
    ) -> Result<TabCompletionResult> {
        // Check if there's a unique completion
        if completion_result.items.len() == 1 {
            let candidate = &completion_result.items[0];
            return Ok(TabCompletionResult::SingleCompletion {
                text: candidate.text.clone(),
                description: candidate.description.clone(),
            });
        }

        // Check for common prefix completion
        if let Some(common_prefix) = self.find_common_prefix(&completion_result.items) {
            if common_prefix.len() > completion_result.prefix.len() {
                return Ok(TabCompletionResult::PartialCompletion {
                    text: common_prefix,
                    remaining_candidates: completion_result.items.len(),
                });
            }
        }

        // Show visual completion panel
        self.show_panel_with_candidates(completion_result.items)
            .await?;

        Ok(TabCompletionResult::PanelShown {
            candidate_count: 0, // Temporarily hardcoded
        })
    }

    /// Handle navigation tab press
    async fn handle_navigation_tab(&mut self) -> Result<TabCompletionResult> {
        if self.completion_state.is_visible {
            // self.completion_panel.select_next()?;
            self.render_panel().await?;
            Ok(TabCompletionResult::NavigationUpdate)
        } else {
            Ok(TabCompletionResult::NoAction)
        }
    }

    /// Handle cycling tab press
    async fn handle_cycling_tab(&mut self) -> Result<TabCompletionResult> {
        if self.completion_state.is_visible {
            // self.completion_panel.select_next()?;
            self.render_panel().await?;
            Ok(TabCompletionResult::NavigationUpdate)
        } else {
            Ok(TabCompletionResult::NoAction)
        }
    }

    /// Show completion panel with candidates
    async fn show_panel_with_candidates(
        &mut self,
        _candidates: Vec<crate::completion_engine::CompletionItem>,
    ) -> Result<()> {
        // self.completion_panel.set_candidates(candidates)?;
        self.completion_state.is_visible = true;
        self.completion_state.navigation_mode = true;
        self.render_panel().await?;
        Ok(())
    }

    /// Render the completion panel at current cursor position
    async fn render_panel(&mut self) -> Result<()> {
        // Update animation state
        // self.completion_panel.update_animation()?;

        // Get current cursor position
        let (_cursor_x, _cursor_y) = self.get_cursor_position()?;

        // Render panel
        // self.completion_panel.render(cursor_x, cursor_y)?;

        Ok(())
    }

    /// Hide the completion panel
    fn hide_panel(&mut self) -> Result<()> {
        if self.completion_state.is_visible {
            // self.completion_panel.hide()?;
            self.completion_state.is_visible = false;
            self.completion_state.navigation_mode = false;
            self.completion_state.tab_sequence_count = 0;
        }
        Ok(())
    }

    /// Accept the currently selected candidate
    async fn accept_selected_candidate(&mut self) -> Result<TabCompletionResult> {
        // Temporarily disabled
        // if let Some(candidate) = self.completion_panel.get_selected_candidate() {
        //     let result = TabCompletionResult::CompletionAccepted {
        //         text: candidate.text.clone(),
        //         description: Some(candidate.description.clone()),
        //     };
        //     self.hide_panel()?;
        //     Ok(result)
        // } else {
        Ok(TabCompletionResult::NoAction)
        // }
    }

    /// Find common prefix among candidates
    fn find_common_prefix(
        &self,
        candidates: &[crate::completion_engine::CompletionItem],
    ) -> Option<String> {
        if candidates.is_empty() {
            return None;
        }

        let first = &candidates[0].text;
        let mut common_len = first.len();

        for candidate in candidates.iter().skip(1) {
            let common = first
                .chars()
                .zip(candidate.text.chars())
                .take_while(|(a, b)| a == b)
                .count();
            common_len = common_len.min(common);
        }

        if common_len > 0 {
            Some(first.chars().take(common_len).collect())
        } else {
            None
        }
    }

    /// Update completion state
    fn update_completion_state(&mut self, input: &str, cursor_pos: usize) {
        let now = Instant::now();

        // Check if this is a new completion sequence
        if let Some(last_time) = self.completion_state.last_request_time {
            if now.duration_since(last_time).as_millis() > 500 {
                // Reset sequence if more than 500ms since last tab
                self.completion_state.tab_sequence_count = 0;
            }
        }

        self.completion_state.tab_sequence_count += 1;
        self.completion_state.last_request_time = Some(now);
        self.completion_state.original_input = input.to_string();
        self.completion_state.cursor_position = cursor_pos;
    }

    /// Update performance metrics
    fn update_metrics(&mut self, start_time: Instant) {
        let elapsed_ms = start_time.elapsed().as_nanos() as f64 / 1_000_000.0;

        self.metrics.requests += 1;

        // Update rolling average
        if self.metrics.requests == 1 {
            self.metrics.avg_response_time_ms = elapsed_ms;
        } else {
            let alpha = 0.1; // Smoothing factor
            self.metrics.avg_response_time_ms =
                alpha * elapsed_ms + (1.0 - alpha) * self.metrics.avg_response_time_ms;
        }

        self.metrics.last_update = Some(Instant::now());
    }

    /// Get current cursor position on screen
    fn get_cursor_position(&self) -> Result<(u16, u16)> {
        // This is a simplified implementation
        // In a real application, you would track the actual cursor position
        let (_term_width, term_height) = size()?;
        Ok((0, term_height.saturating_sub(1)))
    }

    /// Check if completion panel is currently visible
    pub fn is_panel_visible(&self) -> bool {
        self.completion_state.is_visible
    }

    /// Get current completion metrics
    pub fn get_metrics(&self) -> &CompletionMetrics {
        &self.metrics
    }

    /// Add input to history for smart suggestions
    pub fn add_to_history(&mut self, input: String) {
        if !input.trim().is_empty() {
            if self.input_history.len() >= 100 {
                self.input_history.pop_front();
            }
            self.input_history.push_back(input);
        }
    }

    /// Update animation frame
    pub async fn update_animation(&mut self) -> Result<bool> {
        if self.completion_state.is_visible {
            let needs_redraw = true; // Temporarily hardcoded
            if needs_redraw {
                let (_cursor_x, _cursor_y) = self.get_cursor_position()?;
                // self.completion_panel.render(cursor_x, cursor_y)?;
            }
            Ok(needs_redraw)
        } else {
            Ok(false)
        }
    }
}

/// Result of tab completion operation
#[derive(Debug, Clone)]
pub enum TabCompletionResult {
    /// No suggestions available
    NoSuggestions,
    /// Single completion found and can be applied
    SingleCompletion {
        text: String,
        description: Option<String>,
    },
    /// Partial completion applied (common prefix)
    PartialCompletion {
        text: String,
        remaining_candidates: usize,
    },
    /// Visual panel shown with multiple candidates
    PanelShown { candidate_count: usize },
    /// Navigation within panel updated
    NavigationUpdate,
    /// Completion was accepted by user
    CompletionAccepted {
        text: String,
        description: Option<String>,
    },
    /// Completion was cancelled
    Cancelled,
    /// No action taken
    NoAction,
}

impl TabCompletionResult {
    /// Check if this result requires UI update
    pub fn requires_ui_update(&self) -> bool {
        matches!(
            self,
            TabCompletionResult::PanelShown { .. }
                | TabCompletionResult::NavigationUpdate
                | TabCompletionResult::Cancelled
        )
    }

    /// Check if this result completes the input
    pub fn completes_input(&self) -> bool {
        matches!(
            self,
            TabCompletionResult::SingleCompletion { .. }
                | TabCompletionResult::PartialCompletion { .. }
                | TabCompletionResult::CompletionAccepted { .. }
        )
    }

    /// Get completion text if available
    pub fn get_completion_text(&self) -> Option<&str> {
        match self {
            TabCompletionResult::SingleCompletion { text, .. }
            | TabCompletionResult::PartialCompletion { text, .. }
            | TabCompletionResult::CompletionAccepted { text, .. } => Some(text),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tab_completion_creation() {
        let handler = TabCompletionHandler::new().unwrap();
        assert!(!handler.is_panel_visible());
        assert_eq!(handler.metrics.requests, 0);
    }

    #[test]
    fn test_common_prefix_finding() {
        let handler = TabCompletionHandler::new().unwrap();
        let candidates = vec![
            crate::completion_engine::CompletionItem {
                text: "test_file_1.txt".to_string(),
                display_text: "test_file_1.txt".to_string(),
                completion_type: crate::completion_engine::CompletionType::File,
                description: Some("".to_string()),
                score: 1.0,
                source: "test".to_string(),
                metadata: std::collections::HashMap::new(),
            },
            crate::completion_engine::CompletionItem {
                text: "test_file_2.txt".to_string(),
                display_text: "test_file_2.txt".to_string(),
                completion_type: crate::completion_engine::CompletionType::File,
                description: Some("".to_string()),
                score: 0.9,
                source: "test".to_string(),
                metadata: std::collections::HashMap::new(),
            },
        ];

        let prefix = handler.find_common_prefix(&candidates);
        assert_eq!(prefix, Some("test_file_".to_string()));
    }

    #[test]
    fn test_completion_result_methods() {
        let result = TabCompletionResult::SingleCompletion {
            text: "example.txt".to_string(),
            description: Some("Example file".to_string()),
        };

        assert!(result.completes_input());
        assert!(!result.requires_ui_update());
        assert_eq!(result.get_completion_text(), Some("example.txt"));
    }
}
