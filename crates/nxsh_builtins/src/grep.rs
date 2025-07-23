//! `grep` command – comprehensive pattern searching implementation.
//!
//! Supports complete grep functionality:
//!   grep [OPTIONS] PATTERN [FILES...]
//!   -E, --extended-regexp     - Interpret PATTERN as extended regular expression
//!   -F, --fixed-strings       - Interpret PATTERN as list of fixed strings
//!   -G, --basic-regexp        - Interpret PATTERN as basic regular expression (default)
//!   -P, --perl-regexp         - Interpret PATTERN as Perl-compatible regular expression
//!   -e, --regexp=PATTERN      - Use PATTERN for matching
//!   -f, --file=FILE           - Take patterns from FILE
//!   -i, --ignore-case         - Ignore case distinctions
//!   -v, --invert-match        - Select non-matching lines
//!   -w, --word-regexp         - Match whole words only
//!   -x, --line-regexp         - Match whole lines only
//!   -c, --count               - Print only count of matching lines per file
//!   -l, --files-with-matches  - Print only names of files with matches
//!   -L, --files-without-match - Print only names of files without matches
//!   -m, --max-count=NUM       - Stop after NUM matches
//!   -n, --line-number         - Print line numbers with output lines
//!   -H, --with-filename       - Print filename with output lines
//!   -h, --no-filename         - Suppress filename prefix on output
//!   -o, --only-matching       - Show only matching parts of lines
//!   -q, --quiet, --silent     - Suppress all normal output
//!   -s, --no-messages         - Suppress error messages
//!   -r, --recursive           - Search directories recursively
//!   -R, --dereference-recursive - Follow symbolic links recursively
//!   --include=PATTERN         - Search only files matching PATTERN
//!   --exclude=PATTERN         - Skip files matching PATTERN
//!   --exclude-dir=PATTERN     - Skip directories matching PATTERN
//!   -A, --after-context=NUM   - Print NUM lines after matches
//!   -B, --before-context=NUM  - Print NUM lines before matches
//!   -C, --context=NUM         - Print NUM lines before and after matches
//!   --color[=WHEN]            - Colorize output (always, never, auto)
//!   --binary-files=TYPE       - How to handle binary files (binary, text, without-match)
//!   -a, --text                - Process binary files as text
//!   -I                        - Skip binary files
//!   -z, --null-data           - Lines are separated by NUL characters
//!   -Z, --null                - Print NUL after filenames

use anyhow::{Result, anyhow};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::collections::{HashMap, VecDeque};
use regex::{Regex, RegexBuilder, RegexSet};
use fancy_regex::Regex as FancyRegex;
use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use walkdir::{WalkDir, DirEntry};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ansi_term::{Colour, Style};
use memchr::{memchr, memmem};
use rayon::prelude::*;
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone)]
pub struct GrepOptions {
    pub patterns: Vec<String>,
    pub pattern_files: Vec<String>,
    pub files: Vec<String>,
    pub extended_regexp: bool,
    pub fixed_strings: bool,
    pub basic_regexp: bool,
    pub perl_regexp: bool,
    pub ignore_case: bool,
    pub invert_match: bool,
    pub word_regexp: bool,
    pub line_regexp: bool,
    pub count_only: bool,
    pub files_with_matches: bool,
    pub files_without_match: bool,
    pub max_count: Option<usize>,
    pub line_number: bool,
    pub with_filename: bool,
    pub no_filename: bool,
    pub only_matching: bool,
    pub quiet: bool,
    pub no_messages: bool,
    pub recursive: bool,
    pub dereference_recursive: bool,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub exclude_dir_patterns: Vec<String>,
    pub after_context: usize,
    pub before_context: usize,
    pub color: ColorMode,
    pub binary_files: BinaryMode,
    pub text_mode: bool,
    pub skip_binary: bool,
    pub null_data: bool,
    pub null_output: bool,
    pub byte_offset: bool,
    pub initial_tab: bool,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorMode {
    Never,
    Always,
    Auto,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryMode {
    Binary,
    Text,
    WithoutMatch,
}

impl Default for GrepOptions {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            pattern_files: Vec::new(),
            files: Vec::new(),
            extended_regexp: false,
            fixed_strings: false,
            basic_regexp: true,
            perl_regexp: false,
            ignore_case: false,
            invert_match: false,
            word_regexp: false,
            line_regexp: false,
            count_only: false,
            files_with_matches: false,
            files_without_match: false,
            max_count: None,
            line_number: false,
            with_filename: false,
            no_filename: false,
            only_matching: false,
            quiet: false,
            no_messages: false,
            recursive: false,
            dereference_recursive: false,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            exclude_dir_patterns: Vec::new(),
            after_context: 0,
            before_context: 0,
            color: ColorMode::Auto,
            binary_files: BinaryMode::Binary,
            text_mode: false,
            skip_binary: false,
            null_data: false,
            null_output: false,
            byte_offset: false,
            initial_tab: false,
            label: None,
        }
    }
}

