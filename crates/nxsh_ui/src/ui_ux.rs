use anyhow::Result;
use chrono;
use dirs;
use std::{
    collections::HashMap,
    sync::Arc,
    fmt,
};

// 複雑なクロージャ型を簡潔に表現し clippy::type_complexity を解消するための型エイリアス
pub type CompletionHandler = Arc<dyn Fn(&HashMap<String, String>) -> Result<String> + Send + Sync>;

/// Advanced User Interface and Experience system for NexusShell
#[derive(Debug, Clone)]
pub struct UIUXSystem {
    themes: HashMap<String, Theme>,
    current_theme: String,
    layout_manager: LayoutManager,
    interaction_handler: InteractionHandler,
    accessibility: AccessibilityOptions,
    customization: CustomizationSettings,
    animations: AnimationSystem,
}

impl Default for UIUXSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl UIUXSystem {
    pub fn new() -> Self {
        let mut system = Self {
            themes: HashMap::new(),
            current_theme: "default".to_string(),
            layout_manager: LayoutManager::new(),
            interaction_handler: InteractionHandler::new(),
            accessibility: AccessibilityOptions::default(),
            customization: CustomizationSettings::default(),
            animations: AnimationSystem::new(),
        };
        
        system.register_default_themes();
        system
    }

    /// アニメーションが有効か (設定 + アクセシビリティ reduced_motion を考慮)
    pub fn animations_enabled(&self) -> bool {
        self.animations.enabled && !self.accessibility.reduced_motion
    }

    /// Set the current theme
    pub fn set_theme(&mut self, theme_name: &str) -> Result<()> {
        if self.themes.contains_key(theme_name) {
            self.current_theme = theme_name.to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Theme '{}' not found", theme_name))
        }
    }

    /// Get the current theme
    pub fn get_current_theme(&self) -> Option<&Theme> {
        self.themes.get(&self.current_theme)
    }

    /// Register a new theme
    pub fn register_theme(&mut self, name: String, theme: Theme) {
        self.themes.insert(name, theme);
    }

    /// Render the shell prompt
    pub fn render_prompt(&self, context: &PromptContext) -> String {
        let default_theme = Theme::default();
        let theme = self.get_current_theme().unwrap_or(&default_theme);
        let mut prompt = String::new();
        
        // Apply theme colors and styling
        for component in &theme.prompt_components {
            match component {
                PromptComponent::User => {
                    prompt.push_str(&format!("{}{}{}",
                        theme.colors.user_prefix,
                        context.username,
                        theme.colors.reset
                    ));
                },
                PromptComponent::Host => {
                    prompt.push_str(&format!("{}{}{}",
                        theme.colors.host_prefix,
                        context.hostname,
                        theme.colors.reset
                    ));
                },
                PromptComponent::Path => {
                    let display_path = self.format_path(&context.current_path);
                    prompt.push_str(&format!("{}{}{}",
                        theme.colors.path_prefix,
                        display_path,
                        theme.colors.reset
                    ));
                },
                PromptComponent::GitBranch => {
                    if let Some(branch) = &context.git_branch {
                        prompt.push_str(&format!(" {}({}){}",
                            theme.colors.git_prefix,
                            branch,
                            theme.colors.reset
                        ));
                    }
                },
                PromptComponent::Time => {
                    let time = chrono::Local::now().format(&theme.time_format);
                    prompt.push_str(&format!("{}[{}]{}",
                        theme.colors.time_prefix,
                        time,
                        theme.colors.reset
                    ));
                },
                PromptComponent::ExitCode => {
                    if context.last_exit_code != 0 {
                        prompt.push_str(&format!(" {}[{}]{}",
                            theme.colors.error_prefix,
                            context.last_exit_code,
                            theme.colors.reset
                        ));
                    }
                },
                PromptComponent::Symbol => {
                    let symbol = if context.is_admin { &theme.admin_symbol } else { &theme.user_symbol };
                    prompt.push_str(&format!("{}{}{} ",
                        theme.colors.symbol_prefix,
                        symbol,
                        theme.colors.reset
                    ));
                },
                PromptComponent::Custom(text) => {
                    prompt.push_str(text);
                },
            }
        }
        
        prompt
    }

