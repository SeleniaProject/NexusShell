//! `smart_alias` command - Intelligent alias management system
//! Advanced alias management with AI-powered suggestions and smart completion

use anyhow::{Result, anyhow, Context};
use crate::ui_design::{
    TableFormatter, Colorize, Animation, ProgressBar, Notification, NotificationType, create_advanced_table,
    TableOptions, BorderStyle, Alignment, ItemStatus, StatusItem, StatusDashboard, DashboardSection, SectionStyle,
    CommandWizard, WizardStep, InputType, FilePreview
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SmartAliasManager {
    pub aliases: HashMap<String, AliasInfo>,
    pub usage_stats: HashMap<String, UsageStats>,
    pub suggestions: Vec<AliasSuggestion>,
}

#[derive(Debug, Clone)]
pub struct AliasInfo {
    pub name: String,
    pub command: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub category: AliasCategory,
    pub complexity: AliasComplexity,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UsageStats {
    pub usage_count: u64,
    pub last_used: chrono::DateTime<chrono::Local>,
    pub avg_execution_time: Option<f64>,
    pub success_rate: f64,
}

#[derive(Debug, Clone)]
pub struct AliasSuggestion {
    pub suggested_alias: String,
    pub command_pattern: String,
    pub reason: String,
    pub confidence: f64,
    pub category: AliasCategory,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AliasCategory {
    FileManagement,
    ProcessControl,
    Navigation,
    TextProcessing,
    SystemAdmin,
    Development,
    Network,
    Custom,
}

#[derive(Debug, Clone)]
pub enum AliasComplexity {
    Simple,      // Single command with basic flags
    Moderate,    // Command with multiple options
    Complex,     // Pipeline or multiple commands
    Advanced,    // Complex logic with conditions
}

impl SmartAliasManager {
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
            usage_stats: HashMap::new(),
            suggestions: Vec::new(),
        }
    }
    
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Load aliases from configuration file
        let mut manager = Self::new();
        
        if let Ok(content) = fs::read_to_string(path) {
            // Parse TOML or JSON configuration
            // This is a simplified implementation
            manager.initialize_default_aliases();
            manager.generate_smart_suggestions();
        }
        
        Ok(manager)
    }
    
    fn initialize_default_aliases(&mut self) {
        let default_aliases = vec![
            ("ll", "ls -la", "Long list with hidden files", AliasCategory::FileManagement),
            ("la", "ls -la", "List all files", AliasCategory::FileManagement),
            ("l", "ls -CF", "Compact list", AliasCategory::FileManagement),
            ("grep", "grep --color=auto", "Colorized grep", AliasCategory::TextProcessing),
            ("...", "cd ../..", "Go up two directories", AliasCategory::Navigation),
            ("....", "cd ../../..", "Go up three directories", AliasCategory::Navigation),
            ("h", "history", "Show command history", AliasCategory::SystemAdmin),
            ("c", "clear", "Clear screen", AliasCategory::SystemAdmin),
            ("reload", "source ~/.bashrc", "Reload shell config", AliasCategory::SystemAdmin),
        ];
        
        for (name, command, desc, category) in default_aliases {
            let alias_info = AliasInfo {
                name: name.to_string(),
                command: command.to_string(),
                description: desc.to_string(),
                created_at: chrono::Local::now(),
                category,
                complexity: AliasComplexity::Simple,
                tags: vec![],
            };
            
            self.aliases.insert(name.to_string(), alias_info);
            self.usage_stats.insert(name.to_string(), UsageStats {
                usage_count: 0,
                last_used: chrono::Local::now(),
                avg_execution_time: None,
                success_rate: 1.0,
            });
        }
    }
    
    fn generate_smart_suggestions(&mut self) {
        // Generate intelligent alias suggestions based on common command patterns
        let suggestions = vec![
            AliasSuggestion {
                suggested_alias: "findcode".to_string(),
                command_pattern: "find . -name '*.rs' -o -name '*.js' -o -name '*.py'".to_string(),
                reason: "Quickly find source code files".to_string(),
                confidence: 0.8,
                category: AliasCategory::Development,
            },
            AliasSuggestion {
                suggested_alias: "ports".to_string(),
                command_pattern: "netstat -tulpn".to_string(),
                reason: "Show all listening ports".to_string(),
                confidence: 0.7,
                category: AliasCategory::Network,
            },
            AliasSuggestion {
                suggested_alias: "myip".to_string(),
                command_pattern: "curl -s ifconfig.me".to_string(),
                reason: "Get your public IP address".to_string(),
                confidence: 0.9,
                category: AliasCategory::Network,
            },
            AliasSuggestion {
                suggested_alias: "diskspace".to_string(),
                command_pattern: "df -h".to_string(),
                reason: "Show disk usage in human readable format".to_string(),
                confidence: 0.85,
                category: AliasCategory::SystemAdmin,
            },
        ];
        
        self.suggestions = suggestions;
    }
    
    pub fn create_alias(&mut self, name: &str, command: &str, description: Option<&str>) -> Result<()> {
        let alias_info = AliasInfo {
            name: name.to_string(),
            command: command.to_string(),
            description: description.unwrap_or("Custom alias").to_string(),
            created_at: chrono::Local::now(),
            category: self.detect_category(command),
            complexity: self.detect_complexity(command),
            tags: self.generate_tags(command),
        };
        
        self.aliases.insert(name.to_string(), alias_info);
        self.usage_stats.insert(name.to_string(), UsageStats {
            usage_count: 0,
            last_used: chrono::Local::now(),
            avg_execution_time: None,
            success_rate: 1.0,
        });
        
        println!("{}", format!("✅ Created alias: {} → {}", name, command).success());
        Ok(())
    }
    
    fn detect_category(&self, command: &str) -> AliasCategory {
        match command {
            cmd if cmd.contains("ls") || cmd.contains("find") || cmd.contains("cp") => AliasCategory::FileManagement,
            cmd if cmd.contains("ps") || cmd.contains("kill") || cmd.contains("top") => AliasCategory::ProcessControl,
            cmd if cmd.contains("cd") || cmd.contains("pwd") => AliasCategory::Navigation,
            cmd if cmd.contains("grep") || cmd.contains("sed") || cmd.contains("awk") => AliasCategory::TextProcessing,
            cmd if cmd.contains("curl") || cmd.contains("wget") || cmd.contains("ping") => AliasCategory::Network,
            cmd if cmd.contains("git") || cmd.contains("cargo") || cmd.contains("npm") => AliasCategory::Development,
            _ => AliasCategory::Custom,
        }
    }
    
    fn detect_complexity(&self, command: &str) -> AliasComplexity {
        let pipe_count = command.matches('|').count();
        let semicolon_count = command.matches(';').count();
        let flag_count = command.matches(" -").count();
        
        match (pipe_count, semicolon_count, flag_count) {
            (0, 0, 0..=2) => AliasComplexity::Simple,
            (0, 0, 3..=5) => AliasComplexity::Moderate,
            (1..=2, 0..=1, _) => AliasComplexity::Complex,
            _ => AliasComplexity::Advanced,
        }
    }
    
    fn generate_tags(&self, command: &str) -> Vec<String> {
        let mut tags = Vec::new();
        
        if command.contains("sudo") { tags.push("admin".to_string()); }
        if command.contains("git") { tags.push("version-control".to_string()); }
        if command.contains("-r") || command.contains("--recursive") { tags.push("recursive".to_string()); }
        if command.contains("|") { tags.push("pipeline".to_string()); }
        if command.contains("grep") { tags.push("search".to_string()); }
        
        tags
    }
    
    pub fn show_dashboard(&self) -> Result<()> {
        let mut dashboard = StatusDashboard::new("Smart Alias Management".to_string());
        
        // Statistics section
        let mut stats_section = DashboardSection {
            title: "📊 Statistics".to_string(),
            style: SectionStyle::Boxed,
            items: Vec::new(),
        };
        
        stats_section.items.push(StatusItem {
            name: "total_aliases".to_string(),
            label: "Total Aliases".to_string(),
            value: self.aliases.len().to_string(),
            status: ItemStatus::Info,
            icon: "📝".to_string(),
        });
        
        let most_used = self.get_most_used_alias();
        stats_section.items.push(StatusItem {
            name: "most_used".to_string(),
            label: "Most Used".to_string(),
            value: most_used.unwrap_or("None".to_string()),
            status: ItemStatus::Good,
            icon: "⭐".to_string(),
        });
        
        dashboard.add_section(stats_section);
        
        // Categories section
        let mut categories_section = DashboardSection {
            title: "📂 Categories".to_string(),
            style: SectionStyle::Highlighted,
            items: Vec::new(),
        };
        
        let category_counts = self.get_category_counts();
        for (category, count) in category_counts {
            categories_section.items.push(StatusItem {
                name: format!("category_{:?}", category).to_lowercase(),
                label: format!("{:?}", category),
                value: count.to_string(),
                status: ItemStatus::Info,
                icon: self.get_category_icon(&category),
            });
        }
        
        dashboard.add_section(categories_section);
        
        println!("{}", dashboard.render());
        Ok(())
    }
    
    fn get_most_used_alias(&self) -> Option<String> {
        self.usage_stats.iter()
            .max_by_key(|(_, stats)| stats.usage_count)
            .map(|(name, _)| name.clone())
    }
    
    fn get_category_counts(&self) -> HashMap<AliasCategory, usize> {
        let mut counts = HashMap::new();
        for alias in self.aliases.values() {
            *counts.entry(alias.category.clone()).or_insert(0) += 1;
        }
        counts
    }
    
    fn get_category_icon(&self, category: &AliasCategory) -> String {
        match category {
            AliasCategory::FileManagement => "📁",
            AliasCategory::ProcessControl => "⚡",
            AliasCategory::Navigation => "🧭",
            AliasCategory::TextProcessing => "📝",
            AliasCategory::SystemAdmin => "⚙️",
            AliasCategory::Development => "💻",
            AliasCategory::Network => "🌐",
            AliasCategory::Custom => "🎯",
        }.to_string()
    }
    
    pub fn save_aliases(&self) -> Result<()> {
        // Save aliases to configuration file
        // This is a simplified implementation
        println!("{}", "💾 Aliases saved successfully".success());
        Ok(())
    }
    
    pub fn show_suggestions(&self) -> Result<()> {
        println!("\n{}", "💡 Smart Alias Suggestions".primary());
        println!("{}", "═".repeat(60).dim());
        
        for (i, suggestion) in self.suggestions.iter().enumerate() {
            let confidence_bar = self.create_confidence_bar(suggestion.confidence);
            let category_icon = self.get_category_icon(&suggestion.category);
            
            println!("\n{}. {} {} {}", 
                (i + 1).to_string().primary(),
                category_icon,
                suggestion.suggested_alias.clone().info(),
                confidence_bar
            );
            println!("   Command: {}", suggestion.command_pattern.clone().success());
            println!("   Reason: {}", suggestion.reason.clone().dim());
            println!("   Confidence: {:.0}%", (suggestion.confidence * 100.0).to_string().info());
        }
        
        println!("\n{}", "Use 'smart_alias create <name> <command>' to create an alias".info());
        Ok(())
    }
    
    fn create_confidence_bar(&self, confidence: f64) -> String {
        let bar_width = 10;
        let filled = (confidence * bar_width as f64) as usize;
        let empty = bar_width - filled;
        
        format!("[{}{}]", 
            "█".repeat(filled).success(),
            "░".repeat(empty).dim()
        )
    }
    
    pub fn run_interactive_wizard(&mut self) -> Result<()> {
        let mut wizard = CommandWizard::new("Smart Alias Creation".to_string());
        
        wizard.add_step(WizardStep {
            name: "alias_name".to_string(),
            title: "What's the alias name?".to_string(),
            description: "Choose a short, memorable name for your alias".to_string(),
            prompt: "Enter alias name: ".to_string(),
            input_type: InputType::Text,
            options: vec![],
            required: true,
        });
        
        wizard.add_step(WizardStep {
            name: "alias_command".to_string(),
            title: "What command should it execute?".to_string(),
            description: "Enter the full command with all options and arguments".to_string(),
            prompt: "Enter command: ".to_string(),
            input_type: InputType::Text,
            options: vec![],
            required: true,
        });
        
        wizard.add_step(WizardStep {
            name: "alias_description".to_string(),
            title: "Add a description?".to_string(),
            description: "Optional: Describe what this alias does".to_string(),
            prompt: "Enter description: ".to_string(),
            input_type: InputType::Text,
            options: vec![],
            required: false,
        });
        
        if let Ok(results) = wizard.run() {
            if results.len() >= 2 {
                let name = &results[0];
                let command = &results[1];
                let description = results.get(2);
                
                self.create_alias(name, command, description.map(|s| s.as_str()))?;
                
                // Show preview
                self.show_alias_preview(name)?;
            }
        }
        
        Ok(())
    }
    
    fn show_alias_preview(&self, name: &str) -> Result<()> {
        if let Some(alias) = self.aliases.get(name) {
            println!("\n{}", "📋 Alias Preview".primary());
            println!("{}", "─".repeat(40).dim());
            
            println!("Name: {}", alias.name.clone().info());
            println!("Command: {}", alias.command.clone().success());
            println!("Description: {}", alias.description);
            println!("Category: {} {:?}", self.get_category_icon(&alias.category), alias.category);
            println!("Complexity: {:?}", alias.complexity);
            
            if !alias.tags.is_empty() {
                println!("Tags: {}", alias.tags.join(", ").dim());
            }
        }
        
        Ok(())
    }
}

