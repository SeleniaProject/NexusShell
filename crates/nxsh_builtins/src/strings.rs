use anyhow::{Result, anyhow};
use std::io::{self, Read};
use std::fs::File;

#[derive(Debug, Clone, Copy)]
pub enum Encoding {
    /// Single 7-bit-character (ASCII)
    Ascii,
    /// Single 8-bit-character (Latin-1)
    Latin1,
    /// 16-bit little-endian
    Utf16Le,
    /// 16-bit big-endian  
    Utf16Be,
    /// 32-bit little-endian
    Utf32Le,
    /// 32-bit big-endian
    Utf32Be,
    /// UTF-8 variable width encoding
    Utf8,
}

impl Encoding {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "s" | "ascii" => Ok(Self::Ascii),
            "S" | "latin1" => Ok(Self::Latin1),
            "l" | "utf16le" => Ok(Self::Utf16Le),
            "b" | "utf16be" => Ok(Self::Utf16Be),
            "L" | "utf32le" => Ok(Self::Utf32Le),
            "B" | "utf32be" => Ok(Self::Utf32Be),
            "u" | "utf8" => Ok(Self::Utf8),
            _ => Err(anyhow!("Unsupported encoding: {}", s)),
        }
    }

    pub fn char_size(&self) -> usize {
        match self {
            Self::Ascii | Self::Latin1 | Self::Utf8 => 1,
            Self::Utf16Le | Self::Utf16Be => 2,
            Self::Utf32Le | Self::Utf32Be => 4,
        }
    }
}

/// CLI wrapper function for strings command (extract printable strings)
pub fn strings_cli(args: &[String]) -> Result<()> {
    let mut min_length = 4;
    let mut encoding = Encoding::Ascii; // Default: 7-bit ASCII (now fully implemented)
    let mut print_filename = false;
    let mut files = Vec::new();
    let mut all_encodings = false;
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--bytes" => {
                if i + 1 < args.len() {
                    min_length = args[i + 1].parse().unwrap_or(4);
                    i += 1;
                }
            }
            "-e" | "--encoding" => {
                if i + 1 < args.len() {
                    encoding = Encoding::from_str(&args[i + 1])?;
                    i += 1;
                }
            }
            "-f" | "--print-file-name" => {
                print_filename = true;
            }
            "--all-encodings" => {
                all_encodings = true;
            }
            "-h" | "--help" => {
                println!("strings - print the sequences of printable characters in files");
                println!("Usage: strings [OPTION]... [FILE]...");
                println!("  -n, --bytes=MIN-LEN    print sequences of at least MIN-LEN characters");
                println!("  -e, --encoding=ENCODING select character encoding:");
                println!("                           s, ascii    - single 7-bit-character (default)");
                println!("                           S, latin1   - single 8-bit-character");
                println!("                           l, utf16le  - 16-bit little-endian");
                println!("                           b, utf16be  - 16-bit big-endian");
                println!("                           L, utf32le  - 32-bit little-endian");
                println!("                           B, utf32be  - 32-bit big-endian");
                println!("                           u, utf8     - UTF-8 variable width");
                println!("  --all-encodings       scan with all supported encodings (union)");
                println!("  -f, --print-file-name  print the name of the file before each string");
                println!("  -h, --help             display this help and exit");
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                files.push(arg.to_string());
            }
            _ => {
                eprintln!("strings: unrecognized option '{}'", args[i]);
                return Err(anyhow!("Invalid option"));
            }
        }
        i += 1;
    }
    
    let encodings_list: Vec<Encoding> = if all_encodings {
        vec![
            Encoding::Ascii,
            Encoding::Latin1,
            Encoding::Utf8,
            Encoding::Utf16Le,
            Encoding::Utf16Be,
            Encoding::Utf32Le,
            Encoding::Utf32Be,
        ]
    } else {
        vec![encoding]
    };

    if files.is_empty() {
        // Read from stdin
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        for enc in &encodings_list {
            extract_strings(&buffer, min_length, *enc, print_filename, None)?;
        }
    } else {
        // Read from files
        for filename in files {
            let mut file = File::open(&filename)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            for enc in &encodings_list {
                extract_strings(&buffer, min_length, *enc, print_filename, Some(&filename))?;
            }
        }
    }
    
    Ok(())
}

fn extract_strings(data: &[u8], min_length: usize, encoding: Encoding, print_filename: bool, filename: Option<&str>) -> Result<()> {
    match encoding {
        Encoding::Ascii => extract_ascii_strings(data, min_length, print_filename, filename),
        Encoding::Latin1 => extract_latin1_strings(data, min_length, print_filename, filename),
        Encoding::Utf16Le => extract_utf16_strings(data, min_length, print_filename, filename, false),
        Encoding::Utf16Be => extract_utf16_strings(data, min_length, print_filename, filename, true),
        Encoding::Utf32Le => extract_utf32_strings(data, min_length, print_filename, filename, false),
        Encoding::Utf32Be => extract_utf32_strings(data, min_length, print_filename, filename, true),
        Encoding::Utf8 => extract_utf8_strings(data, min_length, print_filename, filename),
    }
}

