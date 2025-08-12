use std::process::Command;
use nxsh_core::{ShellError, ErrorKind};
use nxsh_core::error::{RuntimeErrorKind, SystemErrorKind};

pub fn package_cli(args: &[String]) -> Result<(), ShellError> {
    if args.is_empty() || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let action = &args[0];
    let packages = if args.len() > 1 { &args[1..] } else { &[] };
    
    match action.as_str() {
        "install" => install_packages(packages),
        "remove" | "uninstall" => remove_packages(packages),
        "update" => update_packages(),
        "upgrade" => upgrade_packages(),
        "search" => search_packages(packages),
        "info" => show_package_info(packages),
        "list" => list_packages(),
        _ => Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Unknown action: {action}"))),
    }
}

fn print_help() {
    println!("package - cross-platform package manager interface

USAGE:
    package ACTION [PACKAGES...]

ACTIONS:
    install PACKAGE...    Install packages
    remove PACKAGE...     Remove packages  
    update               Update package lists
    upgrade              Upgrade all packages
    search PATTERN...    Search for packages
    info PACKAGE...      Show package information
    list                 List installed packages

EXAMPLES:
    package install git curl
    package remove old-package
    package search editor
    package info python3");
}

fn install_packages(packages: &[String]) -> Result<(), ShellError> {
    if packages.is_empty() {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::TooFewArguments), "No packages specified for installation"));
    }

    println!("Installing packages: {}", packages.join(", "));
    
    for manager in get_package_managers() {
        if manager.is_available() {
            return manager.install(packages);
        }
    }
    
    Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::UnsupportedOperation), "No package manager available"))
}

fn remove_packages(packages: &[String]) -> Result<(), ShellError> {
    if packages.is_empty() {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::TooFewArguments), "No packages specified for removal"));
    }

    println!("Removing packages: {}", packages.join(", "));
    
    for manager in get_package_managers() {
        if manager.is_available() {
            return manager.remove(packages);
        }
    }
    
    Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::UnsupportedOperation), "No package manager available"))
}

fn update_packages() -> Result<(), ShellError> {
    println!("Updating package lists...");
    
    for manager in get_package_managers() {
        if manager.is_available() {
            return manager.update();
        }
    }
    
    Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::UnsupportedOperation), "No package manager available"))
}

fn upgrade_packages() -> Result<(), ShellError> {
    println!("Upgrading packages...");
    
    for manager in get_package_managers() {
        if manager.is_available() {
            return manager.upgrade();
        }
    }
    
    Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::UnsupportedOperation), "No package manager available"))
}

fn search_packages(patterns: &[String]) -> Result<(), ShellError> {
    if patterns.is_empty() {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::TooFewArguments), "No search pattern specified"));
    }

    for manager in get_package_managers() {
        if manager.is_available() {
            return manager.search(patterns);
        }
    }
    
    Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::UnsupportedOperation), "No package manager available"))
}

fn show_package_info(packages: &[String]) -> Result<(), ShellError> {
    if packages.is_empty() {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::TooFewArguments), "No packages specified"));
    }

    for manager in get_package_managers() {
        if manager.is_available() {
            return manager.info(packages);
        }
    }
    
    Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::UnsupportedOperation), "No package manager available"))
}

fn list_packages() -> Result<(), ShellError> {
    for manager in get_package_managers() {
        if manager.is_available() {
            return manager.list();
        }
    }
    
    Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::UnsupportedOperation), "No package manager available"))
}

trait PackageManager {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn install(&self, packages: &[String]) -> Result<(), ShellError>;
    fn remove(&self, packages: &[String]) -> Result<(), ShellError>;
    fn update(&self) -> Result<(), ShellError>;
    fn upgrade(&self) -> Result<(), ShellError>;
    fn search(&self, patterns: &[String]) -> Result<(), ShellError>;
    fn info(&self, packages: &[String]) -> Result<(), ShellError>;
    fn list(&self) -> Result<(), ShellError>;
}

struct AptManager;
struct YumManager;
struct PacmanManager;
struct BrewManager;
struct ChocolateyManager;
struct ScoopManager;

impl PackageManager for AptManager {
    fn name(&self) -> &str { "apt" }
    
    fn is_available(&self) -> bool {
        Command::new("apt").arg("--version").output().is_ok()
    }
    
    fn install(&self, packages: &[String]) -> Result<(), ShellError> {
        execute_command("apt", &["install", "-y"], packages)
    }
    
    fn remove(&self, packages: &[String]) -> Result<(), ShellError> {
        execute_command("apt", &["remove", "-y"], packages)
    }
    
    fn update(&self) -> Result<(), ShellError> {
        execute_command("apt", &["update"], &[])
    }
    
    fn upgrade(&self) -> Result<(), ShellError> {
        execute_command("apt", &["upgrade", "-y"], &[])
    }
    
    fn search(&self, patterns: &[String]) -> Result<(), ShellError> {
        execute_command("apt", &["search"], patterns)
    }
    
    fn info(&self, packages: &[String]) -> Result<(), ShellError> {
        execute_command("apt", &["show"], packages)
    }
    
    fn list(&self) -> Result<(), ShellError> {
        execute_command("apt", &["list", "--installed"], &[])
    }
}

impl PackageManager for ChocolateyManager {
    fn name(&self) -> &str { "choco" }
    
    fn is_available(&self) -> bool {
        Command::new("choco").arg("--version").output().is_ok()
    }
    
    fn install(&self, packages: &[String]) -> Result<(), ShellError> {
        execute_command("choco", &["install", "-y"], packages)
    }
    
    fn remove(&self, packages: &[String]) -> Result<(), ShellError> {
        execute_command("choco", &["uninstall", "-y"], packages)
    }
    
    fn update(&self) -> Result<(), ShellError> {
        execute_command("choco", &["outdated"], &[])
    }
    
    fn upgrade(&self) -> Result<(), ShellError> {
        execute_command("choco", &["upgrade", "all", "-y"], &[])
    }
    
    fn search(&self, patterns: &[String]) -> Result<(), ShellError> {
        execute_command("choco", &["search"], patterns)
    }
    
    fn info(&self, packages: &[String]) -> Result<(), ShellError> {
        execute_command("choco", &["info"], packages)
    }
    
    fn list(&self) -> Result<(), ShellError> {
        execute_command("choco", &["list", "--local-only"], &[])
    }
}

// Implement other package managers similarly...

fn get_package_managers() -> Vec<Box<dyn PackageManager>> {
    vec![
        Box::new(AptManager),
        Box::new(ChocolateyManager),
        // Add other managers as needed
    ]
}

fn execute_command(cmd: &str, args: &[&str], extra_args: &[String]) -> Result<(), ShellError> {
    let mut command = Command::new(cmd);
    command.args(args);
    command.args(extra_args);
    
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::ProcessError), format!("{} failed with exit code {:?}", cmd, status.code()))),
        Err(e) => Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::ProcessError), format!("Failed to execute {cmd}: {e}"))),
    }
}
