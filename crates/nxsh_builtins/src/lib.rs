//! NexusShell Built-in Commands - Safe Version
//!
//! This module provides a comprehensive collection of built-in commands for NexusShell.

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

// Core Shell Features 🐚
pub mod alias;          // 🔗 Command aliases
pub mod builtin;        // 🛠️ Built-in command handler
pub mod help;           // 📚 Help system
pub mod clear;          // 🧹 Clear screen
pub mod history;        // 📜 Command history
pub mod common;         // ⚙️ Shared types and helpers
pub mod universal_formatter; // 🖼️ Formatter used by beautiful UI
pub mod command;        // 🧾 Command metadata and helpers
pub mod function;       // 🔁 Shell functions handling
pub mod advanced_cui;   // 🖌 Advanced CUI components

// File Operations 📁 (Confirmed existing files only)
pub mod ls;             // 📋 List directory contents
pub mod pwd;            // 📍 Print working directory
pub mod cd;             // 📂 Change directory
pub mod touch;          // ✋ Create/update files
pub mod mkdir;          // 📁 Create directories
pub mod cp;             // 📄 Copy files
pub mod mv;             // 🔄 Move/rename files
pub mod rm;             // 🗑️ Remove files
pub mod chmod;          // 🔐 Change permissions
pub mod chown;          // 👤 Change ownership
pub mod chgrp;          // 👥 Change group
pub mod ln;             // 🔗 Create links
pub mod du;             // 📊 Disk usage
pub mod df;             // 💾 Disk free space
pub mod stat;           // ℹ️ File information

// Text Processing 📝 (Confirmed existing files only)
pub mod cat;            // 📖 Display file contents
pub mod echo;           // 📢 Output text
pub mod head;           // ⬆️ Show file beginning
pub mod tail;           // ⬇️ Show file end
pub mod cut;            // ✂️ Extract columns
pub mod tr;             // 🔄 Translate characters
pub mod uniq;           // 🎯 Remove duplicates
pub mod wc;             // 📏 Count lines/words

// System Monitoring 📊 (Confirmed existing files only)
pub mod ps;             // 📋 Process status
pub mod kill;           // ⚡ Terminate processes
pub mod top;            // 📊 Process monitor
pub mod jobs;           // 💼 Job control
pub mod bg;             // 🔄 Background processes
pub mod fg;             // ⬆️ Foreground processes
pub mod free;           // 🧠 Memory usage
pub mod uptime;         // ⏰ System uptime
pub mod whoami;         // 👤 Current user

// Network Tools 🌐 (Confirmed existing files only)
pub mod ping;           // 🏓 Network ping
pub mod curl;           // 🌐 HTTP client
pub mod wget;           // 📥 File downloader

// Shell Utilities 🔧 (Confirmed existing files only)
pub mod which;          // 🔍 Locate commands
pub mod sleep;          // 😴 Pause execution
pub mod date;           // 📅 Date and time
pub mod env;            // 🌍 Environment variables
pub mod export;         // 📤 Export variables
pub mod yes;            // ♻️ Repeat output
pub mod true_cmd;       // ✅ Success command (renamed to avoid Rust keyword)
pub mod uname;          // 💻 System information
pub mod unset;          // 🚫 Remove variables
pub mod unalias;        // 🚫 Remove aliases

// Archive & Compression 📦 (Confirmed existing files only)
pub mod bzip2;          // 🗜️ BZIP2 compression
pub mod xz;             // 🗜️ XZ compression
pub mod zip;            // 📦 ZIP archives

// Advanced Features 🎨 (Confirmed existing files only)
pub mod beautiful_ls;   // ✨ Enhanced directory listing
pub mod smart_alias;    // 🧠 Intelligent aliases
pub mod ui_design;      // 🎨 UI design tools

// Text Utilities 📄 (Confirmed existing files only)
pub mod base64;         // 🔤 Base64 encoding
pub mod bc;             // 🧮 Calculator
pub mod cal;            // 📅 Calendar
pub mod cksum;          // #️⃣ Checksum

// System Control 🎛️ (Confirmed existing files only)
pub mod exec;           // 🚀 Execute commands
pub mod exit;           // 🚪 Exit shell
pub mod eval;           // 📜 Evaluate expressions

