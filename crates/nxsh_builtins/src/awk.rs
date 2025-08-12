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
                let action = parse_awk_action(action_part)?;
                begin_actions.push(action);
            }
        } else if let Some(rest) = line.strip_prefix("END") {
            current_section = "end";
            let action_part = rest.trim();
            if !action_part.is_empty() {
                let action = parse_awk_action(action_part)?;
                end_actions.push(action);
            }
        } else {
            // パターン + アクション簡易検出: /regex/ { action } もしくは /regex/ action
            if let Some(after_first) = line.strip_prefix('/') {
                if let Some(second) = after_first.find('/') {
                    let pattern_str = &after_first[..second];
                    #[cfg(feature = "advanced-regex")]
                    if let Ok(re) = Regex::new(pattern_str) {
                        let rest = after_first[second + 1..].trim();
                        // If the rest begins with a block, parse the block content; otherwise parse as a single action.
                        let action = if rest.is_empty() {
                            AwkAction::Print(vec![AwkExpression::Field(0)])
                        } else if rest.starts_with('{') {
                            // Ensure it ends with '}' for the simplified parser; otherwise treat as expression
                            if rest.ends_with('}') {
                                parse_awk_action(rest)?
                            } else {
                                // Fallback: incomplete block, treat as expression
                                parse_awk_action(rest)?
                            }
                        } else {
                            parse_awk_action(rest)?
                        };
                        pattern_actions.push((Some(AwkPattern::Regex(re)), action));
                        continue;
                    }
                }
            }
            let action = parse_awk_action(line)?;
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
        // Simple print statement
        let args_part = rest.trim();
        if args_part.is_empty() {
            Ok(AwkAction::Print(vec![AwkExpression::Field(0)]))
        } else {
            // Parse print arguments (simplified)
            let expressions = vec![AwkExpression::String(args_part.to_string())];
            Ok(AwkAction::Print(expressions))
        }
    } else if action_str.starts_with('{') && action_str.ends_with('}') {
        // Block action
        let inner = &action_str[1..action_str.len() - 1];
        let action = parse_awk_action(inner)?;
        Ok(AwkAction::Block(vec![action]))
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

fn process_awk_stream<R: BufRead>(
    reader: &mut R,
    program: &AwkProgram,
    context: &mut AwkContext,
    filename: &str,
) -> ShellResult<()> {
    context.filename = filename.to_string();
    context.variables.insert("FILENAME".to_string(), AwkValue::String(filename.to_string()));

    let mut line = String::new();
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
        for (pattern, action) in &program.pattern_actions {
            let should_execute = if let Some(pattern) = pattern {
                match_awk_pattern(pattern, context, &line)?
            } else {
                true // No pattern means always execute
            };

            if should_execute {
                execute_awk_action(action, context)?;
            }
        }

        line.clear();
    }

    Ok(())
}

fn match_awk_pattern(pattern: &AwkPattern, _context: &AwkContext, line: &str) -> ShellResult<bool> {
    match pattern {
        AwkPattern::Regex(re) => Ok(re.is_match(line)),
        AwkPattern::Expression(_expr) => {
            // Simplified - would need full expression evaluation
            Ok(true)
        }
    AwkPattern::Range(_start, _end) => {
            // Simplified range matching
            Ok(true)
        }
    }
}

fn execute_awk_action(action: &AwkAction, context: &mut AwkContext) -> ShellResult<()> {
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
        }
        AwkAction::Block(actions) => {
            for action in actions {
                execute_awk_action(action, context)?;
            }
        }
        AwkAction::Assignment(var, expr) => {
            let value = evaluate_awk_expression(expr, context)?;
            match value {
                AwkValue::Number(n) => { context.variables.insert(var.clone(), AwkValue::Number(n)); }
                AwkValue::String(s) => { context.variables.insert(var.clone(), AwkValue::String(s)); }
            }
        }
        AwkAction::Expression(expr) => {
            evaluate_awk_expression(expr, context)?;
        }
        _ => {
            // Other actions not implemented in this simplified version
        }
    }
    Ok(())
}

fn evaluate_awk_expression(expr: &AwkExpression, context: &AwkContext) -> ShellResult<AwkValue> {
    match expr {
        AwkExpression::String(s) => Ok(AwkValue::String(s.clone())),
        AwkExpression::Number(n) => Ok(AwkValue::Number(*n)),
        AwkExpression::Field(index) => {
            Ok(AwkValue::String(context.get_field(*index)))
        }
        AwkExpression::Variable(name) => {
            Ok(context.variables.get(name).cloned().unwrap_or(AwkValue::String("".to_string())))
        }
        _ => {
            // Other expressions not implemented in this simplified version
            Ok(AwkValue::String("".to_string()))
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

 