fn extract_ascii_strings(data: &[u8], min_length: usize, print_filename: bool, filename: Option<&str>) -> Result<()> {
    let mut current_string = Vec::new();
    
    for &byte in data {
        if byte.is_ascii_graphic() || byte == b' ' {
            current_string.push(byte);
        } else {
            if current_string.len() >= min_length {
                let string = String::from_utf8_lossy(&current_string);
                print_result(&string, print_filename, filename);
            }
            current_string.clear();
        }
    }
    
    // Handle final string if buffer doesn't end with non-printable character
    if current_string.len() >= min_length {
        let string = String::from_utf8_lossy(&current_string);
        print_result(&string, print_filename, filename);
    }
    
    Ok(())
}

fn extract_latin1_strings(data: &[u8], min_length: usize, print_filename: bool, filename: Option<&str>) -> Result<()> {
    let mut current_string = Vec::new();
    
    for &byte in data {
        // Latin-1 printable characters (0x20-0x7E and 0xA0-0xFF, excluding control chars)
    if (byte >= 0x20 && byte <= 0x7E) || byte >= 0xA0 {
            current_string.push(byte);
        } else {
            if current_string.len() >= min_length {
                // Convert Latin-1 to UTF-8 string
                let string: String = current_string.iter().map(|&b| b as char).collect();
                print_result(&string, print_filename, filename);
            }
            current_string.clear();
        }
    }
    
    if current_string.len() >= min_length {
        let string: String = current_string.iter().map(|&b| b as char).collect();
        print_result(&string, print_filename, filename);
    }
    
    Ok(())
}

fn extract_utf16_strings(data: &[u8], min_length: usize, print_filename: bool, filename: Option<&str>, big_endian: bool) -> Result<()> {
    if data.len() % 2 != 0 {
        return Ok(()); // Invalid UTF-16 data
    }
    
    let mut current_string = Vec::new();
    let mut i = 0;
    
    while i + 1 < data.len() {
        let code_unit = if big_endian {
            u16::from_be_bytes([data[i], data[i + 1]])
        } else {
            u16::from_le_bytes([data[i], data[i + 1]])
        };
        
        // Check if it's a printable character (basic check)
        if (code_unit >= 0x20 && code_unit <= 0x7E) || (code_unit >= 0xA0 && code_unit < 0xD800) || (code_unit >= 0xE000) {
            current_string.push(code_unit);
        } else {
            if current_string.len() >= min_length {
                if let Ok(string) = String::from_utf16(&current_string) {
                    print_result(&string, print_filename, filename);
                }
            }
            current_string.clear();
        }
        i += 2;
    }
    
    if current_string.len() >= min_length {
        if let Ok(string) = String::from_utf16(&current_string) {
            print_result(&string, print_filename, filename);
        }
    }
    
    Ok(())
}

fn extract_utf32_strings(data: &[u8], min_length: usize, print_filename: bool, filename: Option<&str>, big_endian: bool) -> Result<()> {
    if data.len() % 4 != 0 {
        return Ok(()); // Invalid UTF-32 data
    }
    
    let mut current_string = Vec::new();
    let mut i = 0;
    
    while i + 3 < data.len() {
        let code_point = if big_endian {
            u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]])
        } else {
            u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]])
        };
        
        // Check if it's a valid Unicode scalar value
        if let Some(ch) = char::from_u32(code_point) {
            if !ch.is_control() || ch == ' ' {
                current_string.push(ch);
            } else {
                if current_string.len() >= min_length {
                    let string: String = current_string.iter().collect();
                    print_result(&string, print_filename, filename);
                }
                current_string.clear();
            }
        } else {
            if current_string.len() >= min_length {
                let string: String = current_string.iter().collect();
                print_result(&string, print_filename, filename);
            }
            current_string.clear();
        }
        i += 4;
    }
    
    if current_string.len() >= min_length {
        let string: String = current_string.iter().collect();
        print_result(&string, print_filename, filename);
    }
    
    Ok(())
}

fn extract_utf8_strings(data: &[u8], min_length: usize, print_filename: bool, filename: Option<&str>) -> Result<()> {
    let mut current_string = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        // Try to decode next UTF-8 character
        let (_ch, len) = match decode_utf8_char(&data[i..]) {
            Some((ch, len)) if !ch.is_control() || ch == ' ' => {
                current_string.push(ch);
                (ch, len)
            }
            Some((_, len)) => {
                // Control character found, end current string
                if current_string.len() >= min_length {
                    let string: String = current_string.iter().collect();
                    print_result(&string, print_filename, filename);
                }
                current_string.clear();
                ('\0', len)
            }
            None => {
                // Invalid UTF-8 sequence, skip byte
                if current_string.len() >= min_length {
                    let string: String = current_string.iter().collect();
                    print_result(&string, print_filename, filename);
                }
                current_string.clear();
                ('\0', 1)
            }
        };
    i += len;
    }
    
    if current_string.len() >= min_length {
        let string: String = current_string.iter().collect();
        print_result(&string, print_filename, filename);
    }
    
    Ok(())
}

