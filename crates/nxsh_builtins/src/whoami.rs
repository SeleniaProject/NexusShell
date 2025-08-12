use anyhow::Result;

/// CLI wrapper function for whoami command
pub fn whoami_cli(args: &[String]) -> Result<()> {
    // Parse arguments
    let show_help = args.contains(&"--help".to_string()) || args.contains(&"-h".to_string());
    
    if show_help {
        println!("whoami - print effective userid");
        println!("Usage: whoami [OPTION]...");
        println!("  -h, --help     display this help and exit");
        return Ok(());
    }
    
    // Get current username
    match std::env::var("USERNAME").or_else(|_| std::env::var("USER")) {
        Ok(username) => println!("{username}"),
        Err(_) => {
            // Fallback to current user detection
            #[cfg(unix)]
            {
                use std::ffi::CStr;
                unsafe {
                    let uid = libc::getuid();
                    let passwd = libc::getpwuid(uid);
                    if !passwd.is_null() {
                        let name = CStr::from_ptr((*passwd).pw_name);
                        if let Ok(name_str) = name.to_str() {
                            println!("{}", name_str);
                            return Ok(());
                        }
                    }
                }
            }
            
            println!("unknown");
        }
    }
    
    Ok(())
}