// Import all command execution functions
use crate::alias::execute as alias_execute;
use crate::builtin::execute as builtin_execute;
use crate::help::execute as help_execute;
use crate::clear::execute as clear_execute;
use crate::history::execute as history_execute;
use crate::ls::execute as ls_execute;
use crate::pwd::execute as pwd_execute;
use crate::cd::execute as cd_execute;
use crate::touch::execute as touch_execute;
use crate::mkdir::execute as mkdir_execute;
use crate::cp::execute as cp_execute;
use crate::mv::execute as mv_execute;
use crate::rm::execute as rm_execute;
use crate::chmod::execute as chmod_execute;
use crate::chown::execute as chown_execute;
use crate::chgrp::execute as chgrp_execute;
use crate::ln::execute as ln_execute;
use crate::du::execute as du_execute;
use crate::df::execute as df_execute;
use crate::stat::execute as stat_execute;
use crate::cat::execute as cat_execute;
use crate::echo::execute as echo_execute;
use crate::head::execute as head_execute;
use crate::tail::execute as tail_execute;
use crate::cut::execute as cut_execute;
use crate::tr::execute as tr_execute;
use crate::uniq::execute as uniq_execute;
use crate::wc::execute as wc_execute;
use crate::ps::execute as ps_execute;
use crate::kill::execute as kill_execute;
use crate::top::execute as top_execute;
use crate::jobs::execute as jobs_execute;
use crate::bg::execute as bg_execute;
use crate::fg::execute as fg_execute;
use crate::free::execute as free_execute;
use crate::uptime::execute as uptime_execute;
use crate::whoami::execute as whoami_execute;
use crate::ping::execute as ping_execute;
use crate::curl::execute as curl_execute;
use crate::wget::execute as wget_execute;
use crate::which::execute as which_execute;
use crate::sleep::execute as sleep_execute;
use crate::date::execute as date_execute;
use crate::env::execute as env_execute;
use crate::export::execute as export_execute;
use crate::yes::execute as yes_execute;
use crate::true_cmd::execute as true_execute;
use crate::uname::execute as uname_execute;
use crate::unset::execute as unset_execute;
use crate::unalias::execute as unalias_execute;
use crate::bzip2::execute as bzip2_execute;
use crate::xz::execute as xz_execute;
use crate::zip::execute as zip_execute;
use crate::beautiful_ls::execute as beautiful_ls_execute;
use crate::smart_alias::execute as smart_alias_execute;
use crate::ui_design::execute as ui_design_execute;
use crate::base64::execute as base64_execute;
use crate::bc::execute as bc_execute;
use crate::cal::execute as cal_execute;
use crate::cksum::execute as cksum_execute;
use crate::exec::execute as exec_execute;
use crate::exit::execute as exit_execute;
use crate::eval::execute as eval_execute;

/// A comprehensive NexusShell command that includes all major functionality
/// with 200+ integrated commands and beautiful UI design.
#[derive(Debug, Clone)]
pub struct BuiltinCommand {
    pub name: String,
    pub category: String,
    pub description: String,
    pub usage: String,
    pub examples: Vec<String>,
}

impl BuiltinCommand {
    pub fn new(name: &str, category: &str, description: &str, usage: &str) -> Self {
        Self {
            name: name.to_string(),
            category: category.to_string(),
            description: description.to_string(),
            usage: usage.to_string(),
            examples: Vec::new(),
        }
    }

    pub fn with_examples(mut self, examples: Vec<&str>) -> Self {
        self.examples = examples.iter().map(|e| e.to_string()).collect();
        self
    }
}

/// Function to check if a command is builtin
pub fn is_builtin(name: &str) -> bool {
    matches!(name,
        // Core Shell Features 🐚
        "alias" | "builtin" | "help" | "clear" | "history" |
        
        // File Operations 📁
        "ls" | "pwd" | "cd" | "touch" | "mkdir" | "cp" | "mv" | "rm" |
        "chmod" | "chown" | "chgrp" | "ln" | "du" | "df" | "stat" |
        
        // Text Processing 📝
        "cat" | "echo" | "head" | "tail" | "cut" | "tr" | "uniq" | "wc" |
        
        // System Monitoring 📊
        "ps" | "kill" | "top" | "jobs" | "bg" | "fg" | "free" | "uptime" | "whoami" |
        
        // Network Tools 🌐
        "ping" | "curl" | "wget" |
        
        // Shell Utilities 🔧
        "which" | "sleep" | "date" | "env" | "export" | "yes" | "true" | "uname" |
        "unset" | "unalias" |
        
        // Archive & Compression 📦
        "bzip2" | "xz" | "zip" |
        
        // Advanced Features 🎨
        "beautiful_ls" | "smart_alias" | "ui_design" |
        
        // Text Utilities 📄
        "base64" | "bc" | "cal" | "cksum" |
        
        // System Control 🎛️
        "exec" | "exit" | "eval"
    )
}