fn decode_utf8_char(data: &[u8]) -> Option<(char, usize)> {
    if data.is_empty() {
        return None;
    }
    
    let first_byte = data[0];
    let (expected_len, code_point) = if first_byte < 0x80 {
        (1, first_byte as u32)
    } else if first_byte < 0xC0 {
        return None; // Invalid start byte
    } else if first_byte < 0xE0 {
        (2, (first_byte & 0x1F) as u32)
    } else if first_byte < 0xF0 {
        (3, (first_byte & 0x0F) as u32)
    } else if first_byte < 0xF8 {
        (4, (first_byte & 0x07) as u32)
    } else {
        return None; // Invalid start byte
    };
    
    if data.len() < expected_len {
        return None;
    }
    
    let mut code_point = code_point;
    for i in 1..expected_len {
        let byte = data[i];
        if byte & 0xC0 != 0x80 {
            return None; // Invalid continuation byte
        }
        code_point = (code_point << 6) | ((byte & 0x3F) as u32);
    }
    
    char::from_u32(code_point).map(|ch| (ch, expected_len))
}

fn print_result(string: &str, print_filename: bool, filename: Option<&str>) {
    if print_filename && filename.is_some() {
        println!("{}: {}", filename.unwrap(), string);
    } else {
        println!("{}", string);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_from_str() {
        assert!(matches!(Encoding::from_str("s").unwrap(), Encoding::Ascii));
        assert!(matches!(Encoding::from_str("ascii").unwrap(), Encoding::Ascii));
        assert!(matches!(Encoding::from_str("S").unwrap(), Encoding::Latin1));
        assert!(matches!(Encoding::from_str("latin1").unwrap(), Encoding::Latin1));
        assert!(matches!(Encoding::from_str("l").unwrap(), Encoding::Utf16Le));
        assert!(matches!(Encoding::from_str("utf16le").unwrap(), Encoding::Utf16Le));
        assert!(matches!(Encoding::from_str("b").unwrap(), Encoding::Utf16Be));
        assert!(matches!(Encoding::from_str("utf16be").unwrap(), Encoding::Utf16Be));
        assert!(matches!(Encoding::from_str("u").unwrap(), Encoding::Utf8));
        assert!(matches!(Encoding::from_str("utf8").unwrap(), Encoding::Utf8));
        
        assert!(Encoding::from_str("invalid").is_err());
    }

    #[test]
    fn test_extract_ascii_strings() {
        let data = b"Hello\x00World\x01Test123\x02";
        let result = std::panic::catch_unwind(|| {
            extract_ascii_strings(data, 4, false, None)
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_latin1_strings() {
        let data = b"Caf\xe9\x00\xc9\xe9\x01";  // "Café" in Latin-1
        let result = std::panic::catch_unwind(|| {
            extract_latin1_strings(data, 3, false, None)
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_utf8_strings() {
        let data = "Hello 世界\x00Test".as_bytes();
        let result = std::panic::catch_unwind(|| {
            extract_utf8_strings(data, 4, false, None)
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_decode_utf8_char() {
        // ASCII character
        assert_eq!(decode_utf8_char(b"A"), Some(('A', 1)));
        
        // Two-byte UTF-8 character (é)
        assert_eq!(decode_utf8_char(&[0xC3, 0xA9]), Some(('é', 2)));
        
        // Three-byte UTF-8 character (世)
        assert_eq!(decode_utf8_char(&[0xE4, 0xB8, 0x96]), Some(('世', 3)));
        
        // Invalid UTF-8
        assert_eq!(decode_utf8_char(&[0xFF]), None);
        assert_eq!(decode_utf8_char(&[0xC3]), None); // Incomplete sequence
    }

    #[test]
    fn test_char_size() {
        assert_eq!(Encoding::Ascii.char_size(), 1);
        assert_eq!(Encoding::Latin1.char_size(), 1);
        assert_eq!(Encoding::Utf8.char_size(), 1);
        assert_eq!(Encoding::Utf16Le.char_size(), 2);
        assert_eq!(Encoding::Utf16Be.char_size(), 2);
        assert_eq!(Encoding::Utf32Le.char_size(), 4);
        assert_eq!(Encoding::Utf32Be.char_size(), 4);
    }

    #[test]
    fn test_extract_utf16_strings() {
        // "Hello" in UTF-16 LE
        let data = &[0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00, 0x00, 0x00, 0x57, 0x00, 0x6F, 0x00, 0x72, 0x00, 0x6C, 0x00, 0x64, 0x00];
        let result = std::panic::catch_unwind(|| {
            extract_utf16_strings(data, 4, false, None, false)
        });
        assert!(result.is_ok());
    }
}
