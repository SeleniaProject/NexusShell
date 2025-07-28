use anyhow::{Result, Context};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use ratatui::{
    style::{Color, Style},
    text::{Span, Line},
    widgets::Widget,
};
use log::{info, warn, error, debug};

use crate::themes::RgbColor;

/// Accessibility manager for NexusShell UI
pub struct AccessibilityManager {
    color_vision_profile: Arc<RwLock<ColorVisionProfile>>,
    contrast_settings: Arc<RwLock<ContrastSettings>>,
    screen_reader_settings: Arc<RwLock<ScreenReaderSettings>>,
    keyboard_navigation: Arc<RwLock<KeyboardNavigationSettings>>,
    visual_indicators: Arc<RwLock<VisualIndicatorSettings>>,
    config: AccessibilityConfig,
}

impl AccessibilityManager {
    /// Create a new accessibility manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            color_vision_profile: Arc::new(RwLock::new(ColorVisionProfile::default())),
            contrast_settings: Arc::new(RwLock::new(ContrastSettings::default())),
            screen_reader_settings: Arc::new(RwLock::new(ScreenReaderSettings::default())),
            keyboard_navigation: Arc::new(RwLock::new(KeyboardNavigationSettings::default())),
            visual_indicators: Arc::new(RwLock::new(VisualIndicatorSettings::default())),
            config: AccessibilityConfig::default(),
        })
    }
    
    /// Initialize the accessibility system
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing accessibility system");
        
        // Detect system accessibility settings
        self.detect_system_settings().await?;
        
        // Initialize screen reader support if needed
        self.initialize_screen_reader_support().await?;
        
        // Setup keyboard navigation
        self.setup_keyboard_navigation().await?;
        
        info!("Accessibility system initialized successfully");
        Ok(())
    }
    
    /// Set color vision profile for color diversity support
    pub async fn set_color_vision_profile(&self, profile: ColorVisionProfile) -> Result<()> {
        info!("Setting color vision profile: {:?}", profile.profile_type);
        
        let mut current_profile = self.color_vision_profile.write().await;
        *current_profile = profile;
        
        info!("Color vision profile updated");
        Ok(())
    }
    
    /// Adjust color for color vision accessibility
    pub async fn adjust_color_for_vision(&self, color: RgbColor) -> RgbColor {
        let profile = self.color_vision_profile.read().await;
        
        match profile.profile_type {
            ColorVisionType::Normal => color,
            ColorVisionType::Protanopia => self.adjust_for_protanopia(color),
            ColorVisionType::Deuteranopia => self.adjust_for_deuteranopia(color),
            ColorVisionType::Tritanopia => self.adjust_for_tritanopia(color),
            ColorVisionType::Protanomaly => self.adjust_for_protanomaly(color),
            ColorVisionType::Deuteranomaly => self.adjust_for_deuteranomaly(color),
            ColorVisionType::Tritanomaly => self.adjust_for_tritanomaly(color),
            ColorVisionType::Achromatopsia => self.adjust_for_achromatopsia(color),
            ColorVisionType::Achromatomaly => self.adjust_for_achromatomaly(color),
        }
    }
    
    /// Check if color contrast meets WCAG 2.1 AA standards
    pub async fn check_contrast_ratio(&self, foreground: RgbColor, background: RgbColor) -> ContrastResult {
        let ratio = self.calculate_contrast_ratio(foreground, background);
        let settings = self.contrast_settings.read().await;
        
        ContrastResult {
            ratio,
            meets_aa_normal: ratio >= 4.5,
            meets_aa_large: ratio >= 3.0,
            meets_aaa_normal: ratio >= 7.0,
            meets_aaa_large: ratio >= 4.5,
            meets_requirements: ratio >= settings.minimum_ratio,
            recommendation: self.get_contrast_recommendation(ratio),
        }
    }
    
    /// Enhance color contrast to meet accessibility standards
    pub async fn enhance_contrast(&self, foreground: RgbColor, background: RgbColor) -> (RgbColor, RgbColor) {
        let current_ratio = self.calculate_contrast_ratio(foreground, background);
        let settings = self.contrast_settings.read().await;
        
        if current_ratio >= settings.minimum_ratio {
            return (foreground, background);
        }
        
        // Enhance contrast by adjusting foreground color
        let enhanced_foreground = self.adjust_foreground_for_contrast(foreground, background, settings.minimum_ratio);
        
        // If still not enough, adjust background as well
        let final_ratio = self.calculate_contrast_ratio(enhanced_foreground, background);
        if final_ratio >= settings.minimum_ratio {
            (enhanced_foreground, background)
        } else {
            let enhanced_background = self.adjust_background_for_contrast(enhanced_foreground, background, settings.minimum_ratio);
            (enhanced_foreground, enhanced_background)
        }
    }
    
    /// Generate screen reader description for UI elements
    pub async fn generate_screen_reader_description(&self, element: &AccessibleElement) -> String {
        let settings = self.screen_reader_settings.read().await;
        
        let mut description = Vec::new();
        
        // Element type
        description.push(element.element_type.to_string());
        
        // Label or text content
        if !element.label.is_empty() {
            description.push(element.label.clone());
        } else if !element.text_content.is_empty() {
            description.push(element.text_content.clone());
        }
        
        // State information
        if let Some(ref state) = element.state {
            description.push(state.to_description());
        }
        
        // Position information if enabled
        if settings.announce_position && element.position.is_some() {
            let pos = element.position.unwrap();
            description.push(format!("at position {} of {}", pos.current, pos.total));
        }
        
        // Additional context
        if !element.context.is_empty() {
            description.push(element.context.clone());
        }
        
        description.join(", ")
    }
    
    /// Create accessible text with proper styling
    pub async fn create_accessible_text(&self, text: &str, style_type: AccessibleStyleType) -> AccessibleText {
        let profile = self.color_vision_profile.read().await;
        let contrast_settings = self.contrast_settings.read().await;
        let visual_settings = self.visual_indicators.read().await;
        
        let base_style = self.get_base_style_for_type(style_type);
        let adjusted_colors = self.adjust_colors_for_accessibility(base_style, &profile, &contrast_settings).await;
        
        AccessibleText {
            content: text.to_string(),
            style: adjusted_colors,
            semantic_type: style_type,
            screen_reader_text: self.generate_screen_reader_text(text, style_type).await,
            high_contrast_alternative: self.create_high_contrast_style(adjusted_colors).await,
        }
    }
    
    /// Enable TTY blind mode for screen reader users
    pub async fn enable_blind_mode(&self) -> Result<()> {
        info!("Enabling TTY blind mode");
        
        let mut settings = self.screen_reader_settings.write().await;
        settings.blind_mode_enabled = true;
        settings.announce_all_changes = true;
        settings.verbose_descriptions = true;
        
        // Disable visual-only features
        let mut visual_settings = self.visual_indicators.write().await;
        visual_settings.use_colors = false;
        visual_settings.use_icons = false;
        visual_settings.force_text_indicators = true;
        
        info!("TTY blind mode enabled");
        Ok(())
    }
    
    /// Disable TTY blind mode
    pub async fn disable_blind_mode(&self) -> Result<()> {
        info!("Disabling TTY blind mode");
        
        let mut settings = self.screen_reader_settings.write().await;
        settings.blind_mode_enabled = false;
        
        // Re-enable visual features
        let mut visual_settings = self.visual_indicators.write().await;
        visual_settings.use_colors = true;
        visual_settings.use_icons = true;
        visual_settings.force_text_indicators = false;
        
        info!("TTY blind mode disabled");
        Ok(())
    }
    
    /// Get keyboard navigation hints for current context
    pub async fn get_keyboard_navigation_hints(&self, context: NavigationContext) -> Vec<KeyboardHint> {
        let settings = self.keyboard_navigation.read().await;
        
        let mut hints = Vec::new();
        
        match context {
            NavigationContext::MainShell => {
                hints.push(KeyboardHint::new("Tab", "Navigate between elements"));
                hints.push(KeyboardHint::new("Enter", "Execute command"));
                hints.push(KeyboardHint::new("Ctrl+C", "Cancel current operation"));
                hints.push(KeyboardHint::new("Ctrl+L", "Clear screen"));
                hints.push(KeyboardHint::new("F1", "Show help"));
            },
            NavigationContext::Menu => {
                hints.push(KeyboardHint::new("Arrow Keys", "Navigate menu items"));
                hints.push(KeyboardHint::new("Enter", "Select item"));
                hints.push(KeyboardHint::new("Esc", "Close menu"));
                hints.push(KeyboardHint::new("Space", "Toggle selection"));
            },
            NavigationContext::Dialog => {
                hints.push(KeyboardHint::new("Tab", "Move to next control"));
                hints.push(KeyboardHint::new("Shift+Tab", "Move to previous control"));
                hints.push(KeyboardHint::new("Enter", "Confirm"));
                hints.push(KeyboardHint::new("Esc", "Cancel"));
            },
            NavigationContext::List => {
                hints.push(KeyboardHint::new("Arrow Keys", "Navigate list items"));
                hints.push(KeyboardHint::new("Home", "Go to first item"));
                hints.push(KeyboardHint::new("End", "Go to last item"));
                hints.push(KeyboardHint::new("Page Up/Down", "Scroll by page"));
            },
        }
        
        // Add custom shortcuts if enabled
        if settings.show_shortcuts {
            hints.extend(settings.custom_shortcuts.clone());
        }
        
        hints
    }
    
    /// Announce text to screen reader
    pub async fn announce_to_screen_reader(&self, text: &str, priority: AnnouncementPriority) -> Result<()> {
        let settings = self.screen_reader_settings.read().await;
        
        if !settings.enabled {
            return Ok(());
        }
        
        debug!("Announcing to screen reader: {} (priority: {:?})", text, priority);
        
        // In a real implementation, this would interface with system screen readers
        // For now, we'll log the announcement
        match priority {
            AnnouncementPriority::Low => {
                if settings.announce_all_changes {
                    self.send_to_screen_reader(text).await?;
                }
            },
            AnnouncementPriority::Medium => {
                self.send_to_screen_reader(text).await?;
            },
            AnnouncementPriority::High => {
                self.send_to_screen_reader(&format!("Important: {}", text)).await?;
            },
            AnnouncementPriority::Critical => {
                self.send_to_screen_reader(&format!("Alert: {}", text)).await?;
            },
        }
        
        Ok(())
    }
    
    /// Get accessibility status and statistics
    pub async fn get_accessibility_status(&self) -> AccessibilityStatus {
        let color_profile = self.color_vision_profile.read().await;
        let contrast_settings = self.contrast_settings.read().await;
        let screen_reader_settings = self.screen_reader_settings.read().await;
        let keyboard_settings = self.keyboard_navigation.read().await;
        let visual_settings = self.visual_indicators.read().await;
        
        AccessibilityStatus {
            color_vision_profile: color_profile.profile_type.clone(),
            high_contrast_enabled: contrast_settings.high_contrast_mode,
            screen_reader_enabled: screen_reader_settings.enabled,
            blind_mode_enabled: screen_reader_settings.blind_mode_enabled,
            keyboard_navigation_enabled: keyboard_settings.enabled,
            minimum_contrast_ratio: contrast_settings.minimum_ratio,
            font_scaling: visual_settings.font_scaling,
            motion_reduced: visual_settings.reduce_motion,
        }
    }
    
    // Private helper methods
    
    async fn detect_system_settings(&self) -> Result<()> {
        // Detect system accessibility preferences
        // This would interface with system APIs in a real implementation
        
        // Check for high contrast mode
        if let Ok(high_contrast) = std::env::var("HIGH_CONTRAST") {
            if high_contrast == "1" || high_contrast.to_lowercase() == "true" {
                let mut settings = self.contrast_settings.write().await;
                settings.high_contrast_mode = true;
                settings.minimum_ratio = 7.0; // AAA standard
            }
        }
        
        // Check for screen reader
        if let Ok(screen_reader) = std::env::var("SCREEN_READER") {
            if screen_reader == "1" || screen_reader.to_lowercase() == "true" {
                let mut settings = self.screen_reader_settings.write().await;
                settings.enabled = true;
            }
        }
        
        // Check for reduced motion
        if let Ok(reduce_motion) = std::env::var("REDUCE_MOTION") {
            if reduce_motion == "1" || reduce_motion.to_lowercase() == "true" {
                let mut settings = self.visual_indicators.write().await;
                settings.reduce_motion = true;
            }
        }
        
        Ok(())
    }
    
    async fn initialize_screen_reader_support(&self) -> Result<()> {
        let settings = self.screen_reader_settings.read().await;
        
        if settings.enabled {
            info!("Initializing screen reader support");
            
            // In a real implementation, this would:
            // - Connect to system screen reader APIs
            // - Set up accessibility event handlers
            // - Configure text-to-speech if needed
            
            debug!("Screen reader support initialized");
        }
        
        Ok(())
    }
    
    async fn setup_keyboard_navigation(&self) -> Result<()> {
        let mut settings = self.keyboard_navigation.write().await;
        settings.enabled = true;
        
        // Setup default keyboard shortcuts
        settings.custom_shortcuts = vec![
            KeyboardHint::new("Alt+H", "Show accessibility help"),
            KeyboardHint::new("Alt+C", "Toggle high contrast"),
            KeyboardHint::new("Alt+S", "Toggle screen reader announcements"),
        ];
        
        Ok(())
    }
    
    // Color vision adjustment methods
    
    fn adjust_for_protanopia(&self, color: RgbColor) -> RgbColor {
        // Protanopia: Missing L-cones (red-blind)
        // Simulate by removing red component and adjusting green/blue
        RgbColor {
            r: (color.g as f32 * 0.567 + color.b as f32 * 0.433) as u8,
            g: (color.g as f32 * 0.558 + color.b as f32 * 0.442) as u8,
            b: color.b,
        }
    }
    
    fn adjust_for_deuteranopia(&self, color: RgbColor) -> RgbColor {
        // Deuteranopia: Missing M-cones (green-blind)
        RgbColor {
            r: (color.r as f32 * 0.625 + color.g as f32 * 0.375) as u8,
            g: (color.r as f32 * 0.7 + color.g as f32 * 0.3) as u8,
            b: color.b,
        }
    }
    
    fn adjust_for_tritanopia(&self, color: RgbColor) -> RgbColor {
        // Tritanopia: Missing S-cones (blue-blind)
        RgbColor {
            r: color.r,
            g: (color.g as f32 * 0.95 + color.b as f32 * 0.05) as u8,
            b: (color.g as f32 * 0.433 + color.b as f32 * 0.567) as u8,
        }
    }
    
    fn adjust_for_protanomaly(&self, color: RgbColor) -> RgbColor {
        // Protanomaly: Shifted L-cone response
        let factor = 0.8;
        RgbColor {
            r: (color.r as f32 * factor + color.g as f32 * (1.0 - factor)) as u8,
            g: color.g,
            b: color.b,
        }
    }
    
    fn adjust_for_deuteranomaly(&self, color: RgbColor) -> RgbColor {
        // Deuteranomaly: Shifted M-cone response
        let factor = 0.8;
        RgbColor {
            r: color.r,
            g: (color.g as f32 * factor + color.r as f32 * (1.0 - factor)) as u8,
            b: color.b,
        }
    }
    
    fn adjust_for_tritanomaly(&self, color: RgbColor) -> RgbColor {
        // Tritanomaly: Shifted S-cone response
        let factor = 0.8;
        RgbColor {
            r: color.r,
            g: color.g,
            b: (color.b as f32 * factor + color.g as f32 * (1.0 - factor)) as u8,
        }
    }
    
    fn adjust_for_achromatopsia(&self, color: RgbColor) -> RgbColor {
        // Achromatopsia: Complete color blindness (monochrome vision)
        let luminance = (0.299 * color.r as f32 + 0.587 * color.g as f32 + 0.114 * color.b as f32) as u8;
        RgbColor {
            r: luminance,
            g: luminance,
            b: luminance,
        }
    }
    
    fn adjust_for_achromatomaly(&self, color: RgbColor) -> RgbColor {
        // Achromatomaly: Partial color blindness
        let luminance = (0.299 * color.r as f32 + 0.587 * color.g as f32 + 0.114 * color.b as f32) as u8;
        let factor = 0.5; // Blend with original color
        RgbColor {
            r: (color.r as f32 * factor + luminance as f32 * (1.0 - factor)) as u8,
            g: (color.g as f32 * factor + luminance as f32 * (1.0 - factor)) as u8,
            b: (color.b as f32 * factor + luminance as f32 * (1.0 - factor)) as u8,
        }
    }
    
    // Contrast calculation and enhancement methods
    
    fn calculate_contrast_ratio(&self, foreground: RgbColor, background: RgbColor) -> f64 {
        let fg_luminance = self.calculate_relative_luminance(foreground);
        let bg_luminance = self.calculate_relative_luminance(background);
        
        let lighter = fg_luminance.max(bg_luminance);
        let darker = fg_luminance.min(bg_luminance);
        
        (lighter + 0.05) / (darker + 0.05)
    }
    
    fn calculate_relative_luminance(&self, color: RgbColor) -> f64 {
        let r = self.linearize_rgb_component(color.r as f64 / 255.0);
        let g = self.linearize_rgb_component(color.g as f64 / 255.0);
        let b = self.linearize_rgb_component(color.b as f64 / 255.0);
        
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
    
    fn linearize_rgb_component(&self, component: f64) -> f64 {
        if component <= 0.03928 {
            component / 12.92
        } else {
            ((component + 0.055) / 1.055).powf(2.4)
        }
    }
    
    fn adjust_foreground_for_contrast(&self, foreground: RgbColor, background: RgbColor, target_ratio: f64) -> RgbColor {
        let bg_luminance = self.calculate_relative_luminance(background);
        
        // Try making foreground darker or lighter
        let mut best_color = foreground;
        let mut best_ratio = self.calculate_contrast_ratio(foreground, background);
        
        // Try darker
        for factor in (0..=100).step_by(5) {
            let factor = factor as f64 / 100.0;
            let test_color = RgbColor {
                r: (foreground.r as f64 * factor) as u8,
                g: (foreground.g as f64 * factor) as u8,
                b: (foreground.b as f64 * factor) as u8,
            };
            
            let ratio = self.calculate_contrast_ratio(test_color, background);
            if ratio >= target_ratio && ratio > best_ratio {
                best_color = test_color;
                best_ratio = ratio;
            }
        }
        
        // Try lighter
        for factor in (100..=200).step_by(5) {
            let factor = factor as f64 / 100.0;
            let test_color = RgbColor {
                r: ((foreground.r as f64 * factor).min(255.0)) as u8,
                g: ((foreground.g as f64 * factor).min(255.0)) as u8,
                b: ((foreground.b as f64 * factor).min(255.0)) as u8,
            };
            
            let ratio = self.calculate_contrast_ratio(test_color, background);
            if ratio >= target_ratio && ratio > best_ratio {
                best_color = test_color;
                best_ratio = ratio;
            }
        }
        
        best_color
    }
    
    fn adjust_background_for_contrast(&self, foreground: RgbColor, background: RgbColor, target_ratio: f64) -> RgbColor {
        // Similar to adjust_foreground_for_contrast but for background
        let mut best_color = background;
        let mut best_ratio = self.calculate_contrast_ratio(foreground, background);
        
        // Try darker background
        for factor in (0..=100).step_by(5) {
            let factor = factor as f64 / 100.0;
            let test_color = RgbColor {
                r: (background.r as f64 * factor) as u8,
                g: (background.g as f64 * factor) as u8,
                b: (background.b as f64 * factor) as u8,
            };
            
            let ratio = self.calculate_contrast_ratio(foreground, test_color);
            if ratio >= target_ratio && ratio > best_ratio {
                best_color = test_color;
                best_ratio = ratio;
            }
        }
        
        best_color
    }
    
    fn get_contrast_recommendation(&self, ratio: f64) -> ContrastRecommendation {
        if ratio >= 7.0 {
            ContrastRecommendation::Excellent
        } else if ratio >= 4.5 {
            ContrastRecommendation::Good
        } else if ratio >= 3.0 {
            ContrastRecommendation::Acceptable
        } else {
            ContrastRecommendation::Poor
        }
    }
    
    fn get_base_style_for_type(&self, style_type: AccessibleStyleType) -> AccessibleStyle {
        match style_type {
            AccessibleStyleType::Normal => AccessibleStyle {
                foreground: RgbColor { r: 255, g: 255, b: 255 },
                background: RgbColor { r: 0, g: 0, b: 0 },
                bold: false,
                italic: false,
                underline: false,
            },
            AccessibleStyleType::Heading => AccessibleStyle {
                foreground: RgbColor { r: 255, g: 255, b: 255 },
                background: RgbColor { r: 0, g: 0, b: 0 },
                bold: true,
                italic: false,
                underline: false,
            },
            AccessibleStyleType::Error => AccessibleStyle {
                foreground: RgbColor { r: 255, g: 100, b: 100 },
                background: RgbColor { r: 0, g: 0, b: 0 },
                bold: true,
                italic: false,
                underline: false,
            },
            AccessibleStyleType::Warning => AccessibleStyle {
                foreground: RgbColor { r: 255, g: 255, b: 100 },
                background: RgbColor { r: 0, g: 0, b: 0 },
                bold: false,
                italic: false,
                underline: false,
            },
            AccessibleStyleType::Success => AccessibleStyle {
                foreground: RgbColor { r: 100, g: 255, b: 100 },
                background: RgbColor { r: 0, g: 0, b: 0 },
                bold: false,
                italic: false,
                underline: false,
            },
            AccessibleStyleType::Info => AccessibleStyle {
                foreground: RgbColor { r: 100, g: 200, b: 255 },
                background: RgbColor { r: 0, g: 0, b: 0 },
                bold: false,
                italic: false,
                underline: false,
            },
            AccessibleStyleType::Link => AccessibleStyle {
                foreground: RgbColor { r: 100, g: 150, b: 255 },
                background: RgbColor { r: 0, g: 0, b: 0 },
                bold: false,
                italic: false,
                underline: true,
            },
            AccessibleStyleType::Disabled => AccessibleStyle {
                foreground: RgbColor { r: 128, g: 128, b: 128 },
                background: RgbColor { r: 0, g: 0, b: 0 },
                bold: false,
                italic: true,
                underline: false,
            },
        }
    }
    
    async fn adjust_colors_for_accessibility(
        &self,
        style: AccessibleStyle,
        profile: &ColorVisionProfile,
        contrast_settings: &ContrastSettings,
    ) -> AccessibleStyle {
        let adjusted_fg = self.adjust_color_for_vision_sync(style.foreground, &profile.profile_type);
        let adjusted_bg = self.adjust_color_for_vision_sync(style.background, &profile.profile_type);
        
        let (final_fg, final_bg) = if contrast_settings.enforce_minimum_contrast {
            self.enhance_contrast_sync(adjusted_fg, adjusted_bg, contrast_settings.minimum_ratio)
        } else {
            (adjusted_fg, adjusted_bg)
        };
        
        AccessibleStyle {
            foreground: final_fg,
            background: final_bg,
            ..style
        }
    }
    
    fn adjust_color_for_vision_sync(&self, color: RgbColor, vision_type: &ColorVisionType) -> RgbColor {
        match vision_type {
            ColorVisionType::Normal => color,
            ColorVisionType::Protanopia => self.adjust_for_protanopia(color),
            ColorVisionType::Deuteranopia => self.adjust_for_deuteranopia(color),
            ColorVisionType::Tritanopia => self.adjust_for_tritanopia(color),
            ColorVisionType::Protanomaly => self.adjust_for_protanomaly(color),
            ColorVisionType::Deuteranomaly => self.adjust_for_deuteranomaly(color),
            ColorVisionType::Tritanomaly => self.adjust_for_tritanomaly(color),
            ColorVisionType::Achromatopsia => self.adjust_for_achromatopsia(color),
            ColorVisionType::Achromatomaly => self.adjust_for_achromatomaly(color),
        }
    }
    
    fn enhance_contrast_sync(&self, foreground: RgbColor, background: RgbColor, target_ratio: f64) -> (RgbColor, RgbColor) {
        let current_ratio = self.calculate_contrast_ratio(foreground, background);
        
        if current_ratio >= target_ratio {
            return (foreground, background);
        }
        
        let enhanced_foreground = self.adjust_foreground_for_contrast(foreground, background, target_ratio);
        (enhanced_foreground, background)
    }
    
    async fn generate_screen_reader_text(&self, text: &str, style_type: AccessibleStyleType) -> String {
        let type_description = match style_type {
            AccessibleStyleType::Normal => "",
            AccessibleStyleType::Heading => "heading, ",
            AccessibleStyleType::Error => "error, ",
            AccessibleStyleType::Warning => "warning, ",
            AccessibleStyleType::Success => "success, ",
            AccessibleStyleType::Info => "info, ",
            AccessibleStyleType::Link => "link, ",
            AccessibleStyleType::Disabled => "disabled, ",
        };
        
        format!("{}{}", type_description, text)
    }
    
    async fn create_high_contrast_style(&self, style: AccessibleStyle) -> AccessibleStyle {
        AccessibleStyle {
            foreground: if self.calculate_relative_luminance(style.foreground) > 0.5 {
                RgbColor { r: 255, g: 255, b: 255 }
            } else {
                RgbColor { r: 0, g: 0, b: 0 }
            },
            background: if self.calculate_relative_luminance(style.background) > 0.5 {
                RgbColor { r: 255, g: 255, b: 255 }
            } else {
                RgbColor { r: 0, g: 0, b: 0 }
            },
            ..style
        }
    }
    
    async fn send_to_screen_reader(&self, text: &str) -> Result<()> {
        // In a real implementation, this would send to system screen reader APIs
        debug!("Screen reader: {}", text);
        Ok(())
    }
}