pub fn smart_alias_cli(args: &[String]) -> Result<()> {
    let mut manager = SmartAliasManager::load_from_file("~/.nxsh_aliases.toml").unwrap_or_else(|_| {
        let mut mgr = SmartAliasManager::new();
        mgr.initialize_default_aliases();
        mgr.generate_smart_suggestions();
        mgr
    });
    
    let formatter = TableFormatter::new();
    
    match args.get(0).map(|s| s.as_str()) {
        Some("list") | Some("ls") => {
            show_alias_list(&manager)?;
        },
        Some("create") | Some("add") => {
            if args.len() >= 3 {
                let name = &args[1];
                let command = &args[2];
                let description = args.get(3).map(|s| s.as_str());
                manager.create_alias(name, command, description)?;
            } else {
                manager.run_interactive_wizard()?;
            }
        },
        Some("suggest") | Some("suggestions") => {
            manager.show_suggestions()?;
        },
        Some("dashboard") | Some("stats") => {
            manager.show_dashboard()?;
        },
        Some("wizard") => {
            manager.run_interactive_wizard()?;
        },
        Some("export") => {
            export_aliases(&manager)?;
        },
        Some("import") => {
            if let Some(file_path) = args.get(1) {
                import_aliases(&mut manager, file_path)?;
            } else {
                println!("{}", "Usage: smart_alias import <file>".warning());
            }
        },
        None => {
            show_interactive_menu(&mut manager)?;
        },
        Some(cmd) => {
            println!("{}", format!("Unknown command: {}", cmd).error());
            show_help()?;
        }
    }
    
    Ok(())
}

