//! Beautiful Tab Completion Panel UI Component
//! 
//! This module provides a stunning, highly visual completion panel with:
//! - Elegant candidate display with icons and descriptions
//! - Tab navigation through candidates
//! - Category-based organization
//! - Smooth animations and transitions
//! - Theme-aware styling

use anyhow::Result;
use std::{
    collections::HashMap,
    fmt::Write,
    io::{self, Write as IoWrite},
    time::{Duration, Instant},
};
use unicode_width::UnicodeWidthStr;
use crossterm::{
    cursor::{MoveTo, Hide, Show},
    style::{Color, SetForegroundColor, SetBackgroundColor, ResetColor, Attribute, SetAttribute},
    terminal::{Clear, ClearType},
    execute,
};

use crate::completion_engine::{CompletionCandidate, CompletionResult, CompletionContext};

/// Beautiful completion panel with visual enhancements
#[derive(Debug, Clone)]
pub struct CompletionPanel {
    /// Current completion candidates
    candidates: Vec<CompletionCandidate>,
    /// Currently selected candidate index
    selected_index: usize,
    /// Panel display configuration
    config: PanelConfig,
    /// Animation state
    animation_state: AnimationState,
    /// Category grouping
    categories: HashMap<String, Vec<usize>>,
    /// Panel dimensions
    dimensions: PanelDimensions,
    /// Theme configuration
    theme: CompletionTheme,
}

/// Configuration for the completion panel appearance
#[derive(Debug, Clone)]
pub struct PanelConfig {
    /// Maximum width of the panel
    pub max_width: usize,
    /// Maximum height of the panel
    pub max_height: usize,
    /// Number of candidates to show per page
    pub candidates_per_page: usize,
    /// Show category headers
    pub show_categories: bool,
    /// Show candidate icons
    pub show_icons: bool,
    /// Show detailed descriptions
    pub show_descriptions: bool,
    /// Enable animations
    pub enable_animations: bool,
    /// Animation duration in milliseconds
    pub animation_duration_ms: u64,
    /// Auto-hide panel when no matches
    pub auto_hide: bool,
}

/// Animation state for smooth transitions
#[derive(Debug, Clone)]
pub struct AnimationState {
    /// Animation start time
    pub start_time: Option<Instant>,
    /// Current animation phase
    pub phase: AnimationPhase,
    /// Target opacity (0.0 to 1.0)
    pub target_opacity: f32,
    /// Current opacity (0.0 to 1.0)
    pub current_opacity: f32,
    /// Slide offset for entrance animation
    pub slide_offset: i32,
}

/// Animation phases
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationPhase {
    /// Panel is hidden
    Hidden,
    /// Panel is appearing
    FadeIn,
    /// Panel is fully visible
    Visible,
    /// Panel is disappearing
    FadeOut,
    /// Selection change animation
    SelectionChange,
}

/// Panel dimensions and positioning
#[derive(Debug, Clone)]
pub struct PanelDimensions {
    /// Panel width
    pub width: usize,
    /// Panel height
    pub height: usize,
    /// X position on screen
    pub x: u16,
    /// Y position on screen
    pub y: u16,
    /// Content area width (excluding borders)
    pub content_width: usize,
    /// Content area height (excluding borders)
    pub content_height: usize,
}

/// Theme configuration for completion panel
#[derive(Debug, Clone)]
pub struct CompletionTheme {
    /// Border characters
    pub border: BorderChars,
    /// Color scheme
    pub colors: CompletionColors,
    /// Icons for different completion types
    pub icons: CompletionIcons,
    /// Typography settings
    pub typography: Typography,
}

/// Border characters for the panel
#[derive(Debug, Clone)]
pub struct BorderChars {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
    pub tee_down: char,
    pub tee_up: char,
    pub tee_right: char,
    pub tee_left: char,
}

/// Color scheme for completion panel
#[derive(Debug, Clone)]
pub struct CompletionColors {
    /// Border color
    pub border: Color,
    /// Background color
    pub background: Color,
    /// Selected item background
    pub selected_background: Color,
    /// Selected item text
    pub selected_text: Color,
    /// Normal item text
    pub text: Color,
    /// Category header color
    pub category_header: Color,
    /// Description text color
    pub description: Color,
    /// Icon color
    pub icon: Color,
    /// Highlight color for matching text
    pub highlight: Color,
}

