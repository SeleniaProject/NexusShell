# Requires -Version 5.0
<#!
.SYNOPSIS
    Generates a Scoop bucket JSON manifest for NexusShell.
.DESCRIPTION
    Computes the SHA256 of the provided archive and writes a properly
    formatted manifest JSON file.
.EXAMPLE
    ./gen_scoop_manifest.ps1 -Version 1.0.0 -Archive nxsh-1.0.0-windows-x86_64.zip
!#>
param(
    [Parameter(Mandatory = $true)]
    [string]$Version,

    [Parameter(Mandatory = $true)]
    [string]$Archive,

    [Parameter()]
    [string]$OutFile = "nxsh.json"
)

# Compute SHA256 hash of the archive
$sha256 = (Get-FileHash -Algorithm SHA256 -Path $Archive).Hash.ToLower()

$manifest = @{
    version      = $Version
    description  = "NexusShell next-generation CLI shell"
    homepage     = "https://github.com/SeleniaProject/NexusShell"
    license      = "MIT OR Apache-2.0"
    architecture = @{
        64bit = @{
            url  = "https://github.com/SeleniaProject/NexusShell/releases/download/$Version/$Archive"
            hash = $sha256
            bin  = "nxsh.exe"
        }
    }
    notes        = "Run 'nxsh --help' to get started."
} | ConvertTo-Json -Depth 10

$manifest | Out-File -FilePath $OutFile -Encoding UTF8

Write-Host "Generated Scoop manifest: $OutFile" 