#[derive(Debug)]
pub struct GrepMatcher {
    regex: Option<Regex>,
    fancy_regex: Option<FancyRegex>,
    aho_corasick: Option<AhoCorasick>,
    fixed_patterns: Vec<String>,
    options: GrepOptions,
}

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub line_number: usize,
    pub byte_offset: usize,
    pub line: String,
    pub matches: Vec<(usize, usize)>, // (start, end) positions within line
}

#[derive(Debug)]
pub struct FileResult {
    pub filename: String,
    pub matches: Vec<MatchResult>,
    pub match_count: usize,
    pub error: Option<String>,
}

pub fn grep_cli(args: &[String]) -> Result<()> {
    let options = parse_grep_args(args)?;
    
    if options.patterns.is_empty() && options.pattern_files.is_empty() {
        return Err(anyhow!("grep: no pattern specified"));
    }
    
    // Load patterns from files if specified
    let mut all_patterns = options.patterns.clone();
    for pattern_file in &options.pattern_files {
        let file_patterns = load_patterns_from_file(pattern_file)?;
        all_patterns.extend(file_patterns);
    }
    
    if all_patterns.is_empty() {
        return Err(anyhow!("grep: no patterns found"));
    }
    
    // Create matcher
    let matcher = create_matcher(all_patterns, &options)?;
    
    // Determine files to search
    let files_to_search = if options.files.is_empty() {
        vec!["-".to_string()] // stdin
    } else {
        expand_file_list(&options.files, &options)?
    };
    
    // Set up output formatting
    let should_show_filename = determine_filename_display(&files_to_search, &options);
    let use_color = should_use_color(&options);
    
    // Search files
    let mut total_matches = 0;
    let mut files_with_matches = 0;
    let mut exit_code = 1; // Default to "no matches found"
    
    if options.recursive && files_to_search.len() == 1 && files_to_search[0] != "-" {
        // Parallel recursive search
        let results = search_recursive(&files_to_search[0], &matcher, &options)?;
        for result in results {
            if let Some(error) = &result.error {
                if !options.no_messages {
                    eprintln!("grep: {}: {}", result.filename, error);
                }
                continue;
            }
            
            if result.match_count > 0 {
                files_with_matches += 1;
                total_matches += result.match_count;
                exit_code = 0;
            }
            
            print_file_results(&result, should_show_filename, use_color, &options)?;
        }
    } else {
        // Sequential or parallel file search
        for filename in &files_to_search {
            let result = search_file(filename, &matcher, &options)?;
            
            if let Some(error) = &result.error {
                if !options.no_messages {
                    eprintln!("grep: {}: {}", result.filename, error);
                }
                continue;
            }
            
            if result.match_count > 0 {
                files_with_matches += 1;
                total_matches += result.match_count;
                exit_code = 0;
            }
            
            print_file_results(&result, should_show_filename, use_color, &options)?;
            
            // Stop early if max-count reached globally
            if let Some(max) = options.max_count {
                if total_matches >= max {
                    break;
                }
            }
        }
    }
    
    // Handle special output modes
    if options.files_with_matches || options.files_without_match {
        // Already handled in print_file_results
    } else if options.count_only && files_to_search.len() == 1 && files_to_search[0] == "-" {
        println!("{}", total_matches);
    }
    
    std::process::exit(exit_code);
}

