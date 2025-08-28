use std::collections::HashMap;
use std::io::{self, Read};
use crate::common::{BuiltinResult, BuiltinContext};

/// Translate or delete characters
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    if args.is_empty() {
        eprintln!("tr: missing operand");
        return Ok(1);
    }

    let mut delete_mode = false;
    let mut complement = false;
    let mut squeeze_repeats = false;
    let mut truncate_set1 = false;
    
    let mut set2 = String::new();
    let mut positional_args = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--delete" => delete_mode = true,
            "-c" | "-C" | "--complement" => complement = true,
            "-s" | "--squeeze-repeats" => squeeze_repeats = true,
            "-t" | "--truncate-set1" => truncate_set1 = true,
            "-h" | "--help" => {
                print_help();
                return Ok(0);
            }
            arg if arg.starts_with('-') => {
                eprintln!("tr: invalid option '{arg}'");
                return Ok(1);
            }
            _ => positional_args.push(&args[i]),
        }
        i += 1;
    }

    if positional_args.is_empty() {
        eprintln!("tr: missing operand");
        return Ok(1);
    }

    let set1: String = positional_args[0].to_string();
    if positional_args.len() > 1 {
        set2 = positional_args[1].to_string();
    } else if !delete_mode {
        eprintln!("tr: missing operand after '{set1}'");
        return Ok(1);
    }

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut buffer = String::new();

    if let Err(e) = reader.read_to_string(&mut buffer) {
        eprintln!("tr: error reading input: {e}");
        return Ok(1);
    }

    let result = if delete_mode {
        delete_characters(&buffer, &set1, complement)
    } else {
        translate_characters(&buffer, &set1, &set2, truncate_set1)
    };

    let final_result = if squeeze_repeats {
        squeeze_repeated_characters(&result, &set2)
    } else {
        result
    };

    print!("{final_result}");
    Ok(0)
}

fn expand_set(set: &str) -> Vec<char> {
    let mut expanded = Vec::new();
    let chars: Vec<char> = set.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 2 < chars.len() && chars[i + 1] == '-' {
            // Range like a-z
            let start = chars[i] as u8;
            let end = chars[i + 2] as u8;
            
            if start <= end {
                for c in start..=end {
                    expanded.push(c as char);
                }
            } else {
                // Invalid range, treat as literal characters
                expanded.push(chars[i]);
                expanded.push(chars[i + 1]);
                expanded.push(chars[i + 2]);
            }
            i += 3;
        } else {
            expanded.push(chars[i]);
            i += 1;
        }
    }

    expanded
}

fn expand_escape_sequences(set: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = set.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                'r' => result.push('\r'),
                '\\' => result.push('\\'),
                'a' => result.push('\x07'), // bell
                'b' => result.push('\x08'), // backspace
                'f' => result.push('\x0c'), // form feed
                'v' => result.push('\x0b'), // vertical tab
                c => {
                    result.push('\\');
                    result.push(c);
                }
            }
            i += 2;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

fn delete_characters(input: &str, set1: &str, complement: bool) -> String {
    let expanded_set1 = expand_set(&expand_escape_sequences(set1));
    let delete_set: std::collections::HashSet<char> = expanded_set1.into_iter().collect();

    input.chars().filter(|&c| {
        if complement {
            delete_set.contains(&c)
        } else {
            !delete_set.contains(&c)
        }
    }).collect()
}

fn translate_characters(input: &str, set1: &str, set2: &str, truncate_set1: bool) -> String {
    let expanded_set1 = expand_set(&expand_escape_sequences(set1));
    let expanded_set2 = expand_set(&expand_escape_sequences(set2));

    let mut translation_map = HashMap::new();

    if truncate_set1 && expanded_set1.len() > expanded_set2.len() {
        // Truncate set1 to match set2 length
        for (i, &c1) in expanded_set1.iter().take(expanded_set2.len()).enumerate() {
            if let Some(&c2) = expanded_set2.get(i) {
                translation_map.insert(c1, c2);
            }
        }
    } else {
        // Standard behavior
        for (i, &c1) in expanded_set1.iter().enumerate() {
            let c2 = if i < expanded_set2.len() {
                expanded_set2[i]
            } else if !expanded_set2.is_empty() {
                // Repeat last character of set2
                expanded_set2[expanded_set2.len() - 1]
            } else {
                c1 // No translation
            };
            translation_map.insert(c1, c2);
        }
    }

    input.chars().map(|c| {
        translation_map.get(&c).copied().unwrap_or(c)
    }).collect()
}

fn squeeze_repeated_characters(input: &str, set: &str) -> String {
    if set.is_empty() {
        return input.to_string();
    }

    let squeeze_set: std::collections::HashSet<char> = 
        expand_set(&expand_escape_sequences(set)).into_iter().collect();

    let mut result = String::new();
    let mut prev_char: Option<char> = None;

    for c in input.chars() {
        if squeeze_set.contains(&c) {
            if prev_char != Some(c) {
                result.push(c);
            }
        } else {
            result.push(c);
        }
        prev_char = Some(c);
    }

    result
}

fn print_help() {
    println!("Usage: tr [OPTION]... SET1 [SET2]");
    println!("Translate, squeeze, and/or delete characters from standard input,");
    println!("writing to standard output.");
    println!();
    println!("Options:");
    println!("  -c, -C, --complement    use the complement of SET1");
    println!("  -d, --delete            delete characters in SET1, do not translate");
    println!("  -s, --squeeze-repeats   replace each sequence of repeated characters");
    println!("                          that are listed in the last specified SET");
    println!("  -t, --truncate-set1     first truncate SET1 to length of SET2");
    println!("  -h, --help              display this help and exit");
    println!();
    println!("SETs are specified as strings of characters. Most represent themselves.");
    println!("Interpreted sequences are:");
    println!("  \\NNN   character with octal value NNN (1 to 3 octal digits)");
    println!("  \\\\     backslash");
    println!("  \\a     audible BEL");
    println!("  \\b     backspace");
    println!("  \\f     form feed");
    println!("  \\n     new line");
    println!("  \\r     return");
    println!("  \\t     horizontal tab");
    println!("  \\v     vertical tab");
    println!();
    println!("Character ranges can be specified with CHAR1-CHAR2.");
    println!();
    println!("Examples:");
    println!("  tr 'a-z' 'A-Z'      Convert lowercase to uppercase");
    println!("  tr -d '0-9'          Delete all digits");
    println!("  tr -s ' '            Squeeze multiple spaces to single space");
    println!("  tr '\\n' ' '          Replace newlines with spaces");
}
