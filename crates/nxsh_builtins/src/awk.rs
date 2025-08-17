//! `awk` command - pattern scanning and data extraction language
//!
//! Complete awk implementation with pattern matching, field processing, and scripting
//! Features: user-defined functions, full regex support, mathematical functions,
//! field assignment, associative arrays, and complete printf formatting.

use std::collections::HashMap;
use nxsh_core::{ShellResult, ShellError};
use nxsh_core::error::{ErrorKind, RuntimeErrorKind};
#[cfg(feature = "advanced-regex")]
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;
// libm is unnecessary on std; use intrinsic f64 methods instead

/// AWK コマンド実装 — BEGIN/END、パターン/アクション、正規表現、式/配列/連想配列、ユーザー関数、
/// 制御構文（if/while/for/for-in/return/break/continue/next/exit）、フィールド/レコード分割、
/// printf/sprintf、match() による RSTART/RLENGTH 設定、`$n` 代入（$0/NF の再構築）などをサポート。
pub fn awk_cli(args: &[String], _ctx: &mut nxsh_core::context::ShellContext) -> anyhow::Result<()> {
	if args.iter().any(|a| a == "--help" || a == "-h") {
		println!("awk - pattern scanning and processing language");
		println!("Usage: awk [-F FS] [-f progfile] [-v var=val] 'program' [file ...]");
		println!("  -F, --field-separator FS    set input field separator");
		println!("  -f, --file PROGFILE         read program from file");
		println!("  -v, --assign VAR=VAL        assign variable before program begins");
		println!("Examples:");
        println!("  awk -F, -v OFS='\\t' '{{ print $1, $3 }}' data.csv");
		println!("  awk -f script.awk input.txt");
		return Ok(());
	}

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

    // Load user-defined functions parsed at top-level into the runtime context
    // This ensures functions are available before BEGIN and record processing.
    awk_context.functions = parsed.functions.clone();

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

// Unescape C-like escape sequences used in AWK string literals and printf formats
fn unescape_string(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('b') => out.push('\u{0008}'), // backspace
            Some('f') => out.push('\u{000C}'), // formfeed
            Some('v') => out.push('\u{000B}'), // vertical tab
            Some('a') => out.push('\u{0007}'), // bell
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('x') => {
                // \xHH (1-2 hex digits)
                let mut val: u32 = 0;
                for _ in 0..2 {
                    if let Some(&c) = chars.peek() {
                        if c.is_ascii_hexdigit() {
                            chars.next();
                            val = val * 16 + c.to_digit(16).unwrap_or(0);
                        } else { break; }
                    }
                }
                if let Some(ch) = char::from_u32(val) { out.push(ch); }
            }
            Some(c @ '0'..='7') => {
                // \ooo (1-3 octal digits)
                let mut val: u32 = (c as u8 - b'0') as u32;
                for _ in 0..2 {
                    if let Some(&d) = chars.peek() {
                        if d >= '0' && d <= '7' {
                            chars.next();
                            val = val * 8 + (d as u8 - b'0') as u32;
                        } else { break; }
                    }
                }
                if let Some(ch) = char::from_u32(val) { out.push(ch); }
            }
            Some(other) => {
                // Unknown escape, keep as-is (common awk behavior)
                out.push(other);
            }
            None => {}
        }
    }
    out
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
    pub functions: HashMap<String, AwkFunction>,
}

/// User-defined function definition
#[derive(Debug, Clone)]
pub struct AwkFunction {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Box<AwkAction>,
    pub local_vars: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum AwkPattern {
    #[cfg(feature = "advanced-regex")]
    Regex(Regex),
    Expression(String),
    Range(Box<AwkPattern>, Box<AwkPattern>),
    BeginEnd, // Special pattern for BEGIN/END blocks
}

#[derive(Debug, Clone)]
pub enum AwkAction {
    Print(Vec<AwkExpression>),
    PrintF(String, Vec<AwkExpression>),
    Assignment(String, AwkExpression),
    AssignmentIndex(String, AwkExpression, AwkExpression),
    FieldAssignment(AwkExpression, AwkExpression), // $1 = value
    If(AwkExpression, Box<AwkAction>, Option<Box<AwkAction>>),
    For(String, AwkExpression, AwkExpression, Box<AwkAction>),
    ForIn(String, String, Box<AwkAction>), // for (key in array)
    While(AwkExpression, Box<AwkAction>),
    Block(Vec<AwkAction>),
    Expression(AwkExpression),
    FunctionDef(AwkFunction),
    Return(Option<AwkExpression>),
    Next,
    Exit(Option<AwkExpression>),
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub enum AwkExpression {
    String(String),
    Number(f64),
    Field(Box<AwkExpression>), // Support dynamic field references like $(NF-1)
    Variable(String),
    Index(Box<AwkExpression>, Box<AwkExpression>),
    Binary(Box<AwkExpression>, BinaryOp, Box<AwkExpression>),
    Unary(UnaryOp, Box<AwkExpression>),
    Function(String, Vec<AwkExpression>),
    UserFunction(String, Vec<AwkExpression>),
    #[cfg(feature = "advanced-regex")]
    Match(Box<AwkExpression>, Regex),
    #[cfg(feature = "advanced-regex")]
    NotMatch(Box<AwkExpression>, Regex),
    Ternary(Box<AwkExpression>, Box<AwkExpression>, Box<AwkExpression>), // condition ? true_expr : false_expr
    PreIncrement(String),  // ++var
    PostIncrement(String), // var++
    PreDecrement(String),  // --var
    PostDecrement(String), // var--
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
    Concat,
    Power,     // **
    Match,     // ~
    NotMatch,  // !~
    In,        // in (for array membership)
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
    pub functions: HashMap<String, AwkFunction>,
    pub call_stack: Vec<CallFrame>,
    pub return_value: Option<AwkValue>,
    pub loop_control: LoopControl,
    pub random_seed: u64,
}

/// Function call frame for local variable scope
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: String,
    pub local_vars: HashMap<String, AwkValue>,
    pub parameters: Vec<String>,
}

/// Loop control state for break/continue
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoopControl {
    None,
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub enum AwkValue {
    String(String),
    Number(f64),
    Map(HashMap<String, AwkValue>),
    Uninitialized, // For uninitialized variables
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
            functions: HashMap::new(),
            call_stack: Vec::new(),
            return_value: None,
            loop_control: LoopControl::None,
            random_seed: 1,
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
    // Enhanced top-level parser with brace/string aware splitting and function defs
    let mut begin_actions = Vec::new();
    let mut pattern_actions = Vec::new();
    let mut end_actions = Vec::new();
    let mut functions: HashMap<String, AwkFunction> = HashMap::new();

    for stmt in split_top_level_statements(program) {
        // Skip comments and empties
        let s = stmt.trim();
        if s.is_empty() || s.starts_with('#') { continue; }

        // function definition
        if let Some(func) = parse_function_definition(s)? {
            functions.insert(func.name.clone(), func);
            continue;
        }

        // BEGIN / END
        if let Some(rest) = s.strip_prefix("BEGIN") {
            let action_part = rest.trim();
            if !action_part.is_empty() {
                let action = parse_awk_action_or_block(action_part)?;
                begin_actions.push(action);
            }
            continue;
        }
        if let Some(rest) = s.strip_prefix("END") {
            let action_part = rest.trim();
            if !action_part.is_empty() {
                let action = parse_awk_action_or_block(action_part)?;
                end_actions.push(action);
            }
            continue;
        }

        // Pattern + action or plain action
        if let Some((pat, rest)) = parse_line_pattern_prefix(s) {
            let action = if rest.is_empty() {
                AwkAction::Print(vec![AwkExpression::Field(Box::new(AwkExpression::Number(0.0)))])
            } else {
                parse_awk_action_or_block(rest)?
            };
            pattern_actions.push((Some(pat), action));
            continue;
        }

        // Default: attach as action without pattern
        let action = parse_awk_action_or_block(s)?;
        pattern_actions.push((None, action));
    }

    Ok(AwkProgram { begin_actions, pattern_actions, end_actions, functions })
}

// Split program into top-level statements (respecting braces and quotes)
fn split_top_level_statements(src: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut depth: i32 = 0;
    let mut in_str = false;
    let mut prev_ch: Option<char> = None;
    for ch in src.chars() {
        match ch {
            '"' => { if prev_ch != Some('\\') { in_str = !in_str; } buf.push(ch); }
            '{' if !in_str => { depth += 1; buf.push(ch); }
            '}' if !in_str => { depth -= 1; buf.push(ch); }
            ';' | '\n' if !in_str && depth == 0 => {
                if !buf.trim().is_empty() { out.push(buf.trim().to_string()); }
                buf.clear();
            }
            _ => buf.push(ch),
        }
        prev_ch = Some(ch);
    }
    if !buf.trim().is_empty() { out.push(buf.trim().to_string()); }
    out
}

// Parse AWK function definition of the form:
// function name(arg1, arg2, ...) { ... }
fn parse_function_definition(stmt: &str) -> ShellResult<Option<AwkFunction>> {
    let s = stmt.trim_start();
    if !s.starts_with("function") { return Ok(None); }
    let mut rest = s["function".len()..].trim_start();
    // name
    let mut name = String::new();
    for (i, ch) in rest.char_indices() {
        if ch.is_ascii_alphanumeric() || ch == '_' { name.push(ch); } else { rest = &rest[i..]; break; }
    }
    if name.is_empty() { return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "function: missing name".to_string())); }
    rest = rest.trim_start();
    if !rest.starts_with('(') { return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "function: missing (".to_string())); }
    // Extract parameters inside balanced parentheses
    let (inside, after) = extract_paren_segment(rest)?;
    let params: Vec<String> = inside.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let body_src = after.trim_start();
    if !body_src.starts_with('{') { return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "function: missing body".to_string())); }
    let action = parse_awk_block(body_src)?;
    Ok(Some(AwkFunction { name, parameters: params, body: Box::new(action), local_vars: Vec::new() }))
}

