# Test banner display
$nxshPath = "c:\Users\Aqua\Programming\SeleniaProject\NexusShell\target\debug\nxsh.exe"

Write-Host "Testing colorful banner..." -ForegroundColor Cyan

# Start NexusShell and quickly exit
$process = Start-Process -FilePath $nxshPath -NoNewWindow -PassThru
Start-Sleep -Seconds 1
$process.Kill()
$process.WaitForExit()

Write-Host "Banner test completed." -ForegroundColor Green