/// Color vision profile for accessibility
#[derive(Debug, Clone)]
pub struct ColorVisionProfile {
    pub profile_type: ColorVisionType,
    pub severity: f64, // 0.0 to 1.0
    pub custom_adjustments: HashMap<String, f64>,
}

impl Default for ColorVisionProfile {
    fn default() -> Self {
        Self {
            profile_type: ColorVisionType::Normal,
            severity: 1.0,
            custom_adjustments: HashMap::new(),
        }
    }
}

/// Types of color vision differences
#[derive(Debug, Clone, PartialEq)]
pub enum ColorVisionType {
    Normal,
    Protanopia,     // Red-blind
    Deuteranopia,   // Green-blind
    Tritanopia,     // Blue-blind
    Protanomaly,    // Red-weak
    Deuteranomaly,  // Green-weak
    Tritanomaly,    // Blue-weak
    Achromatopsia,  // Complete color blindness
    Achromatomaly,  // Partial color blindness
}

/// Contrast settings for WCAG compliance
#[derive(Debug, Clone)]
pub struct ContrastSettings {
    pub minimum_ratio: f64,
    pub high_contrast_mode: bool,
    pub enforce_minimum_contrast: bool,
    pub auto_adjust_colors: bool,
}

impl Default for ContrastSettings {
    fn default() -> Self {
        Self {
            minimum_ratio: 4.5, // WCAG AA standard
            high_contrast_mode: false,
            enforce_minimum_contrast: true,
            auto_adjust_colors: true,
        }
    }
}

