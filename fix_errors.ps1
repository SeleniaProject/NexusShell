# Error fix automation script for NexusShell built-ins
# This script systematically fixes common compilation errors

# Pattern replacements for common errors
$patterns = @{
    # ShellError::runtime -> ShellError::new with RuntimeError
    'ShellError::runtime\("([^"]+)"\)' = 'ShellError::new(nxsh_core::error::ErrorKind::RuntimeError(nxsh_core::error::RuntimeErrorKind::InvalidArgument), "$1")'
    
    # ShellError::io -> ShellError::new with IoError
    'ShellError::io\("([^"]+)"\)' = 'ShellError::new(nxsh_core::error::ErrorKind::IoError(nxsh_core::error::IoErrorKind::Other), "$1")'
    
    # ctx.args() -> &ctx.args
    '\.args\(\)' = '.args'
    
    # ExecutionResult field fixes
    'ExecutionResult \{[\s\n]*exit_code: ([^,]+),[\s\n]*output: ([^,]+),[\s\n]*error: ([^,]+),[\s\n]*duration: ([^,]+),[\s\n]*pid: ([^,]+),[\s\n]*job_id: ([^,}]+)[\s\n]*\}' = 'ExecutionResult { exit_code: $1, output: None, error: None, duration: $4, pid: Some($5), job_id: $6 }'
}

# Get all Rust files in the builtins crate
$files = Get-ChildItem -Path "C:\Users\Aqua\Programming\SeleniaProject\NexusShell\crates\nxsh_builtins\src" -Filter "*.rs" -Recurse

Write-Host "Processing $($files.Count) Rust files..."

foreach ($file in $files) {
    Write-Host "Processing: $($file.Name)"
    $content = Get-Content -Path $file.FullName -Raw
    $originalContent = $content
    
    # Apply each pattern replacement
    foreach ($pattern in $patterns.Keys) {
        $replacement = $patterns[$pattern]
        $content = $content -replace $pattern, $replacement
    }
    
    # Only write if content changed
    if ($content -ne $originalContent) {
        Set-Content -Path $file.FullName -Value $content -NoNewline
        Write-Host "  Modified: $($file.Name)"
    }
}

Write-Host "Pattern replacement completed."
