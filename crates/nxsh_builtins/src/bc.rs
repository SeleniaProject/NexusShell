//! `bc` builtin - arbitrary precision calculator.
//!
//! Usage:
//!   bc [OPTIONS] [FILE...]
//!
//! Options:
//!   -i          Force interactive mode
//!   -l          Load standard math library
//!   -q          Quiet mode (don't print banner)
//!   -s          Process exactly one line from standard input
//!
//! This implementation provides basic arithmetic operations with arbitrary precision
//! using the `num-bigint` and `num-rational` crates for high precision calculations.

use anyhow::{anyhow, Result};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Zero, One, ToPrimitive};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader};
use std::str::FromStr;

/// BC calculator context with variables and settings
pub struct BcContext {
    variables: HashMap<String, BigRational>,
    scale: usize,
    ibase: u32,
    obase: u32,
    quiet: bool,
}

impl Default for BcContext {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            scale: 0,
            ibase: 10,
            obase: 10,
            quiet: false,
        }
    }
}

impl BcContext {
    fn new() -> Self {
        Self::default()
    }

    fn with_math_lib(&mut self) {
        // Add common mathematical constants and functions
        self.variables.insert("pi".to_string(), 
            BigRational::from_str("3.14159265358979323846").unwrap_or_else(|_| BigRational::zero()));
        self.variables.insert("e".to_string(), 
            BigRational::from_str("2.71828182845904523536").unwrap_or_else(|_| BigRational::zero()));
    }

    fn evaluate_expression(&mut self, expr: &str) -> Result<BigRational> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Ok(BigRational::zero());
        }

        // Handle variable assignments
        if let Some(eq_pos) = expr.find('=') {
            let var_name = expr[..eq_pos].trim();
            let value_expr = expr[eq_pos + 1..].trim();
            let value = self.parse_number_or_expression(value_expr)?;
            self.variables.insert(var_name.to_string(), value.clone());
            return Ok(value);
        }

        // Handle special commands
        match expr {
            "quit" | "q" => std::process::exit(0),
            _ if expr.starts_with("scale=") => {
                let scale_str = &expr[6..];
                self.scale = scale_str.parse().unwrap_or(0);
                return Ok(BigRational::from_integer(BigInt::from(self.scale)));
            }
            _ if expr.starts_with("ibase=") => {
                let ibase_str = &expr[6..];
                self.ibase = ibase_str.parse().unwrap_or(10).clamp(2, 16);
                return Ok(BigRational::from_integer(BigInt::from(self.ibase)));
            }
            _ if expr.starts_with("obase=") => {
                let obase_str = &expr[6..];
                self.obase = obase_str.parse().unwrap_or(10).clamp(2, 16);
                return Ok(BigRational::from_integer(BigInt::from(self.obase)));
            }
            _ => {}
        }

        self.parse_number_or_expression(expr)
    }

    fn parse_number_or_expression(&mut self, expr: &str) -> Result<BigRational> {
        // Simple expression parser for basic arithmetic
        let expr = expr.replace(" ", "");
        
        // Check if it's a variable
        if let Some(value) = self.variables.get(&expr) {
            return Ok(value.clone());
        }

        // Try to parse as a number
        if let Ok(num) = self.parse_number(&expr) {
            return Ok(num);
        }

        // Handle basic arithmetic operations
        if let Some(result) = self.parse_arithmetic(&expr)? {
            return Ok(result);
        }

        Err(anyhow!("bc: invalid expression: {}", expr))
    }

    fn parse_number(&self, s: &str) -> Result<BigRational> {
        // Handle decimal numbers
        if s.contains('.') {
            let parts: Vec<&str> = s.split('.').collect();
            if parts.len() == 2 {
                let integer_part = BigInt::from_str(parts[0]).unwrap_or_else(|_| BigInt::zero());
                let decimal_part = parts[1];
                let decimal_value = BigInt::from_str(decimal_part).unwrap_or_else(|_| BigInt::zero());
                let decimal_places = decimal_part.len();
                let denominator = BigInt::from(10).pow(decimal_places as u32);
                
                let rational = BigRational::new(
                    integer_part * &denominator + decimal_value,
                    denominator
                );
                return Ok(rational);
            }
        }

        // Handle integers
        BigInt::from_str(s)
            .map(BigRational::from_integer)
            .map_err(|_| anyhow!("bc: invalid number: {}", s))
    }

    fn parse_arithmetic(&mut self, expr: &str) -> Result<Option<BigRational>> {
        // Simple arithmetic parser - can be extended for more complex expressions
        for op in &['+', '-', '*', '/', '%', '^'] {
            if let Some(pos) = expr.rfind(*op) {
                let left = &expr[..pos];
                let right = &expr[pos + 1..];
                
                let left_val = self.parse_number_or_expression(left)?;
                let right_val = self.parse_number_or_expression(right)?;
                
                let result = match op {
                    '+' => left_val + right_val,
                    '-' => left_val - right_val,
                    '*' => left_val * right_val,
                    '/' => {
                        if right_val.is_zero() {
                            return Err(anyhow!("bc: division by zero"));
                        }
                        left_val / right_val
                    }
                    '%' => {
                        if right_val.is_zero() {
                            return Err(anyhow!("bc: division by zero"));
                        }
                        left_val % right_val
                    }
                    '^' => {
                        // Simple integer power
                        if let Some(exp) = right_val.to_integer().to_u32() {
                            self.power(&left_val, exp)
                        } else {
                            return Err(anyhow!("bc: non-integer exponents not supported"));
                        }
                    }
                    _ => unreachable!(),
                };
                
                return Ok(Some(result));
            }
        }
        
        Ok(None)
    }

    fn power(&self, base: &BigRational, exp: u32) -> BigRational {
        let mut result = BigRational::one();
        let mut base = base.clone();
        let mut exp = exp;
        
        while exp > 0 {
            if exp % 2 == 1 {
                result *= &base;
            }
            let base_clone = base.clone();
            base *= &base_clone;
            exp /= 2;
        }
        
        result
    }

    fn format_output(&self, value: &BigRational) -> String {
        if value.is_integer() {
            value.to_integer().to_string()
        } else {
            // Format with specified scale
            let scaled = if self.scale > 0 {
                let scale_factor = BigInt::from(10).pow(self.scale as u32);
                let scaled_num = value * BigRational::from_integer(scale_factor.clone());
                let rounded = scaled_num.to_integer();
                BigRational::new(rounded, scale_factor)
            } else {
                value.clone()
            };
            
            format!("{}", scaled.to_f64().unwrap_or(0.0))
        }
    }
}