/// Screen reader settings
#[derive(Debug, Clone)]
pub struct ScreenReaderSettings {
    pub enabled: bool,
    pub blind_mode_enabled: bool,
    pub announce_all_changes: bool,
    pub announce_position: bool,
    pub verbose_descriptions: bool,
    pub speech_rate: f64,
    pub speech_volume: f64,
}

impl Default for ScreenReaderSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            blind_mode_enabled: false,
            announce_all_changes: false,
            announce_position: true,
            verbose_descriptions: false,
            speech_rate: 1.0,
            speech_volume: 0.8,
        }
    }
}

/// Keyboard navigation settings
#[derive(Debug, Clone)]
pub struct KeyboardNavigationSettings {
    pub enabled: bool,
    pub show_shortcuts: bool,
    pub custom_shortcuts: Vec<KeyboardHint>,
    pub focus_indicators: bool,
    pub skip_links: bool,
}

impl Default for KeyboardNavigationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            show_shortcuts: true,
            custom_shortcuts: Vec::new(),
            focus_indicators: true,
            skip_links: true,
        }
    }
}

/// Visual indicator settings
#[derive(Debug, Clone)]
pub struct VisualIndicatorSettings {
    pub use_colors: bool,
    pub use_icons: bool,
    pub force_text_indicators: bool,
    pub font_scaling: f64,
    pub reduce_motion: bool,
    pub high_contrast_borders: bool,
}

