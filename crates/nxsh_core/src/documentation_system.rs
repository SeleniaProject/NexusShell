use crate::compat::Result;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    fs,
    time::SystemTime,
};
use serde::{Deserialize, Serialize};

/// Comprehensive documentation system for NexusShell
#[derive(Clone)]
pub struct DocumentationSystem {
    doc_generators: HashMap<DocumentationType, std::sync::Arc<dyn DocumentGenerator>>,
    templates: HashMap<String, DocumentTemplate>,
    output_formats: Vec<OutputFormat>,
    configuration: DocConfig,
    index: DocumentationIndex,
    search_index: SearchIndex,
    metadata: DocumentationMetadata,
}

impl std::fmt::Debug for DocumentationSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentationSystem")
            .field("doc_generators", &format!("{} generators", self.doc_generators.len()))
            .field("templates", &self.templates)
            .field("output_formats", &self.output_formats)
            .field("configuration", &self.configuration)
            .field("index", &self.index)
            .field("search_index", &self.search_index)
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl Default for DocumentationSystem {
    fn default() -> Self { Self::new() }
}

impl DocumentationSystem {
    pub fn new() -> Self {
        let mut system = Self {
            doc_generators: HashMap::new(),
            templates: HashMap::new(),
            output_formats: Vec::new(),
            configuration: DocConfig::default(),
            index: DocumentationIndex::new(),
            search_index: SearchIndex::new(),
            metadata: DocumentationMetadata::default(),
        };
        
        system.register_generators();
        system.load_templates();
        system.configure_output_formats();
        system
    }
    // methods continue below

    /// Generate comprehensive API documentation
    pub fn generate_api_documentation(&mut self, source_path: &Path) -> Result<ApiDocumentationReport> {
        let start_time = SystemTime::now();
        
        let mut report = ApiDocumentationReport {
            generation_id: Self::generate_doc_id(),
            source_path: source_path.to_path_buf(),
            start_time,
            end_time: None,
            modules_documented: Vec::new(),
            functions_documented: 0,
            types_documented: 0,
            examples_generated: 0,
            output_files: Vec::new(),
            warnings: Vec::new(),
        };

        // Scan source files
        let rust_files = self.scan_rust_files(source_path)?;
        
        for rust_file in rust_files {
            match self.parse_rust_file(&rust_file) {
                Ok(module_docs) => {
                    report.modules_documented.push(module_docs.module_name.clone());
                    report.functions_documented += module_docs.functions.len();
                    report.types_documented += module_docs.types.len();
                    
                    // Generate documentation for this module
                    let output_files = self.generate_module_documentation(&module_docs)?;
                    report.output_files.extend(output_files);
                    
                    // Add to search index
                    self.index_module_documentation(&module_docs)?;
                },
                Err(e) => {
                    report.warnings.push(format!("Failed to parse {}: {e}", rust_file.display()));
                }
            }
        }

        // Generate API index
        let index_file = self.generate_api_index(&report)?;
        report.output_files.push(index_file);

        // Generate examples
        let examples_generated = self.generate_code_examples(&report)?;
        report.examples_generated = examples_generated;

        report.end_time = Some(SystemTime::now());
        Ok(report)
    }

    /// Generate user documentation
    pub fn generate_user_documentation(&mut self) -> Result<UserDocumentationReport> {
        let start_time = SystemTime::now();
        
        let mut report = UserDocumentationReport {
            generation_id: Self::generate_doc_id(),
            start_time,
            end_time: None,
            guides_generated: Vec::new(),
            tutorials_generated: Vec::new(),
            reference_pages: Vec::new(),
            output_files: Vec::new(),
        };

        // Generate user guides
        let user_guides = vec![
            "getting_started",
            "basic_usage",
            "advanced_features",
            "configuration",
            "troubleshooting",
        ];

        for guide_name in user_guides {
            match self.generate_user_guide(guide_name) {
                Ok(guide_files) => {
                    report.guides_generated.push(guide_name.to_string());
                    report.output_files.extend(guide_files);
                },
                Err(e) => {
                    eprintln!("Failed to generate user guide {guide_name}: {e}");
                }
            }
        }

        // Generate tutorials
        let tutorials = vec![
            "first_shell_script",
            "custom_builtins",
            "performance_tuning",
            "security_configuration",
        ];

        for tutorial_name in tutorials {
            match self.generate_tutorial(tutorial_name) {
                Ok(tutorial_files) => {
                    report.tutorials_generated.push(tutorial_name.to_string());
                    report.output_files.extend(tutorial_files);
                },
                Err(e) => {
                    eprintln!("Failed to generate tutorial {tutorial_name}: {e}");
                }
            }
        }

        // Generate command reference
        let command_ref_files = self.generate_command_reference()?;
    report.reference_pages.extend(command_ref_files.iter().map(ToString::to_string));
    report.output_files.extend(command_ref_files.into_iter().map(PathBuf::from));

        report.end_time = Some(SystemTime::now());
        Ok(report)
    }

    /// Generate developer documentation
    pub fn generate_developer_documentation(&mut self) -> Result<DeveloperDocumentationReport> {
        let start_time = SystemTime::now();
        
        let mut report = DeveloperDocumentationReport {
            generation_id: Self::generate_doc_id(),
            start_time,
            end_time: None,
            architecture_docs: Vec::new(),
            contribution_guides: Vec::new(),
            code_standards: Vec::new(),
            build_instructions: Vec::new(),
            output_files: Vec::new(),
        };

        // Generate architecture documentation
        let arch_docs = self.generate_architecture_documentation()?;
    report.architecture_docs.extend(arch_docs.iter().map(ToString::to_string));
    report.output_files.extend(arch_docs.into_iter().map(PathBuf::from));

        // Generate contribution guidelines
        let contrib_files = self.generate_contribution_guidelines()?;
    report.contribution_guides.extend(contrib_files.iter().map(ToString::to_string));
    report.output_files.extend(contrib_files.into_iter().map(PathBuf::from));

        // Generate coding standards
        let standards_files = self.generate_coding_standards()?;
    report.code_standards.extend(standards_files.iter().map(ToString::to_string));
    report.output_files.extend(standards_files.into_iter().map(PathBuf::from));

        // Generate build and development instructions
        let build_files = self.generate_build_instructions()?;
    report.build_instructions.extend(build_files.iter().map(ToString::to_string));
    report.output_files.extend(build_files.into_iter().map(PathBuf::from));

        report.end_time = Some(SystemTime::now());
        Ok(report)
    }

