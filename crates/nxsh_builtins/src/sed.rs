//! `sed` command - stream editor for filtering and transforming text
//!
//! Full sed implementation with pattern matching, substitution, and editing commands

use std::io::Write;
use std::collections::HashMap;
use nxsh_core::{ShellContext, ShellResult, ShellError, ExecutionResult};
#[cfg(feature = "advanced-regex")]
use regex::{Regex, RegexBuilder};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;

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
pub enum SedOperation {
    Substitute {
        pattern: Regex,
        replacement: String,
        flags: SubstituteFlags,
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

/// Sed builtin command
#[derive(Debug, Clone)]
pub struct SedCommand;

#[derive(Debug, Clone)]
pub struct SubstituteFlags {
    pub global: bool,
    pub print: bool,
    pub write_file: Option<String>,
    pub occurrence: Option<usize>,
    pub ignore_case: bool,
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
        let mut all_commands: Vec<SedOperation> = Vec::new();
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

impl SedCommand {
    /// Create a new sed command instance
    pub fn new() -> Self {
        SedCommand
    }
}

impl Default for SedCommand {
    fn default() -> Self {
        Self::new()
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

fn parse_sed_script(script: &str, extended_regex: bool) -> ShellResult<Vec<SedOperation>> {
    let mut commands = Vec::new();
    let lines: Vec<&str> = script.lines().collect();
    
    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        let command = parse_sed_command(line, extended_regex)?;
        commands.push(command);
    }
    
    Ok(commands)
}

fn parse_sed_command(cmd: &str, extended_regex: bool) -> ShellResult<SedOperation> {
    let cmd = cmd.trim();
    
    if let Some(rest) = cmd.strip_prefix('s') {
        // reconstruct full command for substitute parser (needs leading 's')
        parse_substitute_command(&format!("s{rest}"), extended_regex)
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
        Err(ShellError::command_not_found(&format!("Unknown sed command: {cmd}")))
    }
}

fn parse_substitute_command(cmd: &str, _extended_regex: bool) -> ShellResult<SedOperation> {
    #[cfg(not(feature = "advanced-regex"))]
    {
        return Err(ShellError::new(nxsh_core::error::ErrorKind::RuntimeError(nxsh_core::error::RuntimeErrorKind::UnsupportedFeature), "sed substitute: advanced-regex feature disabled"));
    }
    #[cfg(feature = "advanced-regex")]
    {
    if cmd.len() < 4 {
        return Err(ShellError::command_not_found("Invalid substitute command"));
    }
    
    let delimiter = cmd.chars().nth(1).unwrap();
    let parts: Vec<&str> = cmd[2..].split(delimiter).collect();
    
    if parts.len() < 2 {
        return Err(ShellError::command_not_found("Invalid substitute command format"));
    }
    
    let pattern_str = parts[0];
    let replacement = parts[1].to_string();
    let flags_str = if parts.len() > 2 { parts[2] } else { "" };
    
    let mut regex_builder = RegexBuilder::new(pattern_str);
    
    let mut flags = SubstituteFlags {
        global: false,
        print: false,
        write_file: None,
        occurrence: None,
        ignore_case: false,
    };
    
    for flag_char in flags_str.chars() {
        match flag_char {
            'g' => flags.global = true,
            'p' => flags.print = true,
            'i' | 'I' => {
                flags.ignore_case = true;
                regex_builder.case_insensitive(true);
            }
            '1'..='9' => {
                let occurrence = flag_char.to_digit(10).unwrap() as usize;
                flags.occurrence = Some(occurrence);
            }
            _ => {} // Ignore unknown flags
        }
    }
    
    let pattern = regex_builder.build()
        .map_err(|e| ShellError::command_not_found(&format!("Invalid regex pattern: {e}")))?;
    
    Ok(SedOperation::Substitute {
        pattern,
        replacement,
        flags,
    })
    }
}

fn process_sed_file(file_path: &str, commands: &[SedOperation], options: &SedOptions) -> ShellResult<()> {
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

fn process_sed_stream<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    commands: &[SedOperation],
    options: &SedOptions,
) -> ShellResult<()> {
    // pattern_space は最初の読み込み後に生成することで初期不要代入を避ける
    let mut pattern_space: String; 
    let mut hold_space = String::new();
    let mut _line_number = 0; // placeholder until address range logic uses it
    let mut quit = false;
    let mut labels: HashMap<String, usize> = HashMap::new();
    
    // Build label map
    for (i, command) in commands.iter().enumerate() {
        if let SedOperation::Label(label) = command {
            labels.insert(label.clone(), i);
        }
    }
    
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
        
    // Convert buffer to pattern_space after reading to avoid unused init warning
    pattern_space = String::from_utf8_lossy(&buffer).to_string();
    _line_number += 1; // keep increment so future address/range features can rely on it
        
        let mut command_index = 0;
        let mut substitution_made = false;
        
        while command_index < commands.len() && !quit {
            match &commands[command_index] {
                SedOperation::Substitute { pattern, replacement, flags } => {
                    let result = if flags.global {
                        pattern.replace_all(&pattern_space, replacement.as_str())
                    } else if let Some(occurrence) = flags.occurrence {
                        // Replace specific occurrence
                        let mut count = 0;
                        pattern.replace(&pattern_space, |_: &regex::Captures| {
                            count += 1;
                            if count == occurrence {
                                replacement.as_str()
                            } else {
                                ""
                            }
                        })
                    } else {
                        pattern.replace(&pattern_space, replacement.as_str())
                    };
                    
                    if result != pattern_space {
                        pattern_space = result.to_string();
                        substitution_made = true;
                        
                        if flags.print && !options.quiet {
                            writeln!(writer, "{pattern_space}")?;
                        }
                    }
                }
                SedOperation::Delete => {
                    // Skip to next line without printing
                    command_index = commands.len();
                    continue;
                }
                SedOperation::Print => {
                    writeln!(writer, "{pattern_space}")?;
                }
                SedOperation::Append(text) => {
                    if !options.quiet {
                        writeln!(writer, "{pattern_space}")?;
                    }
                    writeln!(writer, "{text}")?;
                    command_index = commands.len();
                    continue;
                }
                SedOperation::Insert(text) => {
                    writeln!(writer, "{text}")?;
                    if !options.quiet {
                        writeln!(writer, "{pattern_space}")?;
                    }
                    command_index = commands.len();
                    continue;
                }
                SedOperation::Change(text) => {
                    writeln!(writer, "{text}")?;
                    command_index = commands.len();
                    continue;
                }
                SedOperation::Next => {
                    if !options.quiet {
                        writeln!(writer, "{pattern_space}")?;
                    }
                    break; // Read next line
                }
                SedOperation::Quit => {
                    if !options.quiet {
                        writeln!(writer, "{pattern_space}")?;
                    }
                    quit = true;
                    break;
                }
                SedOperation::Hold => {
                    hold_space = pattern_space.clone();
                }
                SedOperation::Get => {
                    pattern_space = hold_space.clone();
                }
                SedOperation::Exchange => {
                    std::mem::swap(&mut pattern_space, &mut hold_space);
                }
                SedOperation::Label(_) => {
                    // Labels are processed during execution, skip
                }
                SedOperation::Branch(label) => {
                    if let Some(label) = label {
                        if let Some(&target) = labels.get(label) {
                            command_index = target;
                            continue;
                        }
                    } else {
                        // Branch to end
                        break;
                    }
                }
                SedOperation::Test(label) => {
                    if substitution_made {
                        if let Some(label) = label {
                            if let Some(&target) = labels.get(label) {
                                command_index = target;
                                continue;
                            }
                        } else {
                            break;
                        }
                    }
                }
                SedOperation::Read(filename) => {
                    if let Ok(content) = std::fs::read_to_string(filename) {
                        write!(writer, "{content}")?;
                    }
                }
                SedOperation::Write(filename) => {
                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(filename)
                        .map_err(|e| ShellError::file_not_found(&format!("Cannot write to {filename}: {e}")))?;
                    writeln!(file, "{pattern_space}")?;
                }
            }
            command_index += 1;
        }
        
        if !options.quiet && command_index >= commands.len() {
            writeln!(writer, "{pattern_space}")?;
        }
        
        if quit {
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