impl Default for VisualIndicatorSettings {
    fn default() -> Self {
        Self {
            use_colors: true,
            use_icons: true,
            force_text_indicators: false,
            font_scaling: 1.0,
            reduce_motion: false,
            high_contrast_borders: false,
        }
    }
}

/// Accessible element for screen reader support
#[derive(Debug, Clone)]
pub struct AccessibleElement {
    pub element_type: ElementType,
    pub label: String,
    pub text_content: String,
    pub state: Option<ElementState>,
    pub position: Option<Position>,
    pub context: String,
}

/// UI element types
#[derive(Debug, Clone)]
pub enum ElementType {
    Button,
    TextInput,
    Label,
    Menu,
    MenuItem,
    Dialog,
    List,
    ListItem,
    Heading,
    Link,
    Image,
    Table,
    Cell,
}

impl std::fmt::Display for ElementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementType::Button => write!(f, "button"),
            ElementType::TextInput => write!(f, "text input"),
            ElementType::Label => write!(f, "label"),
            ElementType::Menu => write!(f, "menu"),
            ElementType::MenuItem => write!(f, "menu item"),
            ElementType::Dialog => write!(f, "dialog"),
            ElementType::List => write!(f, "list"),
            ElementType::ListItem => write!(f, "list item"),
            ElementType::Heading => write!(f, "heading"),
            ElementType::Link => write!(f, "link"),
            ElementType::Image => write!(f, "image"),
            ElementType::Table => write!(f, "table"),
            ElementType::Cell => write!(f, "cell"),
        }
    }
}

