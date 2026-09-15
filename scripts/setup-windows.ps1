# BookForge — Windows one-shot setup script.
#
# This script:
#   1. Checks that Rust (cargo), Node.js (npm), and PowerShell 7+ are installed.
#   2. Checks for the Microsoft Visual C++ Build Tools (required by the Rust
#      MSVC toolchain that llama-cpp-2 needs to compile on Windows).
#   3. Runs `npm install` for the frontend.
#   4. Downloads the default GGUF model into src-tauri\model\.
#
# Usage:
#   pwsh -File scripts\setup-windows.ps1
#   # or from Windows PowerShell 5.1:
#   .\scripts\setup-windows.ps1

$ErrorActionPreference = "Stop"

function Write-Step($msg) { Write-Host "[*] $msg" -ForegroundColor Cyan }
function Write-OK($msg)   { Write-Host "[v] $msg" -ForegroundColor Green }
function Write-Warn($msg)  { Write-Host "[!] $msg" -ForegroundColor Yellow }
function Write-Err($msg)  { Write-Host "[x] $msg" -ForegroundColor Red }

# Locate the project root (parent of the scripts directory).
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$ProjectRoot = [System.IO.Path]::GetFullPath((Join-Path $ScriptDir ".."))

# --- 1. Toolchain checks -----------------------------------------------------

Write-Step "Checking Rust toolchain..."
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
    Write-Err "cargo not found. Install Rust from https://rustup.rs/ and re-run."
    exit 1
}
$rustVer = (rustc --version) 2>&1
Write-OK "Rust: $rustVer"

Write-Step "Checking Node.js..."
$node = Get-Command node -ErrorAction SilentlyContinue
if (-not $node) {
    Write-Err "node not found. Install Node.js 18+ from https://nodejs.org/ and re-run."
    exit 1
}
$nodeVer = (node --version) 2>&1
Write-OK "Node.js: $nodeVer"

Write-Step "Checking npm..."
$npm = Get-Command npm -ErrorAction SilentlyContinue
if (-not $npm) {
    Write-Err "npm not found. Install Node.js 18+ from https://nodejs.org/ and re-run."
    exit 1
}
$npmVer = (npm --version) 2>&1
Write-OK "npm: $npmVer"

# --- 2. MSVC build tools check ----------------------------------------------

Write-Step "Checking Microsoft Visual C++ Build Tools..."
$msvcFound = $false

# Method 1: look for cl.exe (the MSVC compiler) on PATH
$cl = Get-Command cl -ErrorAction SilentlyContinue
if ($cl) {
    $msvcFound = $true
    Write-OK "Found cl.exe on PATH"
}

# Method 2: look for vswhere
if (-not $msvcFound) {
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($vsPath) {
            $msvcFound = $true
            Write-OK "Found Visual Studio at: $vsPath"
        }
    }
}

# Method 3: check for Build Tools registry entries
if (-not $msvcFound) {
    $buildTools = Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*" -ErrorAction SilentlyContinue |
        Where-Object { $_.DisplayName -match "Visual Studio Build Tools" }
    if ($buildTools) {
        $msvcFound = $true
        Write-OK "Found Visual Studio Build Tools (registry)"
    }
}

if (-not $msvcFound) {
    Write-Warn "MSVC Build Tools not detected."
    Write-Warn "The Rust MSVC toolchain (default on Windows) requires them."
    Write-Warn "Install the 'Desktop development with C++' workload from:"
    Write-Warn "  https://visualstudio.microsoft.com/visual-cpp-build-tools/"
    Write-Warn "After installing, re-run this script."
    $answer = Read-Host "Continue anyway? (y/N)"
    if ($answer -notmatch "^[yY]") { exit 1 }
}

# --- 3. Install frontend dependencies ---------------------------------------

Write-Step "Installing frontend dependencies (npm install)..."
Push-Location $ProjectRoot
try {
    npm install
    if ($LASTEXITCODE -ne 0) { throw "npm install failed" }
    Write-OK "Frontend dependencies installed"
} finally {
    Pop-Location
}

# --- 4. Download model ------------------------------------------------------

Write-Step "Downloading default GGUF model..."
$dlScript = Join-Path $ScriptDir "download_model.ps1"
if (Test-Path $dlScript) {
    & $dlScript
    if ($LASTEXITCODE -ne 0) {
        Write-Warn "Model download failed. You can re-run scripts\download_model.ps1 later."
    }
} else {
    Write-Warn "download_model.ps1 not found at $dlScript"
}

# --- 5. Done ----------------------------------------------------------------

Write-Host ""
Write-OK "Setup complete!"
Write-Host ""
Write-Host "Next steps:" -ForegroundColor White
Write-Host "  cd $ProjectRoot"
Write-Host "  npm run tauri:dev    # launch in dev mode (hot reload)"
Write-Host "  npm run tauri:build  # produce a Windows installer (.msi/.exe) under src-tauri\target\release\bundle\"
Write-Host ""
Write-Host "To override the model, set `$env:BOOKFORGE_MODEL before launching." -ForegroundColor Gray
