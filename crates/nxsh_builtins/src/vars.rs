use anyhow::{bail, Result};
use meval::Expr;
use std::str::FromStr;
use nxsh_core::context::ShellContext;
use regex::Regex;

/// Evaluate arithmetic expressions and assign to shell variables.
/// Usage examples:
///     let "a = 1+2"
///     let "a += 3"
pub fn let_cli(exprs: &[String], ctx: &ShellContext) -> Result<()> {
    if exprs.is_empty() {
        bail!("let requires expression");
    }
    let joined = exprs.join(" ");
    // Support patterns: "name = expr" or "name += expr"
    let assign_re = Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)\s*(\+?=)\s*(.+)$")?;
    let caps = assign_re.captures(&joined).ok_or_else(|| anyhow::anyhow!("invalid let expression"))?;
    let var = &caps[1];
    let op = &caps[2];
    let rhs = &caps[3];
    // Evaluate RHS numeric expression
    let val: f64 = Expr::from_str(rhs)?.eval()?;
    let new_val = if op == "+=" {
        let cur = ctx.get_var(var).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        cur + val
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
        for entry in ctx.env.iter() {
            println!("{}={}", entry.key(), entry.value());
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
            ctx.set_var(name, "__assoc_array__".into());
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
    let mut out = String::new();
    let mut arg_iter = args.iter().skip(1);
    let spec_re = Regex::new(r"%(0?)(\d*)([dxs])")?;
    let mut last = 0;
    for spec in spec_re.find_iter(format_str) {
        // push preceding literal text
        out.push_str(&format_str[last..spec.start()]);
        last = spec.end();
        let caps = spec_re.captures(spec.as_str()).unwrap();
        let zero = &caps[1] == "0";
        let width: usize = caps[2].parse().unwrap_or(0);
        let ty = &caps[3];
        let arg = arg_iter.next().ok_or_else(|| anyhow::anyhow!("missing printf argument"))?;
        let formatted = match ty {
            "d" => {
                let v: i64 = arg.parse()?;
                if width > 0 {
                    if zero { format!("{:0width$}", v, width = width) } else { format!("{:width$}", v, width = width) }
                } else { format!("{}", v) }
            }
            "x" => {
                let v: i64 = arg.parse()?;
                if width > 0 {
                    if zero { format!("{:0width$x}", v, width = width) } else { format!("{:width$x}", v, width = width) }
                } else { format!("{:x}", v) }
            }
            "s" => arg.clone(),
            _ => spec.as_str().into(),
        };
        out.push_str(&formatted);
    }
    // push remainder of format string
    out.push_str(&format_str[last..]);
    print!("{}", out);
    Ok(())
} 