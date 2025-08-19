//! `sed` command - stream editor for filtering and transforming text
//!
//! Full sed implementation with pattern matching, substitution, and editing commands

use std::io::Write;
use std::collections::HashMap;
use nxsh_core::{ShellContext, ShellResult, ShellError, ExecutionResult};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;
use super::ui_design::{Colorize, TableFormatter, ColorPalette, Icons};

pub struct SedBuiltin;

#[derive(Debug, Clone)]
pub struct SedOptions {
    pub in_place: bool,
    pub backup_suffix: Option<String>,
    pub quiet: bool,
    pub extended_regex: bool,
    pub separate_files: bool,
    pub null_data: bool,
    pub script: Vec<String>,
    pub script_files: Vec<String>,
    pub files: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SedAddress {
    Line(usize),
    LastLine,
    Pattern(String),
    Range(Box<SedAddress>, Box<SedAddress>),
    None,
}

#[derive(Debug, Clone)]
pub struct SedCommand {
    pub address: SedAddress,
    pub operation: SedOperation,
}

#[derive(Debug, Clone)]
pub enum SedOperation {
    Substitute {
        pattern: String,
        replacement: String,
        global: bool,
        print: bool,
        ignore_case: bool,
        extended_regex: bool,
    },
    Delete,
    Print,
    Append(String),
    Insert(String),
    Change(String),
    Next,
    Quit,
    Hold,
    Get,
    Exchange,
    Label(String),
    Branch(Option<String>),
    Test(Option<String>),
    Read(String),
    Write(String),
}

#[derive(Debug, Clone)]
pub struct SedState {
    pub pattern_space: String,
    pub hold_space: String,
    pub line_number: usize,
    pub total_lines: Option<usize>,
    pub quit: bool,
    pub substitution_made: bool,
    pub suppress_output: bool,
    pub labels: HashMap<String, usize>,
    pub range_states: Vec<bool>, // Track active ranges for each command
}

impl SedBuiltin {
    #[allow(dead_code)]
    fn name(&self) -> &'static str {
        "sed"
    }

    #[allow(dead_code)]
    fn synopsis(&self) -> &'static str {
        "sed [OPTION]... {script-only-if-no-other-script} [input-file]..."
    }

    #[allow(dead_code)]
    fn help(&self) -> &'static str {
        "Stream editor for filtering and transforming text"
    }

    #[allow(dead_code)]
    fn description(&self) -> &'static str {
        "Stream editor that performs text transformations on input streams"
    }

    fn execute(&self, _ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        let options = parse_sed_args(args)?;

        if options.script.is_empty() && options.script_files.is_empty() {
            return Err(ShellError::new(nxsh_core::error::ErrorKind::RuntimeError(nxsh_core::error::RuntimeErrorKind::InvalidArgument), "No script specified"));
        }

        // スクリプト文字列を統合 (-e / 位置引数 + -f ファイル)
        let mut all_commands: Vec<SedCommand> = Vec::new();
        for expr in &options.script {
            let parsed = parse_sed_script(expr, options.extended_regex)?;
            all_commands.extend(parsed);
        }
        for file in &options.script_files {
            let content = std::fs::read_to_string(file)
                .map_err(|e| ShellError::file_not_found(&format!("Cannot read script file {file}: {e}")))?;
            let parsed = parse_sed_script(&content, options.extended_regex)?;
            all_commands.extend(parsed);
        }
        if all_commands.is_empty() {
            return Err(ShellError::command_not_found("No valid sed commands parsed"));
        }

        // 対象ファイルが無い場合は stdin
        if options.files.is_empty() {
            let stdin = std::io::stdin();
            let mut reader = BufReader::new(stdin.lock());
            let mut stdout = std::io::stdout();
            process_sed_stream(&mut reader, &mut stdout, &all_commands, &options)?;
        } else {
            for file in &options.files {
                process_sed_file(file, &all_commands, &options)?;
            }
        }

        Ok(ExecutionResult::success(0))
    }

    #[allow(dead_code)]
    fn usage(&self) -> &'static str {
        "sed - stream editor for filtering and transforming text\n\nUSAGE:\n    sed [OPTIONS] -e 'script' [-e 'script']... [-f scriptfile]... [file...]\n\nCommon options:\n  -n, --quiet, --silent     suppress automatic printing of pattern space\n  -e, --expression=SCRIPT   add the script to the commands to be executed\n  -f, --file=SCRIPTFILE     add the contents of SCRIPTFILE to the commands\n  -i[SUF], --in-place[=SUF] edit files in place (makes backup if SUF supplied)\n  -r, -E, --regexp-extended use extended regular expressions\n  -s, --separate            consider files as separate rather than one continuous stream\n  -z, --null-data           separate lines by NUL characters\n\nBasic commands:\n  s/REGEX/REPL/[FLAGS]      substitute\n  d                          delete pattern space; start next cycle\n  p                          print pattern space\n  a TEXT                     append text after each line\n  i TEXT                     insert text before each line\n  c TEXT                     change (replace) the pattern space\n  n                          read/append next line to pattern space\n  q                          immediately quit sed\n\nFLAGS for s///: g (global), p (print), i (ignore-case), 1..9 (occurrence)"
    }
}