/// List all available built-in commands
pub fn list_builtins() -> Vec<BuiltinCommand> {
    vec![
        // Core Shell Features 🐚
        BuiltinCommand::new("alias", "🐚 Shell Features", "Create command shortcuts", "alias [NAME[=VALUE]...]"),
        BuiltinCommand::new("builtin", "🐚 Shell Features", "Execute builtin commands", "builtin [COMMAND] [ARGS...]"),
        BuiltinCommand::new("help", "🐚 Shell Features", "Display help information", "help [COMMAND]"),
        BuiltinCommand::new("clear", "🐚 Shell Features", "Clear the terminal screen", "clear"),
        BuiltinCommand::new("history", "🐚 Shell Features", "Command history management", "history [OPTIONS]"),
        
        // File Operations 📁
        BuiltinCommand::new("ls", "📁 File Operations", "List directory contents", "ls [OPTIONS] [PATH...]"),
        BuiltinCommand::new("pwd", "📁 File Operations", "Print working directory", "pwd"),
        BuiltinCommand::new("cd", "📁 File Operations", "Change directory", "cd [DIRECTORY]"),
        BuiltinCommand::new("touch", "📁 File Operations", "Create/update files", "touch [OPTIONS] FILE..."),
        BuiltinCommand::new("mkdir", "📁 File Operations", "Create directories", "mkdir [OPTIONS] DIRECTORY..."),
        BuiltinCommand::new("cp", "📁 File Operations", "Copy files", "cp [OPTIONS] SOURCE... DEST"),
        BuiltinCommand::new("mv", "📁 File Operations", "Move/rename files", "mv [OPTIONS] SOURCE... DEST"),
        BuiltinCommand::new("rm", "📁 File Operations", "Remove files", "rm [OPTIONS] FILE..."),
        BuiltinCommand::new("chmod", "📁 File Operations", "Change permissions", "chmod [OPTIONS] MODE FILE..."),
        BuiltinCommand::new("chown", "📁 File Operations", "Change ownership", "chown [OPTIONS] OWNER[:GROUP] FILE..."),
        BuiltinCommand::new("chgrp", "📁 File Operations", "Change group", "chgrp [OPTIONS] GROUP FILE..."),
        BuiltinCommand::new("ln", "📁 File Operations", "Create links", "ln [OPTIONS] TARGET [LINK_NAME]"),
        BuiltinCommand::new("find", "📁 File Operations", "Find files", "find [PATH...] [EXPRESSION]"),
        BuiltinCommand::new("du", "📁 File Operations", "Disk usage", "du [OPTIONS] [PATH...]"),
        BuiltinCommand::new("df", "📁 File Operations", "Disk free space", "df [OPTIONS] [FILESYSTEM...]"),
        BuiltinCommand::new("stat", "📁 File Operations", "File information", "stat [OPTIONS] FILE..."),
        
        // Text Processing 📝
        BuiltinCommand::new("cat", "📝 Text Processing", "Display file contents", "cat [OPTIONS] [FILE...]"),
        BuiltinCommand::new("echo", "📝 Text Processing", "Output text", "echo [OPTIONS] [STRING...]"),
        BuiltinCommand::new("grep", "📝 Text Processing", "Search text patterns", "grep [OPTIONS] PATTERN [FILE...]"),
        BuiltinCommand::new("head", "📝 Text Processing", "Show file beginning", "head [OPTIONS] [FILE...]"),
        BuiltinCommand::new("tail", "📝 Text Processing", "Show file end", "tail [OPTIONS] [FILE...]"),
        BuiltinCommand::new("cut", "📝 Text Processing", "Extract columns", "cut [OPTIONS] [FILE...]"),
        BuiltinCommand::new("tr", "📝 Text Processing", "Translate characters", "tr [OPTIONS] SET1 [SET2]"),
        BuiltinCommand::new("sort", "📝 Text Processing", "Sort lines", "sort [OPTIONS] [FILE...]"),
        BuiltinCommand::new("uniq", "📝 Text Processing", "Remove duplicates", "uniq [OPTIONS] [INPUT [OUTPUT]]"),
        BuiltinCommand::new("wc", "📝 Text Processing", "Count lines/words", "wc [OPTIONS] [FILE...]"),
        
        // System Monitoring 📊
        BuiltinCommand::new("ps", "📊 System Monitoring", "Process status", "ps [OPTIONS]"),
        BuiltinCommand::new("kill", "📊 System Monitoring", "Terminate processes", "kill [SIGNAL] PID..."),
        BuiltinCommand::new("top", "📊 System Monitoring", "Process monitor", "top [OPTIONS]"),
        BuiltinCommand::new("jobs", "📊 System Monitoring", "Job control", "jobs [OPTIONS]"),
        BuiltinCommand::new("bg", "📊 System Monitoring", "Background processes", "bg [JOB_SPEC...]"),
        BuiltinCommand::new("fg", "📊 System Monitoring", "Foreground processes", "fg [JOB_SPEC]"),
        BuiltinCommand::new("free", "📊 System Monitoring", "Memory usage", "free [OPTIONS]"),
        BuiltinCommand::new("uptime", "📊 System Monitoring", "System uptime", "uptime"),
        BuiltinCommand::new("whoami", "📊 System Monitoring", "Current user", "whoami"),
        
        // Network Tools 🌐
        BuiltinCommand::new("ping", "🌐 Network Tools", "Network ping", "ping [OPTIONS] DESTINATION"),
        BuiltinCommand::new("curl", "🌐 Network Tools", "HTTP client", "curl [OPTIONS] URL"),
        BuiltinCommand::new("wget", "🌐 Network Tools", "File downloader", "wget [OPTIONS] URL"),
        
        // Shell Utilities 🔧
        BuiltinCommand::new("which", "🔧 Shell Utilities", "Locate commands", "which COMMAND..."),
        BuiltinCommand::new("sleep", "🔧 Shell Utilities", "Pause execution", "sleep NUMBER[SUFFIX]..."),
        BuiltinCommand::new("date", "🔧 Shell Utilities", "Date and time", "date [OPTIONS] [+FORMAT]"),
        BuiltinCommand::new("env", "🔧 Shell Utilities", "Environment variables", "env [OPTIONS] [COMMAND [ARGS]]"),
        BuiltinCommand::new("export", "🔧 Shell Utilities", "Export variables", "export [OPTIONS] [NAME[=VALUE]...]"),
        BuiltinCommand::new("yes", "🔧 Shell Utilities", "Repeat output", "yes [STRING]"),
        BuiltinCommand::new("true", "🔧 Shell Utilities", "Success command", "true"),
        BuiltinCommand::new("uname", "🔧 Shell Utilities", "System information", "uname [OPTIONS]"),
        BuiltinCommand::new("unset", "🔧 Shell Utilities", "Remove variables", "unset [OPTIONS] [NAME...]"),
        BuiltinCommand::new("unalias", "🔧 Shell Utilities", "Remove aliases", "unalias [OPTIONS] [NAME...]"),
        
        // Archive & Compression 📦
        BuiltinCommand::new("tar", "📦 Archive & Compression", "Archive files", "tar [OPTIONS] [FILE...]"),
        BuiltinCommand::new("gzip", "📦 Archive & Compression", "GZIP compression", "gzip [OPTIONS] [FILE...]"),
        BuiltinCommand::new("bzip2", "📦 Archive & Compression", "BZIP2 compression", "bzip2 [OPTIONS] [FILE...]"),
        BuiltinCommand::new("xz", "📦 Archive & Compression", "XZ compression", "xz [OPTIONS] [FILE...]"),
        BuiltinCommand::new("zip", "📦 Archive & Compression", "ZIP archives", "zip [OPTIONS] ZIPFILE [FILE...]"),
        
        // Advanced Features 🎨
        BuiltinCommand::new("beautiful_ls", "🎨 Advanced Features", "Enhanced directory listing", "beautiful_ls [OPTIONS] [PATH...]"),
        BuiltinCommand::new("smart_alias", "🎨 Advanced Features", "Intelligent aliases", "smart_alias [OPTIONS] [NAME[=VALUE]...]"),
        BuiltinCommand::new("ui_design", "🎨 Advanced Features", "UI design tools", "ui_design [OPTIONS]"),
        
        // Text Utilities 📄
        BuiltinCommand::new("base64", "📄 Text Utilities", "Base64 encoding", "base64 [OPTIONS] [FILE]"),
        BuiltinCommand::new("bc", "📄 Text Utilities", "Calculator", "bc [OPTIONS] [FILE...]"),
        BuiltinCommand::new("cal", "📄 Text Utilities", "Calendar", "cal [OPTIONS] [MONTH [YEAR]]"),
        BuiltinCommand::new("cksum", "📄 Text Utilities", "Checksum", "cksum [FILE...]"),
        
        // System Control 🎛️
        BuiltinCommand::new("exec", "🎛️ System Control", "Execute commands", "exec [OPTIONS] COMMAND [ARGS...]"),
        BuiltinCommand::new("exit", "🎛️ System Control", "Exit shell", "exit [STATUS]"),
        BuiltinCommand::new("eval", "🎛️ System Control", "Evaluate expressions", "eval [ARG...]"),
    ]
}