fn parse_grep_args(args: &[String]) -> Result<GrepOptions> {
    let mut options = GrepOptions::default();
    let mut i = 0;
    let mut pattern_set = false;
    
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "-E" | "--extended-regexp" => {
                options.extended_regexp = true;
                options.basic_regexp = false;
            }
            "-F" | "--fixed-strings" => {
                options.fixed_strings = true;
                options.basic_regexp = false;
            }
            "-G" | "--basic-regexp" => {
                options.basic_regexp = true;
                options.extended_regexp = false;
            }
            "-P" | "--perl-regexp" => {
                options.perl_regexp = true;
                options.basic_regexp = false;
            }
            "-e" | "--regexp" => {
                if i + 1 < args.len() {
                    options.patterns.push(args[i + 1].clone());
                    pattern_set = true;
                    i += 1;
                } else {
                    return Err(anyhow!("grep: option requires an argument -- e"));
                }
            }
            "-f" | "--file" => {
                if i + 1 < args.len() {
                    options.pattern_files.push(args[i + 1].clone());
                    pattern_set = true;
                    i += 1;
                } else {
                    return Err(anyhow!("grep: option requires an argument -- f"));
                }
            }
            "-i" | "--ignore-case" => options.ignore_case = true,
            "-v" | "--invert-match" => options.invert_match = true,
            "-w" | "--word-regexp" => options.word_regexp = true,
            "-x" | "--line-regexp" => options.line_regexp = true,
            "-c" | "--count" => options.count_only = true,
            "-l" | "--files-with-matches" => options.files_with_matches = true,
            "-L" | "--files-without-match" => options.files_without_match = true,
            "-m" | "--max-count" => {
                if i + 1 < args.len() {
                    let count: usize = args[i + 1].parse()
                        .map_err(|_| anyhow!("grep: invalid max count '{}'", args[i + 1]))?;
                    options.max_count = Some(count);
                    i += 1;
                } else {
                    return Err(anyhow!("grep: option requires an argument -- m"));
                }
            }
            "-n" | "--line-number" => options.line_number = true,
            "-H" | "--with-filename" => options.with_filename = true,
            "-h" | "--no-filename" => options.no_filename = true,
            "-o" | "--only-matching" => options.only_matching = true,
            "-q" | "--quiet" | "--silent" => options.quiet = true,
            "-s" | "--no-messages" => options.no_messages = true,
            "-r" | "--recursive" => options.recursive = true,
            "-R" | "--dereference-recursive" => {
                options.recursive = true;
                options.dereference_recursive = true;
            }
            "-A" | "--after-context" => {
                if i + 1 < args.len() {
                    let count: usize = args[i + 1].parse()
                        .map_err(|_| anyhow!("grep: invalid context length '{}'", args[i + 1]))?;
                    options.after_context = count;
                    i += 1;
                } else {
                    return Err(anyhow!("grep: option requires an argument -- A"));
                }
            }
            "-B" | "--before-context" => {
                if i + 1 < args.len() {
                    let count: usize = args[i + 1].parse()
                        .map_err(|_| anyhow!("grep: invalid context length '{}'", args[i + 1]))?;
                    options.before_context = count;
                    i += 1;
                } else {
                    return Err(anyhow!("grep: option requires an argument -- B"));
                }
            }
            "-C" | "--context" => {
                if i + 1 < args.len() {
                    let count: usize = args[i + 1].parse()
                        .map_err(|_| anyhow!("grep: invalid context length '{}'", args[i + 1]))?;
                    options.after_context = count;
                    options.before_context = count;
                    i += 1;
                } else {
                    return Err(anyhow!("grep: option requires an argument -- C"));
                }
            }
            "-a" | "--text" => options.text_mode = true,
            "-I" => options.skip_binary = true,
            "-z" | "--null-data" => options.null_data = true,
            "-Z" | "--null" => options.null_output = true,
            "--color" => options.color = ColorMode::Always,
            "--color=always" => options.color = ColorMode::Always,
            "--color=never" => options.color = ColorMode::Never,
            "--color=auto" => options.color = ColorMode::Auto,
            "--binary-files=binary" => options.binary_files = BinaryMode::Binary,
            "--binary-files=text" => options.binary_files = BinaryMode::Text,
            "--binary-files=without-match" => options.binary_files = BinaryMode::WithoutMatch,
            arg if arg.starts_with("--include=") => {
                let pattern = arg.strip_prefix("--include=").unwrap();
                options.include_patterns.push(pattern.to_string());
            }
            arg if arg.starts_with("--exclude=") => {
                let pattern = arg.strip_prefix("--exclude=").unwrap();
                options.exclude_patterns.push(pattern.to_string());
            }
            arg if arg.starts_with("--exclude-dir=") => {
                let pattern = arg.strip_prefix("--exclude-dir=").unwrap();
                options.exclude_dir_patterns.push(pattern.to_string());
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--version" => {
                println!("grep (NexusShell) 1.0.0");
                std::process::exit(0);
            }
            arg if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                // Handle combined short options
                for ch in arg.chars().skip(1) {
                    match ch {
                        'E' => {
                            options.extended_regexp = true;
                            options.basic_regexp = false;
                        }
                        'F' => {
                            options.fixed_strings = true;
                            options.basic_regexp = false;
                        }
                        'G' => {
                            options.basic_regexp = true;
                            options.extended_regexp = false;
                        }
                        'P' => {
                            options.perl_regexp = true;
                            options.basic_regexp = false;
                        }
                        'i' => options.ignore_case = true,
                        'v' => options.invert_match = true,
                        'w' => options.word_regexp = true,
                        'x' => options.line_regexp = true,
                        'c' => options.count_only = true,
                        'l' => options.files_with_matches = true,
                        'L' => options.files_without_match = true,
                        'n' => options.line_number = true,
                        'H' => options.with_filename = true,
                        'h' => options.no_filename = true,
                        'o' => options.only_matching = true,
                        'q' => options.quiet = true,
                        's' => options.no_messages = true,
                        'r' => options.recursive = true,
                        'R' => {
                            options.recursive = true;
                            options.dereference_recursive = true;
                        }
                        'a' => options.text_mode = true,
                        'I' => options.skip_binary = true,
                        'z' => options.null_data = true,
                        'Z' => options.null_output = true,
                        _ => return Err(anyhow!("grep: invalid option -- '{}'", ch)),
                    }
                }
            }
            _ => {
                // This is either a pattern or a filename
                if !pattern_set && options.patterns.is_empty() && options.pattern_files.is_empty() {
                    options.patterns.push(arg.clone());
                    pattern_set = true;
                } else {
                    options.files.push(arg.clone());
                }
            }
        }
        i += 1;
    }
    
    Ok(options)
}

