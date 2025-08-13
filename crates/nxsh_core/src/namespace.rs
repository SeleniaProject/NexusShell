//! Namespace and module system for NexusShell
//!
//! This module provides Rust-like namespace management with support for:
//! - Module declarations and hierarchical organization
//! - Crate-style `use` import statements with path resolution
//! - Visibility controls (public, private, package-local)
//! - Re-exports and alias imports
//! - Scoped symbol resolution and conflict detection
//! - Dynamic module loading and hot-reloading

use crate::error::{ShellError, ErrorKind, ShellResult, RuntimeErrorKind, IoErrorKind, SystemErrorKind};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};
use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn};

/// Namespace management system with module resolution
pub struct NamespaceSystem {
    /// Configuration for namespace behavior
    config: NamespaceConfig,
    /// Root namespace containing all modules
    root_namespace: Arc<RwLock<Namespace>>,
    /// Module cache for performance
    module_cache: Arc<RwLock<HashMap<String, Module>>>,
    /// Import resolution cache
    import_cache: Arc<RwLock<HashMap<String, ImportResolution>>>,
    /// Statistics for monitoring
    statistics: NamespaceStatistics,
}

/// Configuration for namespace system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceConfig {
    /// Enable automatic module discovery
    pub auto_discovery: bool,
    /// Module search paths
    pub search_paths: Vec<PathBuf>,
    /// Enable module caching
    pub enable_caching: bool,
    /// Enable hot-reloading of modules
    pub hot_reload: bool,
    /// Maximum import depth to prevent cycles
    pub max_import_depth: usize,
    /// Enable strict visibility checking
    pub strict_visibility: bool,
    /// Allow dynamic imports
    pub allow_dynamic_imports: bool,
    /// Module file extensions to recognize
    pub module_extensions: Vec<String>,
}

impl Default for NamespaceConfig {
    fn default() -> Self {
        Self {
            auto_discovery: true,
            search_paths: vec![
                PathBuf::from("modules"),
                PathBuf::from("lib"),
                PathBuf::from("src"),
            ],
            enable_caching: true,
            hot_reload: false,
            max_import_depth: 50,
            strict_visibility: true,
            allow_dynamic_imports: true,
            module_extensions: vec!["nxsh".to_string(), "sh".to_string()],
        }
    }
}

/// Statistics for namespace operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceStatistics {
    /// Total modules loaded
    pub modules_loaded: u64,
    /// Import resolutions performed
    pub imports_resolved: u64,
    /// Symbol lookups performed
    pub symbol_lookups: u64,
    /// Cache hits for modules
    pub cache_hits: u64,
    /// Cache misses for modules
    pub cache_misses: u64,
    /// Import resolution failures
    pub import_failures: u64,
    /// Symbol resolution failures
    pub symbol_failures: u64,
}

impl Default for NamespaceStatistics {
    fn default() -> Self {
        Self {
            modules_loaded: 0,
            imports_resolved: 0,
            symbol_lookups: 0,
            cache_hits: 0,
            cache_misses: 0,
            import_failures: 0,
            symbol_failures: 0,
        }
    }
}

/// A namespace containing modules and symbols
#[derive(Debug, Clone)]
pub struct Namespace {
    /// Namespace name
    pub name: String,
    /// Parent namespace (None for root)
    pub parent: Option<String>,
    /// Child namespaces
    pub children: HashMap<String, Namespace>,
    /// Modules in this namespace
    pub modules: HashMap<String, Module>,
    /// Imported symbols from other namespaces
    pub imports: HashMap<String, ImportedSymbol>,
    /// Re-exported symbols
    pub re_exports: HashMap<String, String>,
    /// Visibility level
    pub visibility: Visibility,
}

/// A module containing functions, variables, and other symbols
#[derive(Debug, Clone)]
pub struct Module {
    /// Module name
    pub name: String,
    /// Module path
    pub path: Option<PathBuf>,
    /// Symbols exported by the module
    pub symbols: HashMap<String, Symbol>,
    /// Private symbols (not exported)
    pub private_symbols: HashMap<String, Symbol>,
    /// Dependencies on other modules
    pub dependencies: Vec<Dependency>,
    /// Import statements in this module
    pub imports: Vec<ImportStatement>,
    /// Module metadata
    pub metadata: ModuleMetadata,
    /// Whether the module is loaded
    pub is_loaded: bool,
}

