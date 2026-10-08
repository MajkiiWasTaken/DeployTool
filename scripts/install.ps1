
<#
/************************************************
* File: install.ps1
* Author: Michal Švrček
*
* DeployTool Windows installer
*
* ver. 0.5.0
*************************************************/
#>

[CmdletBinding()]
param(
    [string]$BinaryPath
)

$ErrorActionPreference = "Stop"

$ProjectRoot = Split-Path -Parent $PSScriptRoot

if ([string]::IsNullOrWhiteSpace($BinaryPath)) {
    $BinaryPath = Join-Path $ProjectRoot "target\release\deploy.exe"
}

if (-not (Test-Path -LiteralPath $BinaryPath -PathType Leaf)) {
    throw "deploy.exe not found: $BinaryPath. Build the project first."
}

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\DeployTool"
$Destination = Join-Path $InstallDir "deploy.exe"

Write-Host "DeployTool Windows Installer" -ForegroundColor Cyan

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

$Source = (Resolve-Path -LiteralPath $BinaryPath).Path

if (-not [string]::Equals(
    $Source, $Destination,
    [StringComparison]::OrdinalIgnoreCase
)) {
    Copy-Item -LiteralPath $Source -Destination $Destination -Force
}

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")

$Entries = @(
    $UserPath -split ';' | Where-Object {
        -not [string]::IsNullOrWhiteSpace($_)
    }
)

$Exists = @($Entries | Where-Object {
    $_.TrimEnd('\') -ieq $InstallDir.TrimEnd('\')
}).Count -gt 0

if (-not $Exists) {
    $Entries += $InstallDir

    [Environment]::SetEnvironmentVariable(
        "Path",
        ($Entries -join ';'),
        "User"
    )

    Write-Host "[OK] Added DeployTool to user PATH." -ForegroundColor Green
}

$ConfigDir = Join-Path $env:APPDATA "DeployTool"
New-Item -ItemType Directory -Force -Path $ConfigDir | Out-Null

Write-Host ""
Write-Host "[OK] DeployTool installed successfully!" -ForegroundColor Green
Write-Host "Binary: $Destination"
Write-Host "Configuration: $(Join-Path $ConfigDir 'deploy.toml')"
Write-Host ""

& $Destination --version

Write-Host ""
Write-Host "Open a new terminal and run:"
Write-Host "  deploy --version"
Write-Host "  deploy list"