    /// Generate documentation in multiple formats
    pub fn export_documentation(&self, format: OutputFormat, output_dir: &Path) -> Result<ExportReport> {
        let start_time = SystemTime::now();
        
        let mut report = ExportReport {
            format,
            output_directory: output_dir.to_path_buf(),
            start_time,
            end_time: None,
            files_exported: Vec::new(),
            total_size: 0,
            success: false,
        };

        fs::create_dir_all(output_dir)?;

        match format {
            OutputFormat::Html => {
                self.export_html_documentation(output_dir, &mut report)?;
            },
            OutputFormat::Markdown => {
                self.export_markdown_documentation(output_dir, &mut report)?;
            },
            OutputFormat::Pdf => {
                self.export_pdf_documentation(output_dir, &mut report)?;
            },
            OutputFormat::Json => {
                self.export_json_documentation(output_dir, &mut report)?;
            },
            OutputFormat::EPub => {
                self.export_epub_documentation(output_dir, &mut report)?;
            },
        }

        report.success = !report.files_exported.is_empty();
        report.end_time = Some(SystemTime::now());
        Ok(report)
    }

    /// Search documentation content
    pub fn search_documentation(&self, query: &str) -> Result<Vec<SearchResult>> {
        let results = self.search_index.search(query)?;
        Ok(results.into_iter()
            .map(|item| SearchResult {
                title: item.title,
                content: item.content,
                doc_type: item.doc_type,
                file_path: item.file_path,
                relevance_score: item.score,
            })
            .collect())
    }

    /// Update documentation index
    pub fn update_index(&mut self) -> Result<IndexUpdateReport> {
        let start_time = SystemTime::now();
        
        let mut report = IndexUpdateReport {
            start_time,
            end_time: None,
            documents_indexed: 0,
            documents_updated: 0,
            documents_removed: 0,
            index_size: 0,
        };

        // Rebuild search index
        self.search_index = SearchIndex::new();
        
        // Re-index all documentation
        let doc_files = self.find_all_documentation_files()?;
        
        for doc_file in doc_files {
            match self.index_documentation_file(&doc_file) {
                Ok(_) => report.documents_indexed += 1,
                Err(e) => eprintln!("Failed to index {}: {}", doc_file.display(), e),
            }
        }

        report.index_size = self.search_index.size();
        report.end_time = Some(SystemTime::now());
        Ok(report)
    }

    /// Validate documentation completeness
    pub fn validate_documentation(&self) -> Result<ValidationReport> {
        let mut report = ValidationReport {
            validation_time: SystemTime::now(),
            total_items: 0,
            documented_items: 0,
            missing_documentation: Vec::new(),
            warnings: Vec::new(),
            coverage_percentage: 0.0,
        };

        // Check API documentation coverage
        let api_items = self.get_all_api_items()?;
        report.total_items = api_items.len();

        for item in api_items {
            if self.has_documentation(&item) {
                report.documented_items += 1;
            } else {
                report.missing_documentation.push(item.name.clone());
                
                if item.visibility == ItemVisibility::Public {
                    report.warnings.push(format!("Public {} '{}' lacks documentation", 
                                                item.item_type, item.name));
                }
            }
        }

        report.coverage_percentage = if report.total_items > 0 {
            (report.documented_items as f64 / report.total_items as f64) * 100.0
        } else {
            100.0
        };

        Ok(report)
    }

    // Private implementation methods

    fn register_generators(&mut self) {
        // Register different documentation generators
        self.doc_generators.insert(
            DocumentationType::Api,
            std::sync::Arc::new(ApiDocumentationGenerator::new())
        );
        self.doc_generators.insert(
            DocumentationType::User,
            std::sync::Arc::new(UserDocumentationGenerator::new())
        );
        self.doc_generators.insert(
            DocumentationType::Developer,
            std::sync::Arc::new(DeveloperDocumentationGenerator::new())
        );
    }

    fn load_templates(&mut self) {
        // Load documentation templates
        self.templates.insert("api_module".to_string(), DocumentTemplate {
            name: "API Module".to_string(),
            content: include_str!("../templates/api_module.md").to_string(),
            variables: vec!["module_name".to_string(), "functions".to_string()],
        });

        self.templates.insert("user_guide".to_string(), DocumentTemplate {
            name: "User Guide".to_string(),
            content: include_str!("../templates/user_guide.md").to_string(),
            variables: vec!["title".to_string(), "content".to_string()],
        });
    }

    fn configure_output_formats(&mut self) {
        self.output_formats = vec![
            OutputFormat::Html,
            OutputFormat::Markdown,
            OutputFormat::Pdf,
            OutputFormat::Json,
            OutputFormat::EPub,
        ];
    }

