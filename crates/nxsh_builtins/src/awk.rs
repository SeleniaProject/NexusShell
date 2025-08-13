//! `awk` command - pattern scanning and data extraction language
//!
//! Complete awk implementation with pattern matching, field processing, and scripting

use std::collections::HashMap;
use nxsh_core::{ShellResult, ShellError, ErrorKind};
use nxsh_core::error::RuntimeErrorKind;
#[cfg(feature = "advanced-regex")]
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// AWK コマンド簡易実装 (最小限) – BEGIN/PATTERN/ACTIONS と print のみ対応
/// 今後の高機能化のため内部構造は維持しつつ、スタブから実行可能状態へ昇格させる。
pub fn awk_cli(args: &[String], _ctx: &mut nxsh_core::context::ShellContext) -> anyhow::Result<()> {
    // nxsh の他ビルトインと同じ引数解釈を流用
    let options = parse_awk_args(args)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // プログラム文字列決定 (-f 指定が優先)
    let program_src = if let Some(ref file) = options.program_file {
        std::fs::read_to_string(file)
            .map_err(|e| anyhow::anyhow!(format!("Cannot read program file {file}: {e}")))?
    } else {
        options.program.clone()
    };

    // AWK プログラム構文解析 (簡易)
    let parsed = parse_awk_program(&program_src)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let mut awk_context = AwkContext::new(&options);

    // BEGIN アクション
    for action in &parsed.begin_actions {
        execute_awk_action(action, &mut awk_context)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    }

    // 入力ファイル処理 (無ければ stdin)
    if options.files.is_empty() {
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        process_awk_stream(&mut handle, &parsed, &mut awk_context, "<stdin>")
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    } else {
        for file_path in &options.files {
            awk_context.fnr = 0;
            awk_context.filename = file_path.clone();
            let file = File::open(file_path).map_err(|e| {
                anyhow::anyhow!(format!("Cannot open {file_path}: {e}"))
            })?;
            let mut reader = BufReader::new(file);
            process_awk_stream(&mut reader, &parsed, &mut awk_context, file_path)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        }
    }

    // END アクション
    for action in &parsed.end_actions {
        execute_awk_action(action, &mut awk_context)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct AwkOptions {
    pub field_separator: String,
    pub output_field_separator: String,
    pub record_separator: String,
    pub output_record_separator: String,
    pub program: String,
    pub program_file: Option<String>,
    pub variables: HashMap<String, String>,
    pub files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AwkProgram {
    pub begin_actions: Vec<AwkAction>,
    pub pattern_actions: Vec<(Option<AwkPattern>, AwkAction)>,
    pub end_actions: Vec<AwkAction>,
}

#[derive(Debug, Clone)]
pub enum AwkPattern {
    Regex(Regex),
    Expression(String),
    Range(Box<AwkPattern>, Box<AwkPattern>),
}

#[derive(Debug, Clone)]
pub enum AwkAction {
    Print(Vec<AwkExpression>),
    PrintF(String, Vec<AwkExpression>),
    Assignment(String, AwkExpression),
    If(AwkExpression, Box<AwkAction>, Option<Box<AwkAction>>),
    For(String, AwkExpression, AwkExpression, Box<AwkAction>),
    While(AwkExpression, Box<AwkAction>),
    Block(Vec<AwkAction>),
    Expression(AwkExpression),
    Next,
    Exit(Option<AwkExpression>),
}

#[derive(Debug, Clone)]
pub enum AwkExpression {
    String(String),
    Number(f64),
    Field(usize),
    Variable(String),
    Binary(Box<AwkExpression>, BinaryOp, Box<AwkExpression>),
    Unary(UnaryOp, Box<AwkExpression>),
    Function(String, Vec<AwkExpression>),
    Match(Box<AwkExpression>, Regex),
    NotMatch(Box<AwkExpression>, Regex),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
    Concat,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Not, Neg, Pos,
}

#[derive(Debug)]
pub struct AwkContext {
    pub variables: HashMap<String, AwkValue>,
    pub fields: Vec<String>,
    pub nf: usize,
    pub nr: usize,
    pub fnr: usize,
    pub filename: String,
    pub fs: String,
    pub ofs: String,
    pub rs: String,
    pub ors: String,
}

#[derive(Debug, Clone)]
pub enum AwkValue {
    String(String),
    Number(f64),
}

// Legacy note: The old AwkCommand (Builtin trait impl) was deprecated in favor of a single
// `awk_cli(args, &mut ShellContext)` entry. This keeps the interface consistent with other
// builtins, reduces unused-code warnings, and allows future promotion to Builtin safely.

impl AwkContext {
    fn new(options: &AwkOptions) -> Self {
        let mut variables = HashMap::new();
        
        // Initialize built-in variables
        variables.insert("FS".to_string(), AwkValue::String(options.field_separator.clone()));
        variables.insert("OFS".to_string(), AwkValue::String(options.output_field_separator.clone()));
        variables.insert("RS".to_string(), AwkValue::String(options.record_separator.clone()));
        variables.insert("ORS".to_string(), AwkValue::String(options.output_record_separator.clone()));
        variables.insert("NF".to_string(), AwkValue::Number(0.0));
        variables.insert("NR".to_string(), AwkValue::Number(0.0));
        variables.insert("FNR".to_string(), AwkValue::Number(0.0));
        variables.insert("FILENAME".to_string(), AwkValue::String("".to_string()));
        variables.insert("RSTART".to_string(), AwkValue::Number(0.0));
        variables.insert("RLENGTH".to_string(), AwkValue::Number(0.0));

        // Add user-defined variables
        for (key, value) in &options.variables {
            if let Ok(num) = value.parse::<f64>() {
                variables.insert(key.clone(), AwkValue::Number(num));
            } else {
                variables.insert(key.clone(), AwkValue::String(value.clone()));
            }
        }

        Self {
            variables,
            fields: Vec::new(),
            nf: 0,
            nr: 0,
            fnr: 0,
            filename: String::new(),
            fs: options.field_separator.clone(),
            ofs: options.output_field_separator.clone(),
            rs: options.record_separator.clone(),
            ors: options.output_record_separator.clone(),
        }
    }

    fn split_fields(&mut self, line: &str) {
        self.fields.clear();
        self.fields.push(line.to_string()); // $0 is the whole line
        
        if self.fs == " " {
            // Default FS: split on runs of whitespace
            self.fields.extend(line.split_whitespace().map(|s| s.to_string()));
        } else if self.fs.len() == 1 {
            // Single character separator
            self.fields.extend(line.split(&self.fs).map(|s| s.to_string()));
        } else {
            // Multi-character separator or regex
            #[cfg(feature = "advanced-regex")]
            if let Ok(re) = Regex::new(&self.fs) {
                self.fields.extend(re.split(line).map(|s| s.to_string()));
            } else {
                self.fields.extend(line.split(&self.fs).map(|s| s.to_string()));
            }
        }
        
        self.nf = self.fields.len() - 1; // Don't count $0
        self.variables.insert("NF".to_string(), AwkValue::Number(self.nf as f64));
    }

    fn get_field(&self, index: usize) -> String {
        if index < self.fields.len() {
            self.fields[index].clone()
        } else {
            String::new()
        }
    }
}

fn parse_awk_args(args: &[String]) -> ShellResult<AwkOptions> {
    let mut options = AwkOptions {
        field_separator: " ".to_string(),
        output_field_separator: " ".to_string(),
        record_separator: "\n".to_string(),
        output_record_separator: "\n".to_string(),
        program: String::new(),
        program_file: None,
        variables: HashMap::new(),
        files: Vec::new(),
    };

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        
        if arg == "-F" || arg == "--field-separator" {
            i += 1;
            if i >= args.len() {
                return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Option -F requires an argument".to_string()));
            }
            options.field_separator = args[i].clone();
        } else if let Some(rest) = arg.strip_prefix("-F") {
            options.field_separator = rest.to_string();
        } else if arg == "-f" || arg == "--file" {
            i += 1;
            if i >= args.len() {
                return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Option -f requires an argument".to_string()));
            }
            options.program_file = Some(args[i].clone());
        } else if arg == "-v" || arg == "--assign" {
            i += 1;
            if i >= args.len() {
                return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Option -v requires an argument".to_string()));
            }
            let assignment = &args[i];
            if let Some(eq_pos) = assignment.find('=') {
                let var_name = assignment[..eq_pos].to_string();
                let var_value = assignment[eq_pos + 1..].to_string();
                options.variables.insert(var_name, var_value);
            } else {
                return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid variable assignment format".to_string()));
            }
        } else if let Some(assignment) = arg.strip_prefix("-v") {
            if let Some((name, value)) = assignment.split_once('=') {
                options.variables.insert(name.to_string(), value.to_string());
            }
        } else if arg == "--help" {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Help requested".to_string()));
        } else if arg.starts_with("-") {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Unknown option: {arg}")));
        } else {
            // First non-option argument is program if no -f was used
            if options.program.is_empty() && options.program_file.is_none() {
                options.program = arg.clone();
            } else {
                options.files.push(arg.clone());
            }
        }
        i += 1;
    }

    if options.program.is_empty() && options.program_file.is_none() {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "No program provided".to_string()));
    }

    Ok(options)
}