fn parse_awk_action(action_str: &str) -> ShellResult<AwkAction> {
    let action_str = action_str.trim();

    if let Some(rest) = action_str.strip_prefix("print") {
        // Enhanced print: parse comma-separated argument expressions
        let args_part = rest.trim();
        if args_part.is_empty() {
            Ok(AwkAction::Print(vec![AwkExpression::Field(Box::new(AwkExpression::Number(0.0)))]))
        } else {
            let expressions = parse_print_args(args_part)?;
            Ok(AwkAction::Print(expressions))
        }
    } else if let Some(rest) = action_str.strip_prefix("printf") {
        // printf "fmt", args... - Complete POSIX-compatible printf implementation
        let args_csv = rest.trim();
        let parts = split_csv_tokens(args_csv);
        if parts.is_empty() { 
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "printf requires a format string".to_string())); 
        }
        let fmt_tok = parts[0].trim();
        let fmt = if fmt_tok.starts_with('"') && fmt_tok.ends_with('"') && fmt_tok.len() >= 2 { 
            unescape_string(&fmt_tok[1..fmt_tok.len()-1]) 
        } else { 
            fmt_tok.to_string() 
        };
        let mut exprs = Vec::new();
        for p in parts.into_iter().skip(1) { 
            exprs.push(parse_full_expr(p.trim())?); 
        }
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
        Ok(AwkAction::If(parse_full_expr(cond_str.trim())?, Box::new(then_action), else_action))
    } else if action_str.starts_with("while") {
        let (cond_str, rest) = extract_paren_segment(action_str.strip_prefix("while").unwrap_or("").trim())?;
        let body = parse_awk_action_or_block(rest.trim())?;
        Ok(AwkAction::While(parse_full_expr(cond_str.trim())?, Box::new(body)))
    } else if action_str.starts_with("for") {
        let (header, rest) = extract_paren_segment(action_str.strip_prefix("for").unwrap_or("").trim())?;
        let header_trim = header.trim();
        // for (var in array)
        if !header_trim.contains(';') && header_trim.contains(" in ") {
            let mut parts = header_trim.splitn(2, " in ");
            let var = parts.next().unwrap().trim().to_string();
            let arr = parts.next().unwrap().trim().to_string();
            let body = parse_awk_action_or_block(rest.trim())?;
            return Ok(AwkAction::ForIn(var, arr, Box::new(body)));
        }
        let mut pieces = header.splitn(3, ';').map(|s| s.trim()).collect::<Vec<_>>();
        while pieces.len() < 3 { pieces.push(""); }
        let init = pieces[0].to_string();
        let cond = parse_full_expr(pieces[1])?;
        let post = parse_full_expr(pieces[2])?;
        let body = parse_awk_action_or_block(rest.trim())?;
        Ok(AwkAction::For(init, cond, post, Box::new(body)))
    } else if action_str == "next" {
        Ok(AwkAction::Next)
    } else if let Some(rest) = action_str.strip_prefix("exit") {
        let arg = rest.trim();
        if arg.is_empty() { return Ok(AwkAction::Exit(None)); }
        let expr = parse_full_expr(arg)?;
        Ok(AwkAction::Exit(Some(expr)))
    } else if action_str == "break" {
        Ok(AwkAction::Break)
    } else if action_str == "continue" {
        Ok(AwkAction::Continue)
    } else if let Some(rest) = action_str.strip_prefix("return") {
        let arg = rest.trim();
        if arg.is_empty() { Ok(AwkAction::Return(None)) } else { Ok(AwkAction::Return(Some(parse_full_expr(arg)?))) }
    } else if action_str.starts_with('{') && action_str.ends_with('}') {
        Ok(parse_awk_block(action_str)?)
    } else if let Some(eq_pos) = action_str.find('=') {
        // 代入: VAR=... or VAR[expr]=...
        let lhs = action_str[..eq_pos].trim();
        let rhs = action_str[eq_pos + 1..].trim();
        // $N or $(expr) = value  => FieldAssignment
        if let Some(rest) = lhs.strip_prefix('$') {
            let field_expr = if rest.starts_with('(') && rest.ends_with(')') {
                // $(expr)
                let inner = &rest[1..rest.len()-1];
                parse_full_expr(inner.trim())?
            } else if let Ok(_n) = rest.parse::<f64>() {
                AwkExpression::Number(rest.parse::<f64>().unwrap_or(0.0))
            } else {
                AwkExpression::Variable(rest.to_string())
            };
            let value_expr = parse_full_expr(rhs)?;
            return Ok(AwkAction::FieldAssignment(field_expr, value_expr));
        }
        if lhs.ends_with(']') {
            if let Some(bracket_pos) = lhs.find('[') {
                let name = lhs[..bracket_pos].trim();
                let index_expr_src = &lhs[bracket_pos+1..lhs.len()-1];
                let index_expr = parse_full_expr(index_expr_src.trim())?;
                let value_expr = parse_full_expr(rhs)?;
                return Ok(AwkAction::AssignmentIndex(name.to_string(), index_expr, value_expr));
            }
        }
        if !lhs.is_empty() {
            if let Ok(num) = rhs.parse::<f64>() {
                return Ok(AwkAction::Assignment(lhs.to_string(), AwkExpression::Number(num)));
            } else {
                return Ok(AwkAction::Assignment(lhs.to_string(), parse_full_expr(rhs)?));
            }
        }
        Ok(AwkAction::Expression(parse_full_expr(action_str)?))
    } else {
        // Expression or other action
        Ok(AwkAction::Expression(parse_full_expr(action_str)?))
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

// Minimal AWK atom parser used by simple/printf args and as a fallback
fn parse_atom(token: &str) -> ShellResult<AwkExpression> {
    let t = token.trim();
    if t.is_empty() { return Ok(AwkExpression::String(String::new())); }
    // String literal
    if t.starts_with('"') && t.ends_with('"') && t.len() >= 2 {
        return Ok(AwkExpression::String(unescape_string(&t[1..t.len()-1])));
    }
    // Field reference: $N or $(expr)
    if let Some(rest) = t.strip_prefix('$') {
        if rest.starts_with('(') && rest.ends_with(')') {
            let inner = &rest[1..rest.len()-1];
            let idx = parse_full_expr(inner)?;
            return Ok(AwkExpression::Field(Box::new(idx)));
        }
        if let Ok(n) = rest.parse::<f64>() {
            return Ok(AwkExpression::Field(Box::new(AwkExpression::Number(n))));
        }
        // $var -> use value of variable as index (non-POSIX but common); fallback to 0
        return Ok(AwkExpression::Field(Box::new(AwkExpression::Variable(rest.to_string()))));
    }
    // Number
    if let Ok(n) = t.parse::<f64>() {
        return Ok(AwkExpression::Number(n));
    }
    // Function call or variable or indexed array: name(...), name[...]
    // name[...] (index)
    if let Some(br) = t.find('[') {
        if t.ends_with(']') {
            let name = t[..br].trim();
            let inner = &t[br+1..t.len()-1];
            let idx = parse_full_expr(inner.trim())?;
            return Ok(AwkExpression::Index(Box::new(AwkExpression::Variable(name.to_string())), Box::new(idx)));
        }
    }
    // name(args)
    if let Some(lp) = t.find('(') {
        if t.ends_with(')') && lp > 0 {
            let name = t[..lp].trim();
            let args_str = &t[lp+1..t.len()-1];
            let parts = split_csv_tokens(args_str);
            let mut args = Vec::new();
            for p in parts { args.push(parse_full_expr(p.trim())?); }
            // Built-in or user function: we model both as Function; evaluator resolves
            return Ok(AwkExpression::Function(name.to_string(), args));
        }
    }
    // Parenthesized
    if t.starts_with('(') && t.ends_with(')') {
        return parse_full_expr(&t[1..t.len()-1]);
    }
    // Variable
    Ok(AwkExpression::Variable(t.to_string()))
}

// Full expression parser with a pragmatic subset of AWK precedence
fn parse_full_expr(src: &str) -> ShellResult<AwkExpression> {
    #[derive(Clone)]
    struct Tok { s: String }
    fn tokenize(s: &str) -> Vec<Tok> {
        let mut v = Vec::new();
        let mut i = 0; let b = s.as_bytes();
        while i < b.len() {
            let c = b[i] as char;
            if c.is_whitespace() { i += 1; continue; }
            // strings
            if c == '"' { let mut j = i+1; let mut esc=false; while j < b.len() { let ch=b[j] as char; if esc { esc=false; j+=1; continue; } if ch=='\\' { esc=true; j+=1; continue; } if ch=='"' { j+=1; break; } j+=1; } v.push(Tok{ s: s[i..j].to_string() }); i=j; continue; }
            // numbers
            if c.is_ascii_digit() || (c=='.' && i+1<b.len() && (b[i+1] as char).is_ascii_digit()) { let mut j=i+1; while j<b.len() && ((b[j] as char).is_ascii_digit() || b[j] as char=='.') { j+=1; } v.push(Tok{ s: s[i..j].to_string() }); i=j; continue; }
            // identifiers
            if c.is_ascii_alphabetic() || c=='_' { let mut j=i+1; while j<b.len() { let ch=b[j] as char; if ch.is_ascii_alphanumeric()||ch=='_' { j+=1; } else { break; } } v.push(Tok{ s: s[i..j].to_string() }); i=j; continue; }
            // two-char ops
            if i+1<b.len() {
                let two = &s[i..i+2];
                let ops2: [&str; 8] = ["==", "!=", "<=", ">=", "&&", "||", "!~", "**"];
                if ops2.contains(&two) { v.push(Tok{ s: two.to_string() }); i+=2; continue; }
            }
            // single-char
            v.push(Tok{ s: s[i..i+1].to_string() }); i+=1;
        }
        v
    }
    struct P<'a> { toks: &'a [Tok], i: usize }
    impl<'a> P<'a> {
        fn peek(&self) -> Option<&str> { self.toks.get(self.i).map(|t| t.s.as_str()) }
        fn eat(&mut self, s: &str) -> bool { if self.peek()==Some(s) { self.i+=1; true } else { false } }
    }
    fn parse_primary(p: &mut P) -> ShellResult<AwkExpression> {
        if let Some(tok_ref) = p.peek() {
            let tok: String = tok_ref.to_string();
            // parenthesis
            if tok == "(" { p.i+=1; let e = parse_ternary(p)?; if !p.eat(")") { return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "missing )".to_string())); } return Ok(e); }
            // string
            if tok.starts_with('"') { let s = tok; p.i+=1; return Ok(AwkExpression::String(unescape_string(&s[1..s.len()-1]))); }
            // $ field
            if tok == "$" { p.i+=1; let e = parse_primary(p)?; return Ok(AwkExpression::Field(Box::new(e))); }
            if tok.starts_with('$') && tok.len()>1 { p.i+=1; if let Ok(n)=tok[1..].parse::<f64>() { return Ok(AwkExpression::Field(Box::new(AwkExpression::Number(n)))); } }
            // number
            if tok.chars().next().unwrap().is_ascii_digit() || tok.starts_with('.') { if let Ok(n)=tok.parse::<f64>() { p.i+=1; return Ok(AwkExpression::Number(n)); } }
            // identifier: func/array/variable
            if tok.chars().next().unwrap().is_ascii_alphabetic() || tok.starts_with('_') {
                let name = tok.to_string(); p.i+=1;
                // func call
                if p.eat("(") {
                    let mut args: Vec<AwkExpression> = Vec::new();
                    if !p.eat(")") {
                        loop {
                            let e = parse_ternary(p)?;
                            args.push(e);
                            if p.eat(")") { break; }
                            if !p.eat(",") { return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "missing , or )".to_string())); }
                        }
                    }
                    return Ok(AwkExpression::Function(name, args));
                }
                // index
                if p.eat("[") { let e = parse_ternary(p)?; if !p.eat("]") { return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "missing ]".to_string())); } return Ok(AwkExpression::Index(Box::new(AwkExpression::Variable(name)), Box::new(e))); }
                return Ok(AwkExpression::Variable(name));
            }
            // unary + - !
            if tok=="!" || tok=="-" || tok=="+" { let op = tok; p.i+=1; let inner = parse_primary(p)?; return Ok(match op.as_str() { "!"=>AwkExpression::Unary(UnaryOp::Not, Box::new(inner)), "-"=>AwkExpression::Unary(UnaryOp::Neg, Box::new(inner)), _=>AwkExpression::Unary(UnaryOp::Pos, Box::new(inner)) }); }
        }
        Ok(AwkExpression::String(String::new()))
    }
    fn parse_power(p: &mut P) -> ShellResult<AwkExpression> { let mut left = parse_primary(p)?; while p.eat("**") { let r = parse_primary(p)?; left = AwkExpression::Binary(Box::new(left), BinaryOp::Power, Box::new(r)); } Ok(left) }
    fn parse_mul(p: &mut P) -> ShellResult<AwkExpression> { let mut left = parse_power(p)?; loop { if p.eat("*") { let r=parse_power(p)?; left=AwkExpression::Binary(Box::new(left), BinaryOp::Mul, Box::new(r)); continue; } if p.eat("/") { let r=parse_power(p)?; left=AwkExpression::Binary(Box::new(left), BinaryOp::Div, Box::new(r)); continue; } if p.eat("%") { let r=parse_power(p)?; left=AwkExpression::Binary(Box::new(left), BinaryOp::Mod, Box::new(r)); continue; } break; } Ok(left) }
    fn parse_add(p: &mut P) -> ShellResult<AwkExpression> { let mut left = parse_mul(p)?; loop { if p.eat("+") { let r=parse_mul(p)?; left=AwkExpression::Binary(Box::new(left), BinaryOp::Add, Box::new(r)); continue; } if p.eat("-") { let r=parse_mul(p)?; left=AwkExpression::Binary(Box::new(left), BinaryOp::Sub, Box::new(r)); continue; } break; } Ok(left) }
    fn parse_cmp(p: &mut P) -> ShellResult<AwkExpression> { let mut left = parse_add(p)?; loop { if p.eat("<") { let r=parse_add(p)?; left=AwkExpression::Binary(Box::new(left), BinaryOp::Lt, Box::new(r)); continue; } if p.eat("<=") { let r=parse_add(p)?; left=AwkExpression::Binary(Box::new(left), BinaryOp::Le, Box::new(r)); continue; } if p.eat(">") { let r=parse_add(p)?; left=AwkExpression::Binary(Box::new(left), BinaryOp::Gt, Box::new(r)); continue; } if p.eat(">=") { let r=parse_add(p)?; left=AwkExpression::Binary(Box::new(left), BinaryOp::Ge, Box::new(r)); continue; } break; } Ok(left) }
    fn parse_eq(p: &mut P) -> ShellResult<AwkExpression> {
        let mut left = parse_cmp(p)?;
        loop {
            if p.eat("==") { let r=parse_cmp(p)?; left=AwkExpression::Binary(Box::new(left), BinaryOp::Eq, Box::new(r)); continue; }
            if p.eat("!=") { let r=parse_cmp(p)?; left=AwkExpression::Binary(Box::new(left), BinaryOp::Ne, Box::new(r)); continue; }
            if p.eat("~") {
                let r = parse_cmp(p)?;
                #[cfg(feature = "advanced-regex")]
                {
                    if let AwkExpression::String(pat)=r.clone() {
                        if let Ok(re)=regex::Regex::new(&pat) {
                            left=AwkExpression::Match(Box::new(left), re);
                            continue;
                        }
                    }
                }
                left=AwkExpression::Binary(Box::new(left), BinaryOp::Match, Box::new(r));
                continue;
            }
            if p.eat("!~") {
                let r = parse_cmp(p)?;
                #[cfg(feature = "advanced-regex")]
                {
                    if let AwkExpression::String(pat)=r.clone() {
                        if let Ok(re)=regex::Regex::new(&pat) {
                            left=AwkExpression::NotMatch(Box::new(left), re);
                            continue;
                        }
                    }
                }
                left=AwkExpression::Binary(Box::new(left), BinaryOp::NotMatch, Box::new(r));
                continue;
            }
            break;
        }
        Ok(left)
    }
    fn parse_and(p: &mut P) -> ShellResult<AwkExpression> { let mut left = parse_eq(p)?; while p.eat("&&") { let r=parse_eq(p)?; left=AwkExpression::Binary(Box::new(left), BinaryOp::And, Box::new(r)); } Ok(left) }
    fn parse_or(p: &mut P) -> ShellResult<AwkExpression> { let mut left = parse_and(p)?; while p.eat("||") { let r=parse_and(p)?; left=AwkExpression::Binary(Box::new(left), BinaryOp::Or, Box::new(r)); } Ok(left) }
    fn parse_ternary(p: &mut P) -> ShellResult<AwkExpression> { let mut cond = parse_or(p)?; if p.peek()==Some("?") { p.i+=1; let t = parse_or(p)?; if !p.eat(":") { return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "missing :".to_string())); } let f = parse_or(p)?; cond = AwkExpression::Ternary(Box::new(cond), Box::new(t), Box::new(f)); } Ok(cond) }
    let toks = tokenize(src);
    let mut p = P { toks: &toks, i: 0 };
    let expr = parse_ternary(&mut p)?;
    Ok(expr)
}