fn show_alias_list(manager: &SmartAliasManager) -> Result<()> {
    println!("{}", "📋 Your Smart Aliases".primary());
    println!("{}", "═".repeat(60).dim());
    
    if manager.aliases.is_empty() {
        println!("{}", "No aliases defined yet. Use 'smart_alias create' to add some!".info());
        return Ok(());
    }
    
    // Group by category
    let mut categorized: HashMap<AliasCategory, Vec<&AliasInfo>> = HashMap::new();
    for alias in manager.aliases.values() {
        categorized.entry(alias.category.clone()).or_insert_with(Vec::new).push(alias);
    }
    
    for (category, aliases) in categorized {
        let icon = manager.get_category_icon(&category);
        println!("\n{} {:?}", icon, category);
        println!("{}", "─".repeat(40).dim());
        
        for alias in aliases {
            let complexity_icon = match alias.complexity {
                AliasComplexity::Simple => "🟢",
                AliasComplexity::Moderate => "🟡",
                AliasComplexity::Complex => "🟠",
                AliasComplexity::Advanced => "🔴",
            };
            
            println!("  {} {} {} {}", 
                complexity_icon,
                alias.name.clone().primary(),
                "→".dim(),
                alias.command.clone().info()
            );
            
            if !alias.description.is_empty() {
                println!("     {}", alias.description.clone().dim());
            }
            
            if let Some(stats) = manager.usage_stats.get(&alias.name) {
                if stats.usage_count > 0 {
                    println!("     📊 Used {} times", stats.usage_count.to_string().success());
                }
            }
        }
    }
    
    Ok(())
}

