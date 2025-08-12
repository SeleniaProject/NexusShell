use crate::compat::Result;
use std::collections::HashMap;

/// PowerShell compatibility mode for NexusShell
#[derive(Debug, Clone)]
pub struct PowerShellCompat {
    cmdlets: HashMap<String, CmdletInfo>,
    aliases: HashMap<String, String>,
    variables: HashMap<String, PowerShellVariable>,
    execution_policy: ExecutionPolicy,
    modules: HashMap<String, Module>,
    profiles: HashMap<String, Profile>,
}

impl PowerShellCompat {
    pub fn new() -> Self {
        let mut compat = Self {
            cmdlets: HashMap::new(),
            aliases: HashMap::new(),
            variables: HashMap::new(),
            execution_policy: ExecutionPolicy::RemoteSigned,
            modules: HashMap::new(),
            profiles: HashMap::new(),
        };
        
        compat.register_core_cmdlets();
        compat.register_core_aliases();
        compat.register_automatic_variables();
        compat
    }

    /// Execute a PowerShell-style command
    pub fn execute_command(&mut self, command: &str, args: Vec<String>) -> Result<CommandResult> {
        // Check if it's an alias first
        let actual_command = self.aliases.get(command).unwrap_or(&command.to_string()).clone();
        
        // Check if it's a cmdlet
        if let Some(cmdlet) = self.cmdlets.get(&actual_command).cloned() {
            return self.execute_cmdlet(&cmdlet, args);
        }
        
        // Check if it's a function
        if let Some(function) = self.get_function(&actual_command) {
            return self.execute_function(&function, args);
        }
        
        // Fall back to external command
        self.execute_external_command(&actual_command, args)
    }

    /// Execute a PowerShell cmdlet
    fn execute_cmdlet(&mut self, cmdlet: &CmdletInfo, args: Vec<String>) -> Result<CommandResult> {
        match cmdlet.name.as_str() {
            "Get-ChildItem" => self.get_child_item(args),
            "Get-Content" => self.get_content(args),
            "Set-Content" => self.set_content(args),
            "Get-Process" => self.get_process(args),
            "Stop-Process" => self.stop_process(args),
            "Get-Service" => self.get_service(args),
            "Start-Service" => self.start_service(args),
            "Stop-Service" => self.stop_service(args),
            "Get-Location" => self.get_location(args),
            "Set-Location" => self.set_location(args),
            "Copy-Item" => self.copy_item(args),
            "Move-Item" => self.move_item(args),
            "Remove-Item" => self.remove_item(args),
            "New-Item" => self.new_item(args),
            "Test-Path" => self.test_path(args),
            "Get-Command" => self.get_command(args),
            "Get-Help" => self.get_help(args),
            "Measure-Object" => self.measure_object(args),
            "Sort-Object" => self.sort_object(args),
            "Select-Object" => self.select_object(args),
            "Where-Object" => self.where_object(args),
            "ForEach-Object" => self.foreach_object(args),
            "Group-Object" => self.group_object(args),
            "Out-String" => self.out_string(args),
            "Out-File" => self.out_file(args),
            "Write-Host" => self.write_host(args),
            "Write-Output" => self.write_output(args),
            "Read-Host" => self.read_host(args),
            _ => Err(crate::anyhow!("Cmdlet '{}' not implemented", cmdlet.name))
        }
    }

    /// PowerShell pipeline support
    pub fn execute_pipeline(&mut self, pipeline: &str) -> Result<Vec<PowerShellObject>> {
        let commands: Vec<&str> = pipeline.split(" | ").collect();
        let mut objects = Vec::new();
        
        for (i, command) in commands.iter().enumerate() {
            let (cmd_name, args) = self.parse_command(command)?;
            
            if i == 0 {
                // First command in pipeline
                let result = self.execute_command(&cmd_name, args)?;
                objects = result.objects;
            } else {
                // Subsequent commands receive objects from previous command
                objects = self.execute_pipeline_command(&cmd_name, args, objects)?;
            }
        }
        
        Ok(objects)
    }

