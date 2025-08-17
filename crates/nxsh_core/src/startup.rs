//! Startup optimization system for NexusShell
//! 
//! This module provides startup time optimization targeting ≤ 5ms startup.
//! Key strategies:
//! - Lazy initialization of heavy components
//! - Minimal dependency loading
//! - Fast path execution for simple commands
//! - Cached configuration loading

use std::{
    sync::OnceLock,
    time::Instant,
    collections::HashMap,
};
use serde::{Deserialize, Serialize};

/// Startup configuration for performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    /// Enable lazy loading of modules
    pub lazy_loading: bool,
    /// Enable configuration caching
    pub cache_config: bool,
    /// Enable plugin system lazy loading
    pub lazy_plugins: bool,
    /// Enable history lazy loading
    pub lazy_history: bool,
    /// Maximum startup time target in milliseconds
    pub max_startup_ms: u64,
    /// Enable startup performance tracking
    pub track_performance: bool,
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            lazy_loading: true,
            cache_config: true,
            lazy_plugins: true,
            lazy_history: true,
            max_startup_ms: 5, // ≤ 5ms target
            track_performance: cfg!(debug_assertions),
        }
    }
}

/// Startup timing tracker
#[derive(Debug)]
pub struct StartupTimer {
    start_time: Instant,
    checkpoints: HashMap<String, Instant>,
    config: StartupConfig,
}

impl StartupTimer {
    /// Create a new startup timer
    pub fn new(config: StartupConfig) -> Self {
        Self {
            start_time: Instant::now(),
            checkpoints: HashMap::new(),
            config,
        }
    }

    /// Record a checkpoint
    pub fn checkpoint(&mut self, name: impl Into<String>) {
        if self.config.track_performance {
            self.checkpoints.insert(name.into(), Instant::now());
        }
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Get elapsed time for a specific checkpoint
    pub fn elapsed_since_checkpoint(&self, name: &str) -> Option<std::time::Duration> {
        self.checkpoints.get(name).map(|time| time.elapsed())
    }

    /// Check if we're within the startup time target
    pub fn within_target(&self) -> bool {
        self.elapsed().as_millis() <= self.config.max_startup_ms as u128
    }

    /// Generate a performance report
    pub fn report(&self) -> StartupReport {
        let total_ms = self.elapsed().as_millis();
        let target_ms = self.config.max_startup_ms as u128;
        
        let checkpoints = self.checkpoints.iter().map(|(name, time)| {
            (name.clone(), self.start_time.elapsed().as_millis() - time.elapsed().as_millis())
        }).collect();

        StartupReport {
            total_ms,
            target_ms,
            within_target: total_ms <= target_ms,
            checkpoints,
        }
    }
}

/// Startup performance report
#[derive(Debug, Serialize, Deserialize)]
pub struct StartupReport {
    pub total_ms: u128,
    pub target_ms: u128,
    pub within_target: bool,
    pub checkpoints: HashMap<String, u128>,
}

impl StartupReport {
    pub fn print_summary(&self) {
        let status = if self.within_target { "✅" } else { "❌" };
        println!("{} Startup: {}ms (target: ≤{}ms)", 
                status, self.total_ms, self.target_ms);
        
        if !self.checkpoints.is_empty() {
            println!("Checkpoints:");
            for (name, time) in &self.checkpoints {
                println!("  {name}: {time}ms");
            }
        }
    }
}

/// Lazy initialization system
pub struct LazyInitializer {
    config: StartupConfig,
    initialized_components: HashMap<String, bool>,
}

impl LazyInitializer {
    /// Create a new lazy initializer
    pub fn new(config: StartupConfig) -> Self {
        Self {
            config,
            initialized_components: HashMap::new(),
        }
    }

    /// Check if a component is initialized
    pub fn is_initialized(&self, component: &str) -> bool {
        self.initialized_components.get(component).copied().unwrap_or(false)
    }

    /// Mark a component as initialized
    pub fn mark_initialized(&mut self, component: impl Into<String>) {
        self.initialized_components.insert(component.into(), true);
    }