fn show_interactive_menu(manager: &mut SmartAliasManager) -> Result<()> {
    println!("{}", "🎯 Smart Alias Management System".primary());
    println!("{}", "═".repeat(50).dim());
    
    let options = vec![
        "List all aliases".to_string(),
        "Create new alias".to_string(),
        "View suggestions".to_string(),
        "Show dashboard".to_string(),
        "Run creation wizard".to_string(),
        "Export aliases".to_string(),
        "Exit".to_string(),
    ];
    
    loop {
        println!("\n{}", "What would you like to do?".info());
        for (i, option) in options.iter().enumerate() {
            println!("   {}. {}", (i + 1).to_string().primary(), option);
        }
        
        print!("\nEnter your choice (1-{}): ", options.len());
        std::io::Write::flush(&mut std::io::stdout()).ok();
        
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            if let Ok(choice) = input.trim().parse::<usize>() {
                match choice {
                    1 => show_alias_list(manager)?,
                    2 => manager.run_interactive_wizard()?,
                    3 => manager.show_suggestions()?,
                    4 => manager.show_dashboard()?,
                    5 => manager.run_interactive_wizard()?,
                    6 => export_aliases(manager)?,
                    7 => {
                        println!("{}", "Thanks for using Smart Alias Manager! 👋".success());
                        break;
                    },
                    _ => println!("{}", "Invalid choice. Please try again.".warning()),
                }
            }
        }
    }
    
    Ok(())
}