/// Element state information
#[derive(Debug, Clone)]
pub enum ElementState {
    Normal,
    Focused,
    Selected,
    Disabled,
    Expanded,
    Collapsed,
    Checked,
    Unchecked,
}

impl ElementState {
    pub fn to_description(&self) -> String {
        match self {
            ElementState::Normal => "".to_string(),
            ElementState::Focused => "focused".to_string(),
            ElementState::Selected => "selected".to_string(),
            ElementState::Disabled => "disabled".to_string(),
            ElementState::Expanded => "expanded".to_string(),
            ElementState::Collapsed => "collapsed".to_string(),
            ElementState::Checked => "checked".to_string(),
            ElementState::Unchecked => "unchecked".to_string(),
        }
    }
}

/// Position information for elements
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub current: usize,
    pub total: usize,
}

/// Contrast check result
#[derive(Debug, Clone)]
pub struct ContrastResult {
    pub ratio: f64,
    pub meets_aa_normal: bool,
    pub meets_aa_large: bool,
    pub meets_aaa_normal: bool,
    pub meets_aaa_large: bool,
    pub meets_requirements: bool,
    pub recommendation: ContrastRecommendation,
}

/// Contrast recommendation levels
#[derive(Debug, Clone, PartialEq)]
pub enum ContrastRecommendation {
    Excellent,
    Good,
    Acceptable,
    Poor,
}

