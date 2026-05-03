# install.ps1 - Download and install todotxt-tui (TUI) and/or todotxt (CLI) for Windows.
#
# Usage (run in PowerShell):
#   irm https://raw.githubusercontent.com/radioflyer28/todotxt.rs/master/scripts/install.ps1 | iex
#   # The above installs the TUI only (default).
#   # To install the CLI or both, save the script first and pass a flag:
#   irm https://raw.githubusercontent.com/radioflyer28/todotxt.rs/master/scripts/install.ps1 -OutFile install.ps1
#   .\install.ps1 --cli
#   .\install.ps1 --both
#   # Or set $env:INSTALL before piping to iex:
#   $env:INSTALL='both'; irm .../install.ps1 | iex
#
# Options:
#   --tui   Install todotxt-tui only (default)
#   --cli   Install todotxt only
#   --both  Install both todotxt-tui and todotxt
#
# Environment overrides (set before running):
#   $env:INSTALL        'tui' (default), 'cli', or 'both'
#   $env:INSTALL_DIR    Installation directory (default: %USERPROFILE%\bin)
#   $env:RELEASE_TAG    Specific release tag, e.g. "v1.5.0" (default: latest)

$ErrorActionPreference = 'Stop'

$Repo = 'radioflyer28/todotxt.rs'

# -- Determine what to install ------------------------------------------------

# $args parsing (for direct invocation); $env:INSTALL overrides
$mode = 'tui'
foreach ($arg in $args) {
    switch ($arg) {
        '--tui'  { $mode = 'tui' }
        '--cli'  { $mode = 'cli' }
        '--both' { $mode = 'both' }
        default  { Write-Error "Unknown option: $arg. Use --tui (default), --cli, or --both."; exit 1 }
    }
}
if ($env:INSTALL) { $mode = $env:INSTALL.ToLower() }

$InstallTui = $mode -in @('tui', 'both')
$InstallCli = $mode -in @('cli', 'both')

# -- Resolve install directory -------------------------------------------------

$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { Join-Path $env:USERPROFILE 'bin' }

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

# -- Download helper -----------------------------------------------------------

function Install-Binary {
    param([string]$Asset, [string]$BinaryName)

    $Dest = Join-Path $InstallDir $BinaryName
    $Url = if ($env:RELEASE_TAG) {
        "https://github.com/$Repo/releases/download/$($env:RELEASE_TAG)/$Asset"
    } else {
        "https://github.com/$Repo/releases/latest/download/$Asset"
    }

    Write-Host "Downloading $Asset ..."
    Invoke-WebRequest -Uri $Url -OutFile $Dest -UseBasicParsing

    $Version = & $Dest --version 2>$null
    Write-Host "Installed: $Dest"
    Write-Host "Version:   $($Version ?? 'unknown')"
    Write-Host ""
}

# -- Install requested binaries -----------------------------------------------

if ($InstallTui) {
    Install-Binary 'todotxt-tui-windows-x86_64.exe' 'todotxt-tui.exe'
}

if ($InstallCli) {
    Install-Binary 'todotxt-windows-x86_64.exe' 'todotxt.exe'
}

# -- PATH check ----------------------------------------------------------------

$UserPath = [Environment]::GetEnvironmentVariable('PATH', 'User')
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "Adding $InstallDir to your user PATH ..."
    [Environment]::SetEnvironmentVariable(
        'PATH',
        "$UserPath;$InstallDir",
        'User'
    )
    $env:PATH = "$env:PATH;$InstallDir"
    Write-Host "Done. New terminals will pick this up automatically."
    Write-Host ""
}

# -- Alias suggestion ----------------------------------------------------------

Write-Host "Tip: add short aliases to your PowerShell profile:"
Write-Host ""
if ($InstallTui) { Write-Host "  Set-Alias todo todotxt-tui" }
if ($InstallCli) { Write-Host "  Set-Alias td   todotxt" }
Write-Host ""

$addParts = @()
if ($InstallTui) { $addParts += 'Set-Alias todo todotxt-tui' }
if ($InstallCli) { $addParts += 'Set-Alias td   todotxt' }
$addBlock = $addParts -join "`n"
Write-Host "Add to `$PROFILE with:"
Write-Host "  Add-Content `$PROFILE `"`n$addBlock`""
Write-Host ""