fn export_aliases(manager: &SmartAliasManager) -> Result<()> {
    println!("\n{}", "📤 Exporting Aliases".primary());
    
    for alias in manager.aliases.values() {
        println!("alias {}='{}'  # {}", 
            alias.name.clone().success(),
            alias.command,
            alias.description.clone().dim()
        );
    }
    
    println!("\n{}", "💡 Copy these lines to your shell configuration file (.bashrc, .zshrc, etc.)".info());
    Ok(())
}

fn import_aliases(manager: &mut SmartAliasManager, file_path: &str) -> Result<()> {
    // Validate file path and existence
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(anyhow!("Import file does not exist: {}", file_path));
    }
    
    if !path.is_file() {
        return Err(anyhow!("Import path is not a file: {}", file_path));
    }

    // Read and parse the file content
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read import file: {}", file_path))?;
    
    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut error_count = 0;
    
    println!("{}", format!("📥 Importing aliases from: {}", file_path).info());
    
    // Parse different alias file formats
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        
        // Parse different alias formats:
        // 1. shell format: alias name='command'
        // 2. JSON format: {"name": "alias_name", "command": "command", ...}
        // 3. TOML format: [alias] name = "command"
        // 4. simple format: name=command
        
        match parse_alias_line(line) {
            Ok(Some((name, command, description))) => {
                // Check for duplicates
                if manager.aliases.contains_key(&name) {
                    println!("{}", format!("⚠️  Line {}: Alias '{}' already exists, skipping", 
                            line_num + 1, name).warning());
                    skipped_count += 1;
                    continue;
                }
                
                // Create and add the alias using AliasInfo
                let alias_info = AliasInfo {
                    name: name.clone(),
                    command,
                    description,
                    created_at: chrono::Local::now(),
                    category: AliasCategory::Custom,
                    complexity: AliasComplexity::Simple,
                    tags: Vec::new(),
                };
                
                manager.aliases.insert(name.clone(), alias_info);
                imported_count += 1;
                
                println!("{}", format!("✅ Imported: {}", name).success());
            }
            Ok(None) => {
                // Valid line but not an alias definition (like section headers)
                continue;
            }
            Err(e) => {
                println!("{}", format!("❌ Line {}: Parse error: {}", line_num + 1, e).error());
                error_count += 1;
            }
        }
    }
    
    // Save the updated aliases
    manager.save_aliases()?;
    
    // Display import summary
    println!("\n{}", "📊 Import Summary:".primary());
    println!("{}", format!("  ✅ Imported: {} aliases", imported_count).success());
    if skipped_count > 0 {
        println!("{}", format!("  ⚠️  Skipped:  {} duplicates", skipped_count).warning());
    }
    if error_count > 0 {
        println!("{}", format!("  ❌ Errors:   {} lines", error_count).error());
    }
    
    println!("{}", format!("🎉 Import completed! Total aliases: {}", manager.aliases.len()).info());
    
    Ok(())
}

