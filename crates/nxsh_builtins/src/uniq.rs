//! `uniq` command - report or omit repeated lines
//!
//! Full uniq implementation with various filtering and counting options

use nxsh_core::{Builtin, ExecutionResult, ShellContext, ShellError, ShellResult};
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader, BufWriter};

// Beautiful CUI design
use crate::ui_design::ColorPalette;

// Helper function to create runtime errors more concisely
fn runtime_error(msg: &str) -> ShellError {
    ShellError::new(
        nxsh_core::error::ErrorKind::RuntimeError(
            nxsh_core::error::RuntimeErrorKind::InvalidArgument,
        ),
        msg,
    )
}

// Helper function to create IO errors more concisely
fn io_error(msg: &str) -> ShellError {
    ShellError::new(
        nxsh_core::error::ErrorKind::IoError(nxsh_core::error::IoErrorKind::Other),
        msg,
    )
}

pub struct UniqBuiltin;

#[derive(Debug, Clone)]
pub struct UniqOptions {
    pub count: bool,
    pub repeated: bool,
    pub unique: bool,
    pub all_repeated: bool,
    pub ignore_case: bool,
    pub skip_fields: usize,
    pub skip_chars: usize,
    pub check_chars: Option<usize>,
    pub zero_terminated: bool,
    pub group: bool,
    pub input_file: Option<String>,
    pub output_file: Option<String>,
    pub no_color: bool,
}

impl Builtin for UniqBuiltin {
    fn execute(&self, _ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        let options = parse_uniq_args(args)?;
        process_uniq(&options)?;
        Ok(ExecutionResult::success(0))
    }

    fn name(&self) -> &'static str {
        "uniq"
    }

    fn synopsis(&self) -> &'static str {
        "report or omit repeated lines"
    }

    fn description(&self) -> &'static str {
        "Filter adjacent duplicate lines from input"
    }

    fn help(&self) -> &'static str {
        self.usage()
    }

    fn usage(&self) -> &'static str {
        "uniq - report or omit repeated lines

USAGE:
    uniq [OPTIONS] [INPUT [OUTPUT]]

OPTIONS:
    -c, --count               Prefix lines by the number of occurrences
    -d, --repeated            Only print duplicate lines, one for each group
    -D, --all-repeated        Print all duplicate lines
    -f, --skip-fields=N       Avoid comparing the first N fields
    -i, --ignore-case         Ignore differences in case when comparing
    -s, --skip-chars=N        Avoid comparing the first N characters
    -u, --unique              Only print unique lines
    -w, --check-chars=N       Compare no more than N characters in lines
    -z, --zero-terminated     Line delimiter is NUL, not newline
    --group                   Show all items, separating groups with an empty line
    --help                    Display this help and exit

EXAMPLES:
    uniq file.txt             Remove consecutive duplicate lines
    uniq -c file.txt          Count occurrences of each line
    uniq -d file.txt          Show only duplicate lines
    uniq -u file.txt          Show only unique lines
    sort file.txt | uniq      Remove all duplicate lines (sorted first)
    uniq -f 1 file.txt        Ignore first field when comparing
    uniq -s 5 file.txt        Ignore first 5 characters when comparing"
    }
}

