use std::env;
use crate::command::{CommandInfo, CommandResult, ShellState, CommandType};

/// Environment variable management command
pub struct EnvCommand;

impl EnvCommand {
    fn info(&self) -> CommandInfo {
        CommandInfo {
            command_type: CommandType::Builtin,
            path: None,
            name: "env".to_string(),
            description: "Display or set environment variables".to_string(),
            usage: "env [OPTION]... [NAME[=VALUE]]...".to_string(),
            examples: vec![
                "env".to_string(),
                "env PATH".to_string(),
                "env FOO=bar".to_string(),
                "env -u PATH".to_string(),
                "env --help".to_string(),
            ],
        }
    }

    fn execute(&self, args: &[String], _state: &mut ShellState) -> CommandResult {
        let mut show_help = false;
        let mut null_separator = false;
        let mut ignore_env = false;
        let mut unset_vars: Vec<String> = Vec::new();
        let mut set_vars: Vec<(String, String)> = Vec::new();
        let mut query_vars: Vec<String> = Vec::new();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--help" => show_help = true,
                "-0" | "--null" => null_separator = true,
                "-i" | "--ignore-environment" => ignore_env = true,
                "-u" | "--unset" => {
                    if i + 1 < args.len() {
                        unset_vars.push(args[i + 1].clone());
                        i += 1;
                    } else {
                        return CommandResult::error("--unset requires a variable name");
                    }
                }
                arg if arg.contains('=') => {
                    if let Some((key, value)) = arg.split_once('=') {
                        set_vars.push((key.to_string(), value.to_string()));
                    } else {
                        return CommandResult::error(&format!("Invalid assignment: {}", arg));
                    }
                }
                arg if !arg.starts_with('-') => {
                    query_vars.push(arg.to_string());
                }
                _ => {
                    return CommandResult::error(&format!("Unknown option: {}", args[i]));
                }
            }
            i += 1;
        }

        if show_help {
            return self.show_help();
        }

        // Handle unset operations
        for var in &unset_vars {
            env::remove_var(var);
        }

        // Handle set operations
        for (key, value) in &set_vars {
            env::set_var(key, value);
        }

        // Handle queries or display all variables
        if query_vars.is_empty() {
            self.display_all_env_vars(null_separator, ignore_env)
        } else {
            self.display_specific_env_vars(&query_vars, null_separator)
        }
    }
}

impl EnvCommand {
    fn show_help(&self) -> CommandResult {
        let help_text = r#"Usage: env [OPTION]... [NAME[=VALUE]]... [COMMAND [ARG]...]
Set each NAME to VALUE in the environment and run COMMAND.

  -i, --ignore-environment  start with an empty environment
  -0, --null               end each output line with NUL, not newline
  -u, --unset=NAME         remove variable from the environment
      --help     display this help and exit
      --version  output version information and exit

If no COMMAND, print the resulting environment.

Examples:
  env                      Display all environment variables
  env PATH                 Display the PATH variable
  env FOO=bar              Set FOO to 'bar' and display all variables
  env -u PATH              Remove PATH from environment
  env FOO=bar COMMAND      Set FOO and run COMMAND with new environment
"#;
        CommandResult::success(help_text)
    }

    fn display_all_env_vars(&self, null_separator: bool, ignore_env: bool) -> CommandResult {
        let mut output = String::new();
        let separator = if null_separator { '\0' } else { '\n' };

        if ignore_env {
            // When ignoring environment, only show variables we've explicitly set
            // For now, we'll show nothing since we don't track explicitly set vars
            return CommandResult::success("");
        }

        let mut env_vars: Vec<(String, String)> = env::vars().collect();
        env_vars.sort_by(|a, b| a.0.cmp(&b.0));

        for (key, value) in env_vars {
            output.push_str(&format!("{}={}{}", key, value, separator));
        }

        // Remove the trailing separator if present
        if output.ends_with(separator) {
            output.pop();
        }

        CommandResult::success(&output)
    }

    fn display_specific_env_vars(&self, vars: &[String], null_separator: bool) -> CommandResult {
        let mut output = String::new();
        let separator = if null_separator { '\0' } else { '\n' };

        for var in vars {
            match env::var(var) {
                Ok(value) => {
                    output.push_str(&format!("{}={}{}", var, value, separator));
                }
                Err(env::VarError::NotPresent) => {
                    return CommandResult::error(&format!("env: {}: not set", var));
                }
                Err(env::VarError::NotUnicode(_)) => {
                    return CommandResult::error(&format!("env: {}: contains invalid Unicode", var));
                }
            }
        }

        // Remove the trailing separator if present
        if output.ends_with(separator) {
            output.pop();
        }

        CommandResult::success(&output)
    }
}

impl Default for EnvCommand {
    fn default() -> Self {
        Self
    }
}

/// Execute function stub
pub fn execute(_args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}
