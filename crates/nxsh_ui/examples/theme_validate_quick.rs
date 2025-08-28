// Simple theme validation using nxsh_ui::theme_validator
use nxsh_ui::theme_validator::ThemeValidator;

fn main() -> anyhow::Result<()> {
    println!("NexusShell Theme Validator (quick)");
    let themes = nxsh_ui::themes::list_theme_files()?;
    let validator = ThemeValidator::new()?;
    for t in themes {
        print!("Validating {} ... ", t.display());
        match validator.validate_theme_file(&t) {
            Ok(result) => {
                if result.is_valid() {
                    println!("ok");
                } else {
                    println!("fail");
                    for e in result.errors { println!("  - {e}"); }
                }
            }
            Err(e) => {
                println!("error: {e}");
            }
        }
    }
    Ok(())
}


