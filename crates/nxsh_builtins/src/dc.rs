use nxsh_core::{ShellError, ErrorKind}; use nxsh_core::error::RuntimeErrorKind;
use nxsh_core::mir;

pub fn dc_cli(args: &[String]) -> Result<(), ShellError> {
    if args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let mut calculator = DCCalculator::new();
    
    if args.is_empty() {
        // Interactive mode
        use std::io::{self, BufRead};
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    if let Err(e) = calculator.process_line(&line) {
                        eprintln!("dc: {e}");
                    }
                },
                Err(e) => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("dc: {e}"))),
            }
        }
    } else {
        // Process command line arguments
        let input = args.join(" ");
        calculator.process_line(&input)?;
    }
    
    Ok(())
}

fn print_help() {
    println!("dc - desk calculator (reverse Polish notation)

USAGE:
    dc [expression...]

DESCRIPTION:
    A reverse-Polish desk calculator which supports unlimited precision
    arithmetic. If no expressions are given, dc reads from stdin.

OPERATORS:
    +           Add top two values
    -           Subtract top two values  
    *           Multiply top two values
    /           Divide top two values
    %           Modulo of top two values
    ^           Exponentiation
    p           Print top value
    n           Print top value without newline
    f           Print entire stack
    c           Clear stack
    d           Duplicate top value
    r           Reverse (swap) top two values
    q           Quit
    
EXAMPLES:
    # Calculate 2 + 3
    echo \"2 3 + p\" | dc
    
    # Calculate 10 * (5 + 3)
    echo \"10 5 3 + * p\" | dc");
}

struct DCCalculator {
    stack: Vec<f64>,
}

impl DCCalculator {
    fn new() -> Self {
        Self { stack: Vec::new() }
    }
    
    fn process_line(&mut self, line: &str) -> Result<(), ShellError> {
        for token in line.split_whitespace() {
            self.process_token(token)?;
        }
        Ok(())
    }
    
    fn process_token(&mut self, token: &str) -> Result<(), ShellError> {
        match token {
            "+" => self.binary_op(|a, b| Ok(a + b))?,
            "-" => self.binary_op(|a, b| Ok(b - a))?,
            "*" => self.binary_op(|a, b| Ok(a * b))?,
            "/" => self.binary_op(|a, b| if a == 0.0 { Err(mir::MirError::DivByZero)} else { Ok(b / a) })?,
            "%" => self.binary_op(|a, b| if a == 0.0 { Err(mir::MirError::DivByZero)} else { Ok(b % a) })?,
            "^" => self.binary_op(|a, b| Ok(a.powf(b)))?,
            "p" => self.print_top()?,
            "n" => self.print_top_no_newline()?,
            "f" => self.print_stack(),
            "c" => self.stack.clear(),
            "d" => self.duplicate()?,
            "r" => self.reverse()?,
            "q" => std::process::exit(0),
            _ => {
                // Try to parse as number
                match token.parse::<f64>() {
                    Ok(num) => self.stack.push(num),
                    Err(_) => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Unknown token: {token}"))),
                }
            }
        }
        Ok(())
    }
    
    fn binary_op<F>(&mut self, op: F) -> Result<(), ShellError> 
    where 
        F: Fn(f64, f64) -> Result<f64, mir::MirError>
    {
        if self.stack.len() < 2 {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Stack underflow"));
        }
        
        let a = self.stack.pop().unwrap();
        let b = self.stack.pop().unwrap();
        
        match op(a, b) {
            Ok(result) => {
                self.stack.push(result);
                Ok(())
            },
            Err(err) => Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                err.to_string(),
            )),
        }
    }
    
    fn print_top(&self) -> Result<(), ShellError> {
        if self.stack.is_empty() {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Stack is empty"));
        }
        println!("{}", self.stack.last().unwrap());
        Ok(())
    }
    
    fn print_top_no_newline(&self) -> Result<(), ShellError> {
        if self.stack.is_empty() {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Stack is empty"));
        }
        print!("{}", self.stack.last().unwrap());
        Ok(())
    }
    
    fn print_stack(&self) {
        for (i, value) in self.stack.iter().enumerate() {
            println!("{i}: {value}");
        }
    }
    
    fn duplicate(&mut self) -> Result<(), ShellError> {
        if self.stack.is_empty() {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Stack is empty"));
        }
        let top = *self.stack.last().unwrap();
        self.stack.push(top);
        Ok(())
    }
    
    fn reverse(&mut self) -> Result<(), ShellError> {
        if self.stack.len() < 2 {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Need at least 2 values on stack"));
        }
        let len = self.stack.len();
        self.stack.swap(len - 1, len - 2);
        Ok(())
    }
}


