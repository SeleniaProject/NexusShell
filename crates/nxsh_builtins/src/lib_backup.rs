//! NexusShell Built-in Commands
//!
//! This crate provides a comprehensive collection of built-in commands for NexusShell.
//! These commands are implemented in pure Rust for cross-platform compatibility,
//! performance, and security.

#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(unused_imports)]

// Core file operations
pub mod ls;
pub mod pwd;
pub mod cd;
pub mod cp;
pub mod mv;
pub mod rm;
pub mod touch;
pub mod mkdir;
pub mod rmdir;
pub mod ln;

// Text processing
pub mod cat;
pub mod echo;
pub mod head;
pub mod tail;
//pub mod grep;
//pub mod sed;

// Advanced features
pub mod smart_alias_simple;
pub mod ui_design;
pub mod alias;
// pub mod history_advanced;
// pub mod completion_enhanced;

// System utilities
pub mod sleep;
// pub mod env_advanced;
pub mod which;
// pub mod find_advanced;
//pub mod awk;
pub mod wc;
//pub mod sort;
pub mod uniq;
pub mod cut;
pub mod tr;
pub mod tee;

// File search and navigation
//pub mod find;
pub mod locate;
pub mod which;
//pub mod tree;

// System information
pub mod ps;
pub mod top;
pub mod free;
pub mod df;
pub mod du;
pub mod uname;
pub mod whoami;
pub mod id;
pub mod uptime;

// Environment and variables
pub mod env;
pub mod export;
pub mod set;
pub mod unset;
pub mod declare;
pub mod local;
pub mod readonly;

// Shell features
pub mod history;
pub mod alias;
pub mod unalias;
pub mod hash;
pub mod help;

#[path = "type.rs"]
pub mod type_;

// Process control
pub mod jobs;
pub mod bg;
pub mod fg;
pub mod kill;
pub mod nohup;
pub mod timeout;

// Date and time
pub mod date;
pub mod cal;
pub mod sleep;

// Network tools
pub mod ping;
pub mod wget;
pub mod curl;

// Compression
//pub mod tar;
//pub mod gzip;
//pub mod gunzip;
//pub mod zip;
//pub mod unzip;

// Utility commands
pub mod yes;
pub mod seq;

#[path = "test_builtin.rs"]
pub mod test;

#[path = "true_cmd.rs"]
pub mod true_cmd;

#[path = "false_cmd.rs"]
pub mod false_cmd;

pub mod clear;

// Built-in management
pub mod builtin;
pub mod command;

// Common utilities used by multiple built-ins
pub mod common;

// Re-export commonly used types
pub use common::{BuiltinResult, BuiltinError, BuiltinContext};