    /// Handle user input with advanced features
    pub fn handle_input(&mut self, input: &str) -> Result<InputResult> {
        // Auto-completion
        if let Some(stripped) = input.strip_suffix('\t') {
            return Ok(InputResult::AutoComplete(self.get_completions(stripped)?));
        }
        
        // Syntax highlighting
        let highlighted = self.apply_syntax_highlighting(input);
        
        // Command validation
        let validation = self.validate_command(input);
        
        Ok(InputResult::Processed {
            original: input.to_string(),
            highlighted,
            validation,
        })
    }

    /// Apply syntax highlighting to command text
    pub fn apply_syntax_highlighting(&self, text: &str) -> String {
        let default_theme = Theme::default();
        let theme = self.get_current_theme().unwrap_or(&default_theme);
        let mut highlighted = String::new();
        let tokens = self.tokenize_command(text);
        
        for token in tokens {
            let color = match token.token_type {
                TokenType::Command => &theme.syntax_colors.command,
                TokenType::Argument => &theme.syntax_colors.argument,
                TokenType::Flag => &theme.syntax_colors.flag,
                TokenType::String => &theme.syntax_colors.string,
                TokenType::Number => &theme.syntax_colors.number,
                TokenType::Operator => &theme.syntax_colors.operator,
                TokenType::Comment => &theme.syntax_colors.comment,
                TokenType::Keyword => &theme.syntax_colors.keyword,
            };
            
            highlighted.push_str(&format!("{}{}{}", color, token.text, theme.colors.reset));
        }
        
        highlighted
    }

    /// Get auto-completion suggestions
    pub fn get_completions(&self, partial: &str) -> Result<Vec<Completion>> {
        let mut completions = Vec::new();
        
        // Command completions
        if !partial.contains(' ') {
            completions.extend(self.get_command_completions(partial));
        } else {
            // Argument completions
            let parts: Vec<&str> = partial.split_whitespace().collect();
            if let Some(command) = parts.first() {
                completions.extend(self.get_argument_completions(command, &parts[1..]));
            }
        }
        
        // File path completions
        completions.extend(self.get_path_completions(partial)?);
        
        // Sort by relevance
        completions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(completions)
    }

    /// Display command help and documentation
    pub fn display_help(&self, command: Option<&str>) -> Result<String> {
        match command {
            Some(cmd) => {
                if let Some(help_info) = self.get_command_help(cmd) {
                    Ok(self.format_help(&help_info))
                } else {
                    Err(anyhow::anyhow!("Help not found for command: {}", cmd))
                }
            },
            None => {
                Ok(self.format_general_help())
            }
        }
    }

    /// Handle interactive command building
    pub fn start_interactive_mode(&mut self, command: &str) -> Result<InteractiveSession> {
        let command_owned = command.to_string();
        let session = InteractiveSession {
            command: command_owned.clone(),
            parameters: HashMap::new(),
            current_step: 0,
            steps: self.get_command_steps(command)?,
            completion_handler: Arc::new(move |params| {
                // Command completion logic
                Ok(format!("Executing: {command_owned} with parameters: {params:?}"))
            }),
        };
        
        Ok(session)
    }

    /// Update layout based on terminal size
    pub fn update_layout(&mut self, width: usize, height: usize) -> Result<()> {
        self.layout_manager.update_dimensions(width, height);
        self.layout_manager.recalculate_layout()?;
        Ok(())
    }

    /// Handle accessibility features
    pub fn configure_accessibility(&mut self, options: AccessibilityOptions) {
        self.accessibility = options;
    }

    /// Apply user customizations
    pub fn apply_customization(&mut self, settings: CustomizationSettings) {
        self.customization = settings;
        self.update_theme_with_customization();
    }

    // Private helper methods