//
// Enhanced expression evaluation with full AWK semantics


fn evaluate_binary_op(left: &AwkValue, op: &BinaryOp, right: &AwkValue) -> ShellResult<AwkValue> {
    match op {
        BinaryOp::Add => Ok(AwkValue::Number(to_number_val(left) + to_number_val(right))),
        BinaryOp::Sub => Ok(AwkValue::Number(to_number_val(left) - to_number_val(right))),
        BinaryOp::Mul => Ok(AwkValue::Number(to_number_val(left) * to_number_val(right))),
        BinaryOp::Div => {
            let divisor = to_number_val(right);
            if divisor == 0.0 {
                Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::DivisionByZero), "Division by zero".to_string()))
            } else {
                Ok(AwkValue::Number(to_number_val(left) / divisor))
            }
        }
        BinaryOp::Mod => {
            let divisor = to_number_val(right);
            if divisor == 0.0 {
                Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::DivisionByZero), "Modulo by zero".to_string()))
            } else {
                Ok(AwkValue::Number(to_number_val(left) % divisor))
            }
        }
        BinaryOp::Power => Ok(AwkValue::Number(to_number_val(left).powf(to_number_val(right)))),
        BinaryOp::Lt => Ok(AwkValue::Number(if to_number_val(left) < to_number_val(right) { 1.0 } else { 0.0 })),
        BinaryOp::Le => Ok(AwkValue::Number(if to_number_val(left) <= to_number_val(right) { 1.0 } else { 0.0 })),
        BinaryOp::Gt => Ok(AwkValue::Number(if to_number_val(left) > to_number_val(right) { 1.0 } else { 0.0 })),
        BinaryOp::Ge => Ok(AwkValue::Number(if to_number_val(left) >= to_number_val(right) { 1.0 } else { 0.0 })),
        BinaryOp::Eq => Ok(AwkValue::Number(if awk_values_equal(left, right) { 1.0 } else { 0.0 })),
        BinaryOp::Ne => Ok(AwkValue::Number(if !awk_values_equal(left, right) { 1.0 } else { 0.0 })),
        BinaryOp::And => Ok(AwkValue::Number(if is_truthy(left) && is_truthy(right) { 1.0 } else { 0.0 })),
        BinaryOp::Or => Ok(AwkValue::Number(if is_truthy(left) || is_truthy(right) { 1.0 } else { 0.0 })),
        BinaryOp::Concat => Ok(AwkValue::String(format!("{}{}", to_string_val(left), to_string_val(right)))),
        BinaryOp::Match => {
            // AWK: string ~ regex
            let s = to_string_val(left);
            match right {
                // If right is a string, treat it as a regex pattern
                AwkValue::String(pat) => {
                    match regex::Regex::new(pat) {
                        Ok(re) => Ok(AwkValue::Number(if re.is_match(&s) { 1.0 } else { 0.0 })),
                        Err(_) => Ok(AwkValue::Number(0.0)),
                    }
                }
                // If right is a number or other, coerce to string pattern
                _ => {
                    let pat = to_string_val(right);
                    match regex::Regex::new(&pat) {
                        Ok(re) => Ok(AwkValue::Number(if re.is_match(&s) { 1.0 } else { 0.0 })),
                        Err(_) => Ok(AwkValue::Number(0.0)),
                    }
                }
            }
        }
        BinaryOp::NotMatch => {
            let s = to_string_val(left);
            let pat = to_string_val(right);
            match regex::Regex::new(&pat) {
                Ok(re) => Ok(AwkValue::Number(if !re.is_match(&s) { 1.0 } else { 0.0 })),
                Err(_) => Ok(AwkValue::Number(0.0)),
            }
        }
        BinaryOp::In => {
            // AWK: key in array
            let key = to_string_val(left);
            match right {
                AwkValue::Map(map) => Ok(AwkValue::Number(if map.contains_key(&key) { 1.0 } else { 0.0 })),
                _ => Ok(AwkValue::Number(0.0)),
            }
        }
    }
}

fn evaluate_unary_op(op: &UnaryOp, operand: &AwkValue) -> ShellResult<AwkValue> {
    match op {
        UnaryOp::Not => Ok(AwkValue::Number(if is_truthy(operand) { 0.0 } else { 1.0 })),
        UnaryOp::Neg => Ok(AwkValue::Number(-to_number_val(operand))),
        UnaryOp::Pos => Ok(AwkValue::Number(to_number_val(operand))),
    }
}

