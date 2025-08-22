# Test ls command output
$testDir = "c:\Users\Aqua\Programming\SeleniaProject\NexusShell\test_dir"
$nxshPath = "c:\Users\Aqua\Programming\SeleniaProject\NexusShell\target\debug\nxsh.exe"

Write-Host "Testing ls command in NexusShell..." -ForegroundColor Cyan

# Create a test script that exits after running ls
@"
cd $testDir
ls
exit
"@ | & $nxshPath

Write-Host "`nTest completed." -ForegroundColor Green
