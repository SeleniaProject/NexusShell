//! `seq` builtin - output a sequence of numbers.
//!
//! Usage:
//!   seq LAST                    # from 1 to LAST
//!   seq FIRST LAST              # from FIRST to LAST
//!   seq FIRST INCREMENT LAST    # from FIRST to LAST by INCREMENT
//!
//! Options:
//!   -s STRING    Use STRING as separator (default: newline)
//!   -w           Pad numbers with leading zeros to equal width
//!   -f FORMAT    Use printf-style floating-point FORMAT (default: %g)

use anyhow::{anyhow, Result};
use std::fmt::Write as FmtWrite;

/// Entry point for the seq builtin.
pub fn seq_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("seq: missing operand"));
    }

    let mut separator = "\n".to_string();
    let mut equal_width = false;
    let mut format = "%g".to_string();
    let mut args_iter = args.iter();
    let mut numbers = Vec::new();

    // Parse options and collect numeric arguments
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "-s" => {
                separator = args_iter
                    .next()
                    .ok_or_else(|| anyhow!("seq: option requires an argument -- s"))?
                    .clone();
            }
            "-w" => {
                equal_width = true;
            }
            "-f" => {
                format = args_iter
                    .next()
                    .ok_or_else(|| anyhow!("seq: option requires an argument -- f"))?
                    .clone();
            }
            _ if arg.starts_with('-') => {
                return Err(anyhow!("seq: invalid option -- '{}'", arg.trim_start_matches('-')));
            }
            _ => {
                numbers.push(arg.parse::<f64>()
                    .map_err(|_| anyhow!("seq: invalid floating point argument: '{}'", arg))?);
            }
        }
    }

    if numbers.is_empty() {
        return Err(anyhow!("seq: missing operand"));
    }

    let (first, increment, last) = match numbers.len() {
        1 => (1.0, 1.0, numbers[0]),
        2 => (numbers[0], 1.0, numbers[1]),
        3 => (numbers[0], numbers[1], numbers[2]),
        _ => return Err(anyhow!("seq: too many operands")),
    };

    if increment == 0.0 {
        return Err(anyhow!("seq: increment cannot be zero"));
    }

    // Generate sequence
    let mut current = first;
    let mut output = String::new();
    let mut count = 0;

    // Calculate maximum width for padding if -w is specified
    let max_width = if equal_width {
        let max_val = if increment > 0.0 { last } else { first };
        let min_val = if increment > 0.0 { first } else { last };
        format_number(max_val.max(min_val.abs()), &format).len()
    } else {
        0
    };

    while (increment > 0.0 && current <= last) || (increment < 0.0 && current >= last) {
        if count > 0 {
            output.push_str(&separator);
        }

        let formatted = format_number(current, &format);
        if equal_width && max_width > formatted.len() {
            let padding = max_width - formatted.len();
            for _ in 0..padding {
                output.push('0');
            }
        }
        output.push_str(&formatted);

        current += increment;
        count += 1;

        // Prevent infinite loops with floating point precision issues
        if count > 1_000_000 {
            return Err(anyhow!("seq: sequence too long"));
        }
    }

    if !output.is_empty() {
        println!("{}", output);
    }

    Ok(())
}

fn format_number(num: f64, format: &str) -> String {
    // Simple format support - can be extended for more printf-style formats
    match format {
        "%g" | "%G" => {
            if num.fract() == 0.0 && num.abs() < 1e15 {
                format!("{:.0}", num)
            } else {
                format!("{}", num)
            }
        }
        "%f" => format!("{:.6}", num),
        "%e" => format!("{:e}", num),
        "%E" => format!("{:E}", num),
        _ => {
            // Try to parse custom format like "%.2f"
            if format.starts_with("%.") && format.ends_with('f') {
                if let Ok(precision) = format[2..format.len()-1].parse::<usize>() {
                    return format!("{:.1$}", num, precision);
                }
            }
            format!("{}", num)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seq_single_arg() {
        let result = seq_cli(&["3".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_seq_two_args() {
        let result = seq_cli(&["2".to_string(), "5".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_seq_three_args() {
        let result = seq_cli(&["1".to_string(), "2".to_string(), "10".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1.0, "%g"), "1");
        assert_eq!(format_number(1.5, "%g"), "1.5");
        assert_eq!(format_number(1.0, "%.2f"), "1.00");
    }
} 
