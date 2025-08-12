use nxsh_core::{ShellError, ErrorKind}; use nxsh_core::error::RuntimeErrorKind;

pub fn case_cli(args: &[String]) -> Result<(), ShellError> {
    if args.is_empty() || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    // Basic case statement implementation
    // This is a simplified version - full shell case would require complex parsing
    
    if args.len() < 2 {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "case: missing word or pattern"));
    }

    let word = &args[0];
    
    // Find the 'in' keyword
    let in_pos = args.iter().position(|arg| arg == "in")
        .ok_or_else(|| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "case: missing 'in' keyword"))?;
    
    // Parse patterns and execute matching case
    execute_case_statement(word, &args[in_pos + 1..])
}

fn print_help() {
    println!(r#"case - pattern matching conditional construct

USAGE:
    case WORD in
        PATTERN) COMMANDS ;;
        PATTERN) COMMANDS ;;
        *) COMMANDS ;;
    esac

DESCRIPTION:
    The case statement matches WORD against multiple patterns and executes
    the commands associated with the first matching pattern.

PATTERNS:
    Patterns support shell-style globbing:
    - *          : Matches any string
    - ?          : Matches any single character
    - [abc]      : Matches any character in the set
    - [a-z]      : Matches any character in the range
    - pattern|pattern : Matches either pattern (alternation)

EXAMPLES:
    case "$1" in
        start)
            echo "Starting service"
            ;;
        stop)
            echo "Stopping service"
            ;;
        restart)
            echo "Restarting service"
            ;;
        *)
            echo "Usage: $0 {{start|stop|restart}}"
            exit 1
            ;;
    esac

    case "$filename" in
        *.txt)
            echo "Text file"
            ;;
        *.jpg|*.png|*.gif)
            echo "Image file"
            ;;
        *)
            echo "Unknown file type"
            ;;
    esac"#);
}

fn execute_case_statement(word: &str, args: &[String]) -> Result<(), ShellError> {
    let mut i = 0;
    
    while i < args.len() {
        // Look for pattern ending with ')'
        if let Some(pattern_end) = find_pattern_end(&args[i..]) {
            let pattern = &args[i..i + pattern_end];
            let pattern_str = pattern.join(" ");
            
            // Remove the trailing ')'
            let clean_pattern = if pattern_str.ends_with(')') {
                &pattern_str[..pattern_str.len() - 1]
            } else {
                &pattern_str
            };
            
            if pattern_matches(word, clean_pattern)? {
                // Execute commands until we find ';;' or 'esac'
                let command_start = i + pattern_end + 1;
                let command_end = find_command_end(&args[command_start..]);
                
                if command_start < args.len() {
                    let commands = &args[command_start..command_start + command_end];
                    execute_commands(commands)?;
                }
                
                return Ok(()); // Exit after first match
            }
            
            // Move past this pattern and its commands
            i += pattern_end + 1;
            i += find_command_end(&args[i..]);
            
            // Skip the ';;' separator
            if i < args.len() && args[i] == ";;" {
                i += 1;
            }
        } else if args[i] == "esac" {
            break;
        } else {
            i += 1;
        }
    }
    
    Ok(())
}

fn find_pattern_end(args: &[String]) -> Option<usize> {
    for (i, arg) in args.iter().enumerate() {
        if arg.ends_with(')') {
            return Some(i + 1);
        }
        if arg == ")" {
            return Some(i + 1);
        }
    }
    None
}

fn find_command_end(args: &[String]) -> usize {
    for (i, arg) in args.iter().enumerate() {
        if arg == ";;" || arg == "esac" {
            return i;
        }
    }
    args.len()
}