fn parse_awk_program(program: &str) -> ShellResult<AwkProgram> {
    // Simplified parser - in a real implementation, this would be much more complex
    let mut begin_actions = Vec::new();
    let mut pattern_actions = Vec::new();
    let mut end_actions = Vec::new();

    let lines: Vec<&str> = program.lines().collect();
    let mut current_section = "main";
    
    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(rest) = line.strip_prefix("BEGIN") {
            current_section = "begin";
            let action_part = rest.trim();
            if !action_part.is_empty() {
                let action = parse_awk_action_or_block(action_part)?;
                begin_actions.push(action);
            }
        } else if let Some(rest) = line.strip_prefix("END") {
            current_section = "end";
            let action_part = rest.trim();
            if !action_part.is_empty() {
                let action = parse_awk_action_or_block(action_part)?;
                end_actions.push(action);
            }
        } else {
            // パターン + アクション: /regex/ action または /start/ , /end/ action
            if let Some((pat, rest)) = parse_line_pattern_prefix(line) {
                let action = if rest.is_empty() {
                    AwkAction::Print(vec![AwkExpression::Field(0)])
                } else {
                    parse_awk_action_or_block(rest)?
                };
                pattern_actions.push((Some(pat), action));
                continue;
            }
            let action = parse_awk_action_or_block(line)?;
            match current_section {
                "begin" => begin_actions.push(action),
                "end" => end_actions.push(action),
                _ => pattern_actions.push((None, action)),
            }
        }
    }

    Ok(AwkProgram {
        begin_actions,
        pattern_actions,
        end_actions,
    })
}