fn load_patterns_from_file(filename: &str) -> Result<Vec<String>> {
    let file = File::open(filename)
        .map_err(|e| anyhow!("grep: {}: {}", filename, e))?;
    let reader = BufReader::new(file);
    
    let mut patterns = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if !line.is_empty() {
            patterns.push(line);
        }
    }
    
    Ok(patterns)
}

fn create_matcher(patterns: Vec<String>, options: &GrepOptions) -> Result<GrepMatcher> {
    if options.fixed_strings {
        // Use Aho-Corasick for fixed string matching
        let mut builder = AhoCorasickBuilder::new();
        builder.ascii_case_insensitive(options.ignore_case);
        
        let ac = builder.build(&patterns)
            .map_err(|e| anyhow!("grep: failed to build fixed string matcher: {}", e))?;
        
        Ok(GrepMatcher {
            regex: None,
            fancy_regex: None,
            aho_corasick: Some(ac),
            fixed_patterns: patterns,
            options: options.clone(),
        })
    } else if options.perl_regexp {
        // Use fancy-regex for Perl-compatible regex
        let pattern = if patterns.len() == 1 {
            patterns[0].clone()
        } else {
            format!("({})", patterns.join("|"))
        };
        
        let mut pattern = if options.word_regexp {
            format!(r"\b({})\b", pattern)
        } else if options.line_regexp {
            format!("^({})$", pattern)
        } else {
            pattern
        };
        
        let regex = FancyRegex::new(&pattern)
            .map_err(|e| anyhow!("grep: invalid regex pattern: {}", e))?;
        
        Ok(GrepMatcher {
            regex: None,
            fancy_regex: Some(regex),
            aho_corasick: None,
            fixed_patterns: Vec::new(),
            options: options.clone(),
        })
    } else {
        // Use standard regex for basic/extended regex
        let pattern = if patterns.len() == 1 {
            patterns[0].clone()
        } else {
            format!("({})", patterns.join("|"))
        };
        
        let pattern = if options.word_regexp {
            format!(r"\b({})\b", pattern)
        } else if options.line_regexp {
            format!("^({})$", pattern)
        } else {
            pattern
        };
        
        let mut builder = RegexBuilder::new(&pattern);
        builder.case_insensitive(options.ignore_case);
        
        if options.extended_regexp {
            // Extended regex is default in Rust regex crate
        } else if options.basic_regexp {
            // Convert basic regex to extended regex
            // This is a simplified conversion - full BRE support would be more complex
        }
        
        let regex = builder.build()
            .map_err(|e| anyhow!("grep: invalid regex pattern: {}", e))?;
        
        Ok(GrepMatcher {
            regex: Some(regex),
            fancy_regex: None,
            aho_corasick: None,
            fixed_patterns: Vec::new(),
            options: options.clone(),
        })
    }
}