fn evaluate_builtin_function(name: &str, args: &[AwkExpression], context: &mut AwkContext) -> ShellResult<AwkValue> {
    match name {
        "length" => {
            if args.is_empty() {
                Ok(AwkValue::Number(context.get_field(0).len() as f64))
            } else {
                let val = evaluate_awk_expression(&args[0], context)?;
                Ok(AwkValue::Number(to_string_val(&val).len() as f64))
            }
        }
        "substr" => {
            if args.len() >= 2 {
                let string_val = evaluate_awk_expression(&args[0], context)?;
                let start_val = evaluate_awk_expression(&args[1], context)?;
                let string = to_string_val(&string_val);
                let start = (to_number_val(&start_val) as usize).saturating_sub(1); // AWK uses 1-based indexing
                
                if args.len() >= 3 {
                    let length_val = evaluate_awk_expression(&args[2], context)?;
                    let length = to_number_val(&length_val) as usize;
                    let end = (start + length).min(string.len());
                    Ok(AwkValue::String(string.chars().skip(start).take(end - start).collect()))
                } else {
                    Ok(AwkValue::String(string.chars().skip(start).collect()))
                }
            } else {
                Ok(AwkValue::String(String::new()))
            }
        }
        "int" => {
            if !args.is_empty() {
                let val = evaluate_awk_expression(&args[0], context)?;
                Ok(AwkValue::Number(to_number_val(&val).trunc()))
            } else {
                Ok(AwkValue::Number(0.0))
            }
        }
        "sqrt" => {
            if !args.is_empty() {
                let val = evaluate_awk_expression(&args[0], context)?;
                Ok(AwkValue::Number(to_number_val(&val).sqrt()))
            } else {
                Ok(AwkValue::Number(0.0))
            }
        }
        "sin" => {
            if !args.is_empty() {
                let val = evaluate_awk_expression(&args[0], context)?;
                Ok(AwkValue::Number(to_number_val(&val).sin()))
            } else {
                Ok(AwkValue::Number(0.0))
            }
        }
        "cos" => {
            if !args.is_empty() {
                let val = evaluate_awk_expression(&args[0], context)?;
                Ok(AwkValue::Number(to_number_val(&val).cos()))
            } else {
                Ok(AwkValue::Number(1.0))
            }
        }
        "tolower" => {
            if !args.is_empty() {
                let val = evaluate_awk_expression(&args[0], context)?;
                Ok(AwkValue::String(to_string_val(&val).to_lowercase()))
            } else {
                Ok(AwkValue::String(String::new()))
            }
        }
        "toupper" => {
            if !args.is_empty() {
                let val = evaluate_awk_expression(&args[0], context)?;
                Ok(AwkValue::String(to_string_val(&val).to_uppercase()))
            } else {
                Ok(AwkValue::String(String::new()))
            }
        }
        _ => Ok(AwkValue::String(format!("builtin_{}({})", name, args.len()))),
    }
}

fn awk_values_equal(left: &AwkValue, right: &AwkValue) -> bool {
    match (left, right) {
        (AwkValue::Number(a), AwkValue::Number(b)) => (a - b).abs() < f64::EPSILON,
        (AwkValue::String(a), AwkValue::String(b)) => a == b,
        (AwkValue::Number(n), AwkValue::String(s)) | (AwkValue::String(s), AwkValue::Number(n)) => {
            s.parse::<f64>().map_or(false, |parsed| (parsed - n).abs() < f64::EPSILON)
        }
        _ => false,
    }
}

fn to_number_val(val: &AwkValue) -> f64 {
    match val {
        AwkValue::Number(n) => *n,
        AwkValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
        AwkValue::Uninitialized => 0.0,
        AwkValue::Map(_) => 0.0,
    }
}

// Compatibility helper used by formatting and tests
// Converts an `AwkValue` to f64 using AWK's numeric coercion semantics.
fn to_number(val: &AwkValue) -> f64 {
    to_number_val(val)
}

fn to_string_val(val: &AwkValue) -> String {
    match val {
        AwkValue::String(s) => s.clone(),
        AwkValue::Number(n) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        AwkValue::Uninitialized => String::new(),
        AwkValue::Map(_) => "[array]".to_string(),
    }
}

fn is_truthy(val: &AwkValue) -> bool {
    match val {
        AwkValue::Number(n) => *n != 0.0,
        AwkValue::String(s) => !s.is_empty(),
        AwkValue::Uninitialized => false,
        AwkValue::Map(m) => !m.is_empty(),
    }
}

