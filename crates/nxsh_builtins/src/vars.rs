use anyhow::{bail, Result};
use exmex::Express; // Replaced meval with exmex for better C/C++ dependency elimination
use nxsh_core::context::ShellContext;
use nxsh_core::memory_efficient::MemoryEfficientStringBuilder;

// NOTE: We intentionally avoid pulling in the regex crate here so that super-min
// builds (which omit advanced-regex) do not drag in large dependencies. Lightweight
// manual parsers are implemented instead.

/// Evaluate arithmetic expressions and assign to shell variables.
/// Usage examples:
///     let "a = 1+2"
///     let "a += 3"
pub fn let_cli(exprs: &[String], ctx: &ShellContext) -> Result<()> {
    if exprs.is_empty() {
        bail!("let requires expression");
    }
    let joined = exprs.join(" ");
    // Manual parse: find '=' (supports '+='). Allow whitespace around.
    let eq_pos = joined
        .find('=')
        .ok_or_else(|| anyhow::anyhow!("invalid let expression"))?;
    let (lhs_raw, rhs_raw) = joined.split_at(eq_pos);
    let rhs = &rhs_raw[1..]; // skip '='
    let lhs_trim = lhs_raw.trim_end();
    let (var, op_add) = if let Some(stripped) = lhs_trim.strip_suffix('+') {
        (stripped, true)
    } else {
        (lhs_trim, false)
    };
    let var = var.trim();
    if var.is_empty() || !var.chars().next().unwrap().is_ascii_alphabetic() {
        bail!("invalid variable name")
    }
    let rhs = rhs.trim();
    let expr = exmex::parse::<f64>(rhs)?;
    let val: f64 = expr.eval(&[])?;
    let new_val = if op_add {
        ctx.get_var(var)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0)
            + val
    } else {
        val
    };
    ctx.set_var(var, new_val.to_string());
    Ok(())
}

/// `declare` builtin (subset). Supports associative array (-A) and plain variable.
pub fn declare_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    if args.is_empty() {
        // Print all vars
        if let Ok(vars_guard) = ctx.vars.read() {
            for (key, var) in vars_guard.iter() {
                println!("{}={}", key, var.value);
            }
        }
        return Ok(());
    }
    let mut iter = args.iter();
    let mut assoc = false;
    if let Some(flag) = iter.next() {
        if flag == "-A" {
            assoc = true;
        } else {
            iter = args.iter(); // no flag present
        }
    }
    for name in iter {
        if assoc {
            ctx.set_var(name, "__assoc_array__".to_string());
        } else {
            ctx.set_var(name, String::new());
        }
    }
    Ok(())
}

/// `printf` builtin supporting %d %x %s with width/zero-pad.
pub fn printf_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    let format_str = &args[0];
    let mut out = MemoryEfficientStringBuilder::new(format_str.len() * 2);
    let mut arg_iter = args.iter().skip(1);
    let bytes: Vec<char> = format_str.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == '%' {
            i += 1;
            if i >= bytes.len() {
                break;
            }
            let mut zero = false;
            if bytes[i] == '0' {
                zero = true;
                i += 1;
            }
            let mut width_str = MemoryEfficientStringBuilder::new(8);
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                width_str.push(bytes[i]);
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }
            let ty = bytes[i];
            i += 1;
            let width: usize = width_str.into_string().parse().unwrap_or(0);
            let arg = arg_iter
                .next()
                .ok_or_else(|| anyhow::anyhow!("missing printf argument"))?;
            let formatted = match ty {
                'd' => {
                    let v: i64 = arg.parse()?;
                    if width > 0 {
                        if zero {
                            let mut result = MemoryEfficientStringBuilder::new(width + 2);
                            let num_str = v.to_string();
                            let padding = width.saturating_sub(num_str.len());
                            if v < 0 {
                                result.push('-');
                                for _ in 0..padding {
                                    result.push('0');
                                }
                                result.push_str(&num_str[1..]);
                            } else {
                                for _ in 0..padding {
                                    result.push('0');
                                }
                                result.push_str(&num_str);
                            }
                            result.into_string()
                        } else {
                            let mut result = MemoryEfficientStringBuilder::new(width + 2);
                            let num_str = v.to_string();
                            let padding = width.saturating_sub(num_str.len());
                            for _ in 0..padding {
                                result.push(' ');
                            }
                            result.push_str(&num_str);
                            result.into_string()
                        }
                    } else {
                        v.to_string()
                    }
                }
                'x' => {
                    let v: i64 = arg.parse()?;
                    if width > 0 {
                        if zero {
                            let mut result = MemoryEfficientStringBuilder::new(width + 2);
                            let hex_str = format!("{v:x}");
                            let padding = width.saturating_sub(hex_str.len());
                            for _ in 0..padding {
                                result.push('0');
                            }
                            result.push_str(&hex_str);
                            result.into_string()
                        } else {
                            let mut result = MemoryEfficientStringBuilder::new(width + 2);
                            let hex_str = format!("{v:x}");
                            let padding = width.saturating_sub(hex_str.len());
                            for _ in 0..padding {
                                result.push(' ');
                            }
                            result.push_str(&hex_str);
                            result.into_string()
                        }
                    } else {
                        format!("{v:x}")
                    }
                }
                's' => arg.clone(),
                '%' => "%".into(),
                _ => {
                    // Unknown specifier, emit literally
                    let mut lit = MemoryEfficientStringBuilder::new(8);
                    lit.push('%');
                    if zero {
                        lit.push('0');
                    }
                    lit.push_str(&width.to_string());
                    lit.push(ty);
                    lit.into_string()
                }
            };
            out.push_str(&formatted);
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    print!("{}", out.into_string());
    Ok(())
}

/// Adapter function for the builtin command interface
pub fn execute(
    args: &[String],
    _context: &crate::common::BuiltinContext,
) -> crate::common::BuiltinResult<i32> {
    if args.is_empty() {
        return Err(crate::common::BuiltinError::Other(
            "No command specified".to_string(),
        ));
    }

    // Create a minimal shell context for variable operations
    let shell_ctx = ShellContext::new();

    let result = match args[0].as_str() {
        "let" => let_cli(&args[1..], &shell_ctx),
        "declare" => declare_cli(&args[1..], &shell_ctx),
        "printf" => printf_cli(&args[1..]),
        _ => {
            return Err(crate::common::BuiltinError::Other(format!(
                "Unknown command: {}",
                args[0]
            )))
        }
    };

    result.map_err(|e| crate::common::BuiltinError::Other(e.to_string()))?;
    Ok(0)
}