/// Icons for different completion types
#[derive(Debug, Clone)]
pub struct CompletionIcons {
    /// File icon
    pub file: &'static str,
    /// Directory icon
    pub directory: &'static str,
    /// Command icon
    pub command: &'static str,
    /// Variable icon
    pub variable: &'static str,
    /// Function icon
    pub function: &'static str,
    /// Keyword icon
    pub keyword: &'static str,
    /// Smart suggestion icon
    pub smart_suggestion: &'static str,
    /// Default icon
    pub default: &'static str,
}

/// Typography settings
#[derive(Debug, Clone)]
pub struct Typography {
    /// Use bold for selected items
    pub bold_selected: bool,
    /// Use italic for descriptions
    pub italic_descriptions: bool,
    /// Maximum description length
    pub max_description_length: usize,
    /// Truncation indicator
    pub truncation_indicator: &'static str,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            max_width: 80,
            max_height: 15,
            candidates_per_page: 10,
            show_categories: true,
            show_icons: true,
            show_descriptions: true,
            enable_animations: true,
            animation_duration_ms: 150,
            auto_hide: true,
        }
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            start_time: None,
            phase: AnimationPhase::Hidden,
            target_opacity: 0.0,
            current_opacity: 0.0,
            slide_offset: 0,
        }
    }
}

impl Default for CompletionTheme {
    fn default() -> Self {
        Self {
            border: BorderChars::unicode(),
            colors: CompletionColors::default(),
            icons: CompletionIcons::default(),
            typography: Typography::default(),
        }
    }
}

impl BorderChars {
    /// Unicode box drawing characters
    pub fn unicode() -> Self {
        Self {
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            horizontal: '─',
            vertical: '│',
            tee_down: '┬',
            tee_up: '┴',
            tee_right: '├',
            tee_left: '┤',
        }
    }

    /// ASCII fallback characters
    pub fn ascii() -> Self {
        Self {
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            horizontal: '-',
            vertical: '|',
            tee_down: '+',
            tee_up: '+',
            tee_right: '+',
            tee_left: '+',
        }
    }
}

impl Default for CompletionColors {
    fn default() -> Self {
        Self {
            border: Color::DarkGrey,
            background: Color::Black,
            selected_background: Color::DarkBlue,
            selected_text: Color::White,
            text: Color::White,
            category_header: Color::Cyan,
            description: Color::DarkGrey,
            icon: Color::Yellow,
            highlight: Color::Green,
        }
    }
}

impl Default for CompletionIcons {
    fn default() -> Self {
        Self {
            file: "📄",
            directory: "📁",
            command: "⚡",
            variable: "💫",
            function: "🔧",
            keyword: "🔑",
            smart_suggestion: "🧠",
            default: "🔹",
        }
    }
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            bold_selected: true,
            italic_descriptions: true,
            max_description_length: 40,
            truncation_indicator: "…",
        }
    }
}

impl CompletionPanel {
    /// Create a new completion panel
    pub fn new(config: PanelConfig) -> Self {
        Self {
            candidates: Vec::new(),
            selected_index: 0,
            config,
            animation_state: AnimationState::default(),
            categories: HashMap::new(),
            dimensions: PanelDimensions {
                width: 0,
                height: 0,
                x: 0,
                y: 0,
                content_width: 0,
                content_height: 0,
            },
            theme: CompletionTheme::default(),
        }
    }

    /// Set completion candidates and update display
    pub fn set_candidates(&mut self, candidates: Vec<CompletionCandidate>) -> Result<()> {
        self.candidates = candidates;
        self.selected_index = 0;
        self.organize_categories();
        self.calculate_dimensions()?;
        self.start_fade_in_animation();
        Ok(())
    }

