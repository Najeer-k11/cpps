# cpps installer for Windows
# Run: powershell -ExecutionPolicy Bypass -File install.ps1

$ErrorActionPreference = "Stop"

$InstallDir = "$env:USERPROFILE\.cpps\bin"
$BinaryName = "cpps.exe"

Write-Host ""
Write-Host "  Installing cpps..." -ForegroundColor Cyan
Write-Host ""

# Build release binary
Write-Host "  Building release binary..." -ForegroundColor Yellow
$ErrorActionPreference = "Continue"
cargo build --release 2>&1 | Out-Null
$ErrorActionPreference = "Stop"
if ($LASTEXITCODE -ne 0) {
    Write-Host "  Build failed. Run 'cargo build --release' manually to see errors." -ForegroundColor Red
    exit 1
}

# Create install directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Copy binary
$source = "target\release\$BinaryName"
$dest = "$InstallDir\$BinaryName"
Copy-Item $source $dest -Force

Write-Host "  Installed to: $dest" -ForegroundColor Green

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
    Write-Host "  Added $InstallDir to user PATH" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Restart your terminal for PATH changes to take effect." -ForegroundColor Yellow
} else {
    Write-Host "  PATH already configured." -ForegroundColor Green
}

Write-Host ""
Write-Host "  Done! Run 'cpps --version' to verify." -ForegroundColor Cyan
Write-Host ""