fn parse_uniq_args(args: &[String]) -> ShellResult<UniqOptions> {
    let mut options = UniqOptions {
        count: false,
        repeated: false,
        unique: false,
        all_repeated: false,
        ignore_case: false,
        skip_fields: 0,
        skip_chars: 0,
        check_chars: None,
        zero_terminated: false,
        group: false,
        input_file: None,
        output_file: None,
        no_color: false,
    };

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        match arg.as_str() {
            "-c" | "--count" => options.count = true,
            "-d" | "--repeated" => options.repeated = true,
            "-D" | "--all-repeated" => options.all_repeated = true,
            "-u" | "--unique" => options.unique = true,
            "-i" | "--ignore-case" => options.ignore_case = true,
            "-z" | "--zero-terminated" => options.zero_terminated = true,
            "--group" => options.group = true,
            "-f" | "--skip-fields" => {
                i += 1;
                if i >= args.len() {
                    return Err(runtime_error("Option -f requires an argument"));
                }
                options.skip_fields = args[i]
                    .parse()
                    .map_err(|_| runtime_error("Invalid field count"))?;
            }
            "-s" | "--skip-chars" => {
                i += 1;
                if i >= args.len() {
                    return Err(runtime_error("Option -s requires an argument"));
                }
                options.skip_chars = args[i]
                    .parse()
                    .map_err(|_| runtime_error("Invalid character count"))?;
            }
            "-w" | "--check-chars" => {
                i += 1;
                if i >= args.len() {
                    return Err(runtime_error("Option -w requires an argument"));
                }
                options.check_chars = Some(
                    args[i]
                        .parse()
                        .map_err(|_| runtime_error("Invalid character count"))?,
                );
            }
            "--help" => return Err(runtime_error("Help requested")),
            _ if arg.starts_with("-f") => {
                options.skip_fields = arg[2..]
                    .parse()
                    .map_err(|_| runtime_error("Invalid field count"))?;
            }
            _ if arg.starts_with("-s") => {
                options.skip_chars = arg[2..]
                    .parse()
                    .map_err(|_| runtime_error("Invalid character count"))?;
            }
            _ if arg.starts_with("-w") => {
                options.check_chars = Some(
                    arg[2..]
                        .parse()
                        .map_err(|_| runtime_error("Invalid character count"))?,
                );
            }
            _ if arg.starts_with("-") => {
                // Handle combined short options
                for ch in arg[1..].chars() {
                    match ch {
                        'c' => options.count = true,
                        'd' => options.repeated = true,
                        'D' => options.all_repeated = true,
                        'u' => options.unique = true,
                        'i' => options.ignore_case = true,
                        'z' => options.zero_terminated = true,
                        _ => return Err(runtime_error(&format!("Unknown option: -{ch}"))),
                    }
                }
            }
            _ => {
                // Non-option arguments are input and output files
                if options.input_file.is_none() {
                    options.input_file = Some(arg.clone());
                } else if options.output_file.is_none() {
                    options.output_file = Some(arg.clone());
                } else {
                    return Err(runtime_error("Too many arguments"));
                }
            }
        }
        i += 1;
    }

    Ok(options)
}

fn process_uniq(options: &UniqOptions) -> ShellResult<()> {
    let separator = if options.zero_terminated {
        b'\0'
    } else {
        b'\n'
    };

    // Open input
    let input: Box<dyn BufRead> = if let Some(ref input_file) = options.input_file {
        let file = File::open(input_file)
            .map_err(|e| io_error(&format!("Cannot open {input_file}: {e}")))?;
        Box::new(BufReader::new(file))
    } else {
        Box::new(std::io::stdin().lock())
    };

    // Open output
    let output: Box<dyn Write> = if let Some(ref output_file) = options.output_file {
        let file = File::create(output_file)
            .map_err(|e| io_error(&format!("Cannot create {output_file}: {e}")))?;
        Box::new(BufWriter::new(file))
    } else {
        Box::new(std::io::stdout())
    };

    process_uniq_stream(input, output, options, separator)?;
    Ok(())
}

fn process_uniq_stream<R: BufRead, W: Write>(
    mut reader: R,
    mut writer: W,
    options: &UniqOptions,
    separator: u8,
) -> ShellResult<()> {
    let mut current_line = Vec::new();
    let mut previous_key = String::new();
    let mut current_count = 0;
    let mut group_lines = Vec::new();
    let mut first_line = true;

    loop {
        current_line.clear();
        let bytes_read = reader.read_until(separator, &mut current_line)?;

        if bytes_read == 0 {
            // End of input - process the last group
            if current_count > 0 {
                process_group(&group_lines, current_count, &mut writer, options, separator)?;
            }
            break;
        }

        // Remove separator
        if current_line.last() == Some(&separator) {
            current_line.pop();
        }

        let line_str = String::from_utf8_lossy(&current_line).to_string();
        let key = extract_comparison_key(&line_str, options);

        if first_line || key == previous_key {
            // Same group
            current_count += 1;
            group_lines.push(line_str.clone());
            first_line = false;
        } else {
            // New group - process the previous group
            process_group(&group_lines, current_count, &mut writer, options, separator)?;

            // Start new group
            group_lines.clear();
            group_lines.push(line_str.clone());
            current_count = 1;
        }

        previous_key = key;
    }

    writer.flush()?;
    Ok(())
}

fn extract_comparison_key(line: &str, options: &UniqOptions) -> String {
    let mut key = line.to_string();

    // Skip fields
    if options.skip_fields > 0 {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() > options.skip_fields {
            key = fields[options.skip_fields..].join(" ");
        } else {
            key = String::new();
        }
    }

    // Skip characters
    if options.skip_chars > 0 {
        let chars: Vec<char> = key.chars().collect();
        if chars.len() > options.skip_chars {
            key = chars[options.skip_chars..].iter().collect();
        } else {
            key = String::new();
        }
    }

    // Check only specified number of characters
    if let Some(check_chars) = options.check_chars {
        let chars: Vec<char> = key.chars().collect();
        if chars.len() > check_chars {
            key = chars[..check_chars].iter().collect();
        }
    }

    // Case insensitive comparison
    if options.ignore_case {
        key = key.to_lowercase();
    }

    key
}