/// A symbol (function, variable, type, etc.)
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol type
    pub symbol_type: SymbolType,
    /// Visibility level
    pub visibility: Visibility,
    /// Symbol definition
    pub definition: SymbolDefinition,
    /// Documentation string
    pub documentation: Option<String>,
    /// Symbol metadata
    pub metadata: HashMap<String, String>,
}

/// Types of symbols
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Function,
    Variable,
    Constant,
    Type,
    Module,
    Namespace,
    Alias,
    Macro,
}

/// Visibility levels for symbols and modules
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    /// Public - visible to all
    Public,
    /// Package-local - visible within the package
    Package,
    /// Private - visible only within the module
    Private,
    /// Protected - visible to submodules
    Protected,
}

/// Symbol definition containing the actual implementation
#[derive(Debug, Clone)]
pub enum SymbolDefinition {
    Function {
        parameters: Vec<Parameter>,
        return_type: Option<String>,
        body: String, // Could be AST in real implementation
    },
    Variable {
        value_type: Option<String>,
        value: String, // Could be Value in real implementation
        is_mutable: bool,
    },
    Constant {
        value_type: Option<String>,
        value: String,
    },
    Type {
        definition: String,
    },
    Alias {
        target: String,
    },
    Macro {
        parameters: Vec<Parameter>,
        expansion: String,
    },
}

/// Function/macro parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<String>,
    pub default_value: Option<String>,
    pub is_variadic: bool,
}

/// Module dependency
#[derive(Debug, Clone)]
pub struct Dependency {
    /// Module name or path
    pub module: String,
    /// Version requirement
    pub version: Option<String>,
    /// Whether the dependency is optional
    pub optional: bool,
    /// Features to enable in the dependency
    pub features: Vec<String>,
}

/// Import statement in a module
#[derive(Debug, Clone)]
pub struct ImportStatement {
    /// Source module path
    pub source: ModulePath,
    /// What to import
    pub import_type: ImportType,
    /// Optional alias
    pub alias: Option<String>,
    /// Visibility of the import
    pub visibility: Visibility,
}

/// Module path specification
#[derive(Debug, Clone)]
pub enum ModulePath {
    /// Absolute path from root
    Absolute(Vec<String>),
    /// Relative path from current module
    Relative(Vec<String>),
    /// External crate/package
    External(String),
    /// Self reference
    SelfPath,
    /// Super (parent) reference
    Super,
}

/// Types of imports
#[derive(Debug, Clone)]
pub enum ImportType {
    /// Import everything (use module::*)
    Glob,
    /// Import specific symbols
    Named(Vec<ImportedSymbol>),
    /// Import the module itself
    Module,
    /// Import with renaming
    Renamed {
        original: String,
        alias: String,
    },
}

/// An imported symbol with optional alias
#[derive(Debug, Clone)]
pub struct ImportedSymbol {
    /// Original symbol name
    pub name: String,
    /// Alias name (if renamed)
    pub alias: Option<String>,
    /// Source module path
    pub source: String,
    /// Resolved symbol
    pub symbol: Option<Symbol>,
}

/// Module metadata
#[derive(Debug, Clone)]
pub struct ModuleMetadata {
    /// Module version
    pub version: Option<String>,
    /// Author information
    pub author: Option<String>,
    /// Description
    pub description: Option<String>,
    /// License
    pub license: Option<String>,
    /// Keywords/tags
    pub keywords: Vec<String>,
    /// Last modification time
    pub last_modified: Option<std::time::SystemTime>,
    /// File size in bytes
    pub file_size: Option<u64>,
}

/// Import resolution result
#[derive(Debug, Clone)]
pub struct ImportResolution {
    /// Whether the import was successful
    pub success: bool,
    /// Resolved symbols
    pub symbols: HashMap<String, Symbol>,
    /// Error message if failed
    pub error: Option<String>,
    /// Warnings generated during resolution
    pub warnings: Vec<String>,
}

