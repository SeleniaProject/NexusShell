//! `expr` builtin - evaluate simple expressions.
//!
//! Usage:
//!   expr EXPRESSION
//!
//! Supported operations:
//!   Arithmetic: +, -, *, /, %
//!   Comparison: =, !=, <, <=, >, >=
//!   Logical: &, |
//!   String: length STRING, substr STRING POS LENGTH, index STRING CHARS
//!   Pattern: match STRING REGEX
//!
//! Examples:
//!   expr 1 + 2
//!   expr 5 \* 3
//!   expr length "hello"
//!   expr substr "hello" 2 3

use anyhow::{anyhow, Result};
use regex::Regex;

/// Entry point for the expr builtin.
pub fn expr_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("expr: missing operand"));
    }

    let result = evaluate_expression(args)?;
    println!("{}", result);
    
    // expr returns exit code 1 if result is 0 or empty string
    if result == "0" || result.is_empty() {
        std::process::exit(1);
    }
    
    Ok(())
}

fn evaluate_expression(args: &[String]) -> Result<String> {
    if args.is_empty() {
        return Ok("0".to_string());
    }

    // Handle special functions first
    if args.len() >= 2 {
        match args[0].as_str() {
            "length" => {
                if args.len() != 2 {
                    return Err(anyhow!("expr: length requires exactly one argument"));
                }
                return Ok(args[1].len().to_string());
            }
            "substr" => {
                if args.len() != 4 {
                    return Err(anyhow!("expr: substr requires exactly three arguments"));
                }
                let string = &args[1];
                let pos: usize = args[2].parse()
                    .map_err(|_| anyhow!("expr: non-numeric argument"))?;
                let length: usize = args[3].parse()
                    .map_err(|_| anyhow!("expr: non-numeric argument"))?;
                
                // expr uses 1-based indexing
                let start = if pos > 0 { pos - 1 } else { 0 };
                let end = (start + length).min(string.len());
                
                if start >= string.len() {
                    return Ok("".to_string());
                }
                
                return Ok(string[start..end].to_string());
            }
            "index" => {
                if args.len() != 3 {
                    return Err(anyhow!("expr: index requires exactly two arguments"));
                }
                let string = &args[1];
                let chars = &args[2];
                
                for (i, ch) in string.char_indices() {
                    if chars.contains(ch) {
                        return Ok((i + 1).to_string()); // 1-based indexing
                    }
                }
                return Ok("0".to_string());
            }
            "match" => {
                if args.len() != 3 {
                    return Err(anyhow!("expr: match requires exactly two arguments"));
                }
                let string = &args[1];
                let pattern = &args[2];
                
                match Regex::new(pattern) {
                    Ok(re) => {
                        if let Some(mat) = re.find(string) {
                            return Ok(mat.len().to_string());
                        } else {
                            return Ok("0".to_string());
                        }
                    }
                    Err(_) => return Err(anyhow!("expr: invalid regular expression")),
                }
            }
            _ => {}
        }
    }

    // Parse and evaluate the expression
    evaluate_infix_expression(args)
}

fn evaluate_infix_expression(args: &[String]) -> Result<String> {
    // Convert to a single expression string for easier parsing
    let expr = args.join(" ");
    let tokens = tokenize(&expr);
    
    if tokens.is_empty() {
        return Ok("0".to_string());
    }
    
    if tokens.len() == 1 {
        return Ok(tokens[0].clone());
    }
    
    // Handle binary operations with precedence
    evaluate_tokens(&tokens)
}