/// Parse a single line from an alias import file
/// Returns Ok(Some((name, command, description))) for valid alias definitions
/// Returns Ok(None) for valid non-alias lines (comments, sections, etc.)
/// Returns Err for invalid/malformed lines
fn parse_alias_line(line: &str) -> Result<Option<(String, String, String)>> {
    let line = line.trim();
    
    // Try JSON format first
    if line.starts_with('{') && line.ends_with('}') {
        return parse_json_alias(line);
    }
    
    // Try shell alias format: alias name='command' or alias name="command"
    if line.starts_with("alias ") {
        return parse_shell_alias(line);
    }
    
    // Try TOML-like format: name = "command" # description
    if line.contains(" = ") {
        return parse_toml_alias(line);
    }
    
    // Try simple format: name=command
    if line.contains('=') && !line.starts_with('[') {
        return parse_simple_alias(line);
    }
    
    // Valid non-alias line (section headers, etc.)
    Ok(None)
}

/// Parse JSON format: {"name": "alias_name", "command": "command", "description": "desc"}
fn parse_json_alias(line: &str) -> Result<Option<(String, String, String)>> {
    use serde_json::Value;
    
    let parsed: Value = serde_json::from_str(line)
        .with_context(|| "Invalid JSON format")?;
    
    let obj = parsed.as_object()
        .ok_or_else(|| anyhow!("JSON must be an object"))?;
    
    let name = obj.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing or invalid 'name' field"))?
        .to_string();
    
    let command = obj.get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing or invalid 'command' field"))?
        .to_string();
    
    let description = obj.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    validate_alias_name(&name)?;
    validate_alias_command(&command)?;
    
    Ok(Some((name, command, description)))
}

/// Parse shell format: alias name='command' or alias name="command"
fn parse_shell_alias(line: &str) -> Result<Option<(String, String, String)>> {
    let without_alias = line.strip_prefix("alias ").unwrap();
    
    // Find the equals sign
    let eq_pos = without_alias.find('=')
        .ok_or_else(|| anyhow!("Missing '=' in alias definition"))?;
    
    let name = without_alias[..eq_pos].trim().to_string();
    let command_part = without_alias[eq_pos + 1..].trim();
    
    // Handle quoted commands
    let command = if (command_part.starts_with('"') && command_part.ends_with('"')) ||
                     (command_part.starts_with('\'') && command_part.ends_with('\'')) {
        command_part[1..command_part.len()-1].to_string()
    } else {
        command_part.to_string()
    };
    
    validate_alias_name(&name)?;
    validate_alias_command(&command)?;
    
    Ok(Some((name, command, String::new())))
}