    /// Execute a command in a pipeline context
    fn execute_pipeline_command(&mut self, command: &str, args: Vec<String>, input_objects: Vec<PowerShellObject>) -> Result<Vec<PowerShellObject>> {
        match command {
            "Where-Object" => self.filter_objects(input_objects, &args),
            "Select-Object" => self.select_object_properties(input_objects, &args),
            "Sort-Object" => self.sort_objects(input_objects, &args),
            "ForEach-Object" => self.transform_objects(input_objects, &args),
            "Group-Object" => self.group_objects(input_objects, &args),
            "Measure-Object" => self.measure_objects(input_objects, &args),
            _ => {
                // For other commands, convert objects to strings and execute
                let string_args: Vec<String> = input_objects.iter()
                    .map(|obj| obj.to_string())
                    .collect();
                let result = self.execute_command(command, [args, string_args].concat())?;
                Ok(result.objects)
            }
        }
    }

    /// PowerShell variable expansion
    pub fn expand_variables(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // Replace $variable patterns
        for (name, var) in &self.variables {
            let pattern = format!("${}", name);
            result = result.replace(&pattern, &var.value);
        }
        
        // Replace ${variable} patterns
        for (name, var) in &self.variables {
            let pattern = format!("${{{}}}", name);
            result = result.replace(&pattern, &var.value);
        }
        
        result
    }

    /// PowerShell expression evaluation
    pub fn evaluate_expression(&mut self, expression: &str) -> Result<PowerShellObject> {
        // Simple expression evaluator - would need full PowerShell parser for real implementation
        if expression.starts_with('"') && expression.ends_with('"') {
            // String literal
            Ok(PowerShellObject::String(expression[1..expression.len()-1].to_string()))
        } else if let Ok(num) = expression.parse::<i64>() {
            // Integer literal
            Ok(PowerShellObject::Integer(num))
        } else if let Ok(num) = expression.parse::<f64>() {
            // Float literal
            Ok(PowerShellObject::Float(num))
        } else if expression == "$true" {
            Ok(PowerShellObject::Boolean(true))
        } else if expression == "$false" {
            Ok(PowerShellObject::Boolean(false))
        } else if expression == "$null" {
            Ok(PowerShellObject::Null)
        } else if expression.starts_with('$') {
            // Variable reference
            let var_name = &expression[1..];
            if let Some(var) = self.variables.get(var_name) {
                Ok(PowerShellObject::String(var.value.clone()))
            } else {
                Ok(PowerShellObject::Null)
            }
        } else {
            // Treat as string
            Ok(PowerShellObject::String(expression.to_string()))
        }
    }

    /// Set a PowerShell variable
    pub fn set_variable(&mut self, name: String, value: String, scope: VariableScope) -> Result<()> {
        let variable = PowerShellVariable {
            name: name.clone(),
            value,
            scope,
            data_type: VariableType::String,
            read_only: false,
        };
        
        self.variables.insert(name, variable);
        Ok(())
    }

    /// Get a PowerShell variable
    pub fn get_variable(&self, name: &str) -> Option<&PowerShellVariable> {
        self.variables.get(name)
    }

    // Cmdlet implementations
    fn get_child_item(&mut self, args: Vec<String>) -> Result<CommandResult> {
        let path = args.get(0).map(|s| s.as_str()).unwrap_or(".");
        let entries = std::fs::read_dir(path)?;
        
        let mut objects = Vec::new();
        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            
            objects.push(PowerShellObject::FileInfo {
                name: entry.file_name().to_string_lossy().to_string(),
                full_path: entry.path().to_string_lossy().to_string(),
                size: metadata.len(),
                is_directory: metadata.is_dir(),
                last_modified: metadata.modified().ok(),
            });
        }
        