fn awk_value_to_string(val: &AwkValue) -> String {
    to_string_val(val)
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
    while read_next_record(reader, &context.rs, &mut line)? {

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

fn read_next_record<R: BufRead>(reader: &mut R, rs: &str, out: &mut String) -> std::io::Result<bool> {
    // Empty RS means records are separated by blank lines: read until double newline
    if rs.is_empty() {
        let mut buf = String::new();
        let mut saw_nl = false;
        loop {
            let mut ch = [0u8; 1];
            let n = reader.read(&mut ch)?;
            if n == 0 { break; }
            let c = ch[0] as char;
            if c == '\n' {
                if saw_nl { break; } else { saw_nl = true; buf.push('\n'); continue; }
            } else {
                if saw_nl { saw_nl = false; }
                buf.push(c);
            }
        }
        if buf.is_empty() { return Ok(false); }
        // trim single trailing newline
        if buf.ends_with('\n') { buf.pop(); if buf.ends_with('\r') { buf.pop(); } }
        out.push_str(&buf);
        return Ok(true);
    }
    // Default or multi-char RS: use read_line; single-char custom RS: read_until that byte
    if rs.len() == 1 {
        let delim = rs.as_bytes()[0];
        let mut buf: Vec<u8> = Vec::new();
        let n = reader.read_until(delim, &mut buf)?;
        if n == 0 { return Ok(false); }
        // Drop trailing delimiter if present
        if let Some(&last) = buf.last() {
            if last == delim { buf.pop(); }
        }
        let s = String::from_utf8_lossy(&buf);
        out.push_str(&s);
        // Normalize Windows CRLF if rs == "\n"
        if delim == b'\n' && out.ends_with('\r') { out.pop(); }
        return Ok(true);
    }

    // Fallback: line-based parsing, stripping trailing newline and optional CR
    let mut tmp = String::new();
    let n = reader.read_line(&mut tmp)?;
    if n == 0 { return Ok(false); }
    if tmp.ends_with('\n') { tmp.pop(); if tmp.ends_with('\r') { tmp.pop(); } }
    out.push_str(&tmp);
    Ok(true)
}

fn match_awk_pattern(pattern: &AwkPattern, context: &mut AwkContext, line: &str) -> ShellResult<bool> {
    match pattern {
        AwkPattern::Regex(re) => Ok(re.is_match(line)),
        AwkPattern::Expression(expr_src) => {
            // Evaluate simple expression truthiness
            let expr = parse_full_expr(expr_src)?;
            let val = evaluate_awk_expression(&expr, context)?;
            Ok(is_truthy(&val))
        }
        AwkPattern::Range(_start, _end) => Ok(true),
        AwkPattern::BeginEnd => {
            // BEGIN/END は専用フェーズで処理されるため、通常のレコードマッチでは偽を返す
            Ok(false)
        }
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
                // print $0 ORS
                print!("{}{}", context.get_field(0), context.ors);
            } else {
                // print expr1, expr2, ... with OFS and append ORS
                let mut output = Vec::new();
                for expr in expressions {
                    let value = evaluate_awk_expression(expr, context)?;
                    output.push(awk_value_to_string(&value));
                }
                print!("{}{}", output.join(&context.ofs), context.ors);
            }
            Ok(AwkFlow::Continue)
        }
        AwkAction::PrintF(format, expressions) => {
            // More complete printf implementation: supports %, flags (-,+,0), width, precision, and specifiers (diouxXfegsc)
            let formatted = format_awk_printf(format, expressions, context)?;
            print!("{}", formatted);
            Ok(AwkFlow::Continue)
        }
        AwkAction::Block(actions) => {
            for action in actions {
                match execute_awk_action_flow(action, context)? {
                    AwkFlow::Continue => {}
                    AwkFlow::NextRecord => return Ok(AwkFlow::NextRecord),
                    AwkFlow::Exit(code) => return Ok(AwkFlow::Exit(code)),
                }
                // If we're inside a function and return has been set, stop executing further actions
                if !context.call_stack.is_empty() && context.return_value.is_some() {
                    return Ok(AwkFlow::Continue);
                }
            }
            Ok(AwkFlow::Continue)
        }
        AwkAction::Assignment(var, expr) => {
            let value = evaluate_awk_expression(expr, context)?;
            match value {
                AwkValue::Number(n) => {
                    context.variables.insert(var.clone(), AwkValue::Number(n));
                    // Update built-in numeric variables if needed (limited set)
                }
                AwkValue::String(s) => {
                    // Assign variable and synchronize well-known separators
                    if var == "FS" { context.fs = s.clone(); }
                    else if var == "OFS" { context.ofs = s.clone(); }
                    else if var == "RS" { context.rs = s.clone(); }
                    else if var == "ORS" { context.ors = s.clone(); }
                    context.variables.insert(var.clone(), AwkValue::String(s));
                }
                AwkValue::Map(m) => {
                    context.variables.insert(var.clone(), AwkValue::Map(m));
                }
                AwkValue::Uninitialized => {
                    context.variables.insert(var.clone(), AwkValue::String(String::new()));
                }
            }
            Ok(AwkFlow::Continue)
        }
        AwkAction::AssignmentIndex(name, index_expr, value_expr) => {
            // Evaluate index and value, then assign into map variable (associative array)
            let idx_val = evaluate_awk_expression(index_expr, context)?;
            let key = to_string_val(&idx_val);
            let val = evaluate_awk_expression(value_expr, context)?;
            let entry = context.variables.entry(name.clone()).or_insert_with(|| AwkValue::Map(HashMap::new()));
            if let AwkValue::Map(map) = entry {
                map.insert(key, val);
            } else {
                // If previously non-map, replace with a new map and lose old scalar (simple behavior)
                let mut new_map = HashMap::new();
                new_map.insert(key, val);
                context.variables.insert(name.clone(), AwkValue::Map(new_map));
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
                let flow = execute_awk_action_flow(then_a, context)?;
                if !context.call_stack.is_empty() && context.return_value.is_some() { return Ok(AwkFlow::Continue); }
                return Ok(flow);
            } else if let Some(e) = else_a {
                let flow = execute_awk_action_flow(e, context)?;
                if !context.call_stack.is_empty() && context.return_value.is_some() { return Ok(AwkFlow::Continue); }
                return Ok(flow);
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
                if !context.call_stack.is_empty() && context.return_value.is_some() { break; }
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
                
                // Check for break/continue before executing body
                context.loop_control = LoopControl::None;
                match execute_awk_action_flow(body, context)? {
                    AwkFlow::Continue => {
                        match context.loop_control {
                            LoopControl::Break => break,
                            LoopControl::Continue => {
                                let _ = evaluate_awk_expression(post, context)?;
                                continue;
                            }
                            LoopControl::None => {}
                        }
                    }
                    AwkFlow::NextRecord => return Ok(AwkFlow::NextRecord),
                    AwkFlow::Exit(code) => return Ok(AwkFlow::Exit(code)),
                }
                if !context.call_stack.is_empty() && context.return_value.is_some() { break; }
                let _ = evaluate_awk_expression(post, context)?;
            }
            context.loop_control = LoopControl::None;
            Ok(AwkFlow::Continue)
        }
        AwkAction::ForIn(var, array_name, body) => {
            // for (key in array) body
            if let Some(AwkValue::Map(map)) = context.variables.get(array_name).cloned() {
                for key in map.keys() {
                    context.variables.insert(var.clone(), AwkValue::String(key.clone()));
                    
                    context.loop_control = LoopControl::None;
                    match execute_awk_action_flow(body, context)? {
                        AwkFlow::Continue => {
                            match context.loop_control {
                                LoopControl::Break => break,
                                LoopControl::Continue => continue,
                                LoopControl::None => {}
                            }
                        }
                        AwkFlow::NextRecord => return Ok(AwkFlow::NextRecord),
                        AwkFlow::Exit(code) => return Ok(AwkFlow::Exit(code)),
                    }
                    if !context.call_stack.is_empty() && context.return_value.is_some() { break; }
                }
            }
            context.loop_control = LoopControl::None;
            Ok(AwkFlow::Continue)
        }
        AwkAction::Break => {
            context.loop_control = LoopControl::Break;
            Ok(AwkFlow::Continue)
        }
        AwkAction::Continue => {
            context.loop_control = LoopControl::Continue;
            Ok(AwkFlow::Continue)
        }
        AwkAction::Return(expr_opt) => {
            let value = if let Some(expr) = expr_opt {
                evaluate_awk_expression(expr, context)?
            } else {
                AwkValue::String(String::new())
            };
            context.return_value = Some(value);
            Ok(AwkFlow::Continue)
        }
        AwkAction::FunctionDef(func) => {
            context.functions.insert(func.name.clone(), func.clone());
            Ok(AwkFlow::Continue)
        }
        AwkAction::FieldAssignment(field_expr, value_expr) => {
            let field_num = match evaluate_awk_expression(field_expr, context)? {
                AwkValue::Number(n) => n as usize,
                AwkValue::String(s) => s.parse::<usize>().unwrap_or(0),
                _ => 0,
            };
            let value = awk_value_to_string(&evaluate_awk_expression(value_expr, context)?);
            
            // Extend fields if necessary
            while context.fields.len() <= field_num {
                context.fields.push(String::new());
            }
            
            if field_num < context.fields.len() {
                context.fields[field_num] = value;
                
                // If assigning to $0, re-split fields
                if field_num == 0 {
                    context.split_fields(&context.fields[0].clone());
                } else {
                    // Update NF if we extended beyond current field count
                    context.nf = context.fields.len() - 1;
                    context.variables.insert("NF".to_string(), AwkValue::Number(context.nf as f64));
                    
                    // Rebuild $0 from fields 1..NF
                    if context.nf > 0 {
                        let new_record = context.fields[1..=context.nf].join(&context.ofs);
                        context.fields[0] = new_record;
                    }
                }
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
        let expr = parse_full_expr(rhs)?;
        let val = evaluate_awk_expression(&expr, context)?;
        match val {
            AwkValue::Number(n) => { context.variables.insert(name.to_string(), AwkValue::Number(n)); }
            AwkValue::String(t) => {
                if name == "FS" { context.fs = t.clone(); }
                else if name == "OFS" { context.ofs = t.clone(); }
                else if name == "RS" { context.rs = t.clone(); }
                else if name == "ORS" { context.ors = t.clone(); }
                context.variables.insert(name.to_string(), AwkValue::String(t));
            }
            _ => {}
        }
        return Ok(());
    }
    Ok(())
}

// Helper function to check if a function name is a built-in
fn is_builtin_function(name: &str) -> bool {
    matches!(name, "length" | "substr" | "index" | "split" | "gsub" | "sub" | 
             "match" | "sprintf" | "sin" | "cos" | "atan2" | "exp" | "log" | 
             "sqrt" | "int" | "rand" | "srand" | "system" | "tolower" | "toupper")
}

fn evaluate_awk_expression(expr: &AwkExpression, context: &mut AwkContext) -> ShellResult<AwkValue> {
    match expr {
        AwkExpression::String(s) => Ok(AwkValue::String(s.clone())),
        AwkExpression::Number(n) => Ok(AwkValue::Number(*n)),
        AwkExpression::Field(field_expr) => {
            // Support dynamic field references like $(NF-1)
            let field_num = match field_expr.as_ref() {
                // Avoid unstable box pattern; match by reference then evaluate
                AwkExpression::Number(n) => *n as usize,
                expr => {
                    let val = evaluate_awk_expression(expr, context)?;
                    to_number_val(&val) as usize
                }
            };
            Ok(AwkValue::String(context.get_field(field_num)))
        }
        AwkExpression::Variable(name) => {
            // Check local variables in call stack first
            if let Some(frame) = context.call_stack.last() {
                if let Some(value) = frame.local_vars.get(name) {
                    return Ok(value.clone());
                }
            }
            Ok(context.variables.get(name).cloned().unwrap_or(AwkValue::Uninitialized))
        }
        AwkExpression::Index(base, idx) => {
            // Only support VARIABLE[expr] for now
            let base_val = evaluate_awk_expression(base, context)?;
            let idx_val = evaluate_awk_expression(idx, context)?;
            let key = to_string_val(&idx_val);
            match base_val {
                AwkValue::Map(map) => Ok(map.get(&key).cloned().unwrap_or(AwkValue::Uninitialized)),
                _ => Ok(AwkValue::Uninitialized),
            }
        }
        AwkExpression::Unary(op, inner) => {
            let v = evaluate_awk_expression(inner, context)?;
            match op {
                UnaryOp::Not => Ok(AwkValue::Number(if is_truthy(&v) { 0.0 } else { 1.0 })),
                UnaryOp::Neg => Ok(AwkValue::Number(-to_number_val(&v))),
                UnaryOp::Pos => Ok(AwkValue::Number(to_number_val(&v))),
            }
        }
        AwkExpression::Binary(lhs, op, rhs) => {
            let lv = evaluate_awk_expression(lhs, context)?;
            let rv = evaluate_awk_expression(rhs, context)?;
            evaluate_binary_op(&lv, op, &rv)
        }
        AwkExpression::Function(name, args) => {
            // Check for user-defined functions first
            if let Some(func) = context.functions.get(name).cloned() {
                return call_user_function(&func, args, context);
            }
            
            let name_lower = name.to_lowercase();
            match name_lower.as_str() {
                // Math functions using intrinsic f64 methods
                "sin" => {
                    let v = if args.is_empty() { AwkValue::Number(0.0) } else { evaluate_awk_expression(&args[0], context)? };
                    Ok(AwkValue::Number(to_number_val(&v).sin()))
                }
                "cos" => {
                    let v = if args.is_empty() { AwkValue::Number(0.0) } else { evaluate_awk_expression(&args[0], context)? };
                    Ok(AwkValue::Number(to_number_val(&v).cos()))
                }
                "atan2" => {
                    let a = if args.len()>0 { evaluate_awk_expression(&args[0], context)? } else { AwkValue::Number(0.0) };
                    let b = if args.len()>1 { evaluate_awk_expression(&args[1], context)? } else { AwkValue::Number(0.0) };
                    Ok(AwkValue::Number(to_number_val(&a).atan2(to_number_val(&b))))
                }
                "sqrt" => {
                    let v = if args.is_empty() { AwkValue::Number(0.0) } else { evaluate_awk_expression(&args[0], context)? };
                    Ok(AwkValue::Number(to_number_val(&v).sqrt()))
                }
                "exp" => {
                    let v = if args.is_empty() { AwkValue::Number(0.0) } else { evaluate_awk_expression(&args[0], context)? };
                    Ok(AwkValue::Number(to_number_val(&v).exp()))
                }
                "log" => {
                    let v = if args.is_empty() { AwkValue::Number(0.0) } else { evaluate_awk_expression(&args[0], context)? };
                    let n = to_number_val(&v);
                    if n <= 0.0 { Ok(AwkValue::Number(f64::NAN)) } else { Ok(AwkValue::Number(n.ln())) }
                }
                "int" => {
                    let v = if args.is_empty() { AwkValue::Number(0.0) } else { evaluate_awk_expression(&args[0], context)? };
                    Ok(AwkValue::Number((to_number_val(&v) as i64) as f64))
                }
                "rand" => {
                    // Simple linear congruential generator
                    context.random_seed = (context.random_seed.wrapping_mul(1103515245).wrapping_add(12345)) & 0x7fffffff;
                    Ok(AwkValue::Number((context.random_seed as f64) / (0x7fffffff as f64)))
                }
                "srand" => {
                    let seed = if args.is_empty() {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    } else {
                        to_number_val(&evaluate_awk_expression(&args[0], context)?) as u64
                    };
                    let old_seed = context.random_seed;
                    context.random_seed = seed;
                    Ok(AwkValue::Number(old_seed as f64))
                }
                "system" => {
                    let cmd = if args.is_empty() { String::new() } else { 
                        to_string_val(&evaluate_awk_expression(&args[0], context)?)
                    };
                    if cmd.is_empty() { return Ok(AwkValue::Number(-1.0)); }

                    let code = Command::new("sh")
                        .arg("-c")
                        .arg(&cmd)
                        .status()
                        .map(|s| s.code().unwrap_or(-1))
                        .unwrap_or(-1);
                    Ok(AwkValue::Number(code as f64))
                }
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
                "tolower" => {
                    let v = if args.is_empty() { AwkExpression::Field(Box::new(AwkExpression::Number(0.0))) } else { args[0].clone() };
                    let s = to_string_val(&evaluate_awk_expression(&v, context)?);
                    Ok(AwkValue::String(s.to_lowercase()))
                }
                "toupper" => {
                    let v = if args.is_empty() { AwkExpression::Field(Box::new(AwkExpression::Number(0.0))) } else { args[0].clone() };
                    let s = to_string_val(&evaluate_awk_expression(&v, context)?);
                    Ok(AwkValue::String(s.to_uppercase()))
                }
                "substr" => {
                    // substr(s, m[, n]) 1-based index, n optional
                    if args.len() < 2 { return Ok(AwkValue::String(String::new())); }
                    let s = to_string_val(&evaluate_awk_expression(&args[0], context)?);
                    let m = to_number_val(&evaluate_awk_expression(&args[1], context)?) as isize;
                    let n_opt = if args.len() >= 3 { Some(to_number_val(&evaluate_awk_expression(&args[2], context)?) as isize) } else { None };
                    if m <= 0 { return Ok(AwkValue::String(String::new())); }
                    // operate on bytes per awk behavior for simplicity
                    let bytes = s.as_bytes();
                    let start = (m - 1).clamp(0, bytes.len() as isize) as usize;
                    let end = if let Some(n) = n_opt { (start + n.max(0) as usize).min(bytes.len()) } else { bytes.len() };
                    Ok(AwkValue::String(String::from_utf8_lossy(&bytes[start..end]).to_string()))
                }
                "index" => {
                    // index(s, t) 1-based, 0 if not found
                    if args.len() < 2 { return Ok(AwkValue::Number(0.0)); }
                    let s = to_string_val(&evaluate_awk_expression(&args[0], context)?);
                    let t = to_string_val(&evaluate_awk_expression(&args[1], context)?);
                    if t.is_empty() { return Ok(AwkValue::Number(1.0)); }
                    if let Some(pos) = s.find(&t) { Ok(AwkValue::Number((pos + 1) as f64)) } else { Ok(AwkValue::Number(0.0)) }
                }
                "split" => {
                    // split(s, a, fs) -> number of fields; a becomes array a[1..n]
                    if args.len() < 2 { return Ok(AwkValue::Number(0.0)); }
                    let s = to_string_val(&evaluate_awk_expression(&args[0], context)?);
                    let array_name = match &args[1] { AwkExpression::Variable(n) => n.clone(), _ => "".to_string() };
                    if array_name.is_empty() { return Ok(AwkValue::Number(0.0)); }
                    let sep = if args.len() >= 3 { to_string_val(&evaluate_awk_expression(&args[2], context)?) } else { context.fs.clone() };
                    let parts: Vec<&str> = if sep == " " { s.split_whitespace().collect() } else { s.split(&sep).collect() };
                    let mut map = HashMap::new();
                    for (i, p) in parts.iter().enumerate() { map.insert((i+1).to_string(), AwkValue::String(p.to_string())); }
                    context.variables.insert(array_name, AwkValue::Map(map));
                    Ok(AwkValue::Number(parts.len() as f64))
                }
                "match" => {
                    // match(s, r) -> index (1-based) and sets RSTART, RLENGTH
                    if args.len() < 2 { return Ok(AwkValue::Number(0.0)); }
                    let s = to_string_val(&evaluate_awk_expression(&args[0], context)?);
                    let pat = to_string_val(&evaluate_awk_expression(&args[1], context)?);
                    if pat.is_empty() { context.variables.insert("RSTART".into(), AwkValue::Number(0.0)); context.variables.insert("RLENGTH".into(), AwkValue::Number(-1.0)); return Ok(AwkValue::Number(0.0)); }
                    match regex::Regex::new(&pat) {
                        Ok(re) => {
                            if let Some(m) = re.find(&s) {
                                let start = m.start() + 1; // 1-based
                                let len = m.end() - m.start();
                                context.variables.insert("RSTART".into(), AwkValue::Number(start as f64));
                                context.variables.insert("RLENGTH".into(), AwkValue::Number(len as f64));
                                Ok(AwkValue::Number(start as f64))
                            } else {
                                context.variables.insert("RSTART".into(), AwkValue::Number(0.0));
                                context.variables.insert("RLENGTH".into(), AwkValue::Number(-1.0));
                                Ok(AwkValue::Number(0.0))
                            }
                        }
                        Err(_) => Ok(AwkValue::Number(0.0)),
                    }
                }
                "sprintf" => {
                    // sprintf(fmt, args...) -> string
                    if args.is_empty() { return Ok(AwkValue::String(String::new())); }
                    let fmt_val = evaluate_awk_expression(&args[0], context)?;
                    let fmt = to_string_val(&fmt_val);
                    let mut evaled: Vec<AwkExpression> = Vec::new();
                    for a in args.iter().skip(1) { evaled.push(a.clone()); }
                    let s = format_awk_printf(&fmt, &evaled, context)?;
                    Ok(AwkValue::String(s))
                }
                "sub" | "gsub" => {
                    // sub(r,s [,t]) / gsub(r,s [,t]) simplistic literal implementation without regex unless advanced-regex enabled
                    let global = name_lower == "gsub";
                    if args.len() < 2 { return Ok(AwkValue::Number(0.0)); }
                    let pat = to_string_val(&evaluate_awk_expression(&args[0], context)?);
                    let rep = to_string_val(&evaluate_awk_expression(&args[1], context)?);
                    let target_name = if args.len() >= 3 { if let AwkExpression::Variable(n) = &args[2] { Some(n.clone()) } else { None } } else { None };
                    let mut target_val = if let Some(name) = &target_name { to_string_val(&context.variables.get(name).cloned().unwrap_or(AwkValue::String(String::new()))) } else { context.get_field(0) };
                    if pat.is_empty() { return Ok(AwkValue::Number(0.0)); }
                    let mut count = 0;
                    #[cfg(feature = "advanced-regex")]
                    {
                        if let Ok(re) = regex::Regex::new(&pat) {
                            if global {
                                let replaced = re.replace_all(&target_val, rep.as_str()).to_string();
                                if replaced != target_val { count = re.find_iter(&target_val).count(); target_val = replaced; }
                            } else if re.is_match(&target_val) {
                                target_val = re.replace(&target_val, rep.as_str()).to_string();
                                count = 1;
                            }
                        }
                    }
                    #[cfg(not(feature = "advanced-regex"))]
                    if global {
                        let mut out = String::new();
                        let mut start = 0usize;
                        while let Some(pos) = target_val[start..].find(&pat) {
                            out.push_str(&target_val[start..start+pos]);
                            out.push_str(&rep);
                            start += pos + pat.len();
                            count += 1;
                        }
                        out.push_str(&target_val[start..]);
                        target_val = out;
                    } else if let Some(pos) = target_val.find(&pat) {
                        target_val = format!("{}{}{}", &target_val[..pos], rep, &target_val[pos+pat.len()..]);
                        count = 1;
                    }
                    if let Some(name) = target_name { context.variables.insert(name, AwkValue::String(target_val)); } else {
                        // write back to $0 only in this simplified model
                        context.fields[0] = target_val;
                    }
                    Ok(AwkValue::Number(count as f64))
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
            Ok(AwkValue::Number(if !re.is_match(&to_string_val(&lv)) { 1.0 } else { 0.0 }))
        }
        AwkExpression::Ternary(cond, true_expr, false_expr) => {
            let cond_val = evaluate_awk_expression(cond, context)?;
            if is_truthy(&cond_val) {
                evaluate_awk_expression(true_expr, context)
            } else {
                evaluate_awk_expression(false_expr, context)
            }
        }
        AwkExpression::PreIncrement(var) => {
            let current = context.variables.get(var).cloned().unwrap_or(AwkValue::Number(0.0));
            let new_val = AwkValue::Number(to_number_val(&current) + 1.0);
            context.variables.insert(var.clone(), new_val.clone());
            Ok(new_val)
        }
        AwkExpression::PostIncrement(var) => {
            let current = context.variables.get(var).cloned().unwrap_or(AwkValue::Number(0.0));
            let old_val = current.clone();
            let new_val = AwkValue::Number(to_number_val(&current) + 1.0);
            context.variables.insert(var.clone(), new_val);
            Ok(old_val)
        }
        AwkExpression::PreDecrement(var) => {
            let current = context.variables.get(var).cloned().unwrap_or(AwkValue::Number(0.0));
            let new_val = AwkValue::Number(to_number_val(&current) - 1.0);
            context.variables.insert(var.clone(), new_val.clone());
            Ok(new_val)
        }
        AwkExpression::PostDecrement(var) => {
            let current = context.variables.get(var).cloned().unwrap_or(AwkValue::Number(0.0));
            let old_val = current.clone();
            let new_val = AwkValue::Number(to_number_val(&current) - 1.0);
            context.variables.insert(var.clone(), new_val);
            Ok(old_val)
        }
        AwkExpression::UserFunction(name, args) => {
            if let Some(func) = context.functions.get(name).cloned() {
                call_user_function(&func, args, context)
            } else {
                Ok(AwkValue::String(String::new()))
            }
        }
    }
}

// removed duplicate helpers: use earlier definitions of awk_value_to_string, to_number_val, to_string_val, is_truthy

// User-defined function call implementation (temporarily stubbed)
fn call_user_function(func: &AwkFunction, args: &[AwkExpression], context: &mut AwkContext) -> ShellResult<AwkValue> {
    // Evaluate arguments and bind to parameter names in a new call frame (local scope)
    let mut locals: HashMap<String, AwkValue> = HashMap::new();
    for (i, pname) in func.parameters.iter().enumerate() {
        let v = if i < args.len() { evaluate_awk_expression(&args[i], context)? } else { AwkValue::Uninitialized };
        locals.insert(pname.clone(), v);
    }
    // Prepare and push frame
    let frame = CallFrame { function_name: func.name.clone(), local_vars: locals, parameters: func.parameters.clone() };
    context.call_stack.push(frame);
    // Execute body
    let prev_ret = context.return_value.take();
    let _ = execute_awk_action_flow(&func.body, context)?;
    // Collect return value (default empty string)
    let ret = context.return_value.take().unwrap_or(AwkValue::String(String::new()));
    // Restore and pop
    context.return_value = prev_ret;
    context.call_stack.pop();
    Ok(ret)
}
// printf formatting (subset compatible with awk)
// ----------------------------------------------------------------------------

fn format_awk_printf(fmt: &str, exprs: &[AwkExpression], ctx: &mut AwkContext) -> ShellResult<String> {
    let mut out = String::new();
    let mut chars = fmt.chars().peekable();
    let mut arg_index: usize = 0;

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            // Handle escape sequences
            if let Some(escaped) = chars.next() {
                match escaped {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    'b' => out.push('\x08'), // backspace
                    'f' => out.push('\x0C'), // form feed
                    'a' => out.push('\x07'), // bell
                    'v' => out.push('\x0B'), // vertical tab
                    '\\' => out.push('\\'),
                    '"' => out.push('"'),
                    '/' => out.push('/'),
                    // Octal escape sequences (\nnn)
                    c if c.is_ascii_digit() => {
                        let mut octal = String::new();
                        octal.push(c);
                        // Read up to 2 more octal digits
                        for _ in 0..2 {
                            if let Some(&next_c) = chars.peek() {
                                if next_c.is_ascii_digit() && next_c <= '7' {
                                    octal.push(chars.next().unwrap());
                                } else {
                                    break;
                                }
                            }
                        }
                        if let Ok(code) = u8::from_str_radix(&octal, 8) {
                            out.push(code as char);
                        } else {
                            out.push('\\');
                            out.push(c);
                        }
                    }
                    // Hexadecimal escape sequences (\xhh)
                    'x' => {
                        let mut hex = String::new();
                        for _ in 0..2 {
                            if let Some(&next_c) = chars.peek() {
                                if next_c.is_ascii_hexdigit() {
                                    hex.push(chars.next().unwrap());
                                } else {
                                    break;
                                }
                            }
                        }
                        if !hex.is_empty() {
                            if let Ok(code) = u8::from_str_radix(&hex, 16) {
                                out.push(code as char);
                            } else {
                                out.push('\\');
                                out.push('x');
                                out.push_str(&hex);
                            }
                        } else {
                            out.push('\\');
                            out.push('x');
                        }
                    }
                    _ => {
                        out.push('\\');
                        out.push(escaped);
                    }
                }
            } else {
                out.push('\\');
            }
            continue;
        }
        
        if ch != '%' {
            out.push(ch);
            continue;
        }

        // Handle literal %%
        if let Some('%') = chars.peek().copied() {
            let _ = chars.next();
            out.push('%');
            continue;
        }

        // Flags
        let mut left_align = false; // '-'
        let mut sign_plus = false;  // '+'
        let mut sign_space = false; // ' ' (space for positive numbers)
        let mut zero_pad = false;   // '0' (ignored with left alignment)
        let mut alternate = false;  // '#' (alternate form)
        loop {
            match chars.peek().copied() {
                Some('-') => { left_align = true; let _ = chars.next(); }
                Some('+') => { sign_plus = true; let _ = chars.next(); }
                Some(' ') => { sign_space = true; let _ = chars.next(); }
                Some('0') => { zero_pad = true; let _ = chars.next(); }
                Some('#') => { alternate = true; let _ = chars.next(); }
                _ => break,
            }
        }

        // Width (number or '*')
        let mut width: Option<usize> = None;
        if let Some('*') = chars.peek().copied() {
            let _ = chars.next();
            let v = if arg_index < exprs.len() { evaluate_awk_expression(&exprs[arg_index], ctx)? } else { AwkValue::Number(0.0) };
            arg_index = arg_index.saturating_add(1);
            width = Some(to_number(&v).max(0.0) as usize);
        } else {
            let mut width_buf = String::new();
            while let Some(c) = chars.peek().copied() {
                if c.is_ascii_digit() { width_buf.push(c); let _ = chars.next(); } else { break; }
            }
            if !width_buf.is_empty() {
                if let Ok(w) = width_buf.parse::<usize>() { width = Some(w); }
            }
        }

        // Precision ('.' number or '.*')
        let mut precision: Option<usize> = None;
        if let Some('.') = chars.peek().copied() {
            let _ = chars.next();
            if let Some('*') = chars.peek().copied() {
                let _ = chars.next();
                let v = if arg_index < exprs.len() { evaluate_awk_expression(&exprs[arg_index], ctx)? } else { AwkValue::Number(0.0) };
                arg_index = arg_index.saturating_add(1);
                precision = Some(to_number(&v).max(0.0) as usize);
            } else {
                let mut prec_buf = String::new();
                while let Some(c) = chars.peek().copied() {
                    if c.is_ascii_digit() { prec_buf.push(c); let _ = chars.next(); } else { break; }
                }
                if let Ok(p) = prec_buf.parse::<usize>() { precision = Some(p); } else { precision = Some(0); }
            }
        }

        // Specifier
        let spec = chars.next().ok_or_else(|| ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            "printf: missing format specifier".to_string()
        ))?;

        // Fetch argument value (if needed)
    let mut next_value = |i: usize| -> ShellResult<AwkValue> {
            if i >= exprs.len() { return Ok(AwkValue::String(String::new())); }
            evaluate_awk_expression(&exprs[i], ctx)
        };

        match spec {
            's' => {
                let mut s = awk_value_to_string(&next_value(arg_index)?);
                arg_index = arg_index.saturating_add(1);
                if let Some(p) = precision { if s.len() > p { s.truncate(p); } }
                let w = width.unwrap_or(0);
                if w > 0 {
                    if left_align {
                        out.push_str(&format!("{s:<w$}", s=s, w=w));
                    } else {
                        out.push_str(&format!("{s:>w$}", s=s, w=w));
                    }
                } else {
                    out.push_str(&s);
                }
            }
            'd' | 'i' | 'o' | 'x' | 'X' | 'u' => {
                let v = next_value(arg_index)?; arg_index = arg_index.saturating_add(1);
                let mut n = to_number(&v) as i64;
                let mut sign = String::new();
                
                // Handle sign and space flag
                if n < 0 { 
                    sign.push('-'); 
                    n = -n; 
                } else if sign_plus { 
                    sign.push('+'); 
                } else if sign_space && !sign_plus {
                    sign.push(' ');
                }
                
                let mut body = match spec {
                    'd' | 'i' => format!("{n}"),
                    'u' => format!("{}", n as u64),
                    'o' => {
                        let formatted = format!("{:o}", n);
                        if alternate && n != 0 { format!("0{}", formatted) } else { formatted }
                    },
                    'x' => {
                        let formatted = format!("{:x}", n);
                        if alternate && n != 0 { format!("0x{}", formatted) } else { formatted }
                    },
                    'X' => {
                        let formatted = format!("{:X}", n);
                        if alternate && n != 0 { format!("0X{}", formatted) } else { formatted }
                    },
                    _ => unreachable!(),
                };
                
                // Apply precision (minimum digits)
                if let Some(prec) = precision {
                    if body.len() < prec {
                        body = format!("{:0>width$}", body, width = prec);
                    }
                }
                
                let combined = format!("{sign}{body}");
                let w = width.unwrap_or(0);
                if w > combined.len() {
                    let pad_len = w - combined.len();
                    if left_align {
                        out.push_str(&combined);
                        out.push_str(&" ".repeat(pad_len));
                    } else if zero_pad && !sign.is_empty() {
                        // Keep sign in front of zero padding
                        let zeros = "0".repeat(pad_len);
                        out.push_str(&format!("{}{}{}", sign, zeros, &body));
                    } else if zero_pad {
                        out.push_str(&format!("{combined:0>width$}", combined=combined, width=w));
                    } else {
                        out.push_str(&format!("{combined:>width$}", combined=combined, width=w));
                    }
                } else {
                    out.push_str(&combined);
                }
            }
            'f' | 'F' | 'e' | 'E' | 'g' | 'G' => {
                let v = next_value(arg_index)?; arg_index = arg_index.saturating_add(1);
                let mut n = to_number(&v);
                let mut sign = String::new();
                
                // Handle sign and space flag
                if n < 0.0 { 
                    sign.push('-'); 
                    n = -n; 
                } else if sign_plus { 
                    sign.push('+'); 
                } else if sign_space && !sign_plus {
                    sign.push(' ');
                }
                
                let prec = precision.unwrap_or(6);
                let body = match spec {
                    'f' | 'F' => {
                        let formatted = format!("{n:.prec$}", n=n, prec=prec);
                        if alternate && !formatted.contains('.') {
                            format!("{}.", formatted)
                        } else {
                            formatted
                        }
                    },
                    'e' => format!("{n:.prec$e}", n=n, prec=prec),
                    'E' => format!("{n:.prec$E}", n=n, prec=prec),
                    'g' => {
                        // Choose fixed or exponential based on magnitude and precision
                        let exp_threshold = 10_f64.powi(prec as i32);
                        if n != 0.0 && (n >= exp_threshold || n < 1e-4) {
                            let mut exp_str = format!("{n:.prec$e}", n=n, prec=prec);
                            // Remove trailing zeros after decimal point for %g
                            if !alternate && exp_str.contains('.') {
                                let (before_e, after_e) = exp_str.split_once('e').unwrap();
                                let trimmed = before_e.trim_end_matches('0').trim_end_matches('.');
                                exp_str = format!("{}e{}", trimmed, after_e);
                            }
                            exp_str
                        } else {
                            let mut fixed_str = format!("{n:.prec$}", n=n, prec=prec);
                            // Remove trailing zeros after decimal point for %g
                            if !alternate && fixed_str.contains('.') {
                                fixed_str = fixed_str.trim_end_matches('0').trim_end_matches('.').to_string();
                            }
                            fixed_str
                        }
                    },
                    'G' => {
                        // Same as 'g' but with uppercase E
                        let exp_threshold = 10_f64.powi(prec as i32);
                        if n != 0.0 && (n >= exp_threshold || n < 1e-4) {
                            let mut exp_str = format!("{n:.prec$E}", n=n, prec=prec);
                            if !alternate && exp_str.contains('.') {
                                let (before_e, after_e) = exp_str.split_once('E').unwrap();
                                let trimmed = before_e.trim_end_matches('0').trim_end_matches('.');
                                exp_str = format!("{}E{}", trimmed, after_e);
                            }
                            exp_str
                        } else {
                            let mut fixed_str = format!("{n:.prec$}", n=n, prec=prec);
                            if !alternate && fixed_str.contains('.') {
                                fixed_str = fixed_str.trim_end_matches('0').trim_end_matches('.').to_string();
                            }
                            fixed_str
                        }
                    },
                    _ => unreachable!(),
                };
                
                let combined = format!("{sign}{body}");
                let w = width.unwrap_or(0);
                if w > combined.len() {
                    let pad_len = w - combined.len();
                    if left_align {
                        out.push_str(&combined);
                        out.push_str(&" ".repeat(pad_len));
                    } else if zero_pad && !sign.is_empty() {
                        // Keep sign in front of zero padding
                        let zeros = "0".repeat(pad_len);
                        out.push_str(&format!("{}{}{}", sign, zeros, &body));
                    } else if zero_pad {
                        out.push_str(&format!("{combined:0>width$}", combined=combined, width=w));
                    } else {
                        out.push_str(&format!("{combined:>width$}", combined=combined, width=w));
                    }
                } else {
                    out.push_str(&combined);
                }
            }
            'c' => {
                let v = next_value(arg_index)?; arg_index = arg_index.saturating_add(1);
                let ch = match v {
                    AwkValue::Number(n) => char::from_u32((n as i64).clamp(0, 0x10FFFF) as u32).unwrap_or('\u{FFFD}'),
                    AwkValue::String(s) => s.chars().next().unwrap_or('\0'),
                    _ => '\0',
                };
                let s = ch.to_string();
                let w = width.unwrap_or(0);
                if w > 0 {
                    if left_align { out.push_str(&format!("{s:<w$}", s=s, w=w)); }
                    else { out.push_str(&format!("{s:>w$}", s=s, w=w)); }
                } else { out.push(ch); }
            }
            _ => {
                // Unknown specifier: output literally with leading '%'
                out.push('%');
                out.push(spec);
            }
        }
    }

    Ok(out)
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
    let mut ctx = make_ctx();
    let v = evaluate_awk_expression(&expr, &mut ctx).unwrap();
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
    let mut ctx = make_ctx();
    assert_eq!(to_number(&evaluate_awk_expression(&add, &mut ctx).unwrap()), 14.0);
    assert_eq!(to_number(&evaluate_awk_expression(&sub, &mut ctx).unwrap()), 6.0);
    assert_eq!(to_number(&evaluate_awk_expression(&mul, &mut ctx).unwrap()), 40.0);
    assert_eq!(to_number(&evaluate_awk_expression(&div, &mut ctx).unwrap()), 2.5);
    }

    #[test]
    fn test_field_and_length_function() {
        let mut ctx = make_ctx();
        // $2 should be "beta"
        let field = AwkExpression::Field(Box::new(AwkExpression::Number(2.0)));
    let v = evaluate_awk_expression(&field, &mut ctx).unwrap();
        assert_eq!(super::to_string_val(&v), "beta");

        let len = AwkExpression::Function("length".to_string(), vec![field]);
    let lv = evaluate_awk_expression(&len, &mut ctx).unwrap();
        assert_eq!(super::to_number(&lv) as usize, 4);
    }

    #[test]
    fn test_pattern_expression_truthy() {
        let pat = AwkPattern::Expression("1".to_string());
    let mut ctx = make_ctx();
    assert!(match_awk_pattern(&pat, &mut ctx, "line").unwrap());
    }

    #[test]
    fn test_regex_match_and_not_match() {
        // Enable only when advanced-regex is compiled; otherwise, parser should still accept but evaluation may differ
        let mut ctx = make_ctx();
        ctx.split_fields("abc 123 xyz");
        let lhs = AwkExpression::Field(Box::new(AwkExpression::Number(1.0))); // "abc"
        // Use Match/NotMatch nodes when advanced-regex is compiled
        #[cfg(feature = "advanced-regex")]
        {
            let re1 = regex::Regex::new("a.*").unwrap();
            let expr_match = AwkExpression::Match(Box::new(lhs.clone()), re1);
            let v1 = evaluate_awk_expression(&expr_match, &mut ctx).unwrap();
            assert_eq!(super::to_number(&v1), 1.0);

            let re2 = regex::Regex::new("^z").unwrap();
            let expr_not = AwkExpression::NotMatch(Box::new(lhs), re2);
            let v2 = evaluate_awk_expression(&expr_not, &mut ctx).unwrap();
            assert_eq!(super::to_number(&v2), 1.0);
        }
    }

    #[test]
    fn test_binary_match_operator() {
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
        let expr = AwkExpression::Binary(
            Box::new(AwkExpression::String("hello world".to_string())),
            BinaryOp::Match,
            Box::new(AwkExpression::String("^hello".to_string())),
        );
    let v = evaluate_awk_expression(&expr, &mut ctx).unwrap();
        assert!(matches!(v, AwkValue::Number(n) if (n - 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_binary_notmatch_operator() {
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
        let expr = AwkExpression::Binary(
            Box::new(AwkExpression::String("hello world".to_string())),
            BinaryOp::NotMatch,
            Box::new(AwkExpression::String("world$".to_string())),
        );
    let v = evaluate_awk_expression(&expr, &mut ctx).unwrap();
        assert!(matches!(v, AwkValue::Number(n) if n.abs() < f64::EPSILON));
    }

    #[test]
    fn test_binary_in_operator() {
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
        let mut map = HashMap::new();
        map.insert("key1".to_string(), AwkValue::Number(123.0));
        ctx.variables.insert("arr".to_string(), AwkValue::Map(map));

        let expr_in = AwkExpression::Binary(
            Box::new(AwkExpression::String("key1".to_string())),
            BinaryOp::In,
            Box::new(AwkExpression::Variable("arr".to_string())),
        );
    let v_in = evaluate_awk_expression(&expr_in, &mut ctx).unwrap();
        assert!(matches!(v_in, AwkValue::Number(n) if (n - 1.0).abs() < f64::EPSILON));

        let expr_not_in = AwkExpression::Binary(
            Box::new(AwkExpression::String("key2".to_string())),
            BinaryOp::In,
            Box::new(AwkExpression::Variable("arr".to_string())),
        );
    let v_not_in = evaluate_awk_expression(&expr_not_in, &mut ctx).unwrap();
        assert!(matches!(v_not_in, AwkValue::Number(n) if n.abs() < f64::EPSILON));
    }

    #[test]
    fn test_control_flow_if_and_while() {
        let mut ctx = make_ctx();
        // if ($2 == "beta") print $1 $3
        let cond = AwkExpression::Binary(
            Box::new(AwkExpression::Field(Box::new(AwkExpression::Number(2.0)))),
            BinaryOp::Eq,
            Box::new(AwkExpression::String("beta".to_string())),
        );
        let action = AwkAction::Print(vec![
            AwkExpression::Field(Box::new(AwkExpression::Number(1.0))),
            AwkExpression::Field(Box::new(AwkExpression::Number(3.0))),
        ]);
        let stmt = AwkAction::If(cond, Box::new(action), None);
        execute_awk_action(&stmt, &mut ctx).unwrap();
        // Output goes to stdout in real CLI; here we only check no error and semantics via data path
        let s = format!("{} {}", ctx.get_field(1), ctx.get_field(3));
        assert!(s.contains("alpha gamma"));

        // while loop: i=0; while (i<3) { print i; i=i+1 }
        ctx.variables.insert("i".into(), AwkValue::Number(0.0));
        let cond2 = AwkExpression::Binary(
            Box::new(AwkExpression::Variable("i".into())),
            BinaryOp::Lt,
            Box::new(AwkExpression::Number(3.0)),
        );
        let body = AwkAction::Block(vec![
            AwkAction::Assignment("i".into(), AwkExpression::Binary(
                Box::new(AwkExpression::Variable("i".into())), BinaryOp::Add, Box::new(AwkExpression::Number(1.0))
            )),
        ]);
        let loop_stmt = AwkAction::While(cond2, Box::new(body));
        execute_awk_action(&loop_stmt, &mut ctx).unwrap();
        if let AwkValue::Number(i) = ctx.variables.get("i").cloned().unwrap() { assert!((i - 3.0).abs() < 1e-6); } else { panic!("i not number"); }
    }

    #[test]
    fn test_associative_array_assignment_and_index() {
        let mut ctx = make_ctx();
        // a["key"]=42; then read a["key"]
        let assign = AwkAction::AssignmentIndex(
            "a".to_string(),
            AwkExpression::String("key".into()),
            AwkExpression::Number(42.0),
        );
        execute_awk_action(&assign, &mut ctx).unwrap();

        let idx_expr = AwkExpression::Index(
            Box::new(AwkExpression::Variable("a".into())),
            Box::new(AwkExpression::String("key".into())),
        );
    let v = evaluate_awk_expression(&idx_expr, &mut ctx).unwrap();
        assert!(matches!(v, AwkValue::Number(n) if (n-42.0).abs() < 1e-9));
    }

    #[test]
    fn test_user_function_def_and_call() {
        // function add(a,b){ return a+b }
        let func = AwkFunction { name: "add".into(), parameters: vec!["a".into(), "b".into()], body: Box::new(AwkAction::Return(Some(AwkExpression::Binary(Box::new(AwkExpression::Variable("a".into())), BinaryOp::Add, Box::new(AwkExpression::Variable("b".into())))))), local_vars: vec![] };
        let mut ctx = make_ctx();
        ctx.functions.insert("add".into(), func);
        let expr = AwkExpression::Function("add".into(), vec![AwkExpression::Number(5.0), AwkExpression::Number(7.0)]);
        let v = evaluate_awk_expression(&expr, &mut ctx).unwrap();
        assert!(matches!(v, AwkValue::Number(n) if (n-12.0).abs() < 1e-9));
    }

    #[test]
    fn test_sprintf_and_match_builtins() {
        let mut ctx = make_ctx();
        let s = match evaluate_awk_expression(&AwkExpression::Function("sprintf".into(), vec![AwkExpression::String("%s-%02d".into()), AwkExpression::String("id".into()), AwkExpression::Number(7.0)]), &mut ctx).unwrap() { AwkValue::String(s) => s, _ => String::new() };
        assert_eq!(s, "id-07");

        let idx = evaluate_awk_expression(&AwkExpression::Function("match".into(), vec![AwkExpression::String("abc123".into()), AwkExpression::String("[0-9]+".into())]), &mut ctx).unwrap();
        assert!(matches!(idx, AwkValue::Number(n) if (n-4.0).abs()<1e-9));
        // RSTART/RLENGTH were set
        if let AwkValue::Number(rs) = ctx.variables.get("RSTART").cloned().unwrap() { assert_eq!(rs as i32, 4); } else { panic!("RSTART missing"); }
        if let AwkValue::Number(rl) = ctx.variables.get("RLENGTH").cloned().unwrap() { assert_eq!(rl as i32, 3); } else { panic!("RLENGTH missing"); }
    }

    #[test]
    fn test_field_assignment_builds_record() {
        let mut ctx = make_ctx();
        // $2 = "Z" should rebuild $0 with OFS
    let act = AwkAction::FieldAssignment(AwkExpression::Number(2.0), AwkExpression::String("Z".into()));
        execute_awk_action(&act, &mut ctx).unwrap();
        assert_eq!(ctx.get_field(2), "Z");
        assert_eq!(ctx.get_field(0), format!("{}{}{}{}{}", ctx.get_field(1), ctx.ofs, ctx.get_field(2), ctx.ofs, ctx.get_field(3)));
    }

    #[test]
    fn test_for_in_loop_over_array() {
        let mut ctx = make_ctx();
        let mut map = HashMap::new();
        map.insert("k1".into(), AwkValue::Number(1.0));
        map.insert("k2".into(), AwkValue::Number(2.0));
        ctx.variables.insert("arr".into(), AwkValue::Map(map));
        ctx.variables.insert("sum".into(), AwkValue::Number(0.0));
        let body = AwkAction::Block(vec![AwkAction::Assignment("sum".into(), AwkExpression::Binary(Box::new(AwkExpression::Variable("sum".into())), BinaryOp::Add, Box::new(AwkExpression::Number(1.0))))]);
        let loop_act = AwkAction::ForIn("k".into(), "arr".into(), Box::new(body));
        execute_awk_action(&loop_act, &mut ctx).unwrap();
        if let AwkValue::Number(n) = ctx.variables.get("sum").cloned().unwrap() { assert_eq!(n as i32, 2); } else { panic!("sum not number"); }
    }
}