    fn register_default_themes(&mut self) {
        // Dark theme
        let dark_theme = Theme {
            name: "dark".to_string(),
            colors: ThemeColors {
                background: "\x1b[40m".to_string(),
                foreground: "\x1b[37m".to_string(),
                user_prefix: "\x1b[32m".to_string(),
                host_prefix: "\x1b[33m".to_string(),
                path_prefix: "\x1b[34m".to_string(),
                git_prefix: "\x1b[35m".to_string(),
                time_prefix: "\x1b[36m".to_string(),
                error_prefix: "\x1b[31m".to_string(),
                symbol_prefix: "\x1b[37m".to_string(),
                reset: "\x1b[0m".to_string(),
            },
            syntax_colors: SyntaxColors {
                command: "\x1b[96m".to_string(),
                argument: "\x1b[37m".to_string(),
                flag: "\x1b[93m".to_string(),
                string: "\x1b[92m".to_string(),
                number: "\x1b[91m".to_string(),
                operator: "\x1b[95m".to_string(),
                comment: "\x1b[90m".to_string(),
                keyword: "\x1b[94m".to_string(),
            },
            prompt_components: vec![
                PromptComponent::User,
                PromptComponent::Custom("@".to_string()),
                PromptComponent::Host,
                PromptComponent::Custom(":".to_string()),
                PromptComponent::Path,
                PromptComponent::GitBranch,
                PromptComponent::ExitCode,
                PromptComponent::Symbol,
            ],
            user_symbol: "$".to_string(),
            admin_symbol: "#".to_string(),
            time_format: "%H:%M:%S".to_string(),
        };
        
        // Light theme
        let light_theme = Theme {
            name: "light".to_string(),
            colors: ThemeColors {
                background: "\x1b[47m".to_string(),
                foreground: "\x1b[30m".to_string(),
                user_prefix: "\x1b[32m".to_string(),
                host_prefix: "\x1b[33m".to_string(),
                path_prefix: "\x1b[34m".to_string(),
                git_prefix: "\x1b[35m".to_string(),
                time_prefix: "\x1b[36m".to_string(),
                error_prefix: "\x1b[31m".to_string(),
                symbol_prefix: "\x1b[30m".to_string(),
                reset: "\x1b[0m".to_string(),
            },
            syntax_colors: SyntaxColors {
                command: "\x1b[34m".to_string(),
                argument: "\x1b[30m".to_string(),
                flag: "\x1b[33m".to_string(),
                string: "\x1b[32m".to_string(),
                number: "\x1b[31m".to_string(),
                operator: "\x1b[35m".to_string(),
                comment: "\x1b[90m".to_string(),
                keyword: "\x1b[36m".to_string(),
            },
            prompt_components: vec![
                PromptComponent::Time,
                PromptComponent::Custom(" ".to_string()),
                PromptComponent::User,
                PromptComponent::Custom("@".to_string()),
                PromptComponent::Host,
                PromptComponent::Custom(" ".to_string()),
                PromptComponent::Path,
                PromptComponent::GitBranch,
                PromptComponent::Custom("\n".to_string()),
                PromptComponent::Symbol,
            ],
            user_symbol: "→".to_string(),
            admin_symbol: "⚡".to_string(),
            time_format: "%H:%M".to_string(),
        };
        
        self.themes.insert("dark".to_string(), dark_theme);
        self.themes.insert("light".to_string(), light_theme);
    }

    fn format_path(&self, path: &str) -> String {
        // Shorten long paths, handle home directory substitution, etc.
        let home = dirs::home_dir().unwrap_or_default();
        let home_str = home.to_string_lossy();
        
        if path.starts_with(&*home_str) {
            format!("~{}", &path[home_str.len()..])
        } else {
            path.to_string()
        }
    }

    fn tokenize_command(&self, text: &str) -> Vec<Token> {
        // Simple tokenizer - would need proper lexical analysis for production
        let mut tokens = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            let token_type = if i == 0 {
                TokenType::Command
            } else if word.starts_with('-') {
                TokenType::Flag
            } else if word.starts_with('"') || word.starts_with('\'') {
                TokenType::String
            } else if word.parse::<f64>().is_ok() {
                TokenType::Number
            } else if "&&||>><<|&".contains(word) {
                TokenType::Operator
            } else if word.starts_with('#') {
                TokenType::Comment
            } else {
                TokenType::Argument
            };
            
            tokens.push(Token {
                text: word.to_string(),
                token_type,
            });
        }
        
