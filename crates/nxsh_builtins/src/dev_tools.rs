use anyhow::{Result, Context};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    env,
    fs,
    time::SystemTime,
    sync::Arc,
};
use tokio::{
    process::Command as AsyncCommand,
    sync::RwLock,
    io::{AsyncBufReadExt, BufReader},
};
use serde::{Deserialize, Serialize};
use log::{info, warn, error, debug};
use regex::Regex;

use crate::common::i18n::tr;
use nxsh_core::{context::NxshContext, result::NxshResult};

/// Development tools manager for various development utilities
pub struct DevToolsManager {
    git_repos: Arc<RwLock<HashMap<PathBuf, GitRepository>>>,
    build_cache: Arc<RwLock<HashMap<PathBuf, BuildInfo>>>,
    config: DevToolsConfig,
}

impl DevToolsManager {
    /// Create a new development tools manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            git_repos: Arc::new(RwLock::new(HashMap::new())),
            build_cache: Arc::new(RwLock::new(HashMap::new())),
            config: DevToolsConfig::default(),
        })
    }
    
    /// Execute git command with enhanced features
    pub async fn git(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        if args.is_empty() {
            return self.show_git_help().await;
        }
        
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        
        // Check if we're in a git repository
        let repo_info = self.get_or_create_repo_info(&current_dir).await?;
        
        match args[0].as_str() {
            "status" => self.git_status(&current_dir, &args[1..], &repo_info).await,
            "log" => self.git_log(&current_dir, &args[1..], &repo_info).await,
            "diff" => self.git_diff(&current_dir, &args[1..], &repo_info).await,
            "branch" => self.git_branch(&current_dir, &args[1..], &repo_info).await,
            "checkout" => self.git_checkout(&current_dir, &args[1..], &repo_info).await,
            "add" => self.git_add(&current_dir, &args[1..], &repo_info).await,
            "commit" => self.git_commit(&current_dir, &args[1..], &repo_info).await,
            "push" => self.git_push(&current_dir, &args[1..], &repo_info).await,
            "pull" => self.git_pull(&current_dir, &args[1..], &repo_info).await,
            "clone" => self.git_clone(&current_dir, &args[1..]).await,
            "init" => self.git_init(&current_dir, &args[1..]).await,
            "remote" => self.git_remote(&current_dir, &args[1..], &repo_info).await,
            "merge" => self.git_merge(&current_dir, &args[1..], &repo_info).await,
            "rebase" => self.git_rebase(&current_dir, &args[1..], &repo_info).await,
            "stash" => self.git_stash(&current_dir, &args[1..], &repo_info).await,
            "tag" => self.git_tag(&current_dir, &args[1..], &repo_info).await,
            _ => self.git_passthrough(&current_dir, args).await,
        }
    }
    
    /// Execute make command with enhanced features
    pub async fn make(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        
        // Check for Makefile
        if !current_dir.join("Makefile").exists() && !current_dir.join("makefile").exists() {
            return Err(anyhow::anyhow!("No Makefile found in current directory").into());
        }
        
        info!("Running make in {}", current_dir.display());
        
        let build_info = self.get_or_create_build_info(&current_dir, BuildTool::Make).await?;
        
        let mut cmd = AsyncCommand::new("make");
        cmd.current_dir(&current_dir);
        cmd.args(args);
        
        // Add environment variables
        if let Some(ref env_vars) = build_info.environment {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }
        
        // Execute with output streaming
        self.execute_build_command(cmd, &current_dir, BuildTool::Make).await
    }
    
    /// Execute cargo command with enhanced features
    pub async fn cargo(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        
        // Check for Cargo.toml
        if !current_dir.join("Cargo.toml").exists() {
            return Err(anyhow::anyhow!("No Cargo.toml found in current directory").into());
        }
        
        info!("Running cargo in {}", current_dir.display());
        
        let build_info = self.get_or_create_build_info(&current_dir, BuildTool::Cargo).await?;
        
        let mut cmd = AsyncCommand::new("cargo");
        cmd.current_dir(&current_dir);
        cmd.args(args);
        
        // Add Rust-specific environment
        cmd.env("RUST_BACKTRACE", "1");
        
        // Add custom environment variables
        if let Some(ref env_vars) = build_info.environment {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }
        
        // Execute with enhanced output
        self.execute_cargo_command(cmd, &current_dir, args).await
    }
    
    /// Execute go command with enhanced features
    pub async fn go(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        
        info!("Running go in {}", current_dir.display());
        
        let build_info = self.get_or_create_build_info(&current_dir, BuildTool::Go).await?;
        
        let mut cmd = AsyncCommand::new("go");
        cmd.current_dir(&current_dir);
        cmd.args(args);
        
        // Add Go-specific environment
        if let Some(gopath) = env::var_os("GOPATH") {
            cmd.env("GOPATH", gopath);
        }
        if let Some(goroot) = env::var_os("GOROOT") {
            cmd.env("GOROOT", goroot);
        }
        
        // Add custom environment variables
        if let Some(ref env_vars) = build_info.environment {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }
        
        // Execute with output streaming
        self.execute_build_command(cmd, &current_dir, BuildTool::Go).await
    }
    
    /// Execute npm command with enhanced features
    pub async fn npm(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        
        // Check for package.json
        if !current_dir.join("package.json").exists() {
            return Err(anyhow::anyhow!("No package.json found in current directory").into());
        }
        
        info!("Running npm in {}", current_dir.display());
        
        let build_info = self.get_or_create_build_info(&current_dir, BuildTool::Npm).await?;
        
        let mut cmd = AsyncCommand::new("npm");
        cmd.current_dir(&current_dir);
        cmd.args(args);
        
        // Add Node.js-specific environment
        if let Some(node_env) = env::var_os("NODE_ENV") {
            cmd.env("NODE_ENV", node_env);
        } else {
            cmd.env("NODE_ENV", "development");
        }
        
        // Add custom environment variables
        if let Some(ref env_vars) = build_info.environment {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }
        
        // Execute with output streaming
        self.execute_build_command(cmd, &current_dir, BuildTool::Npm).await
    }
    
    /// Execute yarn command with enhanced features
    pub async fn yarn(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        
        // Check for package.json or yarn.lock
        if !current_dir.join("package.json").exists() {
            return Err(anyhow::anyhow!("No package.json found in current directory").into());
        }
        
        info!("Running yarn in {}", current_dir.display());
        
        let build_info = self.get_or_create_build_info(&current_dir, BuildTool::Yarn).await?;
        
        let mut cmd = AsyncCommand::new("yarn");
        cmd.current_dir(&current_dir);
        cmd.args(args);
        
        // Add Node.js-specific environment
        if let Some(node_env) = env::var_os("NODE_ENV") {
            cmd.env("NODE_ENV", node_env);
        } else {
            cmd.env("NODE_ENV", "development");
        }
        
        // Add custom environment variables
        if let Some(ref env_vars) = build_info.environment {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }
        
        // Execute with output streaming
        self.execute_build_command(cmd, &current_dir, BuildTool::Yarn).await
    }
    
    /// Execute python/pip commands with enhanced features
    pub async fn python(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        
        info!("Running python in {}", current_dir.display());
        
        let build_info = self.get_or_create_build_info(&current_dir, BuildTool::Python).await?;
        
        let mut cmd = AsyncCommand::new("python");
        cmd.current_dir(&current_dir);
        cmd.args(args);
        
        // Add Python-specific environment
        if let Some(virtual_env) = env::var_os("VIRTUAL_ENV") {
            cmd.env("VIRTUAL_ENV", virtual_env);
        }
        
        // Add custom environment variables
        if let Some(ref env_vars) = build_info.environment {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }
        
        // Execute with output streaming
        self.execute_build_command(cmd, &current_dir, BuildTool::Python).await
    }
    
    /// Execute pip command with enhanced features
    pub async fn pip(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        
        info!("Running pip in {}", current_dir.display());
        
        let mut cmd = AsyncCommand::new("pip");
        cmd.current_dir(&current_dir);
        cmd.args(args);
        
        // Add Python-specific environment
        if let Some(virtual_env) = env::var_os("VIRTUAL_ENV") {
            cmd.env("VIRTUAL_ENV", virtual_env);
        }
        
        // Execute with output streaming
        self.execute_build_command(cmd, &current_dir, BuildTool::Pip).await
    }
    
    /// Execute cmake command with enhanced features
    pub async fn cmake(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        
        info!("Running cmake in {}", current_dir.display());
        
        let build_info = self.get_or_create_build_info(&current_dir, BuildTool::CMake).await?;
        
        let mut cmd = AsyncCommand::new("cmake");
        cmd.current_dir(&current_dir);
        cmd.args(args);
        
        // Add CMake-specific environment
        if let Some(cmake_prefix_path) = env::var_os("CMAKE_PREFIX_PATH") {
            cmd.env("CMAKE_PREFIX_PATH", cmake_prefix_path);
        }
        
        // Add custom environment variables
        if let Some(ref env_vars) = build_info.environment {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }
        
        // Execute with output streaming
        self.execute_build_command(cmd, &current_dir, BuildTool::CMake).await
    }
    
    /// Execute gdb command with enhanced features
    pub async fn gdb(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        
        info!("Running gdb in {}", current_dir.display());
        
        let mut cmd = AsyncCommand::new("gdb");
        cmd.current_dir(&current_dir);
        cmd.args(args);
        
        // Interactive mode for gdb
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        
        let status = cmd.status().await.context("Failed to execute gdb")?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("gdb exited with status: {}", status).into());
        }
        
        Ok(())
    }
    
    /// Execute strace command with enhanced features
    pub async fn strace(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        
        info!("Running strace in {}", current_dir.display());
        
        let mut cmd = AsyncCommand::new("strace");
        cmd.current_dir(&current_dir);
        cmd.args(args);
        
        // Execute with output streaming
        self.execute_debug_command(cmd, &current_dir, "strace").await
    }
    
    /// Execute valgrind command with enhanced features
    pub async fn valgrind(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        
        info!("Running valgrind in {}", current_dir.display());
        
        let mut cmd = AsyncCommand::new("valgrind");
        cmd.current_dir(&current_dir);
        cmd.args(args);
        
        // Add valgrind-specific options for better output
        if !args.contains(&"--tool".to_string()) {
            cmd.arg("--tool=memcheck");
        }
        if !args.contains(&"--leak-check".to_string()) {
            cmd.arg("--leak-check=full");
        }
        if !args.contains(&"--show-leak-kinds".to_string()) {
            cmd.arg("--show-leak-kinds=all");
        }
        
        // Execute with output streaming
        self.execute_debug_command(cmd, &current_dir, "valgrind").await
    }
    
    /// Get project information and statistics
    pub async fn project_info(&self, ctx: &NxshContext) -> NxshResult<()> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        
        println!("Project Information for: {}", current_dir.display());
        println!("=" .repeat(50));
        
        // Detect project type
        let project_types = self.detect_project_types(&current_dir).await;
        if !project_types.is_empty() {
            println!("Project Types: {}", project_types.join(", "));
        }
        
        // Git information
        if let Ok(repo_info) = self.get_or_create_repo_info(&current_dir).await {
            println!("\nGit Repository:");
            println!("  Current Branch: {}", repo_info.current_branch.unwrap_or_else(|| "unknown".to_string()));
            println!("  Remote URL: {}", repo_info.remote_url.unwrap_or_else(|| "none".to_string()));
            println!("  Last Commit: {}", repo_info.last_commit_hash.unwrap_or_else(|| "none".to_string()));
            
            if let Ok(status) = self.get_git_status(&current_dir).await {
                println!("  Status: {} modified, {} staged, {} untracked", 
                        status.modified.len(), 
                        status.staged.len(), 
                        status.untracked.len());
            }
        }
        
        // Build information
        let build_cache = self.build_cache.read().await;
        if let Some(build_info) = build_cache.get(&current_dir) {
            println!("\nBuild Information:");
            println!("  Build Tool: {:?}", build_info.tool);
            if let Some(ref last_build) = build_info.last_build_time {
                println!("  Last Build: {:?}", last_build);
            }
            if let Some(ref last_status) = build_info.last_build_status {
                println!("  Last Status: {:?}", last_status);
            }
        }
        
        // Dependencies information
        self.show_dependencies_info(&current_dir).await?;
        
        Ok(())
    }
    
    /// Show build status across multiple projects
    pub async fn build_status(&self, ctx: &NxshContext) -> NxshResult<()> {
        let build_cache = self.build_cache.read().await;
        
        if build_cache.is_empty() {
            println!("No build information available");
            return Ok(());
        }
        
        println!("Build Status Summary");
        println!("=" .repeat(50));
        
        for (path, build_info) in build_cache.iter() {
            println!("Project: {}", path.display());
            println!("  Tool: {:?}", build_info.tool);
            
            if let Some(ref status) = build_info.last_build_status {
                println!("  Status: {:?}", status);
            }
            
            if let Some(ref time) = build_info.last_build_time {
                println!("  Last Build: {:?}", time);
            }
            
            println!();
        }
        
        Ok(())
    }
    
    // Private helper methods
    
    async fn show_git_help(&self) -> NxshResult<()> {
        println!("NexusShell Git Integration");
        println!("=" .repeat(30));
        println!();
        println!("Enhanced git commands:");
        println!("  git status     - Show working tree status with enhanced formatting");
        println!("  git log        - Show commit logs with graph visualization");
        println!("  git diff       - Show changes with syntax highlighting");
        println!("  git branch     - List, create, or delete branches");
        println!("  git checkout   - Switch branches or restore files");
        println!("  git add        - Add file contents to the index");
        println!("  git commit     - Record changes to the repository");
        println!("  git push       - Update remote refs along with associated objects");
        println!("  git pull       - Fetch from and integrate with another repository");
        println!("  git clone      - Clone a repository into a new directory");
        println!("  git init       - Create an empty Git repository");
        println!("  git remote     - Manage set of tracked repositories");
        println!("  git merge      - Join two or more development histories");
        println!("  git rebase     - Reapply commits on top of another base tip");
        println!("  git stash      - Stash the changes in a dirty working directory");
        println!("  git tag        - Create, list, delete or verify tags");
        println!();
        println!("All other git commands are passed through to the system git.");
        
        Ok(())
    }
    
    async fn get_or_create_repo_info(&self, path: &Path) -> Result<GitRepository> {
        let mut repos = self.git_repos.write().await;
        
        if let Some(repo) = repos.get(path) {
            return Ok(repo.clone());
        }
        
        // Check if this is a git repository
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            return Err(anyhow::anyhow!("Not a git repository"));
        }
        
        // Create new repository info
        let mut repo = GitRepository {
            path: path.to_path_buf(),
            current_branch: None,
            remote_url: None,
            last_commit_hash: None,
            status: None,
            branches: Vec::new(),
        };
        
        // Get current branch
        if let Ok(output) = Command::new("git")
            .current_dir(path)
            .args(&["branch", "--show-current"])
            .output()
        {
            if output.status.success() {
                repo.current_branch = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
        
        // Get remote URL
        if let Ok(output) = Command::new("git")
            .current_dir(path)
            .args(&["remote", "get-url", "origin"])
            .output()
        {
            if output.status.success() {
                repo.remote_url = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
        
        // Get last commit hash
        if let Ok(output) = Command::new("git")
            .current_dir(path)
            .args(&["rev-parse", "HEAD"])
            .output()
        {
            if output.status.success() {
                repo.last_commit_hash = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
        
        repos.insert(path.to_path_buf(), repo.clone());
        Ok(repo)
    }
    
    async fn get_or_create_build_info(&self, path: &Path, tool: BuildTool) -> Result<BuildInfo> {
        let mut build_cache = self.build_cache.write().await;
        
        if let Some(build_info) = build_cache.get(path) {
            return Ok(build_info.clone());
        }
        
        let build_info = BuildInfo {
            path: path.to_path_buf(),
            tool,
            last_build_time: None,
            last_build_status: None,
            environment: None,
        };
        
        build_cache.insert(path.to_path_buf(), build_info.clone());
        Ok(build_info)
    }
    
    async fn git_status(&self, path: &Path, args: &[String], repo: &GitRepository) -> NxshResult<()> {
        let status = self.get_git_status(path).await?;
        
        println!("On branch {}", repo.current_branch.as_deref().unwrap_or("unknown"));
        
        if let Some(ref remote) = repo.remote_url {
            println!("Remote: {}", remote);
        }
        
        if !status.staged.is_empty() {
            println!("\nChanges to be committed:");
            for file in &status.staged {
                println!("  \x1b[32mmodified:   {}\x1b[0m", file);
            }
        }
        
        if !status.modified.is_empty() {
            println!("\nChanges not staged for commit:");
            for file in &status.modified {
                println!("  \x1b[31mmodified:   {}\x1b[0m", file);
            }
        }
        
        if !status.untracked.is_empty() {
            println!("\nUntracked files:");
            for file in &status.untracked {
                println!("  \x1b[31m{}\x1b[0m", file);
            }
        }
        
        if status.staged.is_empty() && status.modified.is_empty() && status.untracked.is_empty() {
            println!("Working tree clean");
        }
        
        Ok(())
    }
    
    async fn get_git_status(&self, path: &Path) -> Result<GitStatus> {
        let output = Command::new("git")
            .current_dir(path)
            .args(&["status", "--porcelain"])
            .output()
            .context("Failed to execute git status")?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Git status failed"));
        }
        
        let mut status = GitStatus {
            staged: Vec::new(),
            modified: Vec::new(),
            untracked: Vec::new(),
        };
        
        let status_text = String::from_utf8_lossy(&output.stdout);
        for line in status_text.lines() {
            if line.len() < 3 {
                continue;
            }
            
            let status_chars = &line[0..2];
            let filename = &line[3..];
            
            match status_chars {
                "??" => status.untracked.push(filename.to_string()),
                s if s.chars().nth(0).unwrap() != ' ' => status.staged.push(filename.to_string()),
                s if s.chars().nth(1).unwrap() != ' ' => status.modified.push(filename.to_string()),
                _ => {}
            }
        }
        
        Ok(status)
    }
    
    async fn git_log(&self, path: &Path, args: &[String], repo: &GitRepository) -> NxshResult<()> {
        let mut cmd = AsyncCommand::new("git");
        cmd.current_dir(path);
        cmd.args(&["log", "--graph", "--pretty=format:%C(yellow)%h%C(reset) - %C(green)(%cr)%C(reset) %s %C(bold blue)<%an>%C(reset)%C(red)%d%C(reset)", "--abbrev-commit"]);
        cmd.args(args);
        
        let output = cmd.output().await.context("Failed to execute git log")?;
        
        if output.status.success() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            eprintln!("Git log failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    async fn git_diff(&self, path: &Path, args: &[String], repo: &GitRepository) -> NxshResult<()> {
        let mut cmd = AsyncCommand::new("git");
        cmd.current_dir(path);
        cmd.args(&["diff", "--color=always"]);
        cmd.args(args);
        
        let output = cmd.output().await.context("Failed to execute git diff")?;
        
        if output.status.success() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            eprintln!("Git diff failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    async fn git_branch(&self, path: &Path, args: &[String], repo: &GitRepository) -> NxshResult<()> {
        if args.is_empty() {
            // List branches with enhanced formatting
            let output = Command::new("git")
                .current_dir(path)
                .args(&["branch", "-v"])
                .output()
                .context("Failed to execute git branch")?;
            
            if output.status.success() {
                let branch_text = String::from_utf8_lossy(&output.stdout);
                for line in branch_text.lines() {
                    if line.starts_with('*') {
                        println!("\x1b[32m{}\x1b[0m", line);
                    } else {
                        println!("{}", line);
                    }
                }
            }
        } else {
            // Pass through to git
            self.git_passthrough(path, &["branch".to_string()].iter().chain(args).cloned().collect::<Vec<_>>()).await?;
        }
        
        Ok(())
    }
    
    async fn git_checkout(&self, path: &Path, args: &[String], repo: &GitRepository) -> NxshResult<()> {
        self.git_passthrough(path, &["checkout".to_string()].iter().chain(args).cloned().collect::<Vec<_>>()).await
    }
    
    async fn git_add(&self, path: &Path, args: &[String], repo: &GitRepository) -> NxshResult<()> {
        self.git_passthrough(path, &["add".to_string()].iter().chain(args).cloned().collect::<Vec<_>>()).await
    }
    
    async fn git_commit(&self, path: &Path, args: &[String], repo: &GitRepository) -> NxshResult<()> {
        self.git_passthrough(path, &["commit".to_string()].iter().chain(args).cloned().collect::<Vec<_>>()).await
    }
    
    async fn git_push(&self, path: &Path, args: &[String], repo: &GitRepository) -> NxshResult<()> {
        self.git_passthrough(path, &["push".to_string()].iter().chain(args).cloned().collect::<Vec<_>>()).await
    }
    
    async fn git_pull(&self, path: &Path, args: &[String], repo: &GitRepository) -> NxshResult<()> {
        self.git_passthrough(path, &["pull".to_string()].iter().chain(args).cloned().collect::<Vec<_>>()).await
    }
    
    async fn git_clone(&self, path: &Path, args: &[String]) -> NxshResult<()> {
        self.git_passthrough(path, &["clone".to_string()].iter().chain(args).cloned().collect::<Vec<_>>()).await
    }
    
    async fn git_init(&self, path: &Path, args: &[String]) -> NxshResult<()> {
        self.git_passthrough(path, &["init".to_string()].iter().chain(args).cloned().collect::<Vec<_>>()).await
    }
    
    async fn git_remote(&self, path: &Path, args: &[String], repo: &GitRepository) -> NxshResult<()> {
        self.git_passthrough(path, &["remote".to_string()].iter().chain(args).cloned().collect::<Vec<_>>()).await
    }
    
    async fn git_merge(&self, path: &Path, args: &[String], repo: &GitRepository) -> NxshResult<()> {
        self.git_passthrough(path, &["merge".to_string()].iter().chain(args).cloned().collect::<Vec<_>>()).await
    }
    
    async fn git_rebase(&self, path: &Path, args: &[String], repo: &GitRepository) -> NxshResult<()> {
        self.git_passthrough(path, &["rebase".to_string()].iter().chain(args).cloned().collect::<Vec<_>>()).await
    }
    
    async fn git_stash(&self, path: &Path, args: &[String], repo: &GitRepository) -> NxshResult<()> {
        self.git_passthrough(path, &["stash".to_string()].iter().chain(args).cloned().collect::<Vec<_>>()).await
    }
    
    async fn git_tag(&self, path: &Path, args: &[String], repo: &GitRepository) -> NxshResult<()> {
        self.git_passthrough(path, &["tag".to_string()].iter().chain(args).cloned().collect::<Vec<_>>()).await
    }
    
    async fn git_passthrough(&self, path: &Path, args: &[String]) -> NxshResult<()> {
        let mut cmd = AsyncCommand::new("git");
        cmd.current_dir(path);
        cmd.args(args);
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        
        let status = cmd.status().await.context("Failed to execute git command")?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("Git command failed with status: {}", status).into());
        }
        
        Ok(())
    }
    
    async fn execute_build_command(&self, mut cmd: AsyncCommand, path: &Path, tool: BuildTool) -> NxshResult<()> {
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        let start_time = SystemTime::now();
        let mut child = cmd.spawn().context("Failed to spawn build command")?;
        
        // Stream output
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            
            while let Some(line) = lines.next_line().await.context("Failed to read stdout")? {
                println!("{}", line);
            }
        }
        
        let status = child.wait().await.context("Failed to wait for build command")?;
        let build_time = start_time.elapsed().unwrap();
        
        // Update build info
        let build_status = if status.success() {
            BuildStatus::Success
        } else {
            BuildStatus::Failed
        };
        
        let mut build_cache = self.build_cache.write().await;
        if let Some(build_info) = build_cache.get_mut(path) {
            build_info.last_build_time = Some(start_time);
            build_info.last_build_status = Some(build_status.clone());
        }
        
        if status.success() {
            println!("Build completed successfully in {:.2}s", build_time.as_secs_f64());
        } else {
            return Err(anyhow::anyhow!("Build failed with status: {}", status).into());
        }
        
        Ok(())
    }
    
    async fn execute_cargo_command(&self, mut cmd: AsyncCommand, path: &Path, args: &[String]) -> NxshResult<()> {
        // Add cargo-specific enhancements
        if !args.is_empty() {
            match args[0].as_str() {
                "build" | "check" | "test" | "run" => {
                    // Add color output
                    cmd.arg("--color=always");
                },
                _ => {}
            }
        }
        
        self.execute_build_command(cmd, path, BuildTool::Cargo).await
    }
    
    async fn execute_debug_command(&self, mut cmd: AsyncCommand, path: &Path, tool_name: &str) -> NxshResult<()> {
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        let mut child = cmd.spawn().context("Failed to spawn debug command")?;
        
        // Stream output with debug-specific formatting
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            
            while let Some(line) = lines.next_line().await.context("Failed to read stdout")? {
                // Add color coding for different types of debug output
                if line.contains("ERROR") || line.contains("SIGSEGV") {
                    println!("\x1b[31m{}\x1b[0m", line);
                } else if line.contains("WARNING") {
                    println!("\x1b[33m{}\x1b[0m", line);
                } else if line.contains("INFO") {
                    println!("\x1b[32m{}\x1b[0m", line);
                } else {
                    println!("{}", line);
                }
            }
        }
        
        let status = child.wait().await.context("Failed to wait for debug command")?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("{} failed with status: {}", tool_name, status).into());
        }
        
        Ok(())
    }
    
    async fn detect_project_types(&self, path: &Path) -> Vec<String> {
        let mut types = Vec::new();
        
        // Check for various project files
        if path.join("Cargo.toml").exists() {
            types.push("Rust".to_string());
        }
        if path.join("package.json").exists() {
            types.push("Node.js".to_string());
        }
        if path.join("go.mod").exists() {
            types.push("Go".to_string());
        }
        if path.join("Makefile").exists() || path.join("makefile").exists() {
            types.push("Make".to_string());
        }
        if path.join("CMakeLists.txt").exists() {
            types.push("CMake".to_string());
        }
        if path.join("setup.py").exists() || path.join("pyproject.toml").exists() {
            types.push("Python".to_string());
        }
        if path.join("pom.xml").exists() {
            types.push("Maven".to_string());
        }
        if path.join("build.gradle").exists() || path.join("build.gradle.kts").exists() {
            types.push("Gradle".to_string());
        }
        if path.join(".git").exists() {
            types.push("Git".to_string());
        }
        
        types
    }
    
    async fn show_dependencies_info(&self, path: &Path) -> Result<()> {
        println!("\nDependencies:");
        
        // Rust dependencies
        if path.join("Cargo.toml").exists() {
            if let Ok(content) = fs::read_to_string(path.join("Cargo.toml")) {
                let deps = self.extract_cargo_dependencies(&content);
                if !deps.is_empty() {
                    println!("  Rust (Cargo):");
                    for dep in deps {
                        println!("    {}", dep);
                    }
                }
            }
        }
        
        // Node.js dependencies
        if path.join("package.json").exists() {
            if let Ok(content) = fs::read_to_string(path.join("package.json")) {
                let deps = self.extract_npm_dependencies(&content);
                if !deps.is_empty() {
                    println!("  Node.js (npm):");
                    for dep in deps {
                        println!("    {}", dep);
                    }
                }
            }
        }
        
        // Python requirements
        if path.join("requirements.txt").exists() {
            if let Ok(content) = fs::read_to_string(path.join("requirements.txt")) {
                println!("  Python (pip):");
                for line in content.lines() {
                    if !line.trim().is_empty() && !line.starts_with('#') {
                        println!("    {}", line.trim());
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn extract_cargo_dependencies(&self, content: &str) -> Vec<String> {
        let mut deps = Vec::new();
        let mut in_dependencies = false;
        
        for line in content.lines() {
            let line = line.trim();
            
            if line == "[dependencies]" {
                in_dependencies = true;
                continue;
            }
            
            if line.starts_with('[') && line != "[dependencies]" {
                in_dependencies = false;
                continue;
            }
            
            if in_dependencies && !line.is_empty() && !line.starts_with('#') {
                if let Some(dep_name) = line.split('=').next() {
                    deps.push(dep_name.trim().to_string());
                }
            }
        }
        
        deps
    }
    
    fn extract_npm_dependencies(&self, content: &str) -> Vec<String> {
        let mut deps = Vec::new();
        
        // Simple JSON parsing for dependencies
        let lines: Vec<&str> = content.lines().collect();
        let mut in_dependencies = false;
        
        for line in lines {
            let line = line.trim();
            
            if line.contains("\"dependencies\"") || line.contains("\"devDependencies\"") {
                in_dependencies = true;
                continue;
            }
            
            if in_dependencies && line == "}" {
                in_dependencies = false;
                continue;
            }
            
            if in_dependencies && line.contains(':') {
                if let Some(dep_name) = line.split(':').next() {
                    let dep_name = dep_name.trim().trim_matches('"');
                    if !dep_name.is_empty() {
                        deps.push(dep_name.to_string());
                    }
                }
            }
        }
        
        deps
    }
}

// Structs and enums

#[derive(Debug, Clone)]
struct GitRepository {
    path: PathBuf,
    current_branch: Option<String>,
    remote_url: Option<String>,
    last_commit_hash: Option<String>,
    status: Option<GitStatus>,
    branches: Vec<String>,
}

#[derive(Debug, Clone)]
struct GitStatus {
    staged: Vec<String>,
    modified: Vec<String>,
    untracked: Vec<String>,
}

#[derive(Debug, Clone)]
struct BuildInfo {
    path: PathBuf,
    tool: BuildTool,
    last_build_time: Option<SystemTime>,
    last_build_status: Option<BuildStatus>,
    environment: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq)]
enum BuildTool {
    Make,
    Cargo,
    Go,
    Npm,
    Yarn,
    Python,
    Pip,
    CMake,
    Maven,
    Gradle,
}

#[derive(Debug, Clone, PartialEq)]
enum BuildStatus {
    Success,
    Failed,
    InProgress,
}

#[derive(Debug, Clone)]
struct DevToolsConfig {
    enable_git_integration: bool,
    enable_build_caching: bool,
    show_build_progress: bool,
    auto_detect_project_type: bool,
}

impl Default for DevToolsConfig {
    fn default() -> Self {
        Self {
            enable_git_integration: true,
            enable_build_caching: true,
            show_build_progress: true,
            auto_detect_project_type: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_dev_tools_creation() {
        let manager = DevToolsManager::new().unwrap();
        let repos = manager.git_repos.read().await;
        assert!(repos.is_empty());
    }
    
    #[tokio::test]
    async fn test_project_type_detection() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DevToolsManager::new().unwrap();
        
        // Create a Cargo.toml file
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        
        let types = manager.detect_project_types(temp_dir.path()).await;
        assert!(types.contains(&"Rust".to_string()));
    }
    
    #[test]
    fn test_cargo_dependency_extraction() {
        let manager = DevToolsManager::new().unwrap();
        let content = r#"
[package]
name = "test"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
"#;
        
        let deps = manager.extract_cargo_dependencies(content);
        assert!(deps.contains(&"serde".to_string()));
        assert!(deps.contains(&"tokio".to_string()));
    }
    
    #[test]
    fn test_npm_dependency_extraction() {
        let manager = DevToolsManager::new().unwrap();
        let content = r#"
{
  "name": "test",
  "dependencies": {
    "express": "^4.18.0",
    "lodash": "^4.17.21"
  }
}
"#;
        
        let deps = manager.extract_npm_dependencies(content);
        assert!(deps.contains(&"express".to_string()));
        assert!(deps.contains(&"lodash".to_string()));
    }
    
    #[test]
    fn test_build_tool_enum() {
        let cargo = BuildTool::Cargo;
        let make = BuildTool::Make;
        
        assert_ne!(cargo, make);
        assert_eq!(cargo, BuildTool::Cargo);
    }
    
    #[test]
    fn test_build_status_enum() {
        let success = BuildStatus::Success;
        let failed = BuildStatus::Failed;
        
        assert_ne!(success, failed);
        assert_eq!(success, BuildStatus::Success);
    }
} 