fn parse_awk_action(action_str: &str) -> ShellResult<AwkAction> {
    let action_str = action_str.trim();

    if let Some(rest) = action_str.strip_prefix("print") {
        // Enhanced print: parse comma-separated argument expressions
        let args_part = rest.trim();
        if args_part.is_empty() {
            Ok(AwkAction::Print(vec![AwkExpression::Field(0)]))
        } else {
            let expressions = parse_print_args(args_part)?;
            Ok(AwkAction::Print(expressions))
        }
    } else if let Some(rest) = action_str.strip_prefix("printf") {
        // printf "fmt", args...
        let args_csv = rest.trim();
        let parts = split_csv_tokens(args_csv);
        if parts.is_empty() { return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "printf requires a format string".to_string())); }
        let fmt_tok = parts[0].trim();
        let fmt = if fmt_tok.starts_with('"') && fmt_tok.ends_with('"') && fmt_tok.len() >= 2 { fmt_tok[1..fmt_tok.len()-1].to_string() } else { fmt_tok.to_string() };
        let mut exprs = Vec::new();
        for p in parts.into_iter().skip(1) { exprs.push(parse_simple_expr(p.trim())?); }
        Ok(AwkAction::PrintF(fmt, exprs))
    } else if action_str.starts_with("if") {
        // if (cond) action [else action]
        let (cond_str, rest) = extract_paren_segment(action_str.strip_prefix("if").unwrap_or("").trim())?;
        let then_action = parse_awk_action_or_block(rest.trim())?;
        let mut else_action = None;
        if let Some(pos) = action_str.find(" else ") {
            let else_part = &action_str[pos + 6..];
            else_action = Some(Box::new(parse_awk_action_or_block(else_part.trim())?));
        }
        Ok(AwkAction::If(parse_simple_expr(cond_str.trim())?, Box::new(then_action), else_action))
    } else if action_str.starts_with("while") {
        let (cond_str, rest) = extract_paren_segment(action_str.strip_prefix("while").unwrap_or("").trim())?;
        let body = parse_awk_action_or_block(rest.trim())?;
        Ok(AwkAction::While(parse_simple_expr(cond_str.trim())?, Box::new(body)))
    } else if action_str.starts_with("for") {
        let (header, rest) = extract_paren_segment(action_str.strip_prefix("for").unwrap_or("").trim())?;
        let mut pieces = header.splitn(3, ';').map(|s| s.trim()).collect::<Vec<_>>();
        while pieces.len() < 3 { pieces.push(""); }
        let init = pieces[0].to_string();
        let cond = parse_simple_expr(pieces[1])?;
        let post = parse_simple_expr(pieces[2])?;
        let body = parse_awk_action_or_block(rest.trim())?;
        Ok(AwkAction::For(init, cond, post, Box::new(body)))
    } else if action_str == "next" {
        Ok(AwkAction::Next)
    } else if let Some(rest) = action_str.strip_prefix("exit") {
        let arg = rest.trim();
        if arg.is_empty() { return Ok(AwkAction::Exit(None)); }
        let expr = parse_simple_expr(arg)?;
        Ok(AwkAction::Exit(Some(expr)))
    } else if action_str.starts_with('{') && action_str.ends_with('}') {
        Ok(parse_awk_block(action_str)?)
    } else if let Some(eq_pos) = action_str.find('=') {
        // 簡易代入 VAR=... （空白無し想定）
        let var = action_str[..eq_pos].trim();
        let rhs = action_str[eq_pos + 1..].trim();
        if !var.is_empty() {
            // 数値判定
            if let Ok(num) = rhs.parse::<f64>() {
                return Ok(AwkAction::Assignment(var.to_string(), AwkExpression::Number(num)));
            } else {
                return Ok(AwkAction::Assignment(var.to_string(), AwkExpression::String(rhs.to_string())));
            }
        }
        Ok(AwkAction::Expression(AwkExpression::String(action_str.to_string())))
    } else {
        // Expression or other action
        Ok(AwkAction::Expression(AwkExpression::String(action_str.to_string())))
    }
}