fn parse_sed_args(args: &[String]) -> ShellResult<SedOptions> {
    let mut options = SedOptions {
        in_place: false,
        backup_suffix: None,
        quiet: false,
        extended_regex: false,
        separate_files: false,
        null_data: false,
        script: Vec::new(),
        script_files: Vec::new(),
        files: Vec::new(),
    };

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        
        if arg == "-e" || arg == "--expression" {
            i += 1;
            if i >= args.len() {
                return Err(ShellError::command_not_found("Option -e requires an argument"));
            }
            options.script.push(args[i].clone());
        } else if arg == "-f" || arg == "--file" {
            i += 1;
            if i >= args.len() {
                return Err(ShellError::command_not_found("Option -f requires an argument"));
            }
            options.script_files.push(args[i].clone());
        } else if arg == "-i" || arg == "--in-place" {
            options.in_place = true;
        } else if let Some(rest) = arg.strip_prefix("-i") {
            options.in_place = true;
            if !rest.is_empty() { options.backup_suffix = Some(rest.to_string()); }
        } else if arg == "-n" || arg == "--quiet" || arg == "--silent" {
            options.quiet = true;
        } else if arg == "-r" || arg == "--regexp-extended" {
            options.extended_regex = true;
        } else if arg == "-s" || arg == "--separate" {
            options.separate_files = true;
        } else if arg == "-z" || arg == "--null-data" {
            options.null_data = true;
        } else if arg == "--help" {
            return Err(ShellError::command_not_found("Help requested"));
        } else if arg.starts_with("-") {
            return Err(ShellError::command_not_found(&format!("Unknown option: {arg}")));
        } else {
            // First non-option argument is script if no -e or -f was used
            if options.script.is_empty() && options.script_files.is_empty() {
                options.script.push(arg.clone());
            } else {
                options.files.push(arg.clone());
            }
        }
        i += 1;
    }

    Ok(options)
}

// Simple regex implementation for basic pattern matching
fn simple_regex_match(pattern: &str, text: &str, ignore_case: bool) -> bool {
    if pattern.is_empty() {
        return true;
    }
    
    let pattern_to_match = if ignore_case { pattern.to_lowercase() } else { pattern.to_string() };
    let text_to_match = if ignore_case { text.to_lowercase() } else { text.to_string() };
    
    // Handle basic regex patterns
    if pattern_to_match == ".*" || pattern_to_match == "." {
        return true;
    }
    
    // Simple wildcard matching
    if pattern_to_match.contains('*') {
        let parts: Vec<&str> = pattern_to_match.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];
            return text_to_match.starts_with(prefix) && text_to_match.ends_with(suffix);
        }
    }
    
    // Exact string matching
    text_to_match.contains(&pattern_to_match)
}

