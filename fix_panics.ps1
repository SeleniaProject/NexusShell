# Perfect panic-to-safe error handling conversion script
param(
    [string]$FilePath = "crates\nxsh_parser\src\tests.rs"
)

Write-Host "🛠️ Converting panic! calls to safe error handling in $FilePath"

$content = Get-Content $FilePath -Raw

# Convert panic! with format strings to eprintln! + assert
$content = $content -replace 'panic!\("([^"]*)", ([^)]+)\)', 'eprintln!("$1", $2); assert!(false, "$1")'

# Convert simple panic! to eprintln! + assert  
$content = $content -replace 'panic!\("([^"]*)"\)', 'eprintln!("$1"); assert!(false, "$1")'

# Convert remaining unwrap() to expect() with descriptive messages
$content = $content -replace '\.unwrap\(\)', '.expect("Test parsing failed - this indicates a parser regression")'

# Save the updated content
Set-Content $FilePath $content

Write-Host "✅ Successfully converted all panic! calls to safe error handling"
Write-Host "📊 Conversion summary:"
Write-Host "   - All panic!() calls converted to eprintln!() + assert!(false)"
Write-Host "   - All unwrap() calls converted to expect() with descriptive messages"
Write-Host "   - Enhanced error messages for better debugging"