fn parse_awk_action_or_block(src: &str) -> ShellResult<AwkAction> {
    let s = src.trim();
    if s.starts_with('{') && s.ends_with('}') { return parse_awk_block(s); }
    parse_awk_action(s)
}

fn parse_awk_block(block_src: &str) -> ShellResult<AwkAction> {
    let inner = block_src.trim().trim_start_matches('{').trim_end_matches('}');
    let mut actions = Vec::new();
    let mut buf = String::new();
    let mut depth = 0i32;
    let mut in_str = false;
    for ch in inner.chars() {
        match ch {
            '"' => { in_str = !in_str; buf.push(ch); }
            '{' if !in_str => { depth += 1; buf.push(ch); }
            '}' if !in_str => { depth -= 1; buf.push(ch); }
            ';' | '\n' if !in_str && depth == 0 => {
                let part = buf.trim();
                if !part.is_empty() { actions.push(parse_awk_action(part)?); }
                buf.clear();
            }
            _ => buf.push(ch),
        }
    }
    if !buf.trim().is_empty() { actions.push(parse_awk_action(buf.trim())?); }
    Ok(AwkAction::Block(actions))
}

fn parse_line_pattern_prefix(line: &str) -> Option<(AwkPattern, &str)> {
    let s = line.trim();
    if !s.starts_with('/') { return None; }
    let after = &s[1..];
    let pos1 = after.find('/')?;
    let pat1 = &after[..pos1];
    let rest = after[pos1+1..].trim_start();
    if rest.starts_with(',') {
        let rest2 = rest[1..].trim_start();
        if rest2.starts_with('/') {
            let after2 = &rest2[1..];
            if let Some(pos2) = after2.find('/') {
                let pat2 = &after2[..pos2];
                let action_part = after2[pos2+1..].trim_start();
                #[cfg(feature = "advanced-regex")]
                if let (Ok(re1), Ok(re2)) = (Regex::new(pat1), Regex::new(pat2)) {
                    return Some((AwkPattern::Range(Box::new(AwkPattern::Regex(re1)), Box::new(AwkPattern::Regex(re2))), action_part));
                }
                #[cfg(not(feature = "advanced-regex"))]
                {
                    let _ = (pat1, pat2);
                    return Some((AwkPattern::Expression("true".to_string()), action_part));
                }
            }
        }
    } else {
        let action_part = rest;
        #[cfg(feature = "advanced-regex")]
        if let Ok(re) = Regex::new(pat1) {
            return Some((AwkPattern::Regex(re), action_part));
        }
        #[cfg(not(feature = "advanced-regex"))]
        {
            let _ = pat1;
            return Some((AwkPattern::Expression("true".to_string()), action_part));
        }
    }
    None
}

fn extract_paren_segment(src: &str) -> ShellResult<(&str, &str)> {
    let s = src.trim();
    if !s.starts_with('(') { return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "missing (".to_string())); }
    let mut depth = 0i32;
    let mut idx = 0usize;
    let bytes = s.as_bytes();
    while idx < bytes.len() {
        let ch = bytes[idx] as char;
        if ch == '(' { depth += 1; }
        if ch == ')' { depth -= 1; if depth == 0 { break; } }
        idx += 1;
    }
    if idx >= bytes.len() { return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "missing )".to_string())); }
    let inside = &s[1..idx];
    let rest = &s[idx+1..];
    Ok((inside, rest))
}