/// Accessible text with styling and alternatives
#[derive(Debug, Clone)]
pub struct AccessibleText {
    pub content: String,
    pub style: AccessibleStyle,
    pub semantic_type: AccessibleStyleType,
    pub screen_reader_text: String,
    pub high_contrast_alternative: AccessibleStyle,
}

/// Accessible style definition
#[derive(Debug, Clone)]
pub struct AccessibleStyle {
    pub foreground: RgbColor,
    pub background: RgbColor,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

/// Semantic style types for accessibility
#[derive(Debug, Clone)]
pub enum AccessibleStyleType {
    Normal,
    Heading,
    Error,
    Warning,
    Success,
    Info,
    Link,
    Disabled,
}

/// Navigation context for keyboard hints
#[derive(Debug, Clone)]
pub enum NavigationContext {
    MainShell,
    Menu,
    Dialog,
    List,
}

/// Keyboard navigation hint
#[derive(Debug, Clone)]
pub struct KeyboardHint {
    pub key_combination: String,
    pub description: String,
}

impl KeyboardHint {
    pub fn new(keys: &str, description: &str) -> Self {
        Self {
            key_combination: keys.to_string(),
            description: description.to_string(),
        }
    }
}

/// Announcement priority for screen readers
#[derive(Debug, Clone)]
pub enum AnnouncementPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Overall accessibility status
#[derive(Debug, Clone)]
pub struct AccessibilityStatus {
    pub color_vision_profile: ColorVisionType,
    pub high_contrast_enabled: bool,
    pub screen_reader_enabled: bool,
    pub blind_mode_enabled: bool,
    pub keyboard_navigation_enabled: bool,
    pub minimum_contrast_ratio: f64,
    pub font_scaling: f64,
    pub motion_reduced: bool,
}

/// Accessibility configuration
#[derive(Debug, Clone)]
pub struct AccessibilityConfig {
    pub auto_detect_system_settings: bool,
    pub default_contrast_ratio: f64,
    pub enable_color_vision_simulation: bool,
    pub verbose_screen_reader: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            auto_detect_system_settings: true,
            default_contrast_ratio: 4.5,
            enable_color_vision_simulation: true,
            verbose_screen_reader: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_accessibility_manager_creation() {
        let manager = AccessibilityManager::new().unwrap();
        let status = manager.get_accessibility_status().await;
        assert_eq!(status.color_vision_profile, ColorVisionType::Normal);
    }
    