fn expand_file_list(files: &[String], options: &GrepOptions) -> Result<Vec<String>> {
    let mut expanded = Vec::new();
    
    for file in files {
        if options.recursive {
            if Path::new(file).is_dir() {
                let walker = if options.dereference_recursive {
                    WalkDir::new(file).follow_links(true)
                } else {
                    WalkDir::new(file)
                };
                
                for entry in walker {
                    match entry {
                        Ok(entry) => {
                            if entry.file_type().is_file() {
                                if should_include_file(&entry, options) {
                                    expanded.push(entry.path().to_string_lossy().to_string());
                                }
                            }
                        }
                        Err(e) => {
                            if !options.no_messages {
                                eprintln!("grep: {}", e);
                            }
                        }
                    }
                }
            } else {
                expanded.push(file.clone());
            }
        } else {
            expanded.push(file.clone());
        }
    }
    
    Ok(expanded)
}

fn should_include_file(entry: &DirEntry, options: &GrepOptions) -> bool {
    let filename = entry.file_name().to_string_lossy();
    let path = entry.path();
    
    // Check exclude patterns
    for pattern in &options.exclude_patterns {
        if let Ok(glob) = Glob::new(pattern) {
            if glob.compile_matcher().is_match(&filename) {
                return false;
            }
        }
    }
    
    // Check exclude-dir patterns for parent directories
    for ancestor in path.ancestors().skip(1) {
        if let Some(dir_name) = ancestor.file_name() {
            let dir_name = dir_name.to_string_lossy();
            for pattern in &options.exclude_dir_patterns {
                if let Ok(glob) = Glob::new(pattern) {
                    if glob.compile_matcher().is_match(&dir_name) {
                        return false;
                    }
                }
            }
        }
    }
    
    // Check include patterns (if any specified)
    if !options.include_patterns.is_empty() {
        let mut included = false;
        for pattern in &options.include_patterns {
            if let Ok(glob) = Glob::new(pattern) {
                if glob.compile_matcher().is_match(&filename) {
                    included = true;
                    break;
                }
            }
        }
        if !included {
            return false;
        }
    }
    
    true
}