fn pattern_matches(word: &str, pattern: &str) -> Result<bool, ShellError> {
    // Handle alternation (|)
    if pattern.contains('|') {
        for sub_pattern in pattern.split('|') {
            if pattern_matches(word, sub_pattern.trim())? {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    
    // Convert shell glob pattern to regex-like matching
    if pattern == "*" {
        return Ok(true);
    }
    
    if pattern == word {
        return Ok(true);
    }
    
    // Simple glob matching
    glob_match(word, pattern)
}

fn glob_match(text: &str, pattern: &str) -> Result<bool, ShellError> {
    let text_chars: Vec<char> = text.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();
    
    fn match_recursive(text: &[char], pattern: &[char], ti: usize, pi: usize) -> bool {
        // End of pattern
        if pi >= pattern.len() {
            return ti >= text.len();
        }
        
        // End of text but pattern remains
        if ti >= text.len() {
            // Only '*' patterns can match empty string
            return pattern[pi..].iter().all(|&c| c == '*');
        }
        
        match pattern[pi] {
            '*' => {
                // Try matching zero or more characters
                for i in ti..=text.len() {
                    if match_recursive(text, pattern, i, pi + 1) {
                        return true;
                    }
                }
                false
            },
            '?' => {
                // Match exactly one character
                match_recursive(text, pattern, ti + 1, pi + 1)
            },
            '[' => {
                // Character class matching
                if let Some(end_bracket) = pattern[pi..].iter().position(|&c| c == ']') {
                    let class = &pattern[pi + 1..pi + end_bracket];
                    let matches = match_character_class(text[ti], class);
                    if matches {
                        match_recursive(text, pattern, ti + 1, pi + end_bracket + 1)
                    } else {
                        false
                    }
                } else {
                    // Invalid pattern, treat '[' as literal
                    text[ti] == pattern[pi] && match_recursive(text, pattern, ti + 1, pi + 1)
                }
            },
            c => {
                // Literal character match
                text[ti] == c && match_recursive(text, pattern, ti + 1, pi + 1)
            }
        }
    }
    
    Ok(match_recursive(&text_chars, &pattern_chars, 0, 0))
}

fn match_character_class(ch: char, class: &[char]) -> bool {
    let mut i = 0;
    let mut negate = false;
    
    if !class.is_empty() && class[0] == '^' {
        negate = true;
        i = 1;
    }
    
    let mut matched = false;
    
    while i < class.len() {
        if i + 2 < class.len() && class[i + 1] == '-' {
            // Range pattern like a-z
            if ch >= class[i] && ch <= class[i + 2] {
                matched = true;
                break;
            }
            i += 3;
        } else {
            // Single character
            if ch == class[i] {
                matched = true;
                break;
            }
            i += 1;
        }
    }
    
    if negate { !matched } else { matched }
}

fn execute_commands(commands: &[String]) -> Result<(), ShellError> {
    if commands.is_empty() {
        return Ok(());
    }
    
    // This is a simplified command execution
    // In a real shell, this would integrate with the command execution system
    
    // Handle some basic built-in commands
    match commands[0].as_str() {
        "echo" => {
            let output = commands[1..].join(" ");
            println!("{output}");
        },
        "exit" => {
            let code = if commands.len() > 1 {
                commands[1].parse().unwrap_or(0)
            } else {
                0
            };
            std::process::exit(code);
        },
        _ => {
            // For other commands, just simulate execution
            println!("Would execute: {}", commands.join(" "));
        }
    }
    
    Ok(())
}

// Additional helper functions for case statement features
pub fn case_statement_parser(input: &str) -> Result<CaseStatement, ShellError> {
    let lines: Vec<&str> = input.lines().map(|line| line.trim()).collect();
    let mut statement = CaseStatement::new();
    
    let mut i = 0;
    
    // Parse "case WORD in"
    if i < lines.len() && lines[i].starts_with("case ") && lines[i].ends_with(" in") {
        let case_line = lines[i];
        let word_part = &case_line[5..case_line.len() - 3].trim();
        statement.word = word_part.to_string();
        i += 1;
    } else {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid case statement syntax"));
    }
    
    // Parse patterns and commands
    while i < lines.len() && lines[i] != "esac" {
        if lines[i].ends_with(')') {
            let pattern = lines[i][..lines[i].len() - 1].trim().to_string();
            i += 1;
            
            let mut commands = Vec::new();
            while i < lines.len() && lines[i] != ";;" && lines[i] != "esac" {
                if !lines[i].is_empty() {
                    commands.push(lines[i].to_string());
                }
                i += 1;
            }
            
            statement.cases.push(CaseClause { pattern, commands });
            
            if i < lines.len() && lines[i] == ";;" {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    
    Ok(statement)
}

#[derive(Debug)]
pub struct CaseStatement {
    pub word: String,
    pub cases: Vec<CaseClause>,
}

#[derive(Debug)]
pub struct CaseClause {
    pub pattern: String,
    pub commands: Vec<String>,
}

impl Default for CaseStatement {
    fn default() -> Self {
        Self::new()
    }
}

impl CaseStatement {
    pub fn new() -> Self {
        Self {
            word: String::new(),
            cases: Vec::new(),
        }
    }
    
    pub fn execute(&self) -> Result<(), ShellError> {
        for case in &self.cases {
            if pattern_matches(&self.word, &case.pattern)? {
                for command in &case.commands {
                    execute_commands(&[command.clone()])?;
                }
                break;
            }
        }
        Ok(())
    }
}

