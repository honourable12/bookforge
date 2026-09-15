# Downloads the default GGUF model used by BookForge into src-tauri\model\
#
# The default is Qwen2.5-1.5B-Instruct (Q4_K_M) — small enough to load in
# a few seconds on a laptop while still being useful for prose assistance.
#
# Override the URL/filename with environment variables:
#   $env:BOOKFORGE_MODEL_URL = "https://huggingface.co/.../model.gguf"
#   $env:BOOKFORGE_MODEL_FILENAME = "my-model.gguf"
#
# Usage:
#   pwsh -File scripts\download_model.ps1
#   # or from Windows PowerShell:
#   .\scripts\download_model.ps1

param(
    [string]$ModelUrl = $env:BOOKFORGE_MODEL_URL,
    [string]$ModelFilename = $env:BOOKFORGE_MODEL_FILENAME
)

$ErrorActionPreference = "Stop"

if (-not $ModelFilename) {
    $ModelFilename = "qwen2.5-1.5b-instruct-q4_k_m.gguf"
}
if (-not $ModelUrl) {
    $ModelUrl = "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q4_k_m.gguf"
}

# Resolve the model directory relative to this script.
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$ModelDir = Join-Path $ScriptDir "..\src-tauri\model"
$ModelDir = [System.IO.Path]::GetFullPath($ModelDir)

if (-not (Test-Path $ModelDir)) {
    New-Item -ItemType Directory -Path $ModelDir -Force | Out-Null
}

$Dest = Join-Path $ModelDir $ModelFilename

if (Test-Path $Dest) {
    Write-Host "Model already exists at: $Dest" -ForegroundColor Green
    Write-Host "Skipping download. Delete the file to re-download."
    exit 0
}

Write-Host "Downloading model from:" -ForegroundColor Cyan
Write-Host "  $ModelUrl"
Write-Host "into:"
Write-Host "  $Dest"
Write-Host ""
Write-Host "This is ~1 GB and may take a few minutes depending on your connection." -ForegroundColor Yellow
Write-Host ""

# Use BITS for robustness on Windows; fall back to Invoke-WebRequest.
try {
    if (Get-Command Start-BitsTransfer -ErrorAction SilentlyContinue) {
        Start-BitsTransfer -Source $ModelUrl -Destination $Dest -DisplayName "BookForge model download"
    } else {
        $ProgressPreference = "Continue"
        Invoke-WebRequest -Uri $ModelUrl -OutFile $Dest -UseBasicParsing
    }
} catch {
    Write-Host "Download failed: $_" -ForegroundColor Red
    if (Test-Path $Dest) {
        Remove-Item $Dest -Force
    }
    exit 1
}

$Size = (Get-Item $Dest).Length
$SizeMB = [math]::Round($Size / 1MB, 2)
Write-Host ""
Write-Host "Done. Downloaded $SizeMB MB to $Dest" -ForegroundColor Green
Write-Host "BookForge will pick up the model automatically on next launch."
Write-Host "To use a different model, set `$env:BOOKFORGE_MODEL = 'C:\path\to\your.gguf'"