        tokens
    }

    fn get_command_completions(&self, partial: &str) -> Vec<Completion> {
        let commands = [
            "ls", "cd", "pwd", "echo", "cat", "grep", "find", "ps", "kill", "cp", "mv", "rm",
            "mkdir", "rmdir", "touch", "chmod", "chown", "df", "du", "free", "top", "htop",
            "git", "cargo", "npm", "python", "node", "curl", "wget", "ssh", "scp", "tar",
        ];
        
        commands.iter()
            .filter(|cmd| cmd.starts_with(partial))
            .map(|cmd| Completion {
                text: cmd.to_string(),
                display: cmd.to_string(),
                description: format!("Command: {cmd}"),
                completion_type: CompletionType::Command,
                score: 1.0,
            })
            .collect()
    }

    fn get_argument_completions(&self, command: &str, args: &[&str]) -> Vec<Completion> {
        // Command-specific argument completions
        match command {
            "git" => self.get_git_completions(args),
            "cargo" => self.get_cargo_completions(args),
            _ => Vec::new(),
        }
    }

    fn get_git_completions(&self, args: &[&str]) -> Vec<Completion> {
        if args.is_empty() {
            vec![
                "add", "commit", "push", "pull", "clone", "checkout", "branch", "merge", "status", "log"
            ].into_iter().map(|cmd| Completion {
                text: cmd.to_string(),
                display: cmd.to_string(),
                description: format!("Git subcommand: {cmd}"),
                completion_type: CompletionType::Subcommand,
                score: 1.0,
            }).collect()
        } else {
            Vec::new()
        }
    }

    fn get_cargo_completions(&self, args: &[&str]) -> Vec<Completion> {
        if args.is_empty() {
            vec![
                "build", "run", "test", "check", "clean", "doc", "publish", "install", "update"
            ].into_iter().map(|cmd| Completion {
                text: cmd.to_string(),
                display: cmd.to_string(),
                description: format!("Cargo subcommand: {cmd}"),
                completion_type: CompletionType::Subcommand,
                score: 1.0,
            }).collect()
        } else {
            Vec::new()
        }
    }

    fn get_path_completions(&self, partial: &str) -> Result<Vec<Completion>> {
        let mut completions = Vec::new();
        
        // Extract directory path and filename prefix
        let path = std::path::Path::new(partial);
        let filename_prefix = if partial.ends_with('/') {
            String::new()
        } else {
            path.file_name().unwrap_or_default().to_string_lossy().to_string()
        };
        let dir_path = if partial.ends_with('/') {
            path
        } else {
            path.parent().unwrap_or(path)
        };
        
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&filename_prefix) {
                    let is_dir = entry.metadata().map(|m| m.is_dir()).unwrap_or(false);
                    completions.push(Completion {
                        text: name.clone(),
                        display: if is_dir { format!("{name}/") } else { name.clone() },
                        description: if is_dir { "Directory".to_string() } else { "File".to_string() },
                        completion_type: if is_dir { CompletionType::Directory } else { CompletionType::File },
                        score: 0.8,
                    });
                }
            }
        }
        
        Ok(completions)
    }

    pub fn validate_command(&self, command: &str) -> ValidationResult {
        // Basic command validation
        if command.trim().is_empty() {
            return ValidationResult::Empty;
        }
        
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts[0];
        
        // Check if command exists
        if self.command_exists(cmd) {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid(format!("Command '{cmd}' not found"))
        }
    }

    fn command_exists(&self, command: &str) -> bool {
        // Builtin commands (keep in sync with completion module)
        const BUILTINS: &[&str] = &["cd", "ls", "pwd", "echo", "cat", "exit", "help", "history", "alias", "unalias", "bzip2", "bunzip2", "id"];
        if BUILTINS.contains(&command) { return true; }

        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let candidate = dir.join(command);
                if candidate.is_file() { return true; }
                #[cfg(windows)]
                {
                    if let Ok(pathext) = std::env::var("PATHEXT") {
                        for ext in pathext.split(';') {
                            if ext.is_empty() { continue; }
                            let trimmed = ext.trim_start_matches('.');
                            let with_ext = dir.join(format!("{command}.{}", trimmed));
                            if with_ext.is_file() { return true; }
                        }
                    }
                }
            }
        }
        false
    }

    fn get_command_help(&self, command: &str) -> Option<HelpInfo> {
        // Mock help information - would integrate with actual help system
        Some(HelpInfo {
            command: command.to_string(),
            description: format!("Help for command: {command}"),
            usage: format!("{command} [OPTIONS] [ARGS]"),
            options: vec![
                OptionHelp {
                    short: Some("-h".to_string()),
                    long: Some("--help".to_string()),
                    description: "Show help information".to_string(),
                },
            ],
            examples: vec![
                format!("{} --help", command),
            ],
        })
    }

    fn format_help(&self, help: &HelpInfo) -> String {
        let default_theme = Theme::default();
        let theme = self.get_current_theme().unwrap_or(&default_theme);
        let mut formatted = String::new();
        
        formatted.push_str(&format!("{}{}{}{}:\n", 
            theme.colors.user_prefix, help.command, theme.colors.reset, 
            help.description));
        
        formatted.push_str(&format!("\n{}Usage:{} {}\n", 
            theme.syntax_colors.keyword, theme.colors.reset, help.usage));
        
        if !help.options.is_empty() {
            formatted.push_str(&format!("\n{}Options:{}\n", 
                theme.syntax_colors.keyword, theme.colors.reset));
            
            for option in &help.options {
                let short = option.short.as_deref().unwrap_or("");
                let long = option.long.as_deref().unwrap_or("");
                formatted.push_str(&format!("  {}{}{}, {}{}{}\t{}\n",
                    theme.syntax_colors.flag, short, theme.colors.reset,
                    theme.syntax_colors.flag, long, theme.colors.reset,
                    option.description));
            }
        }
        
        if !help.examples.is_empty() {
            formatted.push_str(&format!("\n{}Examples:{}\n", 
                theme.syntax_colors.keyword, theme.colors.reset));
            
            for example in &help.examples {
                formatted.push_str(&format!("  {}{}{}\n",
                    theme.syntax_colors.string, example, theme.colors.reset));
            }
        }
        
        formatted
    }

    fn format_general_help(&self) -> String {
        "NexusShell Help\n\nAvailable commands:\n- Use 'help <command>' for specific help\n- Use Tab for auto-completion\n- Use Ctrl+C to cancel operations".to_string()
    }

    pub fn get_command_steps(&self, command: &str) -> Result<Vec<InteractiveStep>> {
        // Provide different placeholder steps depending on command to activate code paths
        let steps = match command {
            "grep" => vec![
                InteractiveStep { name: "pattern".into(), description: "Pattern to search".into(), parameter_type: ParameterType::String, required: true, default_value: None },
                InteractiveStep { name: "file".into(), description: "File to search".into(), parameter_type: ParameterType::String, required: true, default_value: None },
            ],
            "cp" => vec![
                InteractiveStep { name: "source".into(), description: "Source path".into(), parameter_type: ParameterType::String, required: true, default_value: None },
                InteractiveStep { name: "destination".into(), description: "Destination path".into(), parameter_type: ParameterType::String, required: true, default_value: None },
            ],
            _ => vec![
                InteractiveStep { name: "target".into(), description: "Primary argument".into(), parameter_type: ParameterType::String, required: true, default_value: None }
            ]
        };
        Ok(steps)
    }

    fn update_theme_with_customization(&mut self) {
        // Apply user customizations to current theme
    if let Some(_theme) = self.themes.get_mut(&self.current_theme) {
            // Apply font settings, colors, etc. from customization
        }
    }
}