        Ok(CommandResult {
            success: true,
            output: format!("Found {} items", objects.len()),
            objects,
        })
    }

    fn get_content(&mut self, args: Vec<String>) -> Result<CommandResult> {
        if args.is_empty() {
            return Err(crate::anyhow!("Get-Content requires a path argument"));
        }
        
        let content = std::fs::read_to_string(&args[0])?;
        let lines: Vec<PowerShellObject> = content.lines()
            .map(|line| PowerShellObject::String(line.to_string()))
            .collect();
        
        Ok(CommandResult {
            success: true,
            output: format!("Read {} lines", lines.len()),
            objects: lines,
        })
    }

    fn set_content(&mut self, args: Vec<String>) -> Result<CommandResult> {
        if args.len() < 2 {
            return Err(crate::anyhow!("Set-Content requires path and value arguments"));
        }
        
        std::fs::write(&args[0], &args[1])?;
        
        Ok(CommandResult {
            success: true,
            output: format!("Content written to {}", args[0]),
            objects: vec![],
        })
    }

    fn get_process(&mut self, args: Vec<String>) -> Result<CommandResult> {
        // Platform-specific process listing would be implemented here
        let processes = vec![
            PowerShellObject::ProcessInfo {
                name: "explorer".to_string(),
                id: 1234,
                cpu: 5.2,
                memory: 104857600, // 100MB
                status: "Running".to_string(),
            },
            PowerShellObject::ProcessInfo {
                name: "chrome".to_string(),
                id: 5678,
                cpu: 15.8,
                memory: 524288000, // 500MB
                status: "Running".to_string(),
            },
        ];
        
        Ok(CommandResult {
            success: true,
            output: format!("Found {} processes", processes.len()),
            objects: processes,
        })
    }

    fn write_host(&mut self, args: Vec<String>) -> Result<CommandResult> {
        let output = args.join(" ");
        println!("{}", output);
        
        Ok(CommandResult {
            success: true,
            output: output.clone(),
            objects: vec![PowerShellObject::String(output)],
        })
    }

    fn write_output(&mut self, args: Vec<String>) -> Result<CommandResult> {
        let output = args.join(" ");
        
        Ok(CommandResult {
            success: true,
            output: output.clone(),
            objects: vec![PowerShellObject::String(output)],
        })
    }

    // Helper methods
    fn parse_command(&self, command: &str) -> Result<(String, Vec<String>)> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(crate::anyhow!("Empty command"));
        }
        
        let cmd_name = parts[0].to_string();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();
        
        Ok((cmd_name, args))
    }

    fn get_function(&self, _name: &str) -> Option<Function> {
        // Function lookup would be implemented here
        None
    }

    fn execute_function(&mut self, _function: &Function, _args: Vec<String>) -> Result<CommandResult> {
        // Function execution would be implemented here
        Ok(CommandResult {
            success: true,
            output: "Function executed".to_string(),
            objects: vec![],
        })
    }

    fn execute_external_command(&mut self, command: &str, args: Vec<String>) -> Result<CommandResult> {
        // External command execution
        use std::process::Command;
        
        let output = Command::new(command)
            .args(&args)
            .output()?;
        
        let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
        
        Ok(CommandResult {
            success: output.status.success(),
            output: if output.status.success() { stdout_str.clone() } else { stderr_str },
            objects: vec![PowerShellObject::String(stdout_str)],
        })
    }

    fn register_core_cmdlets(&mut self) {
        let cmdlets = [
            ("Get-ChildItem", "Gets the items and child items in one or more specified locations"),
            ("Get-Content", "Gets the content of the item at the specified location"),
            ("Set-Content", "Writes new content to a file"),
            ("Get-Process", "Gets the processes that are running on the local computer"),
            ("Stop-Process", "Stops one or more running processes"),
            ("Get-Service", "Gets the services on the computer"),
            ("Start-Service", "Starts one or more stopped services"),
            ("Stop-Service", "Stops one or more running services"),
            ("Get-Location", "Gets information about the current working location"),
            ("Set-Location", "Sets the current working location"),
            ("Copy-Item", "Copies an item from one location to another"),
            ("Move-Item", "Moves an item from one location to another"),
            ("Remove-Item", "Deletes the specified items"),
            ("New-Item", "Creates a new item"),
            ("Test-Path", "Determines whether all elements of a path exist"),
            ("Write-Host", "Writes customized output to a host"),
            ("Write-Output", "Writes objects to the pipeline"),
            ("Read-Host", "Reads a line of input from the console"),
            // Core discovery/help cmdlets used by tests and CLI
            ("Get-Command", "Gets basic information about cmdlets and aliases"),
            ("Get-Help", "Displays help about cmdlets and concepts"),
        ];
        
        for (name, description) in cmdlets {
            self.cmdlets.insert(name.to_string(), CmdletInfo {
                name: name.to_string(),
                description: description.to_string(),
                parameters: vec![], // Would be filled with actual parameter info
            });
        }
    }

    fn register_core_aliases(&mut self) {
        let aliases = [
            ("ls", "Get-ChildItem"),
            ("dir", "Get-ChildItem"),
            ("cat", "Get-Content"),
            ("type", "Get-Content"),
            ("ps", "Get-Process"),
            ("kill", "Stop-Process"),
            ("cd", "Set-Location"),
            ("pwd", "Get-Location"),
            ("cp", "Copy-Item"),
            ("copy", "Copy-Item"),
            ("mv", "Move-Item"),
            ("move", "Move-Item"),
            ("rm", "Remove-Item"),
            ("del", "Remove-Item"),
            ("md", "New-Item"),
            ("mkdir", "New-Item"),
            ("echo", "Write-Output"),
        ];
        
        for (alias, cmdlet) in aliases {
            self.aliases.insert(alias.to_string(), cmdlet.to_string());
        }
    }

    fn register_automatic_variables(&mut self) {
        // PowerShell automatic variables
        self.variables.insert("PSVersionTable".to_string(), PowerShellVariable {
            name: "PSVersionTable".to_string(),
            value: "NexusShell 1.0.0 (PowerShell Compatible)".to_string(),
            scope: VariableScope::Global,
            data_type: VariableType::HashTable,
            read_only: true,
        });

        self.variables.insert("PWD".to_string(), PowerShellVariable {
            name: "PWD".to_string(),
            value: std::env::current_dir().unwrap().to_string_lossy().to_string(),
            scope: VariableScope::Global,
            data_type: VariableType::String,
            read_only: false,
        });

        self.variables.insert("HOME".to_string(), PowerShellVariable {
            name: "HOME".to_string(),
            value: std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_else(|_| "/".to_string()),
            scope: VariableScope::Global,
            data_type: VariableType::String,
            read_only: true,
        });
    }

    // Pipeline helper methods (simplified implementations)
    fn filter_objects(&self, objects: Vec<PowerShellObject>, args: &[String]) -> Result<Vec<PowerShellObject>> {
        // Simple filtering - real implementation would parse PowerShell filter expressions
        Ok(objects) // Placeholder
    }

    fn select_object_properties(&self, objects: Vec<PowerShellObject>, args: &[String]) -> Result<Vec<PowerShellObject>> {
        // Property selection - real implementation would handle complex property paths
        Ok(objects) // Placeholder
    }

    fn sort_objects(&self, mut objects: Vec<PowerShellObject>, args: &[String]) -> Result<Vec<PowerShellObject>> {
        // Simple sorting - real implementation would handle multiple properties and custom comparers
        objects.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
        Ok(objects)
    }

    fn transform_objects(&self, objects: Vec<PowerShellObject>, args: &[String]) -> Result<Vec<PowerShellObject>> {
        // Object transformation - real implementation would execute PowerShell script blocks
        Ok(objects) // Placeholder
    }

    fn group_objects(&self, objects: Vec<PowerShellObject>, args: &[String]) -> Result<Vec<PowerShellObject>> {
        // Object grouping - real implementation would group by specified properties
        Ok(objects) // Placeholder
    }

    fn measure_objects(&self, objects: Vec<PowerShellObject>, args: &[String]) -> Result<Vec<PowerShellObject>> {
        // Object measurement - count, sum, average, etc.
        let count = objects.len();
        Ok(vec![PowerShellObject::Integer(count as i64)])
    }

    // Placeholder implementations for remaining cmdlets
    fn stop_process(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn get_service(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn start_service(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn stop_service(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn get_location(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn set_location(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn copy_item(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn move_item(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn remove_item(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn new_item(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn test_path(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    // get_command implemented below
    fn get_command(&mut self, args: Vec<String>) -> Result<CommandResult> {
        // Return list or specific lookup
        if args.is_empty() {
            let mut objs = Vec::new();
            for (name, info) in &self.cmdlets {
                let mut map = HashMap::new();
                map.insert("Name".to_string(), PowerShellObject::String(name.clone()));
                map.insert("Description".to_string(), PowerShellObject::String(info.description.clone()));
                map.insert("ParameterCount".to_string(), PowerShellObject::Integer(info.parameters.len() as i64));
                objs.push(PowerShellObject::HashTable(map));
            }
            for (alias, target) in &self.aliases {
                let mut map = HashMap::new();
                map.insert("Name".to_string(), PowerShellObject::String(alias.clone()));
                map.insert("AliasTo".to_string(), PowerShellObject::String(target.clone()));
                map.insert("Description".to_string(), PowerShellObject::String(format!("Alias to {}", target)));
                objs.push(PowerShellObject::HashTable(map));
            }
            Ok(CommandResult { success: true, output: String::new(), objects: objs })
        } else {
            let name = &args[0];
            if let Some(info) = self.cmdlets.get(name) {
                let mut map = HashMap::new();
                map.insert("Name".to_string(), PowerShellObject::String(info.name.clone()));
                map.insert("Description".to_string(), PowerShellObject::String(info.description.clone()));
                map.insert("Parameters".to_string(), PowerShellObject::Array(info.parameters.iter().map(|p| PowerShellObject::String(p.name.clone())).collect()));
                return Ok(CommandResult { success: true, output: String::new(), objects: vec![PowerShellObject::HashTable(map)] });
            }
            if let Some(target) = self.aliases.get(name) {
                let mut map = HashMap::new();
                map.insert("Name".to_string(), PowerShellObject::String(name.clone()));
                map.insert("AliasTo".to_string(), PowerShellObject::String(target.clone()));
                return Ok(CommandResult { success: true, output: String::new(), objects: vec![PowerShellObject::HashTable(map)] });
            }
            Ok(CommandResult { success: false, output: format!("{} not found", name), objects: vec![] })
        }
    }
    fn get_help(&mut self, args: Vec<String>) -> Result<CommandResult> {
        if args.is_empty() {
            return Ok(CommandResult { success: true, output: "Usage: Get-Help <Name>".to_string(), objects: vec![] });
        }
        let name = &args[0];
        if let Some(info) = self.cmdlets.get(name) {
            let mut txt = format!("{} - {}\nParameters:\n", info.name, info.description);
            for p in &info.parameters {
                txt.push_str(&format!("  - {} [{}]{}\n", p.name, p.parameter_type, if p.mandatory { " (Mandatory)" } else { "" }));
            }
            return Ok(CommandResult { success: true, output: txt, objects: vec![] });
        }
        if let Some(target) = self.aliases.get(name) {
            return Ok(CommandResult { success: true, output: format!("Alias: {} -> {}", name, target), objects: vec![] });
        }
        Ok(CommandResult { success: false, output: format!("Help not found for {}", name), objects: vec![] })
    }
    fn measure_object(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn sort_object(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn select_object(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn where_object(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn foreach_object(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn group_object(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn out_string(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn out_file(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
    fn read_host(&mut self, _args: Vec<String>) -> Result<CommandResult> { Ok(CommandResult::default()) }
}

// Supporting types and structures

#[derive(Debug, Clone)]
pub struct CmdletInfo {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterInfo>,
}

#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub parameter_type: String,
    pub mandatory: bool,
    pub position: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct PowerShellVariable {
    pub name: String,
    pub value: String,
    pub scope: VariableScope,
    pub data_type: VariableType,
    pub read_only: bool,
}

#[derive(Debug, Clone)]
pub enum VariableScope {
    Global,
    Local,
    Script,
    Private,
}

#[derive(Debug, Clone)]
pub enum VariableType {
    String,
    Integer,
    Array,
    HashTable,
    Object,
}

#[derive(Debug, Clone)]
pub enum ExecutionPolicy {
    Restricted,
    AllSigned,
    RemoteSigned,
    Unrestricted,
    Bypass,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub version: String,
    pub path: String,
    pub cmdlets: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub path: String,
    pub script: String,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub output: String,
    pub objects: Vec<PowerShellObject>,
}

impl Default for CommandResult {
    fn default() -> Self {
        Self {
            success: true,
            output: String::new(),
            objects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PowerShellObject {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<PowerShellObject>),
    HashTable(HashMap<String, PowerShellObject>),
    FileInfo {
        name: String,
        full_path: String,
        size: u64,
        is_directory: bool,
        last_modified: Option<std::time::SystemTime>,
    },
    ProcessInfo {
        name: String,
        id: u32,
        cpu: f64,
        memory: u64,
        status: String,
    },
    Null,
}

impl ToString for PowerShellObject {
    fn to_string(&self) -> String {
        match self {
            PowerShellObject::String(s) => s.clone(),
            PowerShellObject::Integer(i) => i.to_string(),
            PowerShellObject::Float(f) => f.to_string(),
            PowerShellObject::Boolean(b) => b.to_string(),
            PowerShellObject::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|obj| obj.to_string()).collect();
                format!("[{}]", items.join(", "))
            },
            PowerShellObject::HashTable(map) => {
                let items: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{}={}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", items.join("; "))
            },
            PowerShellObject::FileInfo { name, .. } => name.clone(),
            PowerShellObject::ProcessInfo { name, id, .. } => format!("{} ({})", name, id),
            PowerShellObject::Null => String::new(),
        }
    }
}

// Serialization scaffolding for PowerShellObject pipeline (feature gated externally)
impl PowerShellObject {
    pub fn to_json_line(&self) -> String {
        // Minimal manual JSON building to avoid heavy deps; not full escaping
        match self {
            PowerShellObject::String(s) => {
                let esc = s.replace('"', "\\\"");
                format!("{{\"String\":\"{}\"}}", esc)
            },
            PowerShellObject::Integer(i) => format!("{{\"Integer\":{}}}", i),
            PowerShellObject::Float(f) => format!("{{\"Float\":{}}}", f),
            PowerShellObject::Boolean(b) => format!("{{\"Boolean\":{}}}", b),
            PowerShellObject::Null => "{\"Null\":null}".to_string(),
            PowerShellObject::Array(arr) => {
                let parts: Vec<String> = arr.iter().map(|o| o.to_json_line()).collect();
                format!("{{\"Array\":[{}]}}", parts.join(","))
            },
            PowerShellObject::HashTable(map) => {
                let mut parts = Vec::new();
                for (k,v) in map.iter() { parts.push(format!("\"{}\":{}", k, v.to_json_line())); }
                format!("{{\"HashTable\":{{{}}}}}", parts.join(","))
            },
            PowerShellObject::FileInfo { name, full_path, size, is_directory, .. } => {
                format!("{{\"FileInfo\":{{\"name\":\"{}\",\"path\":\"{}\",\"size\":{},\"dir\":{}}}}}", name, full_path, size, is_directory)
            },
            PowerShellObject::ProcessInfo { name, id, cpu, memory, status } => {
                format!("{{\"ProcessInfo\":{{\"name\":\"{}\",\"id\":{},\"cpu\":{},\"mem\":{},\"status\":\"{}\"}}}}", name, id, cpu, memory, status)
            },
        }
    }

    pub fn from_json_line(s: &str) -> Option<Self> {
        // Extremely lightweight parser (assumes structure produced by to_json_line)
        let trimmed = s.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            if trimmed.starts_with("{\"String\":") {
                let inner = trimmed.strip_prefix("{\"String\":\"")?.strip_suffix("\"}")?;
                return Some(PowerShellObject::String(inner.replace("\\\"", "\"")));
            } else if trimmed.starts_with("{\"Integer\":") {
                let num = trimmed.trim_start_matches("{\"Integer\":").trim_end_matches('}');
                if let Ok(v) = num.parse::<i64>() { return Some(PowerShellObject::Integer(v)); }
            } else if trimmed.starts_with("{\"Float\":") {
                let num = trimmed.trim_start_matches("{\"Float\":").trim_end_matches('}');
                if let Ok(v) = num.parse::<f64>() { return Some(PowerShellObject::Float(v)); }
            } else if trimmed.starts_with("{\"Boolean\":") {
                let btxt = trimmed.trim_start_matches("{\"Boolean\":").trim_end_matches('}');
                if btxt == "true" { return Some(PowerShellObject::Boolean(true)); }
                if btxt == "false" { return Some(PowerShellObject::Boolean(false)); }
            } else if trimmed == "{\"Null\":null}" {
                return Some(PowerShellObject::Null);
            }
            // HashTable / Array / FileInfo / ProcessInfo omitted for brevity
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powershell_alias() {
        let ps = PowerShellCompat::new();
        
        // Test that 'ls' alias maps to Get-ChildItem
        assert_eq!(ps.aliases.get("ls"), Some(&"Get-ChildItem".to_string()));
        assert_eq!(ps.aliases.get("dir"), Some(&"Get-ChildItem".to_string()));
        assert_eq!(ps.aliases.get("cat"), Some(&"Get-Content".to_string()));
    }

    #[test]
    fn test_variable_expansion() {
        let mut ps = PowerShellCompat::new();
        
        ps.set_variable("TestVar".to_string(), "Hello World".to_string(), VariableScope::Global).unwrap();
        
        let expanded = ps.expand_variables("Value is: $TestVar");
        assert_eq!(expanded, "Value is: Hello World");
        
        let expanded2 = ps.expand_variables("Value is: ${TestVar}!");
        assert_eq!(expanded2, "Value is: Hello World!");
    }

    #[test]
    fn test_expression_evaluation() {
        let mut ps = PowerShellCompat::new();
        
        let result = ps.evaluate_expression("\"Hello World\"").unwrap();
        assert!(matches!(result, PowerShellObject::String(s) if s == "Hello World"));
        
        let result = ps.evaluate_expression("42").unwrap();
        assert!(matches!(result, PowerShellObject::Integer(42)));
        
        let result = ps.evaluate_expression("$true").unwrap();
        assert!(matches!(result, PowerShellObject::Boolean(true)));
    }

    #[test]
    fn test_cmdlet_registration() {
        let ps = PowerShellCompat::new();
        
        assert!(ps.cmdlets.contains_key("Get-ChildItem"));
        assert!(ps.cmdlets.contains_key("Get-Content"));
        assert!(ps.cmdlets.contains_key("Write-Host"));
    }

    #[test]
    fn test_automatic_variables() {
        let ps = PowerShellCompat::new();
        
        assert!(ps.variables.contains_key("PSVersionTable"));
        assert!(ps.variables.contains_key("PWD"));
        assert!(ps.variables.contains_key("HOME"));
    }

    #[test]
    fn test_get_command_alias_entry() {
        let mut ps = PowerShellCompat::new();
        let res = ps.get_command(vec![]).unwrap();
        // Ensure at least one alias entry present (ls -> Get-ChildItem)
        assert!(res.objects.iter().any(|o| matches!(o, PowerShellObject::HashTable(map) if map.get("Name").map(|v| v.to_string()) == Some("ls".to_string()))));
    }

    #[test]
    fn test_json_line_roundtrip() {
        let objs = vec![
            PowerShellObject::String("hello".into()),
            PowerShellObject::Integer(42),
            PowerShellObject::Boolean(true),
            PowerShellObject::Null,
        ];
        for o in objs.into_iter() {
            let line = o.to_json_line();
            let parsed = PowerShellObject::from_json_line(&line).expect("parse");
            // Compare via string form (simplified equality for subset)
            assert_eq!(o.to_string(), parsed.to_string());
        }
    }
}

// External dependencies
// Replaced 'dirs' crate with standard library - no external dependencies