impl NamespaceSystem {
    /// Create a new namespace system
    pub fn new(config: NamespaceConfig) -> Self {
        let root_namespace = Namespace {
            name: "root".to_string(),
            parent: None,
            children: HashMap::new(),
            modules: HashMap::new(),
            imports: HashMap::new(),
            re_exports: HashMap::new(),
            visibility: Visibility::Public,
        };

        Self {
            config,
            root_namespace: Arc::new(RwLock::new(root_namespace)),
            module_cache: Arc::new(RwLock::new(HashMap::new())),
            import_cache: Arc::new(RwLock::new(HashMap::new())),
            statistics: NamespaceStatistics::default(),
        }
    }

    /// Create a new module
    pub fn create_module(&mut self, name: &str, path: Option<PathBuf>) -> ShellResult<()> {
        info!(name = %name, path = ?path, "Creating new module");

        let module = Module {
            name: name.to_string(),
            path: path.clone(),
            symbols: HashMap::new(),
            private_symbols: HashMap::new(),
            dependencies: Vec::new(),
            imports: Vec::new(),
            metadata: ModuleMetadata {
                version: None,
                author: None,
                description: None,
                license: None,
                keywords: Vec::new(),
                last_modified: Some(std::time::SystemTime::now()),
                file_size: None,
            },
            is_loaded: false,
        };

        // Add to cache
        if self.config.enable_caching {
            self.module_cache.write().unwrap().insert(name.to_string(), module.clone());
        }

        // Add to root namespace
        self.root_namespace.write().unwrap().modules.insert(name.to_string(), module);

        self.statistics.modules_loaded += 1;
        debug!(name = %name, "Module created successfully");
        Ok(())
    }