// Supporting types and structures

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub syntax_colors: SyntaxColors,
    pub prompt_components: Vec<PromptComponent>,
    pub user_symbol: String,
    pub admin_symbol: String,
    pub time_format: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            colors: ThemeColors::default(),
            syntax_colors: SyntaxColors::default(),
            prompt_components: vec![
                PromptComponent::User,
                PromptComponent::Host,
                PromptComponent::Path,
                PromptComponent::Symbol,
            ],
            user_symbol: "$".to_string(),
            admin_symbol: "#".to_string(),
            time_format: "%H:%M:%S".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub background: String,
    pub foreground: String,
    pub user_prefix: String,
    pub host_prefix: String,
    pub path_prefix: String,
    pub git_prefix: String,
    pub time_prefix: String,
    pub error_prefix: String,
    pub symbol_prefix: String,
    pub reset: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            background: "\x1b[40m".to_string(),
            foreground: "\x1b[37m".to_string(),
            user_prefix: "\x1b[32m".to_string(),
            host_prefix: "\x1b[33m".to_string(),
            path_prefix: "\x1b[34m".to_string(),
            git_prefix: "\x1b[35m".to_string(),
            time_prefix: "\x1b[36m".to_string(),
            error_prefix: "\x1b[31m".to_string(),
            symbol_prefix: "\x1b[37m".to_string(),
            reset: "\x1b[0m".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyntaxColors {
    pub command: String,
    pub argument: String,
    pub flag: String,
    pub string: String,
    pub number: String,
    pub operator: String,
    pub comment: String,
    pub keyword: String,
}