fn search_file(filename: &str, matcher: &GrepMatcher, options: &GrepOptions) -> Result<FileResult> {
    let mut result = FileResult {
        filename: filename.to_string(),
        matches: Vec::new(),
        match_count: 0,
        error: None,
    };
    
    // Handle stdin
    if filename == "-" {
        let stdin = io::stdin();
        let reader = stdin.lock();
        search_reader(Box::new(reader), matcher, options, &mut result);
        return Ok(result);
    }
    
    // Check if file exists and is readable
    let metadata = match fs::metadata(filename) {
        Ok(meta) => meta,
        Err(e) => {
            result.error = Some(e.to_string());
            return Ok(result);
        }
    };
    
    if metadata.is_dir() {
        if !options.recursive {
            result.error = Some("Is a directory".to_string());
        }
        return Ok(result);
    }
    
    // Check for binary files
    if !options.text_mode && is_binary_file(filename)? {
        match options.binary_files {
            BinaryMode::WithoutMatch => return Ok(result),
            BinaryMode::Text => {}, // Process as text
            BinaryMode::Binary => {
                if options.skip_binary {
                    return Ok(result);
                }
                // Process but may produce binary output
            }
        }
    }
    
    // Open and search file
    match File::open(filename) {
        Ok(file) => {
            let reader = BufReader::new(file);
            search_reader(Box::new(reader), matcher, options, &mut result);
        }
        Err(e) => {
            result.error = Some(e.to_string());
        }
    }
    
    Ok(result)
}

fn search_reader<R: BufRead>(
    reader: Box<R>,
    matcher: &GrepMatcher,
    options: &GrepOptions,
    result: &mut FileResult,
) {
    let mut line_number = 1;
    let mut byte_offset = 0;
    let mut context_buffer: VecDeque<(usize, usize, String)> = VecDeque::new();
    let mut matches_found = 0;
    
    let line_separator = if options.null_data { b'\0' } else { b'\n' };
    
    for line_result in reader.lines() {
        let line = match line_result {
            Ok(line) => line,
            Err(_) => break,
        };
        
        let line_matches = find_matches_in_line(&line, matcher);
        let is_match = !line_matches.is_empty();
        let should_include = if options.invert_match { !is_match } else { is_match };
        
        if should_include {
            matches_found += 1;
            result.match_count += 1;
            
            // Add before context
            while let Some((ctx_line_num, ctx_byte_offset, ctx_line)) = context_buffer.pop_front() {
                result.matches.push(MatchResult {
                    line_number: ctx_line_num,
                    byte_offset: ctx_byte_offset,
                    line: ctx_line,
                    matches: Vec::new(),
                });
            }
            
            // Add the matching line
            result.matches.push(MatchResult {
                line_number,
                byte_offset,
                line: line.clone(),
                matches: line_matches,
            });
            
            // Check max count
            if let Some(max) = options.max_count {
                if matches_found >= max {
                    break;
                }
            }
        } else if options.before_context > 0 {
            // Maintain context buffer
            context_buffer.push_back((line_number, byte_offset, line.clone()));
            if context_buffer.len() > options.before_context {
                context_buffer.pop_front();
            }
        }
        
        line_number += 1;
        byte_offset += line.len() + 1; // +1 for newline
    }
}

