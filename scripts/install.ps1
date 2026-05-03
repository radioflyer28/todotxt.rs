# install.ps1 — Download and install todotxt-tui for Windows.
#
# Usage (run in PowerShell):
#   irm https://raw.githubusercontent.com/radioflyer28/todotxt.rs/master/scripts/install.ps1 | iex
#   # or clone the repo and run:
#   .\scripts\install.ps1
#
# Environment overrides (set before running):
#   $env:INSTALL_DIR    Installation directory (default: %USERPROFILE%\bin)
#   $env:RELEASE_TAG    Specific release tag, e.g. "v1.5.0" (default: latest)

$ErrorActionPreference = 'Stop'

$Repo       = 'radioflyer28/todotxt.rs'
$BinaryName = 'todotxt-tui'
$Asset      = 'todotxt-tui-windows-x86_64.exe'

# ── Resolve install directory ─────────────────────────────────────────────────

if ($env:INSTALL_DIR) {
    $InstallDir = $env:INSTALL_DIR
} else {
    $InstallDir = Join-Path $env:USERPROFILE 'bin'
}

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

$Dest = Join-Path $InstallDir "$BinaryName.exe"

# ── Resolve download URL ──────────────────────────────────────────────────────

if ($env:RELEASE_TAG) {
    $Url = "https://github.com/$Repo/releases/download/$($env:RELEASE_TAG)/$Asset"
} else {
    $Url = "https://github.com/$Repo/releases/latest/download/$Asset"
}

# ── Download ──────────────────────────────────────────────────────────────────

Write-Host "Downloading $Asset ..."
Invoke-WebRequest -Uri $Url -OutFile $Dest -UseBasicParsing

$Version = & $Dest --version 2>$null
Write-Host ""
Write-Host "Installed: $Dest"
Write-Host "Version:   $($Version ?? 'unknown')"

# ── PATH check ────────────────────────────────────────────────────────────────

$UserPath = [Environment]::GetEnvironmentVariable('PATH', 'User')
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host ""
    Write-Host "Adding $InstallDir to your user PATH ..."
    [Environment]::SetEnvironmentVariable(
        'PATH',
        "$UserPath;$InstallDir",
        'User'
    )
    # Refresh current session
    $env:PATH = "$env:PATH;$InstallDir"
    Write-Host "Done. New terminals will pick this up automatically."
}

# ── Alias suggestion ──────────────────────────────────────────────────────────

Write-Host ""
Write-Host "Tip: add a short alias to your PowerShell profile:"
Write-Host ""
Write-Host "  Set-Alias todo todotxt-tui"
Write-Host ""
Write-Host "Add it with:"
Write-Host "  Add-Content `$PROFILE `"``nSet-Alias todo todotxt-tui`""
Write-Host ""
Write-Host "Then just run: todo"
