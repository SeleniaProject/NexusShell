# Test help command output
$nxshPath = "c:\Users\Aqua\Programming\SeleniaProject\NexusShell\target\debug\nxsh.exe"

Write-Host "Testing help command in NexusShell..." -ForegroundColor Cyan

# Create a test script that runs help then exits
@"
help
exit
"@ | & $nxshPath

Write-Host "`nTest completed." -ForegroundColor Green