    #[tokio::test]
    async fn test_contrast_ratio_calculation() {
        let manager = AccessibilityManager::new().unwrap();
        
        // Black on white should have high contrast
        let black = RgbColor { r: 0, g: 0, b: 0 };
        let white = RgbColor { r: 255, g: 255, b: 255 };
        let result = manager.check_contrast_ratio(black, white).await;
        
        assert!(result.ratio > 20.0);
        assert!(result.meets_aa_normal);
        assert!(result.meets_aaa_normal);
    }
    
    #[tokio::test]
    async fn test_color_vision_adjustment() {
        let manager = AccessibilityManager::new().unwrap();
        
        // Set protanopia profile
        let profile = ColorVisionProfile {
            profile_type: ColorVisionType::Protanopia,
            severity: 1.0,
            custom_adjustments: HashMap::new(),
        };
        manager.set_color_vision_profile(profile).await.unwrap();
        
        // Test color adjustment
        let red = RgbColor { r: 255, g: 0, b: 0 };
        let adjusted = manager.adjust_color_for_vision(red).await;
        
        // Red should be significantly reduced for protanopia
        assert!(adjusted.r < red.r);
    }
    
    #[tokio::test]
    async fn test_screen_reader_description() {
        let manager = AccessibilityManager::new().unwrap();
        
        let element = AccessibleElement {
            element_type: ElementType::Button,
            label: "Submit".to_string(),
            text_content: "".to_string(),
            state: Some(ElementState::Focused),
            position: Some(Position { current: 1, total: 3 }),
            context: "in form".to_string(),
        };
        
        let description = manager.generate_screen_reader_description(&element).await;
        assert!(description.contains("button"));
        assert!(description.contains("Submit"));
        assert!(description.contains("focused"));
    }
    
    #[tokio::test]
    async fn test_accessible_text_creation() {
        let manager = AccessibilityManager::new().unwrap();
        
        let text = manager.create_accessible_text("Error message", AccessibleStyleType::Error).await;
        assert_eq!(text.content, "Error message");
        assert_eq!(text.screen_reader_text, "error, Error message");
    }
    
    #[tokio::test]
    async fn test_keyboard_navigation_hints() {
        let manager = AccessibilityManager::new().unwrap();
        
        let hints = manager.get_keyboard_navigation_hints(NavigationContext::MainShell).await;
        assert!(!hints.is_empty());
        assert!(hints.iter().any(|h| h.key_combination.contains("Tab")));
    }
    
    #[test]
    fn test_color_vision_types() {
        let normal = ColorVisionType::Normal;
        let protanopia = ColorVisionType::Protanopia;
        
        assert_ne!(normal, protanopia);
        assert_eq!(normal, ColorVisionType::Normal);
    }
} 