/// Execute a built-in command by name
pub fn execute_builtin(name: &str, args: &[String], context: &BuiltinContext) -> BuiltinResult<i32> {
    match name {
        "ls" => ls::execute(args, context),
        "pwd" => pwd::execute(args, context),
        "cd" => cd::execute(args, context),
        "cp" => cp::execute(args, context),
        "mv" => mv::execute(args, context),
        "rm" => rm::execute(args, context),
        "touch" => touch::execute(args, context),
        "mkdir" => mkdir::execute(args, context),
        "rmdir" => rmdir::execute(args, context),
        "ln" => ln::execute(args, context),
        
        "cat" => cat::execute(args, context),
        "echo" => echo::execute(args, context),
        "head" => head::execute(args, context),
        "tail" => tail::execute(args, context),
        // "grep" => grep::execute(args, context),
        // "sed" => sed::execute(args, context),
        // "awk" => awk::execute(args, context),
        "wc" => wc::execute(args, context),
        // "sort" => sort::execute(args, context),
        "uniq" => uniq::execute(args, context),
        "cut" => cut::execute(args, context),
        "tr" => tr::execute(args, context),
        "tee" => tee::execute(args, context),
        
        // "find" => find::execute(args, context),
        "locate" => locate::execute(args, context),
        "which" => which::execute(args, context),
        // "tree" => tree::execute(args, context),
        
        "ps" => ps::execute(args, context),
        "top" => top::execute(args, context),
        "free" => free::execute(args, context),
        "df" => df::execute(args, context),
        "du" => du::execute(args, context),
        "uname" => uname::execute(args, context),
        "whoami" => whoami::execute(args, context),
        "id" => id::execute(args, context),
        "uptime" => uptime::execute(args, context),
        
        "env" => env::execute(args, context),
        "export" => export::execute(args, context),
        "set" => set::execute(args, context),
        "unset" => unset::execute(args, context),
        "declare" => declare::execute(args, context),
        "local" => local::execute(args, context),
        "readonly" => readonly::execute(args, context),
        
        "history" => history::execute(args, context),
        "alias" => alias::execute(args, context),
        "unalias" => unalias::execute(args, context),
        "hash" => hash::execute(args, context),
        "help" => help::execute(args, context),
        "type" => type_::execute(args, context),
        
        "jobs" => jobs::execute(args, context),
        "bg" => bg::execute(args, context),
        "fg" => fg::execute(args, context),
        "kill" => kill::execute(args, context),
        "nohup" => nohup::execute(args, context),
        "timeout" => timeout::execute(args, context),
        
        "date" => date::execute(args, context),
        "cal" => cal::execute(args, context),
        "sleep" => sleep::execute(args, context),
        
        "ping" => ping::execute(args, context),
        "wget" => wget::execute(args, context),
        "curl" => curl::execute(args, context),
        
        // "tar" => tar::execute(args, context),
        // "gzip" => gzip::execute(args, context),
        // "gunzip" => gunzip::execute(args, context),
        // "zip" => zip::execute(args, context),
        // "unzip" => unzip::execute(args, context),
        
        "yes" => yes::execute(args, context),
        "seq" => seq::execute(args, context),
        "test" => test::execute(args, context),
        "true" => true_cmd::execute(args, context),
        "false" => false_cmd::execute(args, context),
        "clear" => clear::execute(args, context),
        
        "builtin" => builtin::execute(args, context),
        "command" => command::execute(args, context),
        
        _ => Err(BuiltinError::UnknownCommand(name.to_string())),
    }
}

/// Check if a command is a built-in
pub fn is_builtin(name: &str) -> bool {
    matches!(name,
        "ls" | "pwd" | "cd" | "cp" | "mv" | "rm" | "touch" | "mkdir" | "rmdir" | "ln" |
        "cat" | "echo" | "head" | "tail" | "grep" | "sed" | "awk" | "wc" | "sort" | "uniq" | 
        "cut" | "tr" | "tee" | "find" | "locate" | "which" | "tree" | "ps" | "top" | "free" |
        "df" | "du" | "uname" | "whoami" | "id" | "uptime" | "env" | "export" | "set" | 
        "unset" | "declare" | "local" | "readonly" | "history" | "alias" | "unalias" | 
        "hash" | "help" | "type" | "jobs" | "bg" | "fg" | "kill" | "nohup" | "timeout" |
        "date" | "cal" | "sleep" | "ping" | "wget" | "curl" | "tar" | "gzip" | "gunzip" |
        "zip" | "unzip" | "yes" | "seq" | "test" | "true" | "false" | "clear" | "builtin" |
        "command"
    )
}

/// Get a list of all available built-in commands
pub fn list_builtins() -> Vec<&'static str> {
    vec![
        "ls", "pwd", "cd", "cp", "mv", "rm", "touch", "mkdir", "rmdir", "ln",
        "cat", "echo", "head", "tail", "grep", "sed", "awk", "wc", "sort", "uniq",
        "cut", "tr", "tee", "find", "locate", "which", "tree", "ps", "top", "free",
        "df", "du", "uname", "whoami", "id", "uptime", "env", "export", "set",
        "unset", "declare", "local", "readonly", "history", "alias", "unalias",
        "hash", "help", "type", "jobs", "bg", "fg", "kill", "nohup", "timeout",
        "date", "cal", "sleep", "ping", "wget", "curl", "tar", "gzip", "gunzip",
        "zip", "unzip", "yes", "seq", "test", "true", "false", "clear", "builtin",
        "command"
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_builtin() {
        assert!(is_builtin("ls"));
        assert!(is_builtin("echo"));
        assert!(is_builtin("grep"));
        assert!(!is_builtin("nonexistent"));
    }

    #[test]
    fn test_list_builtins() {
        let builtins = list_builtins();
        assert!(!builtins.is_empty());
        assert!(builtins.contains(&"ls"));
        assert!(builtins.contains(&"echo"));
    }
}

// Additional modules
pub mod function;