fn tokenize(expr: &str) -> Vec<String> {
    expr.split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

fn evaluate_tokens(tokens: &[String]) -> Result<String> {
    if tokens.len() < 3 {
        return Ok(tokens.get(0).unwrap_or(&"0".to_string()).clone());
    }
    
    // Find operators with lowest precedence (right to left)
    // Logical OR has lowest precedence
    if let Some(pos) = find_operator(tokens, "|") {
        let left = evaluate_tokens(&tokens[..pos])?;
        let right = evaluate_tokens(&tokens[pos + 1..])?;
        return Ok(logical_or(&left, &right));
    }
    
    // Logical AND
    if let Some(pos) = find_operator(tokens, "&") {
        let left = evaluate_tokens(&tokens[..pos])?;
        let right = evaluate_tokens(&tokens[pos + 1..])?;
        return Ok(logical_and(&left, &right));
    }
    
    // Comparison operations
    for op in &["=", "!=", "<", "<=", ">", ">="] {
        if let Some(pos) = find_operator(tokens, op) {
            let left = evaluate_tokens(&tokens[..pos])?;
            let right = evaluate_tokens(&tokens[pos + 1..])?;
            return Ok(compare(&left, &right, op)?);
        }
    }
    
    // Arithmetic operations (left to right for same precedence)
    for op in &["+", "-"] {
        if let Some(pos) = find_operator(tokens, op) {
            let left = evaluate_tokens(&tokens[..pos])?;
            let right = evaluate_tokens(&tokens[pos + 1..])?;
            return Ok(arithmetic(&left, &right, op)?);
        }
    }
    
    for op in &["*", "/", "%"] {
        if let Some(pos) = find_operator(tokens, op) {
            let left = evaluate_tokens(&tokens[..pos])?;
            let right = evaluate_tokens(&tokens[pos + 1..])?;
            return Ok(arithmetic(&left, &right, op)?);
        }
    }
    
    // If no operators found, return the first token
    Ok(tokens[0].clone())
}

fn find_operator(tokens: &[String], op: &str) -> Option<usize> {
    tokens.iter().rposition(|token| token == op)
}

fn logical_or(left: &str, right: &str) -> String {
    if is_true(left) {
        left.to_string()
    } else {
        right.to_string()
    }
}

fn logical_and(left: &str, right: &str) -> String {
    if is_true(left) {
        right.to_string()
    } else {
        "0".to_string()
    }
}

fn is_true(value: &str) -> bool {
    !value.is_empty() && value != "0"
}

fn compare(left: &str, right: &str, op: &str) -> Result<String> {
    // Try numeric comparison first
    if let (Ok(left_num), Ok(right_num)) = (left.parse::<i64>(), right.parse::<i64>()) {
        let result = match op {
            "=" => left_num == right_num,
            "!=" => left_num != right_num,
            "<" => left_num < right_num,
            "<=" => left_num <= right_num,
            ">" => left_num > right_num,
            ">=" => left_num >= right_num,
            _ => return Err(anyhow!("expr: unknown operator: {}", op)),
        };
        return Ok(if result { "1".to_string() } else { "0".to_string() });
    }
    
    // Fall back to string comparison
    let result = match op {
        "=" => left == right,
        "!=" => left != right,
        "<" => left < right,
        "<=" => left <= right,
        ">" => left > right,
        ">=" => left >= right,
        _ => return Err(anyhow!("expr: unknown operator: {}", op)),
    };
    
    Ok(if result { "1".to_string() } else { "0".to_string() })
}

fn arithmetic(left: &str, right: &str, op: &str) -> Result<String> {
    let left_num: i64 = left.parse()
        .map_err(|_| anyhow!("expr: non-numeric argument"))?;
    let right_num: i64 = right.parse()
        .map_err(|_| anyhow!("expr: non-numeric argument"))?;
    
    let result = match op {
        "+" => left_num + right_num,
        "-" => left_num - right_num,
        "*" => left_num * right_num,
        "/" => {
            if right_num == 0 {
                return Err(anyhow!("expr: division by zero"));
            }
            left_num / right_num
        }
        "%" => {
            if right_num == 0 {
                return Err(anyhow!("expr: division by zero"));
            }
            left_num % right_num
        }
        _ => return Err(anyhow!("expr: unknown operator: {}", op)),
    };
    
    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic() {
        assert_eq!(evaluate_expression(&["1".to_string(), "+".to_string(), "2".to_string()]).unwrap(), "3");
        assert_eq!(evaluate_expression(&["10".to_string(), "-".to_string(), "3".to_string()]).unwrap(), "7");
        assert_eq!(evaluate_expression(&["4".to_string(), "*".to_string(), "5".to_string()]).unwrap(), "20");
        assert_eq!(evaluate_expression(&["15".to_string(), "/".to_string(), "3".to_string()]).unwrap(), "5");
    }

    #[test]
    fn test_comparison() {
        assert_eq!(evaluate_expression(&["5".to_string(), ">".to_string(), "3".to_string()]).unwrap(), "1");
        assert_eq!(evaluate_expression(&["2".to_string(), "<".to_string(), "1".to_string()]).unwrap(), "0");
        assert_eq!(evaluate_expression(&["hello".to_string(), "=".to_string(), "hello".to_string()]).unwrap(), "1");
    }

    #[test]
    fn test_string_functions() {
        assert_eq!(evaluate_expression(&["length".to_string(), "hello".to_string()]).unwrap(), "5");
        assert_eq!(evaluate_expression(&["substr".to_string(), "hello".to_string(), "2".to_string(), "3".to_string()]).unwrap(), "ell");
        assert_eq!(evaluate_expression(&["index".to_string(), "hello".to_string(), "l".to_string()]).unwrap(), "3");
    }

    #[test]
    fn test_logical() {
        assert_eq!(evaluate_expression(&["1".to_string(), "&".to_string(), "2".to_string()]).unwrap(), "2");
        assert_eq!(evaluate_expression(&["0".to_string(), "&".to_string(), "2".to_string()]).unwrap(), "0");
        assert_eq!(evaluate_expression(&["1".to_string(), "|".to_string(), "2".to_string()]).unwrap(), "1");
        assert_eq!(evaluate_expression(&["0".to_string(), "|".to_string(), "2".to_string()]).unwrap(), "2");
    }
} 
