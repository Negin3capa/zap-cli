# ZapTUI Build Script for Windows
# Alternative to Makefile for Windows users

param(
    [switch]$Clean,
    [switch]$Release,
    [switch]$Run,
    [switch]$Help
)

if ($Help) {
    Write-Host @"
ZapTUI Build Script

Usage:
  .\build.ps1              Build in debug mode
  .\build.ps1 -Release     Build in release mode
  .\build.ps1 -Clean       Clean build artifacts
  .\build.ps1 -Run         Build and run
  .\build.ps1 -Release -Run  Build in release mode and run

"@
    exit 0
}

function Write-Success { Write-Host "✅ $args" -ForegroundColor Green }
function Write-Error-Custom { Write-Host "❌ $args" -ForegroundColor Red }
function Write-Info { Write-Host "ℹ️  $args" -ForegroundColor Cyan }

Write-Host "🔨 ZapTUI Build Script" -ForegroundColor Blue
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue

# Clean build
if ($Clean) {
    Write-Info "Cleaning build artifacts..."
    
    if (Test-Path "target") {
        Remove-Item -Recurse -Force target
        Write-Success "Removed target directory"
    }
    
    if (Test-Path "whatsapp-service/node_modules") {
        Remove-Item -Recurse -Force whatsapp-service/node_modules
        Write-Success "Removed node_modules"
    }
    
    if (-not $Release -and -not $Run) {
        exit 0
    }
}

# Check dependencies
Write-Host "`n📦 Checking dependencies..." -ForegroundColor Blue

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error-Custom "cargo not found. Install Rust from https://rustup.rs/"
    exit 1
}
Write-Success "cargo found"

if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Error-Custom "node not found. Install Node.js from https://nodejs.org/"
    exit 1
}
Write-Success "node found"

if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    Write-Error-Custom "npm not found"
    exit 1
}
Write-Success "npm found"

# Build Rust binary
Write-Host "`n🦀 Building Rust binary..." -ForegroundColor Blue

if ($Release) {
    Write-Info "Building in release mode..."
    cargo build --release
} else {
    Write-Info "Building in debug mode..."
    cargo build
}

if ($LASTEXITCODE -ne 0) {
    Write-Error-Custom "Rust build failed"
    exit 1
}
Write-Success "Rust build successful"

# Build/Install Node.js dependencies
Write-Host "`n🟢 Installing Node.js dependencies..." -ForegroundColor Blue
Push-Location whatsapp-service
npm install --silent
if ($LASTEXITCODE -ne 0) {
    Pop-Location
    Write-Error-Custom "npm install failed"
    exit 1
}
Pop-Location
Write-Success "Node.js dependencies installed"

Write-Host "`n✅ Build complete!" -ForegroundColor Green

# Run if requested
if ($Run) {
    Write-Host "`n🚀 Running ZapTUI..." -ForegroundColor Blue
    if ($Release) {
        & ".\target\release\zaptui.exe"
    } else {
        & ".\target\debug\zaptui.exe"
    }
}