fn split_csv_tokens(src: &str) -> Vec<String> {
    let mut res = Vec::new();
    let mut buf = String::new();
    let mut in_str = false;
    let mut depth = 0i32;
    let s = src.trim();
    for ch in s.chars() {
        match ch {
            '"' => { in_str = !in_str; buf.push(ch); }
            '(' | '{' | '[' if !in_str => { depth += 1; buf.push(ch); }
            ')' | '}' | ']' if !in_str => { depth -= 1; buf.push(ch); }
            ',' if !in_str && depth == 0 => { if !buf.trim().is_empty() { res.push(buf.trim().to_string()); } buf.clear(); }
            _ => buf.push(ch),
        }
    }
    if !buf.trim().is_empty() { res.push(buf.trim().to_string()); }
    res
}

fn parse_print_args(src: &str) -> ShellResult<Vec<AwkExpression>> {
    // Parse comma-separated simple expressions: $N, "string", number, var, concatenations with '+'
    let mut result: Vec<AwkExpression> = Vec::new();
    let mut current = String::new();
    let mut in_str = false;
    let mut chars = src.trim().chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                current.push(ch);
                in_str = !in_str;
            }
            ',' if !in_str => {
                let expr = parse_simple_expr(current.trim())?;
                result.push(expr);
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        let expr = parse_simple_expr(current.trim())?;
        result.push(expr);
    }
    Ok(result)
}

fn parse_simple_expr(token: &str) -> ShellResult<AwkExpression> {
    // Very small expression parser with '+' for numeric/string concat, otherwise atom
    // Split on '+' not inside quotes
    let mut parts: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut in_str = false;
    for ch in token.chars() {
        match ch {
            '"' => { in_str = !in_str; buf.push(ch); }
            '+' if !in_str => {
                if !buf.trim().is_empty() { parts.push(buf.trim().to_string()); }
                buf.clear();
            }
            _ => buf.push(ch),
        }
    }
    if !buf.trim().is_empty() { parts.push(buf.trim().to_string()); }

    if parts.len() > 1 {
        // Build left-associative Binary::Concat
        let mut iter = parts.into_iter();
        let first = parse_atom(&iter.next().unwrap())?;
        let mut acc = first;
        for p in iter {
            let rhs = parse_atom(&p)?;
            acc = AwkExpression::Binary(Box::new(acc), BinaryOp::Concat, Box::new(rhs));
        }
        Ok(acc)
    } else {
        parse_atom(token)
    }
}

fn parse_atom(token: &str) -> ShellResult<AwkExpression> {
    let t = token.trim();
    if t.starts_with('$') {
        let num = t[1..].trim().parse::<usize>()
            .map_err(|_| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Invalid field reference: {t}")))?;
        return Ok(AwkExpression::Field(num));
    }
    if t.starts_with('"') && t.ends_with('"') && t.len() >= 2 {
        return Ok(AwkExpression::String(t[1..t.len()-1].to_string()));
    }
    if let Ok(n) = t.parse::<f64>() {
        return Ok(AwkExpression::Number(n));
    }
    // variable name
    Ok(AwkExpression::Variable(t.to_string()))
}

fn process_awk_stream<R: BufRead>(
    reader: &mut R,
    program: &AwkProgram,
    context: &mut AwkContext,
    filename: &str,
) -> ShellResult<()> {
    context.filename = filename.to_string();
    context.variables.insert("FILENAME".to_string(), AwkValue::String(filename.to_string()));

    let mut line = String::new();
    let mut range_states: Vec<bool> = vec![false; program.pattern_actions.len()];
    while reader.read_line(&mut line)? > 0 {
        // Remove trailing newline
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }

        context.nr += 1;
        context.fnr += 1;
        context.variables.insert("NR".to_string(), AwkValue::Number(context.nr as f64));
        context.variables.insert("FNR".to_string(), AwkValue::Number(context.fnr as f64));

        context.split_fields(&line);

        // Execute pattern-action pairs
        for (idx, (pattern, action)) in program.pattern_actions.iter().enumerate() {
            let mut should_execute = true;
            if let Some(pattern) = pattern {
                should_execute = match pattern {
                    AwkPattern::Range(start, end) => {
                        let active = &mut range_states[idx];
                        let start_match = match_awk_pattern(start, context, &line)?;
                        let end_match = match_awk_pattern(end, context, &line)?;
                        if !*active && start_match { *active = true; }
                        let exec_now = *active;
                        if *active && end_match { *active = false; }
                        exec_now
                    }
                    _ => match_awk_pattern(pattern, context, &line)?,
                };
            }

            if should_execute {
                match execute_awk_action_flow(action, context)? {
                    AwkFlow::Continue => {}
                    AwkFlow::NextRecord => { break; }
                    AwkFlow::Exit(_) => { return Ok(()); }
                }
            }
        }

        line.clear();
    }

    Ok(())
}