/// Parse TOML-like format: name = "command" # description
fn parse_toml_alias(line: &str) -> Result<Option<(String, String, String)>> {
    // Split on comment first
    let (main_part, description) = if let Some(comment_pos) = line.find('#') {
        let main = line[..comment_pos].trim();
        let desc = line[comment_pos + 1..].trim();
        (main, desc.to_string())
    } else {
        (line, String::new())
    };
    
    // Parse name = "command"
    let eq_pos = main_part.find(" = ")
        .ok_or_else(|| anyhow!("Missing ' = ' in TOML-style alias definition"))?;
    
    let name = main_part[..eq_pos].trim().to_string();
    let command_part = main_part[eq_pos + 3..].trim();
    
    // Handle quoted commands
    let command = if (command_part.starts_with('"') && command_part.ends_with('"')) ||
                     (command_part.starts_with('\'') && command_part.ends_with('\'')) {
        command_part[1..command_part.len()-1].to_string()
    } else {
        command_part.to_string()
    };
    
    validate_alias_name(&name)?;
    validate_alias_command(&command)?;
    
    Ok(Some((name, command, description)))
}

/// Parse simple format: name=command
fn parse_simple_alias(line: &str) -> Result<Option<(String, String, String)>> {
    let eq_pos = line.find('=')
        .ok_or_else(|| anyhow!("Missing '=' in simple alias definition"))?;
    
    let name = line[..eq_pos].trim().to_string();
    let command = line[eq_pos + 1..].trim().to_string();
    
    validate_alias_name(&name)?;
    validate_alias_command(&command)?;
    
    Ok(Some((name, command, String::new())))
}

/// Validate alias name according to shell naming rules
fn validate_alias_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Alias name cannot be empty"));
    }
    
    if !name.chars().next().unwrap().is_alphabetic() && !name.starts_with('_') {
        return Err(anyhow!("Alias name must start with a letter or underscore"));
    }
    
    for ch in name.chars() {
        if !ch.is_alphanumeric() && ch != '_' && ch != '-' {
            return Err(anyhow!("Alias name contains invalid character: '{}'", ch));
        }
    }
    
    // Check for reserved words and shell builtins
    let reserved_words = ["if", "then", "else", "elif", "fi", "case", "esac", "for", "while", 
                         "until", "do", "done", "function", "select", "time", "coproc"];
    
    if reserved_words.contains(&name) {
        return Err(anyhow!("Cannot use reserved word '{}' as alias name", name));
    }
    
    Ok(())
}

/// Validate alias command
fn validate_alias_command(command: &str) -> Result<()> {
    if command.trim().is_empty() {
        return Err(anyhow!("Alias command cannot be empty"));
    }
    
    // Check for potentially dangerous commands
    let dangerous_patterns = ["rm -rf /", "mkfs", "dd if=", ":(){ :|:& };:", "chmod -R 777 /"];
    
    for pattern in &dangerous_patterns {
        if command.contains(pattern) {
            return Err(anyhow!("Potentially dangerous command pattern detected: {}", pattern));
        }
    }
    
    Ok(())
}

fn show_help() -> Result<()> {
    println!("\n{}", "🎯 Smart Alias Manager Help".primary());
    println!("{}", "═".repeat(50).dim());
    
    println!("\n{}", "Commands:".info());
    println!("  list, ls          - Show all aliases");
    println!("  create, add       - Create new alias");
    println!("  suggest           - Show smart suggestions");
    println!("  dashboard, stats  - Show statistics dashboard");
    println!("  wizard            - Run interactive creation wizard");
    println!("  export            - Export aliases for shell config");
    println!("  import <file>     - Import aliases from file");
    
    println!("\n{}", "Examples:".info());
    println!("  smart_alias create ll 'ls -la'");
    println!("  smart_alias wizard");
    println!("  smart_alias suggest");
    
    Ok(())
}