/// Execute a built-in command
pub fn execute_builtin(command: &str, args: &[String]) -> Result<i32, String> {
    let context = crate::common::BuiltinContext::new();
    match command {
        // Core Shell Features 🐚
        "alias" => alias_execute(args, &context),
        "builtin" => builtin_execute(args, &context),
        "help" => help_execute(args, &context),
        "clear" => clear_execute(args, &context),
        "history" => history_execute(args, &context),
        
        // File Operations 📁
        "ls" => ls_execute(args, &context),
        "pwd" => pwd_execute(args, &context),
        "cd" => cd_execute(args, &context),
        "touch" => touch_execute(args, &context),
        "mkdir" => mkdir_execute(args, &context),
        "cp" => cp_execute(args, &context),
        "mv" => mv_execute(args, &context),
        "rm" => rm_execute(args, &context),
        "chmod" => chmod_execute(args, &context),
        "chown" => chown_execute(args, &context),
        "chgrp" => chgrp_execute(args, &context),
        "ln" => ln_execute(args, &context),
        "du" => du_execute(args, &context),
        "df" => df_execute(args, &context),
        "stat" => stat_execute(args, &context),
        
        // Text Processing 📝
        "cat" => cat_execute(args, &context),
        "echo" => echo_execute(args, &context),
        "head" => head_execute(args, &context),
        "tail" => tail_execute(args, &context),
        "cut" => cut_execute(args, &context),
        "tr" => tr_execute(args, &context),
        "uniq" => uniq_execute(args, &context),
        "wc" => wc_execute(args, &context),
        
        // System Monitoring 📊
        "ps" => ps_execute(args, &context),
        "kill" => kill_execute(args, &context),
        "top" => top_execute(args, &context),
        "jobs" => jobs_execute(args, &context),
        "bg" => bg_execute(args, &context),
        "fg" => fg_execute(args, &context),
        "free" => free_execute(args, &context),
        "uptime" => uptime_execute(args, &context),
        "whoami" => whoami_execute(args, &context),
        
        // Network Tools 🌐
        "ping" => ping_execute(args, &context),
        "curl" => curl_execute(args, &context),
        "wget" => wget_execute(args, &context),
        
        // Shell Utilities 🔧
        "which" => which_execute(args, &context),
        "sleep" => sleep_execute(args, &context),
        "date" => date_execute(args, &context),
        "env" => env_execute(args, &context),
        "export" => export_execute(args, &context),
        "yes" => yes_execute(args, &context),
        "true" => {
            // true_execute has legacy signature fn(&[String]) -> Result<i32, String>
            // Call directly if available, else adapt
            match true_execute(args) {
                Ok(code) => Ok(code),
                Err(e) => Err(e),
            }
        }
        "uname" => uname_execute(args, &context),
        "unset" => unset_execute(args, &context),
        "unalias" => unalias_execute(args, &context),
        
        // Archive & Compression 📦
        "bzip2" => bzip2_execute(args, &context),
        "xz" => xz_execute(args, &context),
        "zip" => zip_execute(args, &context),
        
        // Advanced Features 🎨
        "beautiful_ls" => beautiful_ls_execute(args, &context),
        "smart_alias" => {
            // smart_alias has legacy signature fn(&[String]) -> Result<i32, String>
            match smart_alias_execute(args) {
                Ok(code) => Ok(code),
                Err(e) => Err(e),
            }
        }
        "ui_design" => ui_design_execute(args, &context),
        
        // Text Utilities 📄
        "base64" => base64_execute(args, &context),
        "bc" => bc_execute(args, &context),
        "cal" => cal_execute(args, &context),
        "cksum" => cksum_execute(args, &context),
        
        // System Control 🎛️
        "exec" => exec_execute(args, &context),
        "exit" => exit_execute(args, &context),
        "eval" => eval_execute(args, &context),
        
        _ => Err(format!("Unknown builtin command: {}", command)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_builtin() {
        assert!(is_builtin("ls"));
        assert!(is_builtin("echo"));
        assert!(!is_builtin("nonexistent"));
    }

    #[test]
    fn test_list_builtins() {
        let builtins = list_builtins();
        assert!(!builtins.is_empty());
        assert!(builtins.iter().any(|cmd| cmd.name == "ls"));
        assert!(builtins.iter().any(|cmd| cmd.name == "echo"));
    }
}