fn match_awk_pattern(pattern: &AwkPattern, context: &AwkContext, line: &str) -> ShellResult<bool> {
    match pattern {
        AwkPattern::Regex(re) => Ok(re.is_match(line)),
        AwkPattern::Expression(expr_src) => {
            // Evaluate simple expression truthiness
            let expr = parse_simple_expr(expr_src)?;
            let val = evaluate_awk_expression(&expr, context)?;
            Ok(is_truthy(&val))
        }
        AwkPattern::Range(_start, _end) => Ok(true),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AwkFlow {
    Continue,
    NextRecord,
    Exit(Option<i32>),
}

fn execute_awk_action_flow(action: &AwkAction, context: &mut AwkContext) -> ShellResult<AwkFlow> {
    match action {
        AwkAction::Print(expressions) => {
            if expressions.is_empty() {
                println!("{}", context.get_field(0));
            } else {
                let mut output = Vec::new();
                for expr in expressions {
                    let value = evaluate_awk_expression(expr, context)?;
                    output.push(awk_value_to_string(&value));
                }
                println!("{}", output.join(&context.ofs));
            }
            Ok(AwkFlow::Continue)
        }
        AwkAction::PrintF(format, expressions) => {
            // Simplified printf implementation
            let mut output = format.clone();
            for expr in expressions {
                let value = evaluate_awk_expression(expr, context)?;
                let str_val = awk_value_to_string(&value);
                output = output.replacen("%s", &str_val, 1);
                if let AwkValue::Number(n) = value {
                    output = output.replacen("%d", &(n as i64).to_string(), 1);
                    output = output.replacen("%f", &n.to_string(), 1);
                }
            }
            print!("{output}");
            Ok(AwkFlow::Continue)
        }
        AwkAction::Block(actions) => {
            for action in actions {
                match execute_awk_action_flow(action, context)? {
                    AwkFlow::Continue => {}
                    AwkFlow::NextRecord => return Ok(AwkFlow::NextRecord),
                    AwkFlow::Exit(code) => return Ok(AwkFlow::Exit(code)),
                }
            }
            Ok(AwkFlow::Continue)
        }
        AwkAction::Assignment(var, expr) => {
            let value = evaluate_awk_expression(expr, context)?;
            match value {
                AwkValue::Number(n) => { context.variables.insert(var.clone(), AwkValue::Number(n)); }
                AwkValue::String(s) => { context.variables.insert(var.clone(), AwkValue::String(s)); }
            }
            Ok(AwkFlow::Continue)
        }
        AwkAction::Expression(expr) => {
            evaluate_awk_expression(expr, context)?;
            Ok(AwkFlow::Continue)
        }
        AwkAction::If(cond, then_a, else_a) => {
            let v = evaluate_awk_expression(cond, context)?;
            if is_truthy(&v) {
                return execute_awk_action_flow(then_a, context);
            } else if let Some(e) = else_a {
                return execute_awk_action_flow(e, context);
            }
            Ok(AwkFlow::Continue)
        }
        AwkAction::While(cond, body) => {
            let mut guard = 0usize;
            loop {
                guard += 1;
                if guard > 1_000_000 { break; }
                if !is_truthy(&evaluate_awk_expression(cond, context)?) { break; }
                match execute_awk_action_flow(body, context)? {
                    AwkFlow::Continue => {}
                    AwkFlow::NextRecord => return Ok(AwkFlow::NextRecord),
                    AwkFlow::Exit(code) => return Ok(AwkFlow::Exit(code)),
                }
            }
            Ok(AwkFlow::Continue)
        }
        AwkAction::For(init, cond, post, body) => {
            apply_simple_statement(init, context)?;
            let mut guard = 0usize;
            loop {
                guard += 1;
                if guard > 1_000_000 { break; }
                if !is_truthy(&evaluate_awk_expression(cond, context)?) { break; }
                match execute_awk_action_flow(body, context)? {
                    AwkFlow::Continue => {}
                    AwkFlow::NextRecord => return Ok(AwkFlow::NextRecord),
                    AwkFlow::Exit(code) => return Ok(AwkFlow::Exit(code)),
                }
                let _ = evaluate_awk_expression(post, context)?;
            }
            Ok(AwkFlow::Continue)
        }
        AwkAction::Next => Ok(AwkFlow::NextRecord),
        AwkAction::Exit(expr_opt) => {
            let code = if let Some(ex) = expr_opt { to_number(&evaluate_awk_expression(ex, context)?) as i32 } else { 0 };
            Ok(AwkFlow::Exit(Some(code)))
        }
    }
}

fn execute_awk_action(action: &AwkAction, context: &mut AwkContext) -> ShellResult<()> {
    let _ = execute_awk_action_flow(action, context)?;
    Ok(())
}

fn apply_simple_statement(stmt: &str, context: &mut AwkContext) -> ShellResult<()> {
    let s = stmt.trim();
    if s.is_empty() { return Ok(()); }
    if let Some(eq) = s.find('=') {
        let name = s[..eq].trim();
        let rhs = s[eq+1..].trim();
        let expr = parse_simple_expr(rhs)?;
        let val = evaluate_awk_expression(&expr, context)?;
        match val {
            AwkValue::Number(n) => { context.variables.insert(name.to_string(), AwkValue::Number(n)); }
            AwkValue::String(t) => { context.variables.insert(name.to_string(), AwkValue::String(t)); }
        }
        return Ok(());
    }
    let _ = evaluate_awk_expression(&parse_simple_expr(s)?, context)?;
    Ok(())
}

fn evaluate_awk_expression(expr: &AwkExpression, context: &AwkContext) -> ShellResult<AwkValue> {
    match expr {
        AwkExpression::String(s) => Ok(AwkValue::String(s.clone())),
        AwkExpression::Number(n) => Ok(AwkValue::Number(*n)),
        AwkExpression::Field(index) => Ok(AwkValue::String(context.get_field(*index))),
        AwkExpression::Variable(name) => Ok(context.variables.get(name).cloned().unwrap_or(AwkValue::String(String::new()))),
        AwkExpression::Unary(op, inner) => {
            let v = evaluate_awk_expression(inner, context)?;
            match op {
                UnaryOp::Not => Ok(AwkValue::Number(if is_truthy(&v) { 0.0 } else { 1.0 })),
                UnaryOp::Neg => Ok(AwkValue::Number(-to_number(&v))),
                UnaryOp::Pos => Ok(AwkValue::Number(to_number(&v))),
            }
        }
        AwkExpression::Binary(lhs, op, rhs) => {
            let lv = evaluate_awk_expression(lhs, context)?;
            let rv = evaluate_awk_expression(rhs, context)?;
            match op {
                BinaryOp::Add => Ok(AwkValue::Number(to_number(&lv) + to_number(&rv))),
                BinaryOp::Sub => Ok(AwkValue::Number(to_number(&lv) - to_number(&rv))),
                BinaryOp::Mul => Ok(AwkValue::Number(to_number(&lv) * to_number(&rv))),
                BinaryOp::Div => Ok(AwkValue::Number(to_number(&lv) / to_number(&rv))),
                BinaryOp::Mod => Ok(AwkValue::Number(to_number(&lv) % to_number(&rv))),
                BinaryOp::Eq => Ok(AwkValue::Number(if to_string_val(&lv) == to_string_val(&rv) { 1.0 } else { 0.0 })),
                BinaryOp::Ne => Ok(AwkValue::Number(if to_string_val(&lv) != to_string_val(&rv) { 1.0 } else { 0.0 })),
                BinaryOp::Lt => Ok(AwkValue::Number(if to_number(&lv) < to_number(&rv) { 1.0 } else { 0.0 })),
                BinaryOp::Le => Ok(AwkValue::Number(if to_number(&lv) <= to_number(&rv) { 1.0 } else { 0.0 })),
                BinaryOp::Gt => Ok(AwkValue::Number(if to_number(&lv) > to_number(&rv) { 1.0 } else { 0.0 })),
                BinaryOp::Ge => Ok(AwkValue::Number(if to_number(&lv) >= to_number(&rv) { 1.0 } else { 0.0 })),
                BinaryOp::And => Ok(AwkValue::Number(if is_truthy(&lv) && is_truthy(&rv) { 1.0 } else { 0.0 })),
                BinaryOp::Or => Ok(AwkValue::Number(if is_truthy(&lv) || is_truthy(&rv) { 1.0 } else { 0.0 })),
                BinaryOp::Concat => Ok(AwkValue::String(format!("{}{}", to_string_val(&lv), to_string_val(&rv)))),
            }
        }
        AwkExpression::Function(name, args) => {
            let name_lower = name.to_lowercase();
            match name_lower.as_str() {
                "length" => {
                    if args.is_empty() {
                        // length($0)
                        let s = context.get_field(0);
                        Ok(AwkValue::Number(s.len() as f64))
                    } else {
                        let v = evaluate_awk_expression(&args[0], context)?;
                        Ok(AwkValue::Number(to_string_val(&v).len() as f64))
                    }
                }
                _ => Ok(AwkValue::String(String::new())),
            }
        }
        AwkExpression::Match(lhs, re) => {
            let lv = evaluate_awk_expression(lhs, context)?;
            #[cfg(feature = "advanced-regex")]
            {
                return Ok(AwkValue::Number(if re.is_match(&to_string_val(&lv)) { 1.0 } else { 0.0 }));
            }
            #[cfg(not(feature = "advanced-regex"))]
            {
                let _ = lv; // regex disabled: always false
                return Ok(AwkValue::Number(0.0));
            }
        }
        AwkExpression::NotMatch(lhs, re) => {
            let lv = evaluate_awk_expression(lhs, context)?;
            #[cfg(feature = "advanced-regex")]
            {
                return Ok(AwkValue::Number(if !re.is_match(&to_string_val(&lv)) { 1.0 } else { 0.0 }));
            }
            #[cfg(not(feature = "advanced-regex"))]
            {
                let _ = lv;
                return Ok(AwkValue::Number(1.0));
            }
        }
    }
}

fn awk_value_to_string(value: &AwkValue) -> String {
    match value {
        AwkValue::String(s) => s.clone(),
        AwkValue::Number(n) => {
            if n.fract() == 0.0 {
                (*n as i64).to_string()
            } else {
                n.to_string()
            }
        }
    }
}

fn to_number(v: &AwkValue) -> f64 {
    match v {
        AwkValue::Number(n) => *n,
        AwkValue::String(s) => s.trim().parse::<f64>().unwrap_or(0.0),
    }
}

fn to_string_val(v: &AwkValue) -> String {
    match v {
        AwkValue::String(s) => s.clone(),
        AwkValue::Number(n) => {
            if n.fract() == 0.0 { (*n as i64).to_string() } else { n.to_string() }
        }
    }
}

fn is_truthy(v: &AwkValue) -> bool {
    match v {
        AwkValue::Number(n) => *n != 0.0,
        AwkValue::String(s) => !s.is_empty(),
    }
}

 
#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx() -> AwkContext {
        let opts = AwkOptions {
            field_separator: " ".to_string(),
            output_field_separator: " ".to_string(),
            record_separator: "\n".to_string(),
            output_record_separator: "\n".to_string(),
            program: String::new(),
            program_file: None,
            variables: HashMap::new(),
            files: vec![],
        };
        let mut ctx = AwkContext::new(&opts);
        ctx.split_fields("alpha beta gamma");
        ctx
    }

    #[test]
    fn test_simple_expr_concat_and_number() {
        let expr = parse_simple_expr("\"x\"+3").unwrap();
        let v = evaluate_awk_expression(&expr, &make_ctx()).unwrap();
        match v { AwkValue::String(s) => assert_eq!(s, "x3"), _ => panic!("expected string") }
    }

    #[test]
    fn test_binary_arithmetic() {
        let left = AwkExpression::Number(10.0);
        let right = AwkExpression::Number(4.0);
        let add = AwkExpression::Binary(Box::new(left.clone()), BinaryOp::Add, Box::new(right.clone()));
        let sub = AwkExpression::Binary(Box::new(left.clone()), BinaryOp::Sub, Box::new(right.clone()));
        let mul = AwkExpression::Binary(Box::new(left.clone()), BinaryOp::Mul, Box::new(right.clone()));
        let div = AwkExpression::Binary(Box::new(left.clone()), BinaryOp::Div, Box::new(right.clone()));
        let ctx = make_ctx();
        assert_eq!(to_number(&evaluate_awk_expression(&add, &ctx).unwrap()), 14.0);
        assert_eq!(to_number(&evaluate_awk_expression(&sub, &ctx).unwrap()), 6.0);
        assert_eq!(to_number(&evaluate_awk_expression(&mul, &ctx).unwrap()), 40.0);
        assert_eq!(to_number(&evaluate_awk_expression(&div, &ctx).unwrap()), 2.5);
    }

    #[test]
    fn test_field_and_length_function() {
        let mut ctx = make_ctx();
        // $2 should be "beta"
        let field = AwkExpression::Field(2);
        let v = evaluate_awk_expression(&field, &ctx).unwrap();
        assert_eq!(super::to_string_val(&v), "beta");

        let len = AwkExpression::Function("length".to_string(), vec![field]);
        let lv = evaluate_awk_expression(&len, &ctx).unwrap();
        assert_eq!(super::to_number(&lv) as usize, 4);
    }

    #[test]
    fn test_pattern_expression_truthy() {
        let pat = AwkPattern::Expression("1".to_string());
        let ctx = make_ctx();
        assert!(match_awk_pattern(&pat, &ctx, "line").unwrap());
    }
}
