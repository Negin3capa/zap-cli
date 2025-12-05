# ZapTUI Windows Installer
# Requires PowerShell 5.1 or later

param(
    [switch]$Global,
    [switch]$Help
)

if ($Help) {
    Write-Host @"
ZapTUI Windows Installer

Usage:
  .\install.ps1           Interactive installation
  .\install.ps1 -Global   Install to user profile (recommended)

Requirements:
  - Rust (cargo)
  - Node.js (18+)
  - npm
"@
    exit 0
}

# Colors for output
function Write-Success { Write-Host "[OK] $args" -ForegroundColor Green }
function Write-Error-Custom { Write-Host "[ERROR] $args" -ForegroundColor Red }
function Write-Info { Write-Host "[INFO] $args" -ForegroundColor Cyan }
function Write-Warning-Custom { Write-Host "[WARNING] $args" -ForegroundColor Yellow }

Write-Host "=== ZapTUI Windows Installer ===" -ForegroundColor Blue
Write-Host "================================================" -ForegroundColor Blue

# Check if running as Administrator (not required, but warn if not)
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Warning-Custom "Not running as Administrator. Some features may be limited."
}

# 1. Check Dependencies
Write-Host "`n[STEP 1] Checking dependencies..." -ForegroundColor Blue
$missingDeps = $false

# Check Cargo
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Success "cargo found ($(cargo --version))"
} else {
    Write-Error-Custom "cargo is not installed."
    Write-Host "Install Rust from: https://rustup.rs/" -ForegroundColor Yellow
    $missingDeps = $true
}

# Check Node.js
if (Get-Command node -ErrorAction SilentlyContinue) {
    Write-Success "node found ($(node --version))"
} else {
    Write-Error-Custom "node is not installed."
    Write-Host "Install Node.js from: https://nodejs.org/" -ForegroundColor Yellow
    $missingDeps = $true
}

# Check npm
if (Get-Command npm -ErrorAction SilentlyContinue) {
    Write-Success "npm found ($(npm --version))"
} else {
    Write-Error-Custom "npm is not installed."
    $missingDeps = $true
}

if ($missingDeps) {
    Write-Error-Custom "Please install missing dependencies and try again."
    exit 1
}

# 2. Build Rust Binary
Write-Host "`n[STEP 2] Building Rust binary..." -ForegroundColor Blue
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "Build failed" }
    Write-Success "Rust build successful."
} catch {
    Write-Error-Custom "Rust build failed: $_"
    exit 1
}

# 3. Install Node Dependencies
Write-Host "`n[STEP 3] Installing WhatsApp service dependencies..." -ForegroundColor Blue
try {
    Push-Location whatsapp-service
    npm install --silent
    if ($LASTEXITCODE -ne 0) { throw "npm install failed" }
    Pop-Location
    Write-Success "Node dependencies installed."
} catch {
    Pop-Location
    Write-Error-Custom "Failed to install Node dependencies: $_"
    exit 1
}

# 4. Installation Verification
Write-Host "`n[STEP 4] Verifying build..." -ForegroundColor Blue
if (Test-Path ".\target\release\zaptui.exe") {
    Write-Success "Binary created at target\release\zaptui.exe"
} else {
    Write-Error-Custom "Binary not found."
    exit 1
}

# 5. Installation Options
Write-Host "`n[STEP 5] Installation Options" -ForegroundColor Blue
Write-Host "1) Local Install - Run from current directory"
Write-Host "2) User Install (Recommended) - Install to user profile"
Write-Host ""

if ($Global) {
    $choice = 2
} else {
    $choice = Read-Host "Choose an option [1-2]"
}

switch ($choice) {
    2 {
        Write-Host "`nInstalling to user profile..." -ForegroundColor Blue
        
        $installDir = "$env:LOCALAPPDATA\zaptui"
        $binDir = "$env:LOCALAPPDATA\zaptui\bin"
        
        # Create installation directory
        New-Item -ItemType Directory -Force -Path $installDir | Out-Null
        New-Item -ItemType Directory -Force -Path $binDir | Out-Null
        
        # Copy and setup WhatsApp service
        Write-Info "Installing WhatsApp service..."
        if (Test-Path "$installDir\whatsapp-service") {
            Remove-Item -Recurse -Force "$installDir\whatsapp-service"
        }
        Copy-Item -Recurse whatsapp-service "$installDir\"
        
        # Install Node dependencies at new location
        Write-Info "Installing service dependencies..."
        Push-Location "$installDir\whatsapp-service"
        npm install --silent
        Pop-Location
        
        # Install binary
        Write-Info "Installing binary..."
        Copy-Item ".\target\release\zaptui.exe" "$installDir\zaptui.exe"
        
        # Create batch launcher
        Write-Info "Creating launcher..."
        $batchContent = @"
@echo off
REM ZapTUI Windows Launcher
set INSTALL_DIR=%LOCALAPPDATA%\zaptui
powershell.exe -ExecutionPolicy Bypass -File "%INSTALL_DIR%\bin\zaptui-launcher.ps1" %*
"@
        $batchContent | Out-File -FilePath "$binDir\zaptui.bat" -Encoding ASCII
        
        # Create PowerShell launcher script
        Copy-Item ".\scripts\zaptui.ps1" "$installDir\bin\zaptui-launcher.ps1"
        
        # Migrate auth data if it exists
        if (Test-Path ".\.wwebjs_auth") {
            Write-Host ""
            Write-Warning-Custom "Found existing WhatsApp authentication data"
            $migrate = Read-Host "Migrate to global installation? [Y/n]"
            
            if ($migrate -ne 'n' -and $migrate -ne 'N') {
                Write-Info "Migrating authentication data..."
                if (Test-Path "$installDir\.wwebjs_auth") {
                    Remove-Item -Recurse -Force "$installDir\.wwebjs_auth"
                }
                Copy-Item -Recurse .\.wwebjs_auth "$installDir\"
                Write-Success "Authentication data migrated"
            }
        }
        
        Write-Success "Installation complete!"
        Write-Host ""
        
        # Check if bin directory is in PATH
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$binDir*") {
            Write-Host "To use 'zaptui' from anywhere, add to PATH:" -ForegroundColor Yellow
            Write-Host "  $binDir" -ForegroundColor Cyan
            Write-Host ""
            $addPath = Read-Host "Add to PATH now? [Y/n]"
            
            if ($addPath -ne 'n' -and $addPath -ne 'N') {
                $newPath = "$userPath;$binDir"
                [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
                Write-Success "Added to PATH. Restart your terminal for changes to take effect."
            }
        } else {
            Write-Success "Already in PATH"
        }
        
        Write-Host ""
        Write-Host "You can now run 'zaptui' from anywhere (after restarting terminal)." -ForegroundColor Green
    }
    default {
        Write-Info "Skipping installation. Run '.\scripts\zaptui.bat' to launch."
    }
}

Write-Host "`n[COMPLETE] Setup finished!" -ForegroundColor Green
Write-Host ""
