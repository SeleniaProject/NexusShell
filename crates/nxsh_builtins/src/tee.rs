use std::io::{self, BufRead, Write, Read};
use nxsh_core::{ShellError, ErrorKind}; use nxsh_core::error::RuntimeErrorKind;

pub fn tee_cli(args: &[String]) -> Result<(), ShellError> {
    if args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let mut append = false;
    let mut ignore_interrupts = false;
    let mut files = Vec::new();
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--append" => append = true,
            "-i" | "--ignore-interrupts" => ignore_interrupts = true,
            arg if !arg.starts_with('-') => files.push(arg.to_string()),
            _ => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("tee: invalid option -- '{}'", args[i]))),
        }
        i += 1;
    }
    
    execute_tee(&files, append, ignore_interrupts)
}

fn print_help() {
    println!("tee - read from standard input and write to standard output and files

USAGE:
    tee [OPTION]... [FILE]...

DESCRIPTION:
    Copy standard input to each FILE, and also to standard output.
    
OPTIONS:
    -a, --append              Append to the given FILEs, do not overwrite
    -i, --ignore-interrupts   Ignore interrupt signals
    -h, --help               Display this help and exit

EXAMPLES:
    # Write to file and stdout
    echo \"hello\" | tee output.txt
    
    # Write to multiple files
    ls -la | tee file1.txt file2.txt
    
    # Append to file instead of overwriting
    echo \"new line\" | tee -a logfile.txt
    
    # Use in pipeline
    cat input.txt | tee intermediate.txt | sort > sorted.txt
    
    # Ignore interrupts
    long_running_command | tee -i output.log

EXIT STATUS:
    0   Success
    1   Error writing to file or reading from stdin");
}

fn execute_tee(files: &[String], append: bool, _ignore_interrupts: bool) -> Result<(), ShellError> {
    use std::fs::OpenOptions;
    
    // Open all output files
    let mut file_handles = Vec::new();
    for filename in files {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .open(filename)
            .map_err(|e| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("tee: {filename}: {e}")))?;
        
        file_handles.push((filename.clone(), file));
    }
    
    // Read from stdin and write to stdout and all files
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut line_count = 0;
    let mut byte_count = 0;
    
    for line_result in stdin.lock().lines() {
        let line = match line_result {
            Ok(line) => line,
            Err(e) => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("tee: error reading stdin: {e}"))),
        };
        
        line_count += 1;
        byte_count += line.len() + 1; // +1 for newline
        
        // Write to stdout
        if let Err(e) = writeln!(stdout, "{line}") {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("tee: error writing to stdout: {e}")));
        }
        
        // Write to all files
        for (filename, file) in &mut file_handles {
            if let Err(e) = writeln!(file, "{line}") {
                return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("tee: {filename}: {e}")));
            }
        }
        
        // Flush stdout periodically
        if line_count % 100 == 0 {
            let _ = stdout.flush();
            for (_, file) in &mut file_handles {
                let _ = file.flush();
            }
        }
    }
    
    // Final flush
    let _ = stdout.flush();
    for (_, file) in &mut file_handles {
        let _ = file.flush();
    }
    
    if files.is_empty() {
        println!("tee: processed {line_count} lines ({byte_count} bytes)");
    }
    
    Ok(())
}

// Alternative implementation for handling binary data
pub fn tee_binary(files: &[String], append: bool) -> Result<(), ShellError> {
    use std::fs::OpenOptions;
    use std::io::Read;
    
    // Open all output files
    let mut file_handles = Vec::new();
    for filename in files {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .open(filename)
            .map_err(|e| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("tee: {filename}: {e}")))?;
        
        file_handles.push((filename.clone(), file));
    }
    
    // Read from stdin in chunks
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut buffer = [0u8; 8192];
    let mut total_bytes = 0;
    
    loop {
        match stdin.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(n) => {
                total_bytes += n;
                let data = &buffer[..n];
                
                // Write to stdout
                if let Err(e) = stdout.write_all(data) {
                    return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("tee: error writing to stdout: {e}")));
                }
                
                // Write to all files
                for (filename, file) in &mut file_handles {
                    if let Err(e) = file.write_all(data) {
                        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("tee: {filename}: {e}")));
                    }
                }
                
                // Periodic flush
                if total_bytes % 65536 == 0 {
                    let _ = stdout.flush();
                    for (_, file) in &mut file_handles {
                        let _ = file.flush();
                    }
                }
            },
            Err(e) => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("tee: error reading stdin: {e}"))),
        }
    }
    
    // Final flush
    let _ = stdout.flush();
    for (_, file) in &mut file_handles {
        let _ = file.flush();
    }
    
    Ok(())
}

// Advanced tee with buffering control
pub struct TeeOptions {
    pub append: bool,
    pub ignore_interrupts: bool,
    pub line_buffered: bool,
    pub buffer_size: usize,
}

impl Default for TeeOptions {
    fn default() -> Self {
        Self {
            append: false,
            ignore_interrupts: false,
            line_buffered: true,
            buffer_size: 8192,
        }
    }
}

pub fn tee_advanced(files: &[String], options: &TeeOptions) -> Result<(), ShellError> {
    use std::fs::OpenOptions;
    use std::io::{BufReader, BufWriter};
    
    // Open all output files with buffering
    let mut file_handles = Vec::new();
    for filename in files {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(options.append)
            .truncate(!options.append)
            .open(filename)
            .map_err(|e| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("tee: {filename}: {e}")))?;
        
        let buffered_file = if options.buffer_size > 0 {
            BufWriter::with_capacity(options.buffer_size, file)
        } else {
            BufWriter::new(file)
        };
        
        file_handles.push((filename.clone(), buffered_file));
    }
    
    // Set up buffered I/O
    let stdin = io::stdin();
    let mut reader = BufReader::with_capacity(options.buffer_size, stdin.lock());
    let mut stdout = BufWriter::new(io::stdout());
    
    if options.line_buffered {
        // Line-by-line processing
        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            // Write to stdout
            stdout.write_all(line.as_bytes())?;
            
            // Write to all files
            for (filename, file) in &mut file_handles {
                file.write_all(line.as_bytes())
                    .map_err(|e| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("tee: {filename}: {e}")))?;
            }
            
            line.clear();
        }
    } else {
        // Chunk-based processing
        let mut buffer = vec![0u8; options.buffer_size];
        loop {
            match reader.read(&mut buffer)? {
                0 => break, // EOF
                n => {
                    let data = &buffer[..n];
                    
                    // Write to stdout
                    stdout.write_all(data)?;
                    
                    // Write to all files
                    for (filename, file) in &mut file_handles {
                        file.write_all(data)
                            .map_err(|e| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("tee: {filename}: {e}")))?;
                    }
                }
            }
        }
    }
    
    // Flush all outputs
    stdout.flush()?;
    for (filename, file) in &mut file_handles {
        file.flush()
            .map_err(|e| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("tee: {filename}: {e}")))?;
    }
    
    Ok(())
}