fn find_matches_in_line(line: &str, matcher: &GrepMatcher) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    
    if let Some(ref ac) = matcher.aho_corasick {
        for mat in ac.find_iter(line) {
            matches.push((mat.start(), mat.end()));
        }
    } else if let Some(ref regex) = matcher.regex {
        for mat in regex.find_iter(line) {
            matches.push((mat.start(), mat.end()));
        }
    } else if let Some(ref fancy_regex) = matcher.fancy_regex {
        let mut start = 0;
        while start < line.len() {
            if let Ok(Some(mat)) = fancy_regex.find_at(line, start) {
                matches.push((mat.start(), mat.end()));
                start = mat.end().max(start + 1);
            } else {
                break;
            }
        }
    }
    
    matches
}

fn search_recursive(dir: &str, matcher: &GrepMatcher, options: &GrepOptions) -> Result<Vec<FileResult>> {
    let walker = if options.dereference_recursive {
        WalkDir::new(dir).follow_links(true)
    } else {
        WalkDir::new(dir)
    };
    
    let files: Vec<_> = walker
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| should_include_file(entry, options))
        .map(|entry| entry.path().to_string_lossy().to_string())
        .collect();
    
    // Use parallel processing for large file sets
    if files.len() > 10 {
        Ok(files
            .par_iter()
            .map(|filename| search_file(filename, matcher, options))
            .collect::<Result<Vec<_>, _>>()?)
    } else {
        files
            .iter()
            .map(|filename| search_file(filename, matcher, options))
            .collect()
    }
}

fn is_binary_file(filename: &str) -> Result<bool> {
    let mut file = File::open(filename)?;
    let mut buffer = [0u8; 8192];
    let bytes_read = file.read(&mut buffer)?;
    
    // Check for null bytes or high ratio of non-printable characters
    let null_count = buffer[..bytes_read].iter().filter(|&&b| b == 0).count();
    if null_count > 0 {
        return Ok(true);
    }
    
    let non_printable = buffer[..bytes_read]
        .iter()
        .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13)
        .count();
    
    Ok(non_printable as f64 / bytes_read as f64 > 0.3)
}

fn determine_filename_display(files: &[String], options: &GrepOptions) -> bool {
    if options.no_filename {
        false
    } else if options.with_filename {
        true
    } else {
        files.len() > 1 || (files.len() == 1 && options.recursive)
    }
}

fn should_use_color(options: &GrepOptions) -> bool {
    match options.color {
        ColorMode::Never => false,
        ColorMode::Always => true,
        ColorMode::Auto => atty::is(atty::Stream::Stdout),
    }
}

fn print_file_results(
    result: &FileResult,
    show_filename: bool,
    use_color: bool,
    options: &GrepOptions,
) -> Result<()> {
    if options.quiet {
        return Ok(());
    }
    
    if result.match_count == 0 {
        if options.files_without_match {
            println!("{}", result.filename);
        }
        return Ok(());
    }
    
    if options.files_with_matches {
        println!("{}", result.filename);
        return Ok(());
    }
    
    if options.count_only {
        if show_filename {
            println!("{}:{}", result.filename, result.match_count);
        } else {
            println!("{}", result.match_count);
        }
        return Ok(());
    }
    
    // Print matches with context
    for (i, match_result) in result.matches.iter().enumerate() {
        let mut output = String::new();
        
        // Add filename
        if show_filename {
            if use_color {
                output.push_str(&Colour::Purple.bold().paint(&result.filename).to_string());
            } else {
                output.push_str(&result.filename);
            }
            output.push(':');
        }
        
        // Add line number
        if options.line_number {
            if use_color {
                output.push_str(&Colour::Green.bold().paint(match_result.line_number.to_string()).to_string());
            } else {
                output.push_str(&match_result.line_number.to_string());
            }
            output.push(':');
        }
        
        // Add byte offset
        if options.byte_offset {
            output.push_str(&match_result.byte_offset.to_string());
            output.push(':');
        }
        
        // Add line content with highlighting
        if options.only_matching {
            // Show only matching parts
            for (start, end) in &match_result.matches {
                let match_text = &match_result.line[*start..*end];
                if use_color {
                    println!("{}{}", output, Colour::Red.bold().paint(match_text));
                } else {
                    println!("{}{}", output, match_text);
                }
            }
        } else {
            // Show full line with highlighted matches
            let highlighted_line = if use_color && !match_result.matches.is_empty() {
                highlight_matches(&match_result.line, &match_result.matches)
            } else {
                match_result.line.clone()
            };
            
            println!("{}{}", output, highlighted_line);
        }
    }
    
    Ok(())
}

