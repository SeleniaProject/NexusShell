use anyhow::Result;
use crate::{list_builtin_names, powershell_object::{PowerShellObject, emit}};

/// List available builtins with simple metadata (placeholder only name & type)
pub fn get_command_cli(_args: &[String]) -> Result<()> {
    let names = list_builtin_names();
    let objs: Vec<PowerShellObject> = names.into_iter()
        .map(|n| PowerShellObject::Map(vec![
            ("Name".into(), PowerShellObject::from(n)),
            ("Type".into(), PowerShellObject::from("Builtin")),
        ])).collect();
    emit(&objs);
    Ok(())
}