    #[allow(clippy::only_used_in_recursion)]
    fn scan_rust_files(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut rust_files = Vec::new();
        
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() && !path.file_name().unwrap().to_str().unwrap().starts_with('.') {
                    rust_files.extend(self.scan_rust_files(&path)?);
                } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    rust_files.push(path);
                }
            }
        }
        
        Ok(rust_files)
    }

    fn parse_rust_file(&self, file_path: &Path) -> Result<ModuleDocumentation> {
        let content = fs::read_to_string(file_path)?;
        
        // Simplified Rust parsing - in reality would use syn or similar
        let module_name = file_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut module_docs = ModuleDocumentation {
            module_name: module_name.clone(),
            file_path: file_path.to_path_buf(),
            description: String::new(),
            functions: Vec::new(),
            types: Vec::new(),
            constants: Vec::new(),
            examples: Vec::new(),
        };

        // Extract doc comments and function signatures
        for line in content.lines() {
            if line.trim().starts_with("/// ") {
                if module_docs.description.is_empty() {
                    module_docs.description = line.trim_start_matches("/// ").to_string();
                }
            } else if line.trim().starts_with("pub fn ") {
                let func_name = self.extract_function_name(line)?;
                module_docs.functions.push(FunctionDocumentation {
                    name: func_name,
                    signature: line.trim().to_string(),
                    description: String::new(),
                    parameters: Vec::new(),
                    return_type: None,
                    examples: Vec::new(),
                });
            }
        }

        Ok(module_docs)
    }

    fn extract_function_name(&self, line: &str) -> Result<String> {
        let line = line.trim();
        if let Some(start) = line.find("fn ") {
            let after_fn = &line[start + 3..];
            if let Some(end) = after_fn.find('(') {
                return Ok(after_fn[..end].trim().to_string());
            }
        }
    Err(crate::anyhow!("Could not extract function name"))
    }

    fn generate_module_documentation(&self, module_docs: &ModuleDocumentation) -> Result<Vec<PathBuf>> {
        let mut output_files = Vec::new();
        
        // Generate HTML documentation
        let html_content = self.render_module_html(module_docs)?;
    let html_path = PathBuf::from(format!("docs/api/{}.html", module_docs.module_name));
        fs::create_dir_all(html_path.parent().unwrap())?;
        fs::write(&html_path, html_content)?;
        output_files.push(html_path);

        // Generate Markdown documentation
        let md_content = self.render_module_markdown(module_docs)?;
    let md_path = PathBuf::from(format!("docs/api/{}.md", module_docs.module_name));
        fs::write(&md_path, md_content)?;
        output_files.push(md_path);

        Ok(output_files)
    }

    fn render_module_html(&self, module_docs: &ModuleDocumentation) -> Result<String> {
        let mut html = String::from("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str(&format!("<title>{} - NexusShell API</title>\n", module_docs.module_name));
        html.push_str("<style>\nbody { font-family: Arial, sans-serif; margin: 40px; }\n");
        html.push_str("h1 { color: #333; }\nh2 { color: #666; }\n");
        html.push_str(".function { margin: 20px 0; padding: 10px; border-left: 3px solid #007acc; }\n");
        html.push_str("</style>\n</head>\n<body>\n");
        
    html.push_str(&format!("<h1>Module: {}</h1>\n", module_docs.module_name));
    html.push_str(&format!("<p>{}</p>\n", module_docs.description));
        
        html.push_str("<h2>Functions</h2>\n");
        for func in &module_docs.functions {
            html.push_str("<div class=\"function\">\n");
            html.push_str(&format!("<h3>{}</h3>\n", func.name));
            html.push_str(&format!("<code>{}</code>\n", func.signature));
            html.push_str(&format!("<p>{}</p>\n", func.description));
            html.push_str("</div>\n");
        }
        
        html.push_str("</body>\n</html>");
        Ok(html)
    }

    fn render_module_markdown(&self, module_docs: &ModuleDocumentation) -> Result<String> {
        let mut md = String::new();
        
        md.push_str(&format!("# Module: {}\n\n", module_docs.module_name));
        md.push_str(&format!("{}\n\n", module_docs.description));
        
        md.push_str("## Functions\n\n");
        for func in &module_docs.functions {
            md.push_str(&format!("### {}\n\n", func.name));
            md.push_str(&format!("```rust\n{}\n```\n\n", func.signature));
            md.push_str(&format!("{}\n\n", func.description));
        }
        
        Ok(md)
    }

    fn index_module_documentation(&mut self, module_docs: &ModuleDocumentation) -> Result<()> {
        // Add module to search index
        self.search_index.add_document(SearchDocument {
            title: format!("Module: {}", module_docs.module_name),
            content: module_docs.description.clone(),
            doc_type: DocumentationType::Api,
            file_path: module_docs.file_path.clone(),
            score: 1.0,
        });

        // Add functions to search index
        for func in &module_docs.functions {
            self.search_index.add_document(SearchDocument {
                title: format!("Function: {}", func.name),
                content: func.description.clone(),
                doc_type: DocumentationType::Api,
                file_path: module_docs.file_path.clone(),
                score: 1.0,
            });
        }

        Ok(())
    }

    fn generate_api_index(&self, report: &ApiDocumentationReport) -> Result<PathBuf> {
        let index_content = format!(
            "# NexusShell API Documentation Index\n\n\
             Generated: {:?}\n\
             Modules: {}\n\
             Functions: {}\n\
             Types: {}\n\n\
             ## Modules\n\n{}",
            report.start_time,
            report.modules_documented.len(),
            report.functions_documented,
            report.types_documented,
            report.modules_documented.iter()
                .map(|m| format!("- [{m}](api/{m}.html)"))
                .map(|m| format!("- [{m}](api/{m}.html)"))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let index_path = PathBuf::from("docs/api/index.html");
        fs::create_dir_all(index_path.parent().unwrap())?;
        
        // Convert markdown to HTML for index
        let html_content = format!(
            "<!DOCTYPE html>\n<html>\n<head><title>NexusShell API Documentation</title></head>\n\
             <body>\n<pre>{index_content}</pre>\n</body>\n</html>"
        );
        
        fs::write(&index_path, html_content)?;
        Ok(index_path)
    }

    fn generate_code_examples(&self, _report: &ApiDocumentationReport) -> Result<usize> {
        // Generate code examples for common use cases
        let examples = vec![
            ("basic_usage", "Basic shell usage examples"),
            ("advanced_features", "Advanced feature examples"),
            ("error_handling", "Error handling examples"),
        ];

        for (name, description) in examples.clone() {
            let example_content = format!(
                "# {description}\n\n```rust\n// Example code for {name}\nfn main() {{\n    println!(\"Example\");\n}}\n```"
            );
            
            let example_path = PathBuf::from(format!("docs/examples/{name}.md"));
            fs::create_dir_all(example_path.parent().unwrap())?;
            fs::write(example_path, example_content)?;
        }

        Ok(examples.len())
    }

    fn generate_user_guide(&self, guide_name: &str) -> Result<Vec<PathBuf>> {
        let guide_content = match guide_name {
            "getting_started" => self.generate_getting_started_guide()?,
            "basic_usage" => self.generate_basic_usage_guide()?,
            "advanced_features" => self.generate_advanced_features_guide()?,
            "configuration" => self.generate_configuration_guide()?,
            "troubleshooting" => self.generate_troubleshooting_guide()?,
            _ => format!("# {guide_name}\n\nContent for {guide_name} guide."),
        };

    let _guide_path = PathBuf::from(format!("docs/user/{guide_name}.md"));
    let guide_path = PathBuf::from(format!("docs/user/{guide_name}.md"));
        fs::create_dir_all(guide_path.parent().unwrap())?;
        fs::write(&guide_path, guide_content)?;
        
        Ok(vec![guide_path])
    }

    fn generate_getting_started_guide(&self) -> Result<String> {
        Ok(r#"# Getting Started with NexusShell

## Installation

### From Binary Release
Download the latest release from our GitHub releases page.

### From Source
```bash
git clone https://github.com/nexusshell/nexusshell
cd nexusshell
cargo build --release
```

## First Steps

1. Start NexusShell: `nxsh`
2. Try basic commands: `ls`, `cd`, `pwd`
3. Explore builtin help: `help`

## Configuration

NexusShell creates a configuration file at `~/.nxsh_config.toml`.

Example configuration:
```toml
[shell]
prompt = "nxsh> "
history_size = 1000

[security]
enable_audit_logging = true
```
"#.to_string())
    }

    fn generate_basic_usage_guide(&self) -> Result<String> {
        Ok(r#"# Basic Usage Guide

## Command Execution

### Running Commands
```bash
# Basic commands
ls -la
cat file.txt
grep "pattern" file.txt
```

### Pipes and Redirection
```bash
# Pipe output
ls | grep ".txt"

# Redirect output
echo "Hello" > output.txt
cat file.txt >> append.txt
```

## Variables and Environment
```bash
# Set variables
export VAR=value
echo $VAR

# Environment variables
export PATH=$PATH:/new/path
```

## Job Control
```bash
# Background jobs
command &

# Job management
jobs
fg %1
bg %2
```
"#.to_string())
    }

    fn generate_advanced_features_guide(&self) -> Result<String> {
        Ok(r#"# Advanced Features Guide

## Script Language Features

### Functions and Closures
```bash
# Define function
function greet() {
    echo "Hello, $1!"
}

impl Default for DocumentationSystem {
    fn default() -> Self { Self::new() }
}

impl Default for DocumentationSystem {
    fn default() -> Self { Self::new() }
}

# Use closure
map { |x| x * 2 } [1, 2, 3, 4]
```

### Error Handling
```bash
# Try-catch blocks
try {
    risky_command
} catch {
    echo "Command failed"
}
```

### Macros
```bash
# Define macro
macro repeat {
    for i in 1..$1 {
        $2
    }
}

# Use macro
repeat 3 { echo "Hello" }
```

## Performance Features

### Profiling
```bash
# Profile command
profile { complex_operation }

# Benchmark
benchmark { command_to_test }
```

### Optimization
```bash
# Apply performance profile
optimize --profile high_performance

# Monitor resources
monitor --duration 60s
```
"#.to_string())
    }

    fn generate_configuration_guide(&self) -> Result<String> {
        Ok(r#"# Configuration Guide

## Configuration File

NexusShell uses TOML format for configuration.

### Location
- Linux/macOS: `~/.config/nxsh/config.toml`
- Windows: `%APPDATA%\nxsh\config.toml`

## Configuration Sections

### Shell Settings
```toml
[shell]
prompt = "nxsh> "
history_size = 10000
max_completion_items = 100
enable_colors = true
```

### Security Settings
```toml
[security]
enable_audit_logging = true
audit_log_path = "/var/log/nxsh.log"
max_command_length = 4096
restricted_commands = ["rm", "dd"]
```

### Performance Settings
```toml
[performance]
enable_optimization = true
cache_size = "100MB"
parallel_jobs = 4
```

### Network Settings
```toml
[network]
enable_monitoring = true
max_connections = 100
timeout = 30
```
"#.to_string())
    }

    fn generate_troubleshooting_guide(&self) -> Result<String> {
        Ok(r#"# Troubleshooting Guide

## Common Issues

### Shell Won't Start
1. Check permissions: `chmod +x nxsh`
2. Verify dependencies: `ldd nxsh`
3. Check logs: `journalctl -u nxsh`

### Performance Issues
1. Check resource usage: `monitor --resources`
2. Profile slow commands: `profile { slow_command }`
3. Apply optimization: `optimize --auto`

### Configuration Problems
1. Validate config: `nxsh --check-config`
2. Reset to defaults: `nxsh --reset-config`
3. Check syntax: Use TOML validator

## Error Messages

### "Command not found"
- Check PATH: `echo $PATH`
- Install missing program
- Use full path: `/usr/bin/command`

### "Permission denied"
- Check file permissions: `ls -la file`
- Run with sudo if needed: `sudo command`
- Check SELinux: `getenforce`

## Getting Help

1. Built-in help: `help <command>`
2. Manual pages: `man nxsh`
3. Community forum: https://forum.nexusshell.org
4. GitHub issues: https://github.com/nexusshell/nexusshell/issues
"#.to_string())
    }

    fn generate_tutorial(&self, tutorial_name: &str) -> Result<Vec<PathBuf>> {
        let tutorial_content = match tutorial_name {
            "first_shell_script" => self.generate_first_script_tutorial()?,
            "custom_builtins" => self.generate_custom_builtins_tutorial()?,
            "performance_tuning" => self.generate_performance_tuning_tutorial()?,
            "security_configuration" => self.generate_security_config_tutorial()?,
            _ => format!("# Tutorial: {tutorial_name}\n\nTutorial content for {tutorial_name}."),
        };

    let _tutorial_path = PathBuf::from(format!("docs/tutorials/{tutorial_name}.md"));
    let tutorial_path = PathBuf::from(format!("docs/tutorials/{tutorial_name}.md"));
        fs::create_dir_all(tutorial_path.parent().unwrap())?;
        fs::write(&tutorial_path, tutorial_content)?;
        
        Ok(vec![tutorial_path])
    }

    fn generate_first_script_tutorial(&self) -> Result<String> {
        Ok(r#"# Your First NexusShell Script

## Step 1: Create Script File
Create a file called `hello.nxsh`:

```bash
#!/usr/bin/env nxsh

# This is a comment
echo "Hello, NexusShell!"

# Variables
name = "World"
echo "Hello, $name!"

# Function
function greet(person) {
    echo "Greetings, $person!"
}

greet("Developer")
```

## Step 2: Make Executable
```bash
chmod +x hello.nxsh
```

## Step 3: Run Script
```bash
./hello.nxsh
```

## Expected Output
```
Hello, NexusShell!
Hello, World!
Greetings, Developer!
```

## Next Steps
- Try adding loops: `for i in 1..10 { echo $i }`
- Add error handling: `try { ... } catch { ... }`
- Use closures: `map { |x| x * 2 } [1, 2, 3]`
"#.to_string())
    }

    fn generate_custom_builtins_tutorial(&self) -> Result<String> {
        Ok(r#"# Creating Custom Builtin Commands

## Overview
NexusShell allows you to extend functionality with custom builtin commands.

## Step 1: Create Builtin Module
Create `my_builtin.rs`:

```rust
use nxsh_core::{Builtin, ExecutionResult};

pub struct MyBuiltin;

impl Builtin for MyBuiltin {
    fn name(&self) -> &str {
        "mycommand"
    }
    
    fn execute(&self, args: Vec<String>) -> ExecutionResult {
        println!("My custom command: {:?}", args);
        ExecutionResult::success()
    }
}
```

## Step 2: Register Builtin
Add to your shell configuration:

```toml
[builtins]
my_builtin = "path/to/my_builtin.so"
```

## Step 3: Use Custom Builtin
```bash
mycommand arg1 arg2
```

## Advanced Features
- Add help text
- Implement command completion
- Handle errors gracefully
- Support options and flags
"#.to_string())
    }

    fn generate_performance_tuning_tutorial(&self) -> Result<String> {
        Ok(r#"# Performance Tuning Tutorial

## Baseline Measurement
```bash
# Profile your shell usage
profile --duration 1h

# Benchmark specific commands
benchmark { find /home -name "*.txt" }
```

## Apply Optimizations

### CPU Optimization
```bash
# Set high-performance profile
optimize --profile performance

# Tune CPU settings
optimize --resource cpu --target maximize
```

### Memory Optimization
```bash
# Reduce memory usage
optimize --resource memory --target minimize

# Clear caches
optimize --clear-caches
```

### I/O Optimization
```bash
# Optimize disk access
optimize --resource disk --target balance

# Enable caching
optimize --enable-cache
```

## Monitor Results
```bash
# Continuous monitoring
monitor --resources --duration 30m

# Generate performance report
report --type performance --output results.html
```

## Advanced Tuning
- Custom optimization profiles
- Workload-specific tuning
- Automated optimization rules
"#.to_string())
    }

    fn generate_security_config_tutorial(&self) -> Result<String> {
        Ok(r#"# Security Configuration Tutorial

## Enable Security Features
Add to your config file:

```toml
[security]
enable_audit_logging = true
audit_log_path = "/var/log/nxsh-audit.log"
enable_network_monitoring = true
restricted_commands = ["rm", "dd", "mkfs"]
```

## Set Up Security Auditing
```bash
# Run security audit
audit --scope filesystem,network,configuration

# Check compliance
audit --framework CIS

# Generate security report
audit --export-format pdf --output security_report.pdf
```

## Configure Access Controls
```toml
[access_control]
enable_rbac = true
default_policy = "deny"

[[access_control.rules]]
user = "admin"
commands = ["*"]
resources = ["*"]

[[access_control.rules]]
user = "user"
commands = ["ls", "cat", "grep"]
resources = ["/home/$USER/*"]
```

## Network Security
```bash
# Enable network monitoring
network-monitor --enable

# Set network policies
network-policy --restrict-outbound
network-policy --allow-host "trusted.example.com"
```

## Hardening Guide
```bash
# Generate hardening recommendations
harden --generate-guide

# Apply security hardening
harden --apply-recommendations
```
"#.to_string())
    }

    fn generate_command_reference(&self) -> Result<Vec<String>> {
        // Generate command reference documentation
        let commands = [
            ("ls", "List directory contents"),
            ("cd", "Change directory"),
            ("pwd", "Print working directory"),
            ("echo", "Display text"),
            ("cat", "Display file contents"),
            ("grep", "Search text patterns"),
            // Add more commands...
        ];

        let reference_content = commands.iter()
            .map(|(cmd, desc)| format!("## {cmd}\n\n{desc}\n\n"))
            .collect::<String>();

        let content = format!("# Command Reference\n\n{reference_content}");
        
        let ref_path = PathBuf::from("docs/reference/commands.md");
        fs::create_dir_all(ref_path.parent().unwrap())?;
        fs::write(&ref_path, content)?;
        
        Ok(vec!["commands".to_string()])
    }

    fn generate_architecture_documentation(&self) -> Result<Vec<String>> {
        let arch_content = r#"# NexusShell Architecture

## Overview
NexusShell is built with a modular, layered architecture designed for performance, security, and extensibility.

## Core Components

### nxsh_core
The core library containing fundamental shell operations:
- Command parsing and execution
- Job management
- I/O handling
- Memory management

### nxsh_parser
Advanced parsing engine with support for:
- Complex command syntax
- Script language features
- Macro expansion

### nxsh_hal
Hardware Abstraction Layer providing:
- Cross-platform compatibility
- System resource access
- Network operations

## Module Dependencies
```
nxsh_cli -> nxsh_core -> nxsh_parser
     |          |            |
     v          v            v
nxsh_ui    nxsh_hal    nxsh_builtins
```

## Design Principles
1. **Performance**: Zero-cost abstractions, efficient memory usage
2. **Security**: Memory safety, input validation, audit logging
3. **Extensibility**: Plugin architecture, custom builtins
4. **Compatibility**: POSIX compliance, cross-platform support
"#;

        let arch_path = PathBuf::from("docs/architecture/overview.md");
        fs::create_dir_all(arch_path.parent().unwrap())?;
        fs::write(arch_path, arch_content)?;
        
        Ok(vec!["overview".to_string()])
    }

    fn generate_contribution_guidelines(&self) -> Result<Vec<String>> {
        let contrib_content = r#"# Contributing to NexusShell

## Getting Started
1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/nexusshell`
3. Create a feature branch: `git checkout -b feature-name`

## Development Setup
```bash
# Install dependencies
cargo install cargo-watch cargo-tarpaulin

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy
```

## Contribution Process
1. Write tests for new features
2. Ensure all tests pass
3. Follow coding standards
4. Update documentation
5. Submit pull request

## Code Review
- All changes require review
- Address reviewer feedback
- Ensure CI passes
- Maintain test coverage > 80%

## Communication
- Use GitHub issues for bugs
- Use discussions for questions
- Join our Discord community
"#;

        let contrib_path = PathBuf::from("docs/contributing/guidelines.md");
        fs::create_dir_all(contrib_path.parent().unwrap())?;
        fs::write(contrib_path, contrib_content)?;
        
        Ok(vec!["guidelines".to_string()])
    }

    fn generate_coding_standards(&self) -> Result<Vec<String>> {
        let standards_content = r#"# Coding Standards

## Rust Style Guide
Follow the official Rust style guide with these additions:

### Naming Conventions
- Functions: `snake_case`
- Types: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### Documentation
- All public items must have doc comments
- Include examples for complex functions
- Document error conditions
- Use `#[doc(hidden)]` for internal APIs

### Error Handling
- Use `Result<T, E>` for fallible operations
- Define custom error types
- Provide helpful error messages
- Use `anyhow` for application errors

### Testing
- Unit tests in same file as implementation
- Integration tests in `tests/` directory
- Use descriptive test names
- Include edge cases and error conditions

### Performance
- Profile before optimizing
- Prefer zero-cost abstractions
- Use `#[inline]` judiciously
- Document performance characteristics

## Git Conventions
- Use conventional commit messages
- Keep commits focused and atomic
- Include issue numbers in commit messages
- Write descriptive commit messages
"#;

        let standards_path = PathBuf::from("docs/contributing/coding_standards.md");
        fs::create_dir_all(standards_path.parent().unwrap())?;
        fs::write(standards_path, standards_content)?;
        
        Ok(vec!["coding_standards".to_string()])
    }

    fn generate_build_instructions(&self) -> Result<Vec<String>> {
        let build_content = r#"# Build Instructions

## Prerequisites
- Rust 1.70 or later
- Git
- Platform-specific build tools

### Linux/macOS
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install build dependencies (Ubuntu/Debian)
sudo apt-get install build-essential pkg-config libssl-dev

# Install build dependencies (macOS)
xcode-select --install
```

### Windows
```powershell
# Install Rust
# Download and run rustup-init.exe from https://rustup.rs/

# Install Visual Studio Build Tools
# Download from https://visualstudio.microsoft.com/build-tools/
```

## Building
```bash
# Clone repository
git clone https://github.com/nexusshell/nexusshell
cd nexusshell

# Build debug version
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Build documentation
cargo doc --open
```

## Development Workflow
```bash
# Watch for changes and rebuild
cargo watch -x build

# Run with live reloading
cargo watch -x run

# Run tests on changes
cargo watch -x test
```

## Cross Compilation
```bash
# Add target
rustup target add x86_64-pc-windows-gnu

# Build for target
cargo build --target x86_64-pc-windows-gnu --release
```

## Troubleshooting
- Link errors: Check system dependencies
- Compilation errors: Update Rust version
- Test failures: Check environment setup
"#;

        let build_path = PathBuf::from("docs/development/build_instructions.md");
        fs::create_dir_all(build_path.parent().unwrap())?;
        fs::write(build_path, build_content)?;
        
        Ok(vec!["build_instructions".to_string()])
    }

    fn export_html_documentation(&self, output_dir: &Path, report: &mut ExportReport) -> Result<()> {
        // Copy generated HTML files to output directory
        let html_files = self.find_files_with_extension("docs", "html")?;
        
        for html_file in html_files {
            let relative_path = html_file.strip_prefix("docs")?;
            let output_path = output_dir.join(relative_path);
            
            fs::create_dir_all(output_path.parent().unwrap())?;
            fs::copy(&html_file, &output_path)?;
            
            report.files_exported.push(output_path);
            report.total_size += fs::metadata(&html_file)?.len();
        }
        
        Ok(())
    }

    fn export_markdown_documentation(&self, output_dir: &Path, report: &mut ExportReport) -> Result<()> {
        let md_files = self.find_files_with_extension("docs", "md")?;
        
        for md_file in md_files {
            let relative_path = md_file.strip_prefix("docs")?;
            let output_path = output_dir.join(relative_path);
            
            fs::create_dir_all(output_path.parent().unwrap())?;
            fs::copy(&md_file, &output_path)?;
            
            report.files_exported.push(output_path);
            report.total_size += fs::metadata(&md_file)?.len();
        }
        
        Ok(())
    }

    fn export_pdf_documentation(&self, output_dir: &Path, report: &mut ExportReport) -> Result<()> {
        // Simplified PDF export - in reality would use a PDF generation library
        let pdf_content = "PDF Documentation Content".as_bytes();
        let pdf_path = output_dir.join("nexusshell_documentation.pdf");
        
        fs::write(&pdf_path, pdf_content)?;
        report.files_exported.push(pdf_path);
        report.total_size += pdf_content.len() as u64;
        
        Ok(())
    }

    fn export_json_documentation(&self, output_dir: &Path, report: &mut ExportReport) -> Result<()> {
        let json_data = serde_json::json!({
            "documentation": {
                "version": "1.0.0",
                "generated": SystemTime::now(),
                "modules": []
            }
        });
        
        let json_content = serde_json::to_string_pretty(&json_data)?;
        let json_path = output_dir.join("documentation.json");
        
        fs::write(&json_path, &json_content)?;
        report.files_exported.push(json_path);
        report.total_size += json_content.len() as u64;
        
        Ok(())
    }

    fn export_epub_documentation(&self, output_dir: &Path, report: &mut ExportReport) -> Result<()> {
        // Simplified ePub export - in reality would use an ePub generation library
        let epub_content = "ePub Documentation Content".as_bytes();
        let epub_path = output_dir.join("nexusshell_documentation.epub");
        
        fs::write(&epub_path, epub_content)?;
        report.files_exported.push(epub_path);
        report.total_size += epub_content.len() as u64;
        
        Ok(())
    }

    #[allow(clippy::only_used_in_recursion)]
    fn find_files_with_extension(&self, dir: &str, extension: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let path = Path::new(dir);
        
        if path.exists() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    files.extend(self.find_files_with_extension(
                        path.to_str().unwrap(), 
                        extension
                    )?);
                } else if path.extension().and_then(|s| s.to_str()) == Some(extension) {
                    files.push(path);
                }
            }
        }
        
        Ok(files)
    }

    fn find_all_documentation_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        files.extend(self.find_files_with_extension("docs", "md")?);
        files.extend(self.find_files_with_extension("docs", "html")?);
        Ok(files)
    }

    fn index_documentation_file(&mut self, file_path: &Path) -> Result<()> {
        let content = fs::read_to_string(file_path)?;
        let title = self.extract_title_from_content(&content);
        
        self.search_index.add_document(SearchDocument {
            title,
            content,
            doc_type: DocumentationType::User,
            file_path: file_path.to_path_buf(),
            score: 1.0,
        });
        
        Ok(())
    }

    fn extract_title_from_content(&self, content: &str) -> String {
        for line in content.lines() {
            if let Some(stripped) = line.strip_prefix("# ") {
                return stripped.to_string();
            }
        }
        "Untitled".to_string()
    }

    fn get_all_api_items(&self) -> Result<Vec<ApiItem>> {
        // Simplified API item discovery
        Ok(vec![
            ApiItem {
                name: "example_function".to_string(),
                item_type: "function".to_string(),
                visibility: ItemVisibility::Public,
            }
        ])
    }

    fn has_documentation(&self, item: &ApiItem) -> bool {
        // Simplified documentation check
        !item.name.starts_with("_") // Private items typically start with underscore
    }

    fn generate_doc_id() -> String {
        format!("DOC_{}", 
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs())
    }
}

// Supporting types and structures

#[derive(Debug, Clone)]
pub struct DocumentTemplate {
    pub name: String,
    pub content: String,
    pub variables: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DocConfig {
    pub output_directory: PathBuf,
    pub include_private: bool,
    pub generate_examples: bool,
    pub theme: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DocumentationIndex {
    entries: HashMap<String, IndexEntry>,
}

impl DocumentationIndex {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

impl Default for DocumentationIndex {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub path: PathBuf,
    pub title: String,
    pub doc_type: DocumentationType,
    pub last_modified: SystemTime,
}

#[derive(Debug, Clone)]
pub struct SearchIndex {
    documents: Vec<SearchDocument>,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }

    pub fn add_document(&mut self, document: SearchDocument) {
        self.documents.push(document);
    }

    pub fn search(&self, query: &str) -> Result<Vec<SearchDocument>> {
        let query_lower = query.to_lowercase();
        
        let mut results: Vec<_> = self.documents.iter()
            .filter(|doc| {
                doc.title.to_lowercase().contains(&query_lower) ||
                doc.content.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();
        
        // Simple scoring based on title vs content matches
        for doc in &mut results {
            if doc.title.to_lowercase().contains(&query_lower) {
                doc.score = 2.0;
            } else {
                doc.score = 1.0;
            }
        }
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(results)
    }

    pub fn size(&self) -> usize {
        self.documents.len()
    }
}

impl Default for SearchIndex {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone)]
pub struct SearchDocument {
    pub title: String,
    pub content: String,
    pub doc_type: DocumentationType,
    pub file_path: PathBuf,
    pub score: f64,
}

#[derive(Debug, Clone, Default)]
pub struct DocumentationMetadata {
    pub version: String,
    pub generated_at: Option<SystemTime>,
    pub total_pages: usize,
    pub last_updated: Option<SystemTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentationType {
    Api,
    User,
    Developer,
    Tutorial,
    Reference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Html,
    Markdown,
    Pdf,
    Json,
    EPub,
}

// Report structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDocumentationReport {
    pub generation_id: String,
    pub source_path: PathBuf,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub modules_documented: Vec<String>,
    pub functions_documented: usize,
    pub types_documented: usize,
    pub examples_generated: usize,
    pub output_files: Vec<PathBuf>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDocumentationReport {
    pub generation_id: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub guides_generated: Vec<String>,
    pub tutorials_generated: Vec<String>,
    pub reference_pages: Vec<String>,
    pub output_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperDocumentationReport {
    pub generation_id: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub architecture_docs: Vec<String>,
    pub contribution_guides: Vec<String>,
    pub code_standards: Vec<String>,
    pub build_instructions: Vec<String>,
    pub output_files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ExportReport {
    pub format: OutputFormat,
    pub output_directory: PathBuf,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub files_exported: Vec<PathBuf>,
    pub total_size: u64,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub content: String,
    pub doc_type: DocumentationType,
    pub file_path: PathBuf,
    pub relevance_score: f64,
}

#[derive(Debug, Clone)]
pub struct IndexUpdateReport {
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub documents_indexed: usize,
    pub documents_updated: usize,
    pub documents_removed: usize,
    pub index_size: usize,
}

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub validation_time: SystemTime,
    pub total_items: usize,
    pub documented_items: usize,
    pub missing_documentation: Vec<String>,
    pub warnings: Vec<String>,
    pub coverage_percentage: f64,
}

// Documentation data structures

#[derive(Debug, Clone)]
pub struct ModuleDocumentation {
    pub module_name: String,
    pub file_path: PathBuf,
    pub description: String,
    pub functions: Vec<FunctionDocumentation>,
    pub types: Vec<TypeDocumentation>,
    pub constants: Vec<ConstantDocumentation>,
    pub examples: Vec<ExampleDocumentation>,
}

#[derive(Debug, Clone)]
pub struct FunctionDocumentation {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub parameters: Vec<ParameterDocumentation>,
    pub return_type: Option<String>,
    pub examples: Vec<ExampleDocumentation>,
}

#[derive(Debug, Clone)]
pub struct TypeDocumentation {
    pub name: String,
    pub type_kind: String, // struct, enum, trait, etc.
    pub description: String,
    pub fields: Vec<FieldDocumentation>,
    pub methods: Vec<FunctionDocumentation>,
}

#[derive(Debug, Clone)]
pub struct ConstantDocumentation {
    pub name: String,
    pub value: String,
    pub description: String,
    pub type_info: String,
}

#[derive(Debug, Clone)]
pub struct ParameterDocumentation {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub optional: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FieldDocumentation {
    pub name: String,
    pub field_type: String,
    pub description: String,
    pub visibility: ItemVisibility,
}

#[derive(Debug, Clone)]
pub struct ExampleDocumentation {
    pub title: String,
    pub code: String,
    pub description: String,
    pub expected_output: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApiItem {
    pub name: String,
    pub item_type: String,
    pub visibility: ItemVisibility,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemVisibility {
    Public,
    Private,
    Crate,
}

// Document generator trait and implementations

pub trait DocumentGenerator: Send + Sync {
    fn generate(&self, input: &Path) -> Result<Vec<PathBuf>>;
    fn supported_formats(&self) -> Vec<OutputFormat>;
}

pub struct ApiDocumentationGenerator;

impl ApiDocumentationGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ApiDocumentationGenerator {
    fn default() -> Self { Self::new() }
}

impl DocumentGenerator for ApiDocumentationGenerator {
    fn generate(&self, _input: &Path) -> Result<Vec<PathBuf>> {
        // Implementation for API documentation generation
        Ok(vec![])
    }

    fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![OutputFormat::Html, OutputFormat::Markdown, OutputFormat::Json]
    }
}

pub struct UserDocumentationGenerator;

impl UserDocumentationGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UserDocumentationGenerator {
    fn default() -> Self { Self::new() }
}

impl DocumentGenerator for UserDocumentationGenerator {
    fn generate(&self, _input: &Path) -> Result<Vec<PathBuf>> {
        // Implementation for user documentation generation
        Ok(vec![])
    }

    fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![OutputFormat::Html, OutputFormat::Markdown, OutputFormat::Pdf]
    }
}

pub struct DeveloperDocumentationGenerator;

impl DeveloperDocumentationGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeveloperDocumentationGenerator {
    fn default() -> Self { Self::new() }
}

impl DocumentGenerator for DeveloperDocumentationGenerator {
    fn generate(&self, _input: &Path) -> Result<Vec<PathBuf>> {
        // Implementation for developer documentation generation
        Ok(vec![])
    }

    fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![OutputFormat::Html, OutputFormat::Markdown]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_documentation_system_creation() {
        let _doc_system = DocumentationSystem::new();
        // Test passes if constructor doesn't panic
    }

    #[test]
    fn test_search_index() {
        let mut index = SearchIndex::new();
        
        index.add_document(SearchDocument {
            title: "Test Document".to_string(),
            content: "This is test content".to_string(),
            doc_type: DocumentationType::User,
            file_path: PathBuf::from("test.md"),
            score: 1.0,
        });

        let results = index.search("test").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Document");
    }

    #[test]
    fn test_documentation_metadata() {
        let metadata = DocumentationMetadata::default();
        assert_eq!(metadata.version, "");
        assert_eq!(metadata.total_pages, 0);
    }

    #[test]
    fn test_api_item() {
        let item = ApiItem {
            name: "test_function".to_string(),
            item_type: "function".to_string(),
            visibility: ItemVisibility::Public,
        };
        
        assert_eq!(item.name, "test_function");
        assert_eq!(item.visibility, ItemVisibility::Public);
    }

    #[test]
    fn test_output_formats() {
        let formats = [
            OutputFormat::Html,
            OutputFormat::Markdown,
            OutputFormat::Pdf,
            OutputFormat::Json,
            OutputFormat::EPub,
        ];
        
        assert_eq!(formats.len(), 5);
    }
}