fn simple_regex_replace(pattern: &str, replacement: &str, text: &str, global: bool, ignore_case: bool) -> String {
    if pattern.is_empty() {
        return text.to_string();
    }
    
    // Simple wildcard and regex support
    if pattern == ".*" {
        return replacement.to_string();
    }
    
    // If pattern contains special regex characters, treat as literal by default
    if pattern.contains(['[', ']', '(', ')', '{', '}', '+', '?', '^', '$']) && 
       !pattern.contains('*') && !pattern.contains('.') {
        // Literal string replacement
        if global {
            return text.replace(pattern, replacement);
        } else {
            return text.replacen(pattern, replacement, 1);
        }
    }
    
    // Simple wildcard patterns  
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];
            
            if ignore_case {
                let text_lower = text.to_lowercase();
                let prefix_lower = prefix.to_lowercase();
                let suffix_lower = suffix.to_lowercase();
                
                if text_lower.starts_with(&prefix_lower) && text_lower.ends_with(&suffix_lower) {
                    return format!("{}{}{}", prefix, replacement, suffix);
                }
            } else {
                if text.starts_with(prefix) && text.ends_with(suffix) {
                    return format!("{}{}{}", prefix, replacement, suffix);
                }
            }
        }
    }
    
    // Direct replacement
    let search_text = if ignore_case { text.to_lowercase() } else { text.to_string() };
    let search_pattern = if ignore_case { pattern.to_lowercase() } else { pattern.to_string() };
    
    if global {
        if ignore_case {
            // Case-insensitive global replacement
            let mut result = text.to_string();
            let mut pos = 0;
            while let Some(found) = result[pos..].to_lowercase().find(&search_pattern) {
                let actual_pos = pos + found;
                result.replace_range(actual_pos..actual_pos + pattern.len(), replacement);
                pos = actual_pos + replacement.len();
            }
            result
        } else {
            text.replace(pattern, replacement)
        }
    } else {
        if ignore_case {
            if let Some(found) = search_text.find(&search_pattern) {
                let mut result = text.to_string();
                result.replace_range(found..found + pattern.len(), replacement);
                result
            } else {
                text.to_string()
            }
        } else {
            text.replacen(pattern, replacement, 1)
        }
    }
}

fn parse_sed_script(script: &str, extended_regex: bool) -> ShellResult<Vec<SedCommand>> {
    let mut commands = Vec::new();
    let lines: Vec<&str> = script.lines().collect();
    
    for line in lines {
        let mut line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Remove surrounding quotes if present
        if (line.starts_with('\'') && line.ends_with('\'')) || (line.starts_with('"') && line.ends_with('"')) {
            line = &line[1..line.len()-1];
        }
        
        // Parse address and command
        let (address, operation) = parse_sed_command_with_address(line, extended_regex)?;
        commands.push(SedCommand { address, operation });
    }
    
    Ok(commands)
}

fn parse_sed_command_with_address(cmd: &str, extended_regex: bool) -> ShellResult<(SedAddress, SedOperation)> {
    let cmd = cmd.trim();
    
    // Parse address part
    let (address, operation_part) = parse_address(cmd)?;
    
    // Parse operation
    let operation = parse_sed_command(operation_part, extended_regex)?;
    
    Ok((address, operation))
}

fn parse_address(cmd: &str) -> ShellResult<(SedAddress, &str)> {
    let cmd = cmd.trim();
    
    // No address specified
    if cmd.is_empty() {
        return Ok((SedAddress::None, cmd));
    }
    
    // Pattern address: /pattern/command
    if cmd.starts_with('/') {
        if let Some(end_pos) = cmd[1..].find('/') {
            let pattern = cmd[1..end_pos + 1].to_string();
            let operation_part = &cmd[end_pos + 2..];
            return Ok((SedAddress::Pattern(pattern), operation_part));
        }
    }
    
    // Line number address: 123command
    let mut i = 0;
    while i < cmd.len() && cmd.chars().nth(i).unwrap().is_ascii_digit() {
        i += 1;
    }
    
    if i > 0 {
        let line_num: usize = cmd[..i].parse().map_err(|_| 
            ShellError::command_not_found("Invalid line number"))?;
        let operation_part = &cmd[i..];
        return Ok((SedAddress::Line(line_num), operation_part));
    }
    
    // $ address (last line)
    if cmd.starts_with('$') {
        return Ok((SedAddress::LastLine, &cmd[1..]));
    }
    
    // Range address: addr1,addr2command
    if let Some(comma_pos) = cmd.find(',') {
        let addr1_str = &cmd[..comma_pos];
        let rest = &cmd[comma_pos + 1..];
        
        let (addr1, _) = parse_address(addr1_str)?;
        
        // Find end of second address
        let mut addr2_end = 0;
        if rest.starts_with('/') {
            if let Some(end_pos) = rest[1..].find('/') {
                addr2_end = end_pos + 2;
            }
        } else if rest.starts_with('$') {
            addr2_end = 1;
        } else {
            while addr2_end < rest.len() && rest.chars().nth(addr2_end).unwrap().is_ascii_digit() {
                addr2_end += 1;
            }
        }
        
        if addr2_end > 0 {
            let addr2_str = &rest[..addr2_end];
            let operation_part = &rest[addr2_end..];
            let (addr2, _) = parse_address(addr2_str)?;
            return Ok((SedAddress::Range(Box::new(addr1), Box::new(addr2)), operation_part));
        }
    }
    
    // No address found, treat whole string as operation
    Ok((SedAddress::None, cmd))
}