/// Entry point for the bc builtin.
pub fn bc_cli(args: &[String]) -> Result<()> {
    let mut interactive = false;
    let mut load_math = false;
    let mut quiet = false;
    let mut single_line = false;
    let mut files = Vec::new();

    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-i" => interactive = true,
            "-l" => load_math = true,
            "-q" => quiet = true,
            "-s" => single_line = true,
            arg if arg.starts_with('-') => {
                return Err(anyhow!("bc: invalid option: {}", arg));
            }
            _ => files.push(&args[i]),
        }
        i += 1;
    }

    let mut ctx = BcContext::new();
    ctx.quiet = quiet;

    if load_math {
        ctx.with_math_lib();
    }

    if !quiet && (interactive || files.is_empty()) {
        println!("bc - arbitrary precision calculator");
        println!("Type 'quit' or 'q' to exit");
    }

    // Process files if provided
    let files_empty = files.is_empty();
    for file_path in files {
        let file = std::fs::File::open(file_path)
            .map_err(|e| anyhow!("bc: cannot open {}: {}", file_path, e))?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            process_line(&mut ctx, &line)?;
        }
    }

    // Interactive mode or stdin processing
    if files_empty || interactive {
        let stdin = io::stdin();
        
        if single_line {
            let mut line = String::new();
            stdin.read_line(&mut line)?;
            process_line(&mut ctx, line.trim())?;
        } else {
            for line in stdin.lock().lines() {
                let line = line?;
                if line.trim() == "quit" || line.trim() == "q" {
                    break;
                }
                process_line(&mut ctx, &line)?;
            }
        }
    }

    Ok(())
}

fn process_line(ctx: &mut BcContext, line: &str) -> Result<()> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(());
    }

    match ctx.evaluate_expression(line) {
        Ok(result) => {
            println!("{}", ctx.format_output(&result));
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let mut ctx = BcContext::new();
        
        assert_eq!(ctx.evaluate_expression("2+3").unwrap(), BigRational::from_integer(BigInt::from(5)));
        assert_eq!(ctx.evaluate_expression("10-4").unwrap(), BigRational::from_integer(BigInt::from(6)));
        assert_eq!(ctx.evaluate_expression("3*4").unwrap(), BigRational::from_integer(BigInt::from(12)));
    }

    #[test]
    fn test_variable_assignment() {
        let mut ctx = BcContext::new();
        
        ctx.evaluate_expression("x=5").unwrap();
        assert_eq!(ctx.evaluate_expression("x").unwrap(), BigRational::from_integer(BigInt::from(5)));
    }

    #[test]
    fn test_decimal_numbers() {
        let ctx = BcContext::new();
        
        let result = ctx.parse_number("3.14").unwrap();
    use std::f64::consts::PI;
    assert!((result.to_f64().unwrap() - PI).abs() < 0.01); // Allow small tolerance
    }
}

/// Execute function for bc command
pub fn execute(args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    match bc_cli(args) {
        Ok(_) => Ok(0),
        Err(e) => {
            eprintln!("{}", e);
            Ok(1)
        }
    }
}

