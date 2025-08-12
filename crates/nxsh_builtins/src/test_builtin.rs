use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use nxsh_core::{ErrorKind, ShellError};
use nxsh_core::error::RuntimeErrorKind;

pub fn test_builtin_cli(args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        // Empty test always returns false
        std::process::exit(1);
    }

    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let result = evaluate_test_expression(&args)?;
    
    // Exit with appropriate code
    if result {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

fn print_help() {
    println!("Usage: test EXPRESSION");
    println!("   or: [ EXPRESSION ]");
    println!();
    println!("Evaluate conditional expression and exit with status 0 (true) or 1 (false).");
    println!();
    println!("File operators:");
    println!("  -e FILE        True if FILE exists");
    println!("  -f FILE        True if FILE is a regular file");
    println!("  -d FILE        True if FILE is a directory");
    println!("  -r FILE        True if FILE is readable");
    println!("  -w FILE        True if FILE is writable");
    println!("  -x FILE        True if FILE is executable");
    println!("  -s FILE        True if FILE exists and is not empty");
    println!("  -L FILE        True if FILE is a symbolic link");
    println!();
    println!("String operators:");
    println!("  -z STRING      True if STRING is empty");
    println!("  -n STRING      True if STRING is not empty");
    println!("  STRING         True if STRING is not empty");
    println!("  STR1 = STR2    True if strings are equal");
    println!("  STR1 != STR2   True if strings are not equal");
    println!("  STR1 < STR2    True if STR1 sorts before STR2");
    println!("  STR1 > STR2    True if STR1 sorts after STR2");
    println!();
    println!("Numeric operators:");
    println!("  INT1 -eq INT2  True if integers are equal");
    println!("  INT1 -ne INT2  True if integers are not equal");
    println!("  INT1 -lt INT2  True if INT1 is less than INT2");
    println!("  INT1 -le INT2  True if INT1 is less than or equal to INT2");
    println!("  INT1 -gt INT2  True if INT1 is greater than INT2");
    println!("  INT1 -ge INT2  True if INT1 is greater than or equal to INT2");
    println!();
    println!("Logical operators:");
    println!("  ! EXPR         True if EXPR is false");
    println!("  EXPR1 -a EXPR2 True if both expressions are true");
    println!("  EXPR1 -o EXPR2 True if either expression is true");
    println!("  ( EXPR )       Force precedence");
    println!();
    println!("Examples:");
    println!("  test -f /etc/passwd      # Check if file exists");
    println!("  test -z \"$VAR\"           # Check if variable is empty");
    println!("  test 5 -gt 3             # Numeric comparison");
    println!("  test \"$a\" = \"$b\"        # String comparison");
    println!("  [ -d /home ] && echo \"Home exists\"");
}

fn evaluate_test_expression(args: &[String]) -> Result<bool> {
    if args.is_empty() {
        return Ok(false);
    }

    // Handle single argument cases
    if args.len() == 1 {
        return Ok(evaluate_single_argument(&args[0]));
    }

    // Handle unary operators
    if args.len() == 2 {
        return evaluate_unary_expression(&args[0], &args[1]);
    }

    // Handle binary operators
    if args.len() == 3 {
        return evaluate_binary_expression(&args[0], &args[1], &args[2]);
    }

    // Handle complex expressions with logical operators
    evaluate_complex_expression(args)
}

fn evaluate_single_argument(arg: &str) -> bool {
    // Non-empty string is true
    !arg.is_empty()
}

fn evaluate_unary_expression(operator: &str, operand: &str) -> Result<bool> {
    match operator {
        "-e" => Ok(path_exists(operand)),
        "-f" => Ok(is_regular_file(operand)),
        "-d" => Ok(is_directory(operand)),
        "-r" => Ok(is_readable(operand)),
        "-w" => Ok(is_writable(operand)),
        "-x" => Ok(is_executable(operand)),
        "-s" => Ok(is_non_empty_file(operand)),
        "-L" => Ok(is_symlink(operand)),
        "-z" => Ok(operand.is_empty()),
        "-n" => Ok(!operand.is_empty()),
        "!" => Ok(!evaluate_single_argument(operand)),
        _ => Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            format!("Unknown unary operator: {operator}")
        ).into()),
    }
}