fn process_group<W: Write>(
    group_lines: &[String],
    count: usize,
    writer: &mut W,
    options: &UniqOptions,
    separator: u8,
) -> ShellResult<()> {
    if group_lines.is_empty() {
        return Ok(());
    }

    let is_duplicate = count > 1;
    let is_unique = count == 1;

    // Determine if we should output this group
    let should_output = if options.unique {
        is_unique
    } else if options.repeated || options.all_repeated {
        is_duplicate
    } else {
        true // Default: output all groups (but only one line per group)
    };

    if !should_output {
        return Ok(());
    }

    // Output format
    if options.all_repeated {
        // Output all lines in the group
        for (i, line) in group_lines.iter().enumerate() {
            if options.count {
                let count_str = if options.no_color {
                    format!("{count:7}")
                } else {
                    let colors = ColorPalette::new();
                    if count > 1 {
                        format!("{}{:7}{}", colors.warning, count, colors.reset)
                    } else {
                        format!("{}{:7}{}", colors.info, count, colors.reset)
                    }
                };
                write!(writer, "{count_str} {line}")?;
            } else {
                write!(writer, "{line}")?;
            }
            writer.write_all(&[separator])?;

            // Add separator between groups (except for the last group)
            if options.group && i == group_lines.len() - 1 && is_duplicate {
                writer.write_all(&[separator])?;
            }
        }
    } else {
        // Output only the first line of the group
        let line = &group_lines[0];

        if options.count {
            let count_str = if options.no_color {
                format!("{count:7}")
            } else {
                let colors = ColorPalette::new();
                if count > 1 {
                    format!("{}{:7}{}", colors.warning, count, colors.reset)
                } else {
                    format!("{}{:7}{}", colors.info, count, colors.reset)
                }
            };
            write!(writer, "{count_str} {line}")?;
        } else {
            write!(writer, "{line}")?;
        }
        writer.write_all(&[separator])?;

        // Add separator between groups
        if options.group && is_duplicate {
            writer.write_all(&[separator])?;
        }
    }

    Ok(())
}

/// CLI wrapper function for uniq command
pub fn uniq_cli(args: &[String]) -> anyhow::Result<()> {
    let options = parse_uniq_args(args).unwrap();
    match process_uniq(&options) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("uniq command failed: {}", e)),
    }
}

/// Execute function stub
pub fn execute(
    _args: &[String],
    _context: &crate::common::BuiltinContext,
) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_extract_comparison_key() {
        let options = UniqOptions {
            count: false,
            repeated: false,
            unique: false,
            all_repeated: false,
            ignore_case: false,
            skip_fields: 0,
            skip_chars: 0,
            check_chars: None,
            zero_terminated: false,
            group: false,
            input_file: None,
            output_file: None,
            no_color: false,
        };

        assert_eq!(
            extract_comparison_key("hello world", &options),
            "hello world"
        );

        let mut options_skip_fields = options.clone();
        options_skip_fields.skip_fields = 1;
        assert_eq!(
            extract_comparison_key("hello world test", &options_skip_fields),
            "world test"
        );

        let mut options_skip_chars = options.clone();
        options_skip_chars.skip_chars = 2;
        assert_eq!(extract_comparison_key("hello", &options_skip_chars), "llo");

        let mut options_ignore_case = options.clone();
        options_ignore_case.ignore_case = true;
        assert_eq!(
            extract_comparison_key("HELLO", &options_ignore_case),
            "hello"
        );
    }

    #[test]
    fn test_uniq_basic() {
        let input = "line1\nline1\nline2\nline2\nline2\nline3\n";
        let expected = "line1\nline2\nline3\n";

        let options = UniqOptions {
            count: false,
            repeated: false,
            unique: false,
            all_repeated: false,
            ignore_case: false,
            skip_fields: 0,
            skip_chars: 0,
            check_chars: None,
            zero_terminated: false,
            group: false,
            input_file: None,
            output_file: None,
            no_color: false,
        };

        let mut output = Vec::new();
        process_uniq_stream(Cursor::new(input.as_bytes()), &mut output, &options, b'\n').unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), expected);
    }

    #[test]
    fn test_uniq_count() {
        let input = "line1\nline1\nline2\nline2\nline2\nline3\n";
        let expected = "      2 line1\n      3 line2\n      1 line3\n";

        let options = UniqOptions {
            count: true,
            repeated: false,
            unique: false,
            all_repeated: false,
            ignore_case: false,
            skip_fields: 0,
            skip_chars: 0,
            check_chars: None,
            zero_terminated: false,
            group: false,
            input_file: None,
            output_file: None,
            no_color: true,
        };

        let mut output = Vec::new();
        process_uniq_stream(Cursor::new(input.as_bytes()), &mut output, &options, b'\n').unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), expected);
    }
}