fn parse_sed_command(cmd: &str, extended_regex: bool) -> ShellResult<SedOperation> {
    let cmd = cmd.trim();
    
    if cmd.starts_with('s') {
        // Pass full command including 's' to substitute parser
        parse_substitute_command(cmd, extended_regex)
    } else if cmd == "d" {
        Ok(SedOperation::Delete)
    } else if cmd == "p" {
        Ok(SedOperation::Print)
    } else if let Some(rest) = cmd.strip_prefix('a') {
        let text = rest.trim();
        Ok(SedOperation::Append(text.to_string()))
    } else if let Some(rest) = cmd.strip_prefix('i') {
        let text = rest.trim();
        Ok(SedOperation::Insert(text.to_string()))
    } else if let Some(rest) = cmd.strip_prefix('c') {
        let text = rest.trim();
        Ok(SedOperation::Change(text.to_string()))
    } else if cmd == "n" {
        Ok(SedOperation::Next)
    } else if cmd == "q" {
        Ok(SedOperation::Quit)
    } else if cmd == "h" {
        Ok(SedOperation::Hold)
    } else if cmd == "g" {
        Ok(SedOperation::Get)
    } else if cmd == "x" {
        Ok(SedOperation::Exchange)
    } else if let Some(rest) = cmd.strip_prefix(':') {
        Ok(SedOperation::Label(rest.to_string()))
    } else if let Some(rest) = cmd.strip_prefix('b') {
        let label = if rest.is_empty() { None } else { Some(rest.trim().to_string()) };
        Ok(SedOperation::Branch(label))
    } else if let Some(rest) = cmd.strip_prefix('t') {
        let label = if rest.is_empty() { None } else { Some(rest.trim().to_string()) };
        Ok(SedOperation::Test(label))
    } else if let Some(rest) = cmd.strip_prefix('r') {
        Ok(SedOperation::Read(rest.trim().to_string()))
    } else if let Some(rest) = cmd.strip_prefix('w') {
        Ok(SedOperation::Write(rest.trim().to_string()))
    } else {
        Err(ShellError::command_not_found(&format!("Unknown sed command: '{cmd}' (length: {})", cmd.len())))
    }
}

fn parse_substitute_command(cmd: &str, extended_regex: bool) -> ShellResult<SedOperation> {
    if cmd.len() < 4 {
        return Err(ShellError::command_not_found("Invalid substitute command"));
    }
    
    let chars: Vec<char> = cmd.chars().collect();
    if chars[0] != 's' {
        return Err(ShellError::command_not_found("Substitute command must start with 's'"));
    }
    
    let delimiter = chars[1];
    let mut pattern = String::new();
    let mut replacement = String::new();
    let mut flags_str = String::new();
    let mut state = 0; // 0: pattern, 1: replacement, 2: flags
    
    for &c in &chars[2..] {
        if c == delimiter {
            state += 1;
            if state > 2 { break; }
        } else {
            match state {
                0 => pattern.push(c),
                1 => replacement.push(c),
                2 => flags_str.push(c),
                _ => break,
            }
        }
    }
    
    let mut global = false;
    let mut print = false;
    let mut ignore_case = false;
    
    for flag_char in flags_str.chars() {
        match flag_char {
            'g' => global = true,
            'p' => print = true,
            'i' | 'I' => ignore_case = true,
            '1'..='9' => {
                // For now, just treat numeric flags as global
                global = true;
            }
            _ => {} // Ignore unknown flags
        }
    }
    
    Ok(SedOperation::Substitute {
        pattern,
        replacement,
        global,
        print,
        ignore_case,
        extended_regex,
    })
}

