use anyhow::Result;
use crate::powershell_object::{PowerShellObject, emit};

/// Display help for a builtin (placeholder).
pub fn get_help_cli(args: &[String]) -> Result<()> {
    if args.is_empty() { println!("Usage: Get-Help <builtin>"); return Ok(()); }
    let name = &args[0];
    let text = format!("Help for {name}: (documentation placeholder)");
    emit(&[PowerShellObject::Map(vec![
        ("Name".into(), PowerShellObject::from(name.as_str())),
        ("Summary".into(), PowerShellObject::from(text.as_str())),
    ])]);
    Ok(())
}