fn evaluate_binary_expression(left: &str, operator: &str, right: &str) -> Result<bool> {
    match operator {
        "=" | "==" => Ok(left == right),
        "!=" => Ok(left != right),
        "<" => Ok(left < right),
        ">" => Ok(left > right),
        "-eq" => {
            let left_num: i64 = left.parse()
                .map_err(|_| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid number"))?;
            let right_num: i64 = right.parse()
                .map_err(|_| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid number"))?;
            Ok(left_num == right_num)
        }
        "-ne" => {
            let left_num: i64 = left.parse()
                .map_err(|_| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid number"))?;
            let right_num: i64 = right.parse()
                .map_err(|_| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid number"))?;
            Ok(left_num != right_num)
        }
        "-lt" => {
            let left_num: i64 = left.parse()
                .map_err(|_| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid number"))?;
            let right_num: i64 = right.parse()
                .map_err(|_| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid number"))?;
            Ok(left_num < right_num)
        }
        "-le" => {
            let left_num: i64 = left.parse()
                .map_err(|_| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid number"))?;
            let right_num: i64 = right.parse()
                .map_err(|_| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid number"))?;
            Ok(left_num <= right_num)
        }
        "-gt" => {
            let left_num: i64 = left.parse()
                .map_err(|_| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid number"))?;
            let right_num: i64 = right.parse()
                .map_err(|_| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid number"))?;
            Ok(left_num > right_num)
        }
        "-ge" => {
            let left_num: i64 = left.parse()
                .map_err(|_| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid number"))?;
            let right_num: i64 = right.parse()
                .map_err(|_| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid number"))?;
            Ok(left_num >= right_num)
        }
        _ => Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            format!("Unknown binary operator: {operator}")
        ).into()),
    }
}

fn evaluate_complex_expression(args: &[String]) -> Result<bool> {
    // Handle parentheses
    if args.len() >= 3 && args[0] == "(" && args[args.len() - 1] == ")" {
        let inner_args = &args[1..args.len() - 1];
        return evaluate_test_expression(inner_args);
    }

    // Handle negation
    if !args.is_empty() && args[0] == "!" {
        let rest = &args[1..];
        let result = evaluate_test_expression(rest)?;
        return Ok(!result);
    }

    // Find logical operators (-a, -o) with lowest precedence
    // -o has lower precedence than -a
    for (i, arg) in args.iter().enumerate() {
        if arg == "-o" && i > 0 && i < args.len() - 1 {
            let left = &args[..i];
            let right = &args[i + 1..];
            let left_result = evaluate_test_expression(left)?;
            let right_result = evaluate_test_expression(right)?;
            return Ok(left_result || right_result);
        }
    }

    for (i, arg) in args.iter().enumerate() {
        if arg == "-a" && i > 0 && i < args.len() - 1 {
            let left = &args[..i];
            let right = &args[i + 1..];
            let left_result = evaluate_test_expression(left)?;
            let right_result = evaluate_test_expression(right)?;
            return Ok(left_result && right_result);
        }
    }

    // If no logical operators found, try as a simple expression
    if args.len() >= 3 {
        evaluate_binary_expression(&args[0], &args[1], &args[2])
    } else if args.len() == 2 {
        evaluate_unary_expression(&args[0], &args[1])
    } else if args.len() == 1 {
        Ok(evaluate_single_argument(&args[0]))
    } else {
        Ok(false)
    }
}

// File test implementations

fn path_exists(path: &str) -> bool {
    PathBuf::from(path).exists()
}

fn is_regular_file(path: &str) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        metadata.is_file()
    } else {
        false
    }
}

fn is_directory(path: &str) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        metadata.is_dir()
    } else {
        false
    }
}

fn is_readable(path: &str) -> bool {
    if !path_exists(path) {
        return false;
    }

    // Try to open file for reading
    fs::File::open(path).is_ok()
}

fn is_writable(path: &str) -> bool {
    let path_buf = PathBuf::from(path);
    
    if path_buf.exists() {
        // Check if we can open existing file for writing
        fs::OpenOptions::new().write(true).open(&path_buf).is_ok()
    } else {
        // Check if we can create file in parent directory
        if let Some(parent) = path_buf.parent() {
            parent.exists() && is_writable(parent.to_str().unwrap_or(""))
        } else {
            false
        }
    }
}

fn is_executable(path: &str) -> bool {
    if !path_exists(path) {
        return false;
    }

    if cfg!(windows) {
        // On Windows, check file extension
        let path_lower = path.to_lowercase();
        path_lower.ends_with(".exe") ||
        path_lower.ends_with(".bat") ||
        path_lower.ends_with(".cmd") ||
        path_lower.ends_with(".ps1")
    } else {
        // On Unix-like systems, check execute permission
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(path) {
                let permissions = metadata.permissions();
                permissions.mode() & 0o111 != 0
            } else {
                false
            }
        }
        #[cfg(not(unix))] {
            // On non-Unix systems, check file extension
            let path_lower = path.to_lowercase();
            path_lower.ends_with(".exe") || path_lower.ends_with(".com") ||
            path_lower.ends_with(".bat") || path_lower.ends_with(".cmd")
        }
    }
}

fn is_non_empty_file(path: &str) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        metadata.is_file() && metadata.len() > 0
    } else {
        false
    }
}

fn is_symlink(path: &str) -> bool {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        metadata.file_type().is_symlink()
    } else {
        false
    }
}

// Utility functions for shell integration

pub fn test_file_exists(path: &str) -> bool {
    path_exists(path)
}

pub fn test_is_directory(path: &str) -> bool {
    is_directory(path)
}

pub fn test_is_file(path: &str) -> bool {
    is_regular_file(path)
}

pub fn test_string_empty(s: &str) -> bool {
    s.is_empty()
}

pub fn test_strings_equal(a: &str, b: &str) -> bool {
    a == b
}

pub fn test_numbers_equal(a: i64, b: i64) -> bool {
    a == b
}

pub fn test_number_greater(a: i64, b: i64) -> bool {
    a > b
}

pub fn test_number_less(a: i64, b: i64) -> bool {
    a < b
}

// Shell condition evaluation
pub fn evaluate_condition(condition: &str) -> Result<bool> {
    let args: Vec<String> = condition.split_whitespace().map(|s| s.to_string()).collect();
    evaluate_test_expression(&args)
}