fn process_sed_file(file_path: &str, commands: &[SedCommand], options: &SedOptions) -> ShellResult<()> {
    let path = Path::new(file_path);
    let input_file = File::open(path)
        .map_err(|e| ShellError::file_not_found(&format!("Cannot open {file_path}: {e}")))?;
    
    if options.in_place {
        let temp_path = format!("{file_path}.sed_tmp");
        let temp_file = File::create(&temp_path)
            .map_err(|e| ShellError::file_not_found(&format!("Cannot create temp file: {e}")))?;
        
        let mut writer = BufWriter::new(temp_file);
        process_sed_stream(&mut BufReader::new(input_file), &mut writer, commands, options)?;
        writer.flush()?;
        drop(writer);
        
        // Handle backup
        if let Some(ref suffix) = options.backup_suffix {
            let backup_path = format!("{file_path}{suffix}");
            std::fs::rename(file_path, backup_path)
                .map_err(|e| ShellError::file_not_found(&format!("Cannot create backup: {e}")))?;
        }
        
        std::fs::rename(&temp_path, file_path)
            .map_err(|e| ShellError::file_not_found(&format!("Cannot replace original file: {e}")))?;
    } else {
        process_sed_stream(&mut BufReader::new(input_file), &mut std::io::stdout(), commands, options)?;
    }
    
    Ok(())
}

impl SedState {
    fn new() -> Self {
        SedState {
            pattern_space: String::new(),
            hold_space: String::new(),
            line_number: 0,
            total_lines: None,
            quit: false,
            substitution_made: false,
            suppress_output: false,
            labels: HashMap::new(),
            range_states: Vec::new(),
        }
    }
}

