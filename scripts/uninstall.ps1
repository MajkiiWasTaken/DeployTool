<#
/************************************************
* File: uninstall.ps1
* Author: Michal Švrček
*
* DeployTool Windows uninstaller
*
* ver. 0.5.0
*************************************************/
#>

$ErrorActionPreference = "Stop"

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\DeployTool"
$Binary = Join-Path $InstallDir "deploy.exe"

Write-Host "DeployTool Windows Uninstaller" -ForegroundColor Cyan

if (Test-Path -LiteralPath $Binary) {
    Remove-Item -LiteralPath $Binary -Force
}

if ((Test-Path -LiteralPath $InstallDir) -and
    -not (Get-ChildItem -LiteralPath $InstallDir -Force)) {
    Remove-Item -LiteralPath $InstallDir
}

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")

$Entries = @(
    $UserPath -split ';' | Where-Object {
        -not [string]::IsNullOrWhiteSpace($_) -and
        $_.TrimEnd('\') -ine $InstallDir.TrimEnd('\')
    }
)

[Environment]::SetEnvironmentVariable(
    "Path",
    ($Entries -join ';'),
    "User"
)

Write-Host ""
Write-Host "Open a new terminal and run:"
Write-Host "  deploy --version"
Write-Host "  deploy list"