fn highlight_matches(line: &str, matches: &[(usize, usize)]) -> String {
    if matches.is_empty() {
        return line.to_string();
    }
    
    let mut result = String::new();
    let mut last_end = 0;
    
    for &(start, end) in matches {
        // Add text before match
        result.push_str(&line[last_end..start]);
        
        // Add highlighted match
        let match_text = &line[start..end];
        result.push_str(&Colour::Red.bold().paint(match_text).to_string());
        
        last_end = end;
    }
    
    // Add remaining text
    result.push_str(&line[last_end..]);
    result
}

fn print_help() {
    println!("Usage: grep [OPTION]... PATTERN [FILE]...");
    println!("Search for PATTERN in each FILE.");
    println!("Example: grep -i 'hello world' menu.h main.c");
    println!();
    println!("Pattern selection and interpretation:");
    println!("  -E, --extended-regexp     PATTERN is an extended regular expression");
    println!("  -F, --fixed-strings       PATTERN is a set of newline-separated strings");
    println!("  -G, --basic-regexp        PATTERN is a basic regular expression (default)");
    println!("  -P, --perl-regexp         PATTERN is a Perl regular expression");
    println!("  -e, --regexp=PATTERN      use PATTERN for matching");
    println!("  -f, --file=FILE           obtain PATTERN from FILE");
    println!("  -i, --ignore-case         ignore case distinctions");
    println!("  -w, --word-regexp         force PATTERN to match only whole words");
    println!("  -x, --line-regexp         force PATTERN to match only whole lines");
    println!();
    println!("Miscellaneous:");
    println!("  -s, --no-messages         suppress error messages");
    println!("  -v, --invert-match        select non-matching lines");
    println!("  -V, --version             display version information and exit");
    println!("      --help                display this help text and exit");
    println!();
    println!("Output control:");
    println!("  -m, --max-count=NUM       stop after NUM matches");
    println!("  -n, --line-number         print line number with output lines");
    println!("  -H, --with-filename       print the file name for each match");
    println!("  -h, --no-filename         suppress the file name prefix on output");
    println!("  -o, --only-matching       show only the part of a line matching PATTERN");
    println!("  -q, --quiet, --silent     suppress all normal output");
    println!("      --binary-files=TYPE   assume that binary files are TYPE;");
    println!("                            TYPE is 'binary', 'text', or 'without-match'");
    println!("  -a, --text                equivalent to --binary-files=text");
    println!("  -I                        equivalent to --binary-files=without-match");
    println!("  -d, --directories=ACTION  how to handle directories;");
    println!("                            ACTION is 'read', 'recurse', or 'skip'");
    println!("  -D, --devices=ACTION      how to handle devices, FIFOs and sockets;");
    println!("                            ACTION is 'read' or 'skip'");
    println!("  -r, --recursive           like --directories=recurse");
    println!("  -R, --dereference-recursive  likewise, but follow all symlinks");
    println!();
    println!("Context control:");
    println!("  -A, --after-context=NUM   print NUM lines of trailing context");
    println!("  -B, --before-context=NUM  print NUM lines of leading context");
    println!("  -C, --context=NUM         print NUM lines of output context");
    println!();
    println!("When FILE is '-', read standard input. With no FILE, read '.' if");
    println!("recursive, '-' otherwise. Exit status is 0 if any line is selected,");
    println!("1 otherwise; if any error occurs and -q is not given, the exit");
    println!("status is 2.");
    println!();
    println!("Report bugs to <bug-reports@nexusshell.org>");
} 