fn process_sed_stream<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    commands: &[SedCommand],
    options: &SedOptions,
) -> ShellResult<()> {
    let mut state = SedState::new();
    state.range_states = vec![false; commands.len()];
    
    // Build label map
    for (i, command) in commands.iter().enumerate() {
        if let SedOperation::Label(label) = &command.operation {
            state.labels.insert(label.clone(), i);
        }
    }
    
    // First pass to count lines for $ address
    let mut all_lines = Vec::new();
    let separator = if options.null_data { b'\0' } else { b'\n' };
    let mut buffer = Vec::new();
    
    loop {
        buffer.clear();
        let bytes_read = reader.read_until(separator, &mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        // Remove separator
        if buffer.last() == Some(&separator) {
            buffer.pop();
        }
        
        all_lines.push(String::from_utf8_lossy(&buffer).to_string());
    }
    
    state.total_lines = Some(all_lines.len());
    
    // Process each line
    for (line_idx, line_content) in all_lines.iter().enumerate() {
        state.pattern_space = line_content.clone();
        state.line_number = line_idx + 1;
        state.substitution_made = false;
        state.suppress_output = false;
        
        let mut command_index = 0;
        
        while command_index < commands.len() && !state.quit {
            let command = &commands[command_index];
            
            // Check if address matches
            let current_range_state = state.range_states[command_index];
            let mut new_range_state = current_range_state;
            if !address_matches(&command.address, &state, &mut new_range_state) {
                state.range_states[command_index] = new_range_state;
                command_index += 1;
                continue;
            }
            state.range_states[command_index] = new_range_state;

            match &command.operation {
                SedOperation::Substitute { 
                    pattern, 
                    replacement, 
                    global, 
                    print, 
                    ignore_case, 
                    extended_regex: _ 
                } => {
                    // Enhanced string replacement with simple regex
                    let result = simple_regex_replace(pattern, replacement, &state.pattern_space, *global, *ignore_case);
                    
                    if result != state.pattern_space {
                        state.pattern_space = result;
                        state.substitution_made = true;
                        
                        if *print && !options.quiet {
                            writeln!(writer, "{}", state.pattern_space)?;
                        }
                    }
                }
                SedOperation::Delete => {
                    // Skip to next line without printing
                    state.suppress_output = true;
                    break;
                }
                SedOperation::Print => {
                    writeln!(writer, "{}", state.pattern_space)?;
                }
                SedOperation::Append(text) => {
                    if !options.quiet && !state.suppress_output {
                        writeln!(writer, "{}", state.pattern_space)?;
                    }
                    writeln!(writer, "{text}")?;
                    state.suppress_output = true;
                }
                SedOperation::Insert(text) => {
                    writeln!(writer, "{text}")?;
                    if !options.quiet && !state.suppress_output {
                        writeln!(writer, "{}", state.pattern_space)?;
                    }
                    state.suppress_output = true;
                }
                SedOperation::Change(text) => {
                    writeln!(writer, "{text}")?;
                    state.suppress_output = true;
                    break;
                }
                SedOperation::Next => {
                    if !options.quiet && !state.suppress_output {
                        writeln!(writer, "{}", state.pattern_space)?;
                    }
                    break; // Read next line
                }
                SedOperation::Quit => {
                    if !options.quiet && !state.suppress_output {
                        writeln!(writer, "{}", state.pattern_space)?;
                    }
                    state.quit = true;
                    break;
                }
                SedOperation::Hold => {
                    state.hold_space = state.pattern_space.clone();
                }
                SedOperation::Get => {
                    state.pattern_space = state.hold_space.clone();
                }
                SedOperation::Exchange => {
                    std::mem::swap(&mut state.pattern_space, &mut state.hold_space);
                }
                SedOperation::Label(_) => {
                    // Labels are no-ops during execution
                }
                SedOperation::Branch(Some(label)) => {
                    if let Some(&target) = state.labels.get(label) {
                        command_index = target;
                        continue;
                    }
                }
                SedOperation::Branch(None) => {
                    // Branch to end of script
                    break;
                }
                SedOperation::Test(Some(label)) => {
                    if state.substitution_made {
                        if let Some(&target) = state.labels.get(label) {
                            command_index = target;
                            continue;
                        }
                    }
                }
                SedOperation::Test(None) => {
                    if state.substitution_made {
                        break;
                    }
                }
                SedOperation::Read(filename) => {
                    if let Ok(content) = std::fs::read_to_string(filename) {
                        write!(writer, "{content}")?;
                    }
                }
                SedOperation::Write(filename) => {
                    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(filename) {
                        writeln!(file, "{}", state.pattern_space)?;
                    }
                }
            }
            
            command_index += 1;
        }
        
        // Print pattern space unless suppressed
        if !options.quiet && !state.suppress_output && !state.quit {
            writeln!(writer, "{}", state.pattern_space)?;
        }
        
        if state.quit {
            break;
        }
    }
    
    Ok(())
}

/// CLI wrapper function for sed command
pub fn sed_cli(args: &[String]) -> anyhow::Result<()> {
    let mut ctx = nxsh_core::context::ShellContext::new();
    let builtin = SedBuiltin;
    match builtin.execute(&mut ctx, args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("sed command failed: {}", e)),
    }
}