impl Default for SyntaxColors {
    fn default() -> Self {
        Self {
            command: "\x1b[96m".to_string(),
            argument: "\x1b[37m".to_string(),
            flag: "\x1b[93m".to_string(),
            string: "\x1b[92m".to_string(),
            number: "\x1b[91m".to_string(),
            operator: "\x1b[95m".to_string(),
            comment: "\x1b[90m".to_string(),
            keyword: "\x1b[94m".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PromptComponent {
    User,
    Host,
    Path,
    GitBranch,
    Time,
    ExitCode,
    Symbol,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct PromptContext {
    pub username: String,
    pub hostname: String,
    pub current_path: String,
    pub git_branch: Option<String>,
    pub last_exit_code: i32,
    pub is_admin: bool,
}

#[derive(Debug, Clone)]
pub struct LayoutManager {
    width: usize,
    height: usize,
    regions: Vec<LayoutRegion>,
}

impl LayoutManager {
    fn new() -> Self {
        Self {
            width: 80,
            height: 24,
            regions: Vec::new(),
        }
    }
    
    fn update_dimensions(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }
    
    fn recalculate_layout(&mut self) -> Result<()> {
        // Layout calculation logic
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LayoutRegion {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub region_type: RegionType,
}

#[derive(Debug, Clone)]
pub enum RegionType {
    Prompt,
    Output,
    StatusBar,
    Sidebar,
}

#[derive(Debug, Clone)]
pub struct InteractionHandler {
    key_bindings: HashMap<String, Action>,
}

impl InteractionHandler {
    fn new() -> Self {
        let mut handler = Self {
            key_bindings: HashMap::new(),
        };
        handler.register_default_bindings();
        handler
    }
    
    fn register_default_bindings(&mut self) {
        self.key_bindings.insert("Ctrl+C".to_string(), Action::Cancel);
        self.key_bindings.insert("Ctrl+D".to_string(), Action::Exit);
        self.key_bindings.insert("Tab".to_string(), Action::AutoComplete);
        self.key_bindings.insert("Up".to_string(), Action::HistoryPrevious);
        self.key_bindings.insert("Down".to_string(), Action::HistoryNext);
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    Cancel,
    Exit,
    AutoComplete,
    HistoryPrevious,
    HistoryNext,
    Custom(String),
}

#[derive(Debug, Clone, Default)]
pub struct AccessibilityOptions {
    pub high_contrast: bool,
    pub large_text: bool,
    pub screen_reader_support: bool,
    pub reduced_motion: bool,
    pub keyboard_only: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CustomizationSettings {
    pub font_family: Option<String>,
    pub font_size: Option<usize>,
    pub line_height: Option<f32>,
    pub cursor_style: Option<CursorStyle>,
    pub custom_css: Option<String>,
}

#[derive(Debug, Clone)]
pub enum CursorStyle {
    Block,
    Line,
    Underline,
}

#[derive(Debug, Clone)]
pub struct AnimationSystem {
    enabled: bool,
    duration_ms: u64,
}

impl AnimationSystem {
    fn new() -> Self {
        Self {
            enabled: true,
            duration_ms: 200,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InputResult {
    AutoComplete(Vec<Completion>),
    Processed {
        original: String,
        highlighted: String,
        validation: ValidationResult,
    },
}

#[derive(Debug, Clone)]
pub struct Completion {
    pub text: String,
    pub display: String,
    pub description: String,
    pub completion_type: CompletionType,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub enum CompletionType {
    Command,
    Subcommand,
    File,
    Directory,
    Variable,
    Function,
}

#[derive(Debug, Clone)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
    Empty,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub token_type: TokenType,
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Command,
    Argument,
    Flag,
    String,
    Number,
    Operator,
    Comment,
    Keyword,
}

pub struct InteractiveSession {
    pub command: String,
    pub parameters: HashMap<String, String>,
    pub current_step: usize,
    pub steps: Vec<InteractiveStep>,
    pub completion_handler: CompletionHandler,
}

impl fmt::Debug for InteractiveSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InteractiveSession")
            .field("command", &self.command)
            .field("parameters", &self.parameters)
            .field("current_step", &self.current_step)
            .field("steps", &self.steps)
            .field("completion_handler", &"<function>")
            .finish()
    }
}

impl Clone for InteractiveSession {
    fn clone(&self) -> Self {
        Self {
            command: self.command.clone(),
            parameters: self.parameters.clone(),
            current_step: self.current_step,
            steps: self.steps.clone(),
            completion_handler: Arc::clone(&self.completion_handler),
        }
    }
}

impl InteractiveSession {
    /// Return the current step if any.
    pub fn current_step(&self) -> Option<&InteractiveStep> {
        self.steps.get(self.current_step)
    }

    /// Set a parameter value by name.
    pub fn set_param(&mut self, name: &str, value: impl Into<String>) -> Result<()> {
        if !self.steps.iter().any(|s| s.name == name) {
            return Err(anyhow::anyhow!(format!("Unknown parameter: {name}")));
        }
        self.parameters.insert(name.to_string(), value.into());
        Ok(())
    }

    /// Check whether all required parameters up to and including the current step are provided.
    pub fn can_advance(&self) -> bool {
        self.steps
            .iter()
            .take(self.current_step + 1)
            .all(|s| !s.required || self.parameters.contains_key(&s.name))
    }

    /// Advance to next step if possible.
    pub fn advance(&mut self) -> Result<()> {
        if !self.can_advance() {
            return Err(anyhow::anyhow!("Cannot advance: missing required parameter(s)"));
        }
        if self.current_step + 1 < self.steps.len() {
            self.current_step += 1;
        }
        Ok(())
    }

    /// Whether all required parameters for all steps are provided.
    pub fn is_complete(&self) -> bool {
        self.steps
            .iter()
            .all(|s| !s.required || self.parameters.contains_key(&s.name))
    }

    /// Complete the interactive session by invoking the completion handler.
    pub fn try_complete(&self) -> Result<String> {
        if !self.is_complete() {
            return Err(anyhow::anyhow!("Cannot complete: missing required parameter(s)"));
        }
        (self.completion_handler)(&self.parameters)
    }
}

#[derive(Debug, Clone)]
pub struct InteractiveStep {
    pub name: String,
    pub description: String,
    pub parameter_type: ParameterType,
    pub required: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    File,
    Directory,
    Choice(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct HelpInfo {
    pub command: String,
    pub description: String,
    pub usage: String,
    pub options: Vec<OptionHelp>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OptionHelp {
    pub short: Option<String>,
    pub long: Option<String>,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_registration() {
        let mut ui_system = UIUXSystem::new();
        
        let custom_theme = Theme {
            name: "custom".to_string(),
            ..Default::default()
        };
        
        ui_system.register_theme("custom".to_string(), custom_theme);
        assert!(ui_system.themes.contains_key("custom"));
    }

    #[test]
    fn test_prompt_rendering() {
        let ui_system = UIUXSystem::new();
        
        let context = PromptContext {
            username: "user".to_string(),
            hostname: "localhost".to_string(),
            current_path: "/home/user".to_string(),
            git_branch: Some("main".to_string()),
            last_exit_code: 0,
            is_admin: false,
        };
        
        let prompt = ui_system.render_prompt(&context);
        assert!(!prompt.is_empty());
        assert!(prompt.contains("user"));
        assert!(prompt.contains("localhost"));
    }

    #[test]
    fn test_syntax_highlighting() {
        let ui_system = UIUXSystem::new();
        
        let highlighted = ui_system.apply_syntax_highlighting("git commit -m \"test\"");
        assert!(!highlighted.is_empty());
        // Would test actual ANSI color codes in real implementation
    }

    #[test]
    fn test_command_validation() {
        let ui_system = UIUXSystem::new();
        
        let result = ui_system.validate_command("echo hello");
        assert!(matches!(result, ValidationResult::Valid));
        
        let result = ui_system.validate_command("");
        assert!(matches!(result, ValidationResult::Empty));
    }

    #[test]
    fn test_completions() {
        let ui_system = UIUXSystem::new();
        
        let completions = ui_system.get_completions("gi").unwrap();
        assert!(completions.iter().any(|c| c.text.starts_with("gi")));
    }
}