    /// Initialize component only if needed
    pub fn initialize_if_needed<F, R>(&mut self, component: &str, initializer: F) -> Option<R>
    where
        F: FnOnce() -> R,
    {
        // Initialize when either not yet initialized under lazy mode, or when lazy loading is disabled.
    if !self.is_initialized(component) || !self.config.lazy_loading {
            let result = initializer();
            self.mark_initialized(component);
            return Some(result);
        }
        None
    }
}

/// Global startup optimization state
static STARTUP_OPTIMIZER: OnceLock<StartupOptimizer> = OnceLock::new();

/// Main startup optimization system
#[allow(dead_code)]
pub struct StartupOptimizer {
    config: StartupConfig,
    timer: StartupTimer,
    lazy_init: LazyInitializer,
}

impl StartupOptimizer {
    /// Initialize the global startup optimizer
    pub fn init(config: StartupConfig) -> &'static Self {
        STARTUP_OPTIMIZER.get_or_init(|| {
            let timer = StartupTimer::new(config.clone());
            let lazy_init = LazyInitializer::new(config.clone());
            
            Self {
                config,
                timer,
                lazy_init,
            }
        })
    }

    /// Get the global startup optimizer instance
    pub fn instance() -> Option<&'static Self> {
        STARTUP_OPTIMIZER.get()
    }

    /// Record a checkpoint
    pub fn checkpoint(&self, name: impl Into<String>) {
        // Safety: This is safe because we only mutate timer in single-threaded startup phase
        unsafe {
            let timer_ptr = &self.timer as *const StartupTimer as *mut StartupTimer;
            (*timer_ptr).checkpoint(name);
        }
    }

    /// Check if we should use fast path for simple operations
    pub fn should_use_fast_path(&self) -> bool {
        self.config.lazy_loading && !self.timer.within_target()
    }

    /// Get the startup configuration
    pub fn config(&self) -> &StartupConfig {
        &self.config
    }

    /// Get the startup timer
    pub fn timer(&self) -> &StartupTimer {
        &self.timer
    }
}

/// Optimized module loading with lazy initialization
pub struct OptimizedModuleLoader {
    modules: HashMap<String, Box<dyn ModuleInitializer>>,
    loaded: HashMap<String, bool>,
}

impl OptimizedModuleLoader {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            loaded: HashMap::new(),
        }
    }

    pub fn register_module(&mut self, name: impl Into<String>, initializer: Box<dyn ModuleInitializer>) {
        self.modules.insert(name.into(), initializer);
    }

    pub fn load_module(&mut self, name: &str) -> crate::compat::Result<()> {
        if self.loaded.get(name).copied().unwrap_or(false) {
            return Ok(());
        }

        if let Some(initializer) = self.modules.get(name) {
            initializer.initialize()?;
            self.loaded.insert(name.to_string(), true);
        }

        Ok(())
    }

    pub fn load_essential_modules(&mut self) -> crate::compat::Result<()> {
        // Load only the most essential modules for startup
        let essential = ["parser", "executor", "builtin_commands"];
        
        for module in essential {
            self.load_module(module)?;
        }
        
        Ok(())
    }
}

impl Default for OptimizedModuleLoader {
    fn default() -> Self { Self::new() }
}

pub trait ModuleInitializer: Send + Sync {
    fn initialize(&self) -> crate::compat::Result<()>;
    fn name(&self) -> &str;
    fn is_essential(&self) -> bool { false }
}

/// Fast path execution for simple commands
pub fn execute_fast_path_command(cmd: &str) -> Option<crate::compat::Result<()>> {
    // Handle very simple commands with minimal overhead
    match cmd.trim() {
        "exit" | "quit" => {
            std::process::exit(0);
        }
        cmd if cmd.starts_with("echo ") => {
            let text = &cmd[5..];
            println!("{text}");
            Some(Ok(()))
        }
        "pwd" => {
            if let Ok(dir) = std::env::current_dir() {
                println!("{}", dir.display());
                Some(Ok(()))
            } else {
                Some(Err(crate::compat::anyhow("Failed to get current directory")))
            }
        }
        "help" | "--help" | "-h" => {
            println!("NexusShell - High-Performance Command Line Interface");
            println!("Usage: nxsh [OPTIONS] [COMMAND]");
            println!("Options:");
            println!("  --fast-boot    Enable fast boot mode");
            println!("  --check-cui    Check CUI compatibility");
            println!("  -h, --help     Show this help");
            Some(Ok(()))
        }
        _ => None,
    }
}