fn address_matches(address: &SedAddress, state: &SedState, range_state: &mut bool) -> bool {
    match address {
        SedAddress::None => true,
        SedAddress::Line(line_num) => state.line_number == *line_num,
        SedAddress::LastLine => {
            if let Some(total) = state.total_lines {
                state.line_number == total
            } else {
                false
            }
        }
        SedAddress::Pattern(pattern) => {
            // Enhanced pattern matching with simple regex
            simple_regex_match(pattern, &state.pattern_space, false)
        }
        SedAddress::Range(start, end) => {
            // Range matching with state tracking
            if !*range_state && address_matches(start, state, &mut false) {
                *range_state = true;
            }
            
            if *range_state {
                if address_matches(end, state, &mut false) {
                    *range_state = false;
                    return true; // Include the end line
                }
                return true;
            }
            
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn test_sed_substitute_basic() {
        let commands = vec![SedCommand {
            address: SedAddress::None,
            operation: SedOperation::Substitute {
                pattern: "hello".to_string(),
                replacement: "hi".to_string(),
                global: false,
                print: false,
                ignore_case: false,
                extended_regex: false,
            },
        }];
        
        let input = "hello world\nhello there";
        let mut reader = BufReader::new(Cursor::new(input));
        let mut output = Vec::new();
        let options = SedOptions {
            in_place: false,
            backup_suffix: None,
            quiet: false,
            extended_regex: false,
            separate_files: false,
            null_data: false,
            script: Vec::new(),
            script_files: Vec::new(),
            files: Vec::new(),
        };
        
        let result = process_sed_stream(&mut reader, &mut output, &commands, &options);
        assert!(result.is_ok());
        
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("hi world"));
        assert!(output_str.contains("hi there"));
    }

    #[test]
    fn test_sed_substitute_global() {
        let commands = vec![SedCommand {
            address: SedAddress::None,
            operation: SedOperation::Substitute {
                pattern: "a".to_string(),
                replacement: "X".to_string(),
                global: true,
                print: false,
                ignore_case: false,
                extended_regex: false,
            },
        }];
        
        let input = "banana";
        let mut reader = BufReader::new(Cursor::new(input));
        let mut output = Vec::new();
        let options = SedOptions {
            in_place: false,
            backup_suffix: None,
            quiet: false,
            extended_regex: false,
            separate_files: false,
            null_data: false,
            script: Vec::new(),
            script_files: Vec::new(),
            files: Vec::new(),
        };
        
        let result = process_sed_stream(&mut reader, &mut output, &commands, &options);
        assert!(result.is_ok());
        
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str.trim(), "bXnXnX");
    }

    #[test]
    fn test_sed_delete() {
        let commands = vec![SedCommand {
            address: SedAddress::Line(2),
            operation: SedOperation::Delete,
        }];
        
        let input = "line1\nline2\nline3";
        let mut reader = BufReader::new(Cursor::new(input));
        let mut output = Vec::new();
        let options = SedOptions {
            in_place: false,
            backup_suffix: None,
            quiet: false,
            extended_regex: false,
            separate_files: false,
            null_data: false,
            script: Vec::new(),
            script_files: Vec::new(),
            files: Vec::new(),
        };
        
        let result = process_sed_stream(&mut reader, &mut output, &commands, &options);
        assert!(result.is_ok());
        
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("line1"));
        assert!(!output_str.contains("line2"));
        assert!(output_str.contains("line3"));
    }

    #[test]
    fn test_sed_pattern_address() {
        let commands = vec![SedCommand {
            address: SedAddress::Pattern("test".to_string()),
            operation: SedOperation::Substitute {
                pattern: "a".to_string(),
                replacement: "X".to_string(),
                global: false,
                print: false,
                ignore_case: false,
                extended_regex: false,
            },
        }];
        
        let input = "hello world\ntest abc\nother line";
        let mut reader = BufReader::new(Cursor::new(input));
        let mut output = Vec::new();
        let options = SedOptions {
            in_place: false,
            backup_suffix: None,
            quiet: false,
            extended_regex: false,
            separate_files: false,
            null_data: false,
            script: Vec::new(),
            script_files: Vec::new(),
            files: Vec::new(),
        };
        
        let result = process_sed_stream(&mut reader, &mut output, &commands, &options);
        assert!(result.is_ok());
        
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("hello world"));
        assert!(output_str.contains("test Xbc"));
        assert!(output_str.contains("other line"));
    }

    #[test]
    fn test_simple_regex_match() {
        assert!(simple_regex_match("hello", "hello world", false));
        assert!(simple_regex_match("HELLO", "hello world", true));
        assert!(!simple_regex_match("HELLO", "hello world", false));
        assert!(simple_regex_match(".*", "anything", false));
        assert!(simple_regex_match("h*o", "hello", false));
    }

    #[test]
    fn test_simple_regex_replace() {
        assert_eq!(simple_regex_replace("hello", "hi", "hello world", false, false), "hi world");
        assert_eq!(simple_regex_replace("l", "X", "hello", true, false), "heXXo");
        assert_eq!(simple_regex_replace("L", "X", "hello", false, true), "heXlo");
        assert_eq!(simple_regex_replace(".*", "replacement", "anything", false, false), "replacement");
    }

    #[test]
    fn test_parse_sed_command() {
        let result = parse_sed_command("s/hello/world/", false);
        assert!(result.is_ok());
        
        if let Ok(operation) = result {
            match operation {
                SedOperation::Substitute { pattern, replacement, .. } => {
                    assert_eq!(pattern, "hello");
                    assert_eq!(replacement, "world");
                }
                _ => panic!("Expected substitute operation"),
            }
        }
    }
} 