    /// Move selection to next candidate
    pub fn select_next(&mut self) -> Result<()> {
        if !self.candidates.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.candidates.len();
            self.start_selection_animation();
        }
        Ok(())
    }

    /// Move selection to previous candidate
    pub fn select_previous(&mut self) -> Result<()> {
        if !self.candidates.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.candidates.len() - 1
            } else {
                self.selected_index - 1
            };
            self.start_selection_animation();
        }
        Ok(())
    }

    /// Get the currently selected candidate
    pub fn get_selected_candidate(&self) -> Option<&CompletionCandidate> {
        self.candidates.get(self.selected_index)
    }

    /// Check if panel should be visible
    pub fn should_show(&self) -> bool {
        !self.candidates.is_empty() && self.animation_state.phase != AnimationPhase::Hidden
    }

    /// Start fade-in animation
    fn start_fade_in_animation(&mut self) {
        if self.config.enable_animations {
            self.animation_state = AnimationState {
                start_time: Some(Instant::now()),
                phase: AnimationPhase::FadeIn,
                target_opacity: 1.0,
                current_opacity: 0.0,
                slide_offset: -5,
            };
        } else {
            self.animation_state.phase = AnimationPhase::Visible;
            self.animation_state.current_opacity = 1.0;
        }
    }

    /// Start selection change animation
    fn start_selection_animation(&mut self) {
        if self.config.enable_animations {
            self.animation_state.phase = AnimationPhase::SelectionChange;
            self.animation_state.start_time = Some(Instant::now());
        }
    }

    /// Organize candidates into categories
    fn organize_categories(&mut self) {
        self.categories.clear();
        
        for (index, candidate) in self.candidates.iter().enumerate() {
            let category = match candidate.candidate_type {
                crate::completion_engine::CandidateType::Command => "Commands",
                crate::completion_engine::CandidateType::File => "Files",
                crate::completion_engine::CandidateType::Directory => "Directories", 
                crate::completion_engine::CandidateType::Variable => "Variables",
                crate::completion_engine::CandidateType::Builtin => "Builtins",
                crate::completion_engine::CandidateType::SmartSuggestion => "Smart Suggestions",
                _ => "Other",
            };
            
            self.categories
                .entry(category.to_string())
                .or_insert_with(Vec::new)
                .push(index);
        }
    }

    /// Calculate optimal panel dimensions
    fn calculate_dimensions(&mut self) -> Result<()> {
        // Calculate content requirements
        let max_candidate_width = self.candidates
            .iter()
            .map(|c| self.calculate_candidate_display_width(c))
            .max()
            .unwrap_or(20);

        // Account for borders, padding, and icons
        let total_width = (max_candidate_width + 6).min(self.config.max_width);
        let total_height = (self.candidates.len() + 4).min(self.config.max_height);

        self.dimensions = PanelDimensions {
            width: total_width,
            height: total_height,
            x: 0, // Will be set during rendering
            y: 0, // Will be set during rendering
            content_width: total_width.saturating_sub(4),
            content_height: total_height.saturating_sub(4),
        };

        Ok(())
    }

    /// Calculate display width needed for a candidate
    fn calculate_candidate_display_width(&self, candidate: &CompletionCandidate) -> usize {
        let icon_width = if self.config.show_icons { 3 } else { 0 };
        let text_width = candidate.text.width();
        let description_width = if self.config.show_descriptions {
            candidate.description.as_ref()
                .map(|d| d.width().min(self.theme.typography.max_description_length))
                .unwrap_or(0) + 3 // " - " separator
        } else {
            0
        };
        
        icon_width + text_width + description_width
    }

    /// Update animation state
    pub fn update_animation(&mut self) -> Result<bool> {
        if !self.config.enable_animations {
            return Ok(false);
        }

        let Some(start_time) = self.animation_state.start_time else {
            return Ok(false);
        };

        let elapsed = start_time.elapsed().as_millis() as u64;
        let duration = self.config.animation_duration_ms;
        
        let progress = (elapsed as f32 / duration as f32).min(1.0);
        
        match self.animation_state.phase {
            AnimationPhase::FadeIn => {
                self.animation_state.current_opacity = progress;
                self.animation_state.slide_offset = (-5.0 * (1.0 - progress)) as i32;
                
                if progress >= 1.0 {
                    self.animation_state.phase = AnimationPhase::Visible;
                    self.animation_state.start_time = None;
                }
                Ok(true)
            }
            AnimationPhase::FadeOut => {
                self.animation_state.current_opacity = 1.0 - progress;
                
                if progress >= 1.0 {
                    self.animation_state.phase = AnimationPhase::Hidden;
                    self.animation_state.start_time = None;
                }
                Ok(true)
            }
            AnimationPhase::SelectionChange => {
                if progress >= 1.0 {
                    self.animation_state.phase = AnimationPhase::Visible;
                    self.animation_state.start_time = None;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Render the completion panel at the specified position
    pub fn render(&self, cursor_x: u16, cursor_y: u16) -> Result<()> {
        if !self.should_show() {
            return Ok(());
        }

        let mut stdout = io::stdout();
        
        // Calculate panel position (below cursor, but stay within screen bounds)
        let panel_x = cursor_x;
        let panel_y = cursor_y + 1;

        // Apply animation offset
        let render_x = panel_x;
        let render_y = (panel_y as i32 + self.animation_state.slide_offset).max(0) as u16;

        execute!(stdout, Hide)?;

        // Render panel background and border
        self.render_panel_background(render_x, render_y)?;
        
        // Render candidates
        self.render_candidates(render_x + 1, render_y + 1)?;
        
        // Render category headers if enabled
        if self.config.show_categories {
            self.render_category_headers(render_x + 1, render_y + 1)?;
        }

        execute!(stdout, Show)?;
        stdout.flush()?;

        Ok(())
    }

    /// Render panel background and border
    fn render_panel_background(&self, x: u16, y: u16) -> Result<()> {
        let mut stdout = io::stdout();
        let chars = &self.theme.border;
        
        // Apply opacity for animation
        let alpha = (self.animation_state.current_opacity * 255.0) as u8;
        
        execute!(stdout, SetForegroundColor(self.theme.colors.border))?;

        // Top border
        execute!(stdout, MoveTo(x, y))?;
        print!("{}", chars.top_left);
        for _ in 1..self.dimensions.width - 1 {
            print!("{}", chars.horizontal);
        }
        print!("{}", chars.top_right);

        // Side borders and content area
        for row in 1..self.dimensions.height - 1 {
            execute!(stdout, MoveTo(x, y + row as u16))?;
            print!("{}", chars.vertical);
            
            // Clear content area
            execute!(stdout, SetBackgroundColor(self.theme.colors.background))?;
            for _ in 1..self.dimensions.width - 1 {
                print!(" ");
            }
            execute!(stdout, ResetColor, SetForegroundColor(self.theme.colors.border))?;
            print!("{}", chars.vertical);
        }

        // Bottom border
        execute!(stdout, MoveTo(x, y + self.dimensions.height as u16 - 1))?;
        print!("{}", chars.bottom_left);
        for _ in 1..self.dimensions.width - 1 {
            print!("{}", chars.horizontal);
        }
        print!("{}", chars.bottom_right);

        execute!(stdout, ResetColor)?;
        Ok(())
    }

    /// Render completion candidates
    fn render_candidates(&self, x: u16, y: u16) -> Result<()> {
        let mut stdout = io::stdout();
        let visible_candidates = self.get_visible_candidates();
        
        for (display_index, &candidate_index) in visible_candidates.iter().enumerate() {
            let candidate = &self.candidates[candidate_index];
            let is_selected = candidate_index == self.selected_index;
            let render_y = y + display_index as u16;
            
            self.render_single_candidate(x, render_y, candidate, is_selected)?;
        }

        execute!(stdout, ResetColor)?;
        Ok(())
    }

    /// Render a single completion candidate
    fn render_single_candidate(
        &self,
        x: u16,
        y: u16,
        candidate: &CompletionCandidate,
        is_selected: bool,
    ) -> Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, MoveTo(x, y))?;

        // Set background for selected item
        if is_selected {
            execute!(stdout, SetBackgroundColor(self.theme.colors.selected_background))?;
            execute!(stdout, SetForegroundColor(self.theme.colors.selected_text))?;
            if self.theme.typography.bold_selected {
                execute!(stdout, SetAttribute(Attribute::Bold))?;
            }
        } else {
            execute!(stdout, SetForegroundColor(self.theme.colors.text))?;
        }

        let mut output = String::new();

        // Add icon if enabled
        if self.config.show_icons {
            let icon = self.get_icon_for_candidate(candidate);
            write!(output, "{} ", icon)?;
        }

        // Add candidate text
        output.push_str(&candidate.text);

        // Add description if enabled and available
        if self.config.show_descriptions {
            if !candidate.description.is_empty() {
                let truncated_desc = self.truncate_description(&candidate.description);
                write!(output, " - {}", truncated_desc)?;
            }
        }

        // Ensure the line fills the available width
        let padding_width = self.dimensions.content_width.saturating_sub(output.width());
        output.push_str(&" ".repeat(padding_width));

        print!("{}", output);
        execute!(stdout, ResetColor)?;

        Ok(())
    }

    /// Get visible candidates for current page
    fn get_visible_candidates(&self) -> Vec<usize> {
        let start_index = (self.selected_index / self.config.candidates_per_page) 
            * self.config.candidates_per_page;
        let end_index = (start_index + self.config.candidates_per_page)
            .min(self.candidates.len());
        
        (start_index..end_index).collect()
    }

    /// Get appropriate icon for candidate type
    fn get_icon_for_candidate(&self, candidate: &CompletionCandidate) -> &str {
        match candidate.candidate_type {
            crate::completion_engine::CandidateType::File => self.theme.icons.file,
            crate::completion_engine::CandidateType::Directory => self.theme.icons.directory,
            crate::completion_engine::CandidateType::Command => self.theme.icons.command,
            crate::completion_engine::CandidateType::Variable => self.theme.icons.variable,
            crate::completion_engine::CandidateType::Builtin => self.theme.icons.function,
            crate::completion_engine::CandidateType::SmartSuggestion => self.theme.icons.smart_suggestion,
            _ => self.theme.icons.default,
        }
    }

    /// Truncate description to fit display
    fn truncate_description(&self, description: &str) -> String {
        if description.width() <= self.theme.typography.max_description_length {
            description.to_string()
        } else {
            let mut truncated = String::new();
            let mut current_width = 0;
            
            for ch in description.chars() {
                let char_width = ch.width().unwrap_or(0);
                if current_width + char_width > self.theme.typography.max_description_length - 1 {
                    break;
                }
                truncated.push(ch);
                current_width += char_width;
            }
            
            truncated.push_str(self.theme.typography.truncation_indicator);
            truncated
        }
    }

    /// Render category headers (placeholder for future implementation)
    fn render_category_headers(&self, _x: u16, _y: u16) -> Result<()> {
        // TODO: Implement category header rendering
        Ok(())
    }

    /// Hide the panel with fade-out animation
    pub fn hide(&mut self) -> Result<()> {
        if self.config.enable_animations {
            self.animation_state = AnimationState {
                start_time: Some(Instant::now()),
                phase: AnimationPhase::FadeOut,
                target_opacity: 0.0,
                current_opacity: self.animation_state.current_opacity,
                slide_offset: 0,
            };
        } else {
            self.animation_state.phase = AnimationPhase::Hidden;
        }
        Ok(())
    }

    /// Clear the panel area from screen
    pub fn clear(&self, x: u16, y: u16) -> Result<()> {
        let mut stdout = io::stdout();
        
        for row in 0..self.dimensions.height {
            execute!(stdout, MoveTo(x, y + row as u16))?;
            execute!(stdout, Clear(ClearType::UntilNewLine))?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_creation() {
        let config = PanelConfig::default();
        let panel = CompletionPanel::new(config);
        assert_eq!(panel.candidates.len(), 0);
        assert_eq!(panel.selected_index, 0);
    }

    #[test]
    fn test_candidate_navigation() {
        let mut panel = CompletionPanel::new(PanelConfig::default());
        let candidates = vec![
            CompletionCandidate {
                text: "test1".to_string(),
                description: "Test 1".to_string(),
                candidate_type: crate::completion_engine::CandidateType::Command,
                base_score: 1.0,
                boost_score: 0.0,
                metadata: std::collections::HashMap::new(),
            },
            CompletionCandidate {
                text: "test2".to_string(),
                description: "Test 2".to_string(),
                candidate_type: crate::completion_engine::CandidateType::Command,
                base_score: 0.9,
                boost_score: 0.0,
                metadata: std::collections::HashMap::new(),
            },
        ];
        
        panel.set_candidates(candidates).unwrap();
        assert_eq!(panel.selected_index, 0);
        
        panel.select_next().unwrap();
        assert_eq!(panel.selected_index, 1);
        
        panel.select_next().unwrap();
        assert_eq!(panel.selected_index, 0); // Wrap around
        
        panel.select_previous().unwrap();
        assert_eq!(panel.selected_index, 1);
    }

    #[test]
    fn test_description_truncation() {
        let panel = CompletionPanel::new(PanelConfig::default());
        let long_description = "This is a very long description that should be truncated";
        let truncated = panel.truncate_description(long_description);
        assert!(truncated.len() <= panel.theme.typography.max_description_length);
        assert!(truncated.ends_with("…"));
    }
}