    /// Load a module from file
    pub fn load_module(&mut self, path: &Path) -> ShellResult<String> {
        info!(path = ?path, "Loading module from file");

        let module_name = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::PathNotFound),
                format!("Invalid module path: {:?}", path),
            ))?
            .to_string();

        // Check cache first
        if self.config.enable_caching {
            if let Some(cached_module) = self.module_cache.read().unwrap().get(&module_name) {
                self.statistics.cache_hits += 1;
                debug!(name = %module_name, "Module loaded from cache");
                return Ok(module_name);
            }
        }

        self.statistics.cache_misses += 1;

        // Read module file
        let content = std::fs::read_to_string(path)
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(IoErrorKind::FileReadError),
                format!("Failed to read module file {:?}: {}", path, e),
            ))?;

        // Parse module content (simplified)
        let module = self.parse_module_content(&module_name, &content, Some(path.to_path_buf()))?;

        // Cache the module
        if self.config.enable_caching {
            self.module_cache.write().unwrap().insert(module_name.clone(), module.clone());
        }

        // Add to namespace
        self.root_namespace.write().unwrap().modules.insert(module_name.clone(), module);

        self.statistics.modules_loaded += 1;
        info!(name = %module_name, "Module loaded successfully");
        Ok(module_name)
    }

    /// Parse module content and extract symbols
    fn parse_module_content(&self, name: &str, content: &str, path: Option<PathBuf>) -> ShellResult<Module> {
        debug!(name = %name, content_len = content.len(), "Parsing module content");

        let mut module = Module {
            name: name.to_string(),
            path,
            symbols: HashMap::new(),
            private_symbols: HashMap::new(),
            dependencies: Vec::new(),
            imports: Vec::new(),
            metadata: ModuleMetadata {
                version: None,
                author: None,
                description: None,
                license: None,
                keywords: Vec::new(),
                last_modified: Some(std::time::SystemTime::now()),
                file_size: Some(content.len() as u64),
            },
            is_loaded: true,
        };

        // Simplified parsing - in real implementation, use the AST parser
        for line in content.lines() {
            let line = line.trim();
            
            // Parse function definitions
            if line.starts_with("function ") || line.starts_with("fn ") {
                if let Some(func_name) = self.extract_function_name(line) {
                    let symbol = Symbol {
                        name: func_name.clone(),
                        symbol_type: SymbolType::Function,
                        visibility: if line.starts_with("pub ") { Visibility::Public } else { Visibility::Private },
                        definition: SymbolDefinition::Function {
                            parameters: vec![], // Simplified
                            return_type: None,
                            body: line.to_string(),
                        },
                        documentation: None,
                        metadata: HashMap::new(),
                    };

                    if symbol.visibility == Visibility::Public {
                        module.symbols.insert(func_name, symbol);
                    } else {
                        module.private_symbols.insert(func_name, symbol);
                    }
                }
            }
            
            // Parse variable assignments
            else if line.contains('=') && !line.starts_with('#') {
                if let Some((var_name, value)) = self.extract_variable_assignment(line) {
                    let symbol = Symbol {
                        name: var_name.clone(),
                        symbol_type: SymbolType::Variable,
                        visibility: if line.starts_with("export ") { Visibility::Public } else { Visibility::Private },
                        definition: SymbolDefinition::Variable {
                            value_type: None,
                            value: value.to_string(),
                            is_mutable: true,
                        },
                        documentation: None,
                        metadata: HashMap::new(),
                    };

                    if symbol.visibility == Visibility::Public {
                        module.symbols.insert(var_name, symbol);
                    } else {
                        module.private_symbols.insert(var_name, symbol);
                    }
                }
            }
            
            // Parse import statements
            else if line.starts_with("use ") || line.starts_with("import ") {
                if let Some(import) = self.parse_import_statement(line) {
                    module.imports.push(import);
                }
            }
        }

        debug!(
            name = %name,
            public_symbols = module.symbols.len(),
            private_symbols = module.private_symbols.len(),
            imports = module.imports.len(),
            "Module parsing completed"
        );

        Ok(module)
    }

    /// Extract function name from function definition line
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let line = line.trim_start_matches("pub ").trim_start_matches("function ").trim_start_matches("fn ");
        if let Some(paren_pos) = line.find('(') {
            let name = line[..paren_pos].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
        None
    }

    /// Extract variable assignment
    fn extract_variable_assignment(&self, line: &str) -> Option<(String, String)> {
        let line = line.trim_start_matches("export ").trim_start_matches("local ");
        if let Some(eq_pos) = line.find('=') {
            let name = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();
            if !name.is_empty() {
                return Some((name.to_string(), value.to_string()));
            }
        }
        None
    }

    /// Parse import statement
    fn parse_import_statement(&self, line: &str) -> Option<ImportStatement> {
        let raw = line.trim_start_matches("use ").trim_start_matches("import ").trim();
        if raw.is_empty() { return None; }

        // Support patterns:
        // use net::*
        // use net::http::{Client,Request as Req}
        // use net::http as http
        // use self::foo::bar
        // (current implementation: treat all as Absolute unless starts with self:: or super::)

        // Split alias (" as ") if present (only top-level alias e.g. module as alias)
        let (path_part, alias_part) = if let Some(idx) = raw.find(" as ") {
            (&raw[..idx], Some(raw[idx+4..].trim()))
        } else { (raw, None) };

        // Detect named import block {...}
        if let Some(open_brace) = path_part.find('{') {
            if path_part.ends_with('}') {
                let prefix = path_part[..open_brace].trim_end_matches("::").trim();
                let inner = &path_part[open_brace+1..path_part.len()-1];
                let items: Vec<&str> = inner.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                let mut imported = Vec::new();
                for item in items {
                    if item == "*" { // glob inside braces -> treat as glob of prefix
                        let path_parts: Vec<String> = prefix.split("::").filter(|s|!s.is_empty()).map(|s| s.to_string()).collect();
                        return Some(ImportStatement { source: ModulePath::Absolute(path_parts), import_type: ImportType::Glob, alias: alias_part.map(|a| a.to_string()), visibility: Visibility::Private });
                    }
                    // Handle rename "Name as Alias"
                    let (name, alias) = if let Some(pos) = item.find(" as ") {
                        (item[..pos].trim(), Some(item[pos+4..].trim()))
                    } else { (item, None) };
                    imported.push(ImportedSymbol { name: name.to_string(), alias: alias.map(|a| a.to_string()), source: prefix.to_string(), symbol: None });
                }
                let path_parts: Vec<String> = prefix.split("::").filter(|s|!s.is_empty()).map(|s| s.to_string()).collect();
                return Some(ImportStatement { source: ModulePath::Absolute(path_parts), import_type: ImportType::Named(imported), alias: alias_part.map(|a| a.to_string()), visibility: Visibility::Private });
            }
        }

        // Glob form module::*
        if path_part.ends_with("::*") {
            let base = path_part.trim_end_matches("::*");
            let path_parts: Vec<String> = base.split("::").filter(|s|!s.is_empty()).map(|s| s.to_string()).collect();
            return Some(ImportStatement { source: ModulePath::Absolute(path_parts), import_type: ImportType::Glob, alias: alias_part.map(|a| a.to_string()), visibility: Visibility::Private });
        }

        // Plain module path
        let path_parts: Vec<String> = path_part.split("::").filter(|s|!s.is_empty()).map(|s| s.to_string()).collect();
        if path_parts.is_empty() { return None; }
        Some(ImportStatement { source: ModulePath::Absolute(path_parts), import_type: ImportType::Module, alias: alias_part.map(|a| a.to_string()), visibility: Visibility::Private })
    }

    /// Resolve an import statement
    pub fn resolve_import(&mut self, module_name: &str, import: &ImportStatement) -> ShellResult<ImportResolution> {
        info!(module = %module_name, import = ?import, "Resolving import");
        self.statistics.imports_resolved += 1;

        // Check cache first
        let cache_key = format!("{}::{:?}", module_name, import);
        if self.config.enable_caching {
            if let Some(cached) = self.import_cache.read().unwrap().get(&cache_key) {
                debug!("Import resolution found in cache");
                return Ok(cached.clone());
            }
        }

        let resolution = match &import.source {
            ModulePath::Absolute(path) => {
                self.resolve_absolute_import(path, &import.import_type)?
            },
            ModulePath::Relative(path) => {
                self.resolve_relative_import(module_name, path, &import.import_type)?
            },
            ModulePath::External(crate_name) => {
                self.resolve_external_import(crate_name, &import.import_type)?
            },
            ModulePath::SelfPath => {
                self.resolve_self_import(module_name, &import.import_type)?
            },
            ModulePath::Super => {
                self.resolve_super_import(module_name, &import.import_type)?
            },
        };

        // Cache the resolution
        if self.config.enable_caching {
            self.import_cache.write().unwrap().insert(cache_key, resolution.clone());
        }

        if resolution.success {
            debug!(symbols_count = resolution.symbols.len(), "Import resolved successfully");
        } else {
            self.statistics.import_failures += 1;
            warn!(error = ?resolution.error, "Import resolution failed");
        }

        Ok(resolution)
    }

    /// Resolve absolute import path
    fn resolve_absolute_import(&self, path: &[String], import_type: &ImportType) -> ShellResult<ImportResolution> {
        if path.is_empty() {
            return Ok(ImportResolution {
                success: false,
                symbols: HashMap::new(),
                error: Some("Empty import path".to_string()),
                warnings: vec![],
            });
        }

        let module_name = &path[0];
        let namespace = self.root_namespace.read().unwrap();
        
        if let Some(module) = namespace.modules.get(module_name) {
            match import_type {
                ImportType::Module => {
                    let mut symbols = HashMap::new();
                    // Import all public symbols
                    for (name, symbol) in &module.symbols {
                        if symbol.visibility == Visibility::Public {
                            symbols.insert(name.clone(), symbol.clone());
                        }
                    }
                    Ok(ImportResolution {
                        success: true,
                        symbols,
                        error: None,
                        warnings: vec![],
                    })
                },
                ImportType::Named(imports) => {
                    let mut symbols = HashMap::new();
                    let mut warnings = vec![];
                    
                    for imported in imports {
                        if let Some(symbol) = module.symbols.get(&imported.name) {
                            if symbol.visibility == Visibility::Public || !self.config.strict_visibility {
                                let key = imported.alias.as_ref().unwrap_or(&imported.name);
                                symbols.insert(key.clone(), symbol.clone());
                            } else {
                                warnings.push(format!("Symbol '{}' is not public", imported.name));
                            }
                        } else {
                            warnings.push(format!("Symbol '{}' not found in module '{}'", imported.name, module_name));
                        }
                    }
                    
                    Ok(ImportResolution {
                        success: !symbols.is_empty(),
                        symbols,
                        error: None,
                        warnings,
                    })
                },
                ImportType::Glob => {
                    let mut symbols = HashMap::new();
                    // Import all public symbols
                    for (name, symbol) in &module.symbols {
                        if symbol.visibility == Visibility::Public {
                            symbols.insert(name.clone(), symbol.clone());
                        }
                    }
                    Ok(ImportResolution {
                        success: true,
                        symbols,
                        error: None,
                        warnings: vec![],
                    })
                },
                ImportType::Renamed { original, alias } => {
                    if let Some(symbol) = module.symbols.get(original) {
                        if symbol.visibility == Visibility::Public || !self.config.strict_visibility {
                            let mut symbols = HashMap::new();
                            symbols.insert(alias.clone(), symbol.clone());
                            Ok(ImportResolution {
                                success: true,
                                symbols,
                                error: None,
                                warnings: vec![],
                            })
                        } else {
                            Ok(ImportResolution {
                                success: false,
                                symbols: HashMap::new(),
                                error: Some(format!("Symbol '{}' is not public", original)),
                                warnings: vec![],
                            })
                        }
                    } else {
                        Ok(ImportResolution {
                            success: false,
                            symbols: HashMap::new(),
                            error: Some(format!("Symbol '{}' not found", original)),
                            warnings: vec![],
                        })
                    }
                },
            }
        } else {
            Ok(ImportResolution {
                success: false,
                symbols: HashMap::new(),
                error: Some(format!("Module '{}' not found", module_name)),
                warnings: vec![],
            })
        }
    }

    /// Resolve relative import (simplified)
    fn resolve_relative_import(&self, current_module: &str, path: &[String], import_type: &ImportType) -> ShellResult<ImportResolution> {
        debug!(current = %current_module, path = ?path, "Resolving relative import");
        // For simplicity, treat relative imports as absolute for now
        self.resolve_absolute_import(path, import_type)
    }

    /// Resolve external import (simplified)
    fn resolve_external_import(&self, crate_name: &str, _import_type: &ImportType) -> ShellResult<ImportResolution> {
        debug!(crate_name = %crate_name, "Resolving external import");
        // Placeholder for external crate resolution
        Ok(ImportResolution {
            success: false,
            symbols: HashMap::new(),
            error: Some(format!("External imports not yet supported: {}", crate_name)),
            warnings: vec![],
        })
    }

    /// Resolve self import
    fn resolve_self_import(&self, module_name: &str, import_type: &ImportType) -> ShellResult<ImportResolution> {
        debug!(module = %module_name, "Resolving self import");
        self.resolve_absolute_import(&[module_name.to_string()], import_type)
    }

    /// Resolve super import (simplified)
    fn resolve_super_import(&self, module_name: &str, import_type: &ImportType) -> ShellResult<ImportResolution> {
        debug!(module = %module_name, "Resolving super import");
        // Determine parent by splitting module name on '::' and dropping the last segment.
        let mut parts: Vec<&str> = module_name.split("::").collect();
        if parts.is_empty() { return Ok(ImportResolution { success: false, symbols: HashMap::new(), error: Some("Invalid module path".to_string()), warnings: vec![] }); }
        parts.pop();
        if parts.is_empty() {
            return Ok(ImportResolution { success: false, symbols: HashMap::new(), error: Some("No parent module".to_string()), warnings: vec![] });
        }
        let parent = parts.join("::");
        // Delegate to absolute import using the parent module
        self.resolve_absolute_import(&[parent], import_type)
    }

    /// Lookup a symbol by name
    pub fn lookup_symbol(&mut self, module_name: &str, symbol_name: &str) -> ShellResult<Option<Symbol>> {
        debug!(module = %module_name, symbol = %symbol_name, "Looking up symbol");
        self.statistics.symbol_lookups += 1;

        let namespace = self.root_namespace.read().unwrap();
        
        if let Some(module) = namespace.modules.get(module_name) {
            // Check module's own symbols first
            if let Some(symbol) = module.symbols.get(symbol_name) {
                return Ok(Some(symbol.clone()));
            }
            
            // Check private symbols if not in strict mode
            if !self.config.strict_visibility {
                if let Some(symbol) = module.private_symbols.get(symbol_name) {
                    return Ok(Some(symbol.clone()));
                }
            }
            
            // Check imported symbols
            if let Some(imported) = module.imports.iter().find_map(|import| {
                // Simplified lookup in imports
                None // Would implement proper lookup here
            }) {
                return Ok(Some(imported));
            }
        }

        self.statistics.symbol_failures += 1;
        Ok(None)
    }

    /// List all modules in the namespace
    pub fn list_modules(&self) -> Vec<String> {
        let namespace = self.root_namespace.read().unwrap();
        namespace.modules.keys().cloned().collect()
    }

    /// List all symbols in a module
    pub fn list_symbols(&self, module_name: &str) -> ShellResult<Vec<String>> {
        let namespace = self.root_namespace.read().unwrap();
        
        if let Some(module) = namespace.modules.get(module_name) {
            let mut symbols: Vec<String> = module.symbols.keys().cloned().collect();
            if !self.config.strict_visibility {
                symbols.extend(module.private_symbols.keys().cloned());
            }
            symbols.sort();
            Ok(symbols)
        } else {
            Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::FileNotFound),
                format!("Module '{}' not found", module_name),
            ))
        }
    }

    /// Get module information
    pub fn get_module_info(&self, module_name: &str) -> ShellResult<ModuleMetadata> {
        let namespace = self.root_namespace.read().unwrap();
        
        if let Some(module) = namespace.modules.get(module_name) {
            Ok(module.metadata.clone())
        } else {
            Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::FileNotFound),
                format!("Module '{}' not found", module_name),
            ))
        }
    }

    /// Get current statistics
    pub fn get_statistics(&self) -> &NamespaceStatistics {
        &self.statistics
    }

    /// Clear all caches
    pub fn clear_caches(&mut self) {
        self.module_cache.write().unwrap().clear();
        self.import_cache.write().unwrap().clear();
        info!("All caches cleared");
    }

    /// Reload a module (if hot-reload is enabled)
    pub fn reload_module(&mut self, module_name: &str) -> ShellResult<()> {
        if !self.config.hot_reload {
            return Err(ShellError::new(
                ErrorKind::SystemError(SystemErrorKind::UnsupportedOperation),
                "Hot reload is not enabled".to_string(),
            ));
        }

        info!(module = %module_name, "Reloading module");
        
        // Remove from cache
        self.module_cache.write().unwrap().remove(module_name);
        self.import_cache.write().unwrap().retain(|k, _| !k.starts_with(&format!("{}::", module_name)));
        
        // Find module path and reload
        let path_opt = {
            let namespace = self.root_namespace.read().unwrap();
            if let Some(module) = namespace.modules.get(module_name) {
                module.path.clone()
            } else {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::FileNotFound),
                    format!("Module '{}' not found", module_name),
                ));
            }
        };

        if let Some(path) = path_opt {
            self.load_module(&path)?;
            info!(module = %module_name, "Module reloaded successfully");
        } else {
            return Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                format!("Module '{}' has no associated file path", module_name),
            ));
        }
        
        Ok(())
    }
}

impl Default for NamespaceSystem {
    fn default() -> Self {
        Self::new(NamespaceConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    

    #[test]
    fn test_module_creation() {
        let mut ns = NamespaceSystem::default();
        assert!(ns.create_module("test_module", None).is_ok());
        assert!(ns.list_modules().contains(&"test_module".to_string()));
    }

    #[test]
    fn test_symbol_lookup() {
        let mut ns = NamespaceSystem::default();
        ns.create_module("test_module", None).unwrap();
        
        // In a real test, we would add symbols to the module
        let result = ns.lookup_symbol("test_module", "test_symbol");
        assert!(result.is_ok());
    }

    #[test]
    fn test_import_resolution() {
        let mut ns = NamespaceSystem::default();
        ns.create_module("source_module", None).unwrap();
        
        let import = ImportStatement {
            source: ModulePath::Absolute(vec!["source_module".to_string()]),
            import_type: ImportType::Module,
            alias: None,
            visibility: Visibility::Private,
        };
        
        let result = ns.resolve_import("target_module", &import);
        assert!(result.is_ok());
    }
}
