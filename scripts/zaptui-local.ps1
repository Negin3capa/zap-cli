# ZapTUI Local Windows Launcher (for development)
# Run from the project directory

param(
    [Parameter(ValueFromRemainingArguments=$true)]
    [string[]]$PassThruArgs
)

$ErrorActionPreference = "Stop"

# Change to script directory
Set-Location $PSScriptRoot\..

# Paths
$binary = ".\target\release\zaptui.exe"
$serviceDir = ".\whatsapp-service"

# Handle info flags
if ($PassThruArgs -contains "--version" -or $PassThruArgs -contains "--help") {
    if (Test-Path $binary) {
        & $binary $PassThruArgs
        exit $LASTEXITCODE
    } else {
        Write-Host "❌ Binary not found at $binary" -ForegroundColor Red
        Write-Host "Run 'install.ps1' first." -ForegroundColor Yellow
        exit 1
    }
}

# Check binary exists
if (-not (Test-Path $binary)) {
    Write-Host "❌ Binary not found at $binary" -ForegroundColor Red
    Write-Host "Run 'install.ps1' first." -ForegroundColor Yellow
    exit 1
}

Write-Host "🚀 ZapTUI - Starting..." -ForegroundColor Cyan
Write-Host ""

# Check Node.js
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Error: Node.js is not installed" -ForegroundColor Red
    exit 1
}

# Check WhatsApp service
if (-not (Test-Path "$serviceDir\node_modules")) {
    Write-Host "📦 WhatsApp service dependencies missing. Installing..." -ForegroundColor Yellow
    Push-Location $serviceDir
    npm install --silent
    Pop-Location
}

# Check if port 8080 is in use
$connection = Get-NetTCPConnection -LocalPort 8080 -State Listen -ErrorAction SilentlyContinue
if ($connection) {
    Write-Host "❌ Error: Port 8080 is already in use" -ForegroundColor Red
    Write-Host "To fix, run this in PowerShell as Administrator:" -ForegroundColor Yellow
    Write-Host "  Get-Process -Id $($connection.OwningProcess) | Stop-Process -Force" -ForegroundColor Cyan
    exit 1
}

# Cleanup function
$serviceProcess = $null
function Cleanup {
    Write-Host ""
    Write-Host "🛑 Stopping WhatsApp service..." -ForegroundColor Yellow
    
    if ($serviceProcess -and -not $serviceProcess.HasExited) {
        $serviceProcess.Kill()
    }
    
    # Cleanup any remaining node processes
    Get-Process -Name node -ErrorAction SilentlyContinue | Where-Object {
        $_.Path -and $_.CommandLine -like "*server.js*"
    } | Stop-Process -Force -ErrorAction SilentlyContinue
    
    # Force cleanup port 8080
    Start-Sleep -Milliseconds 500
    $conn = Get-NetTCPConnection -LocalPort 8080 -State Listen -ErrorAction SilentlyContinue
    if ($conn) {
        Stop-Process -Id $conn.OwningProcess -Force -ErrorAction SilentlyContinue
    }
    
    Write-Host "✅ Goodbye!" -ForegroundColor Green
}

# Handle Ctrl+C
[Console]::TreatControlCAsInput = $false
$null = Register-EngineEvent PowerShell.Exiting -Action { Cleanup }

# Start WhatsApp Service
Write-Host "🔌 Starting WhatsApp service..." -ForegroundColor Cyan
Push-Location $serviceDir
$serviceProcess = Start-Process -FilePath "node" -ArgumentList "server.js" -NoNewWindow -PassThru -RedirectStandardOutput "$env:TEMP\zaptui-service.log" -RedirectStandardError "$env:TEMP\zaptui-service-error.log"
Pop-Location

# Wait for service
Start-Sleep -Seconds 2
if ($serviceProcess.HasExited) {
    Write-Host "❌ Failed to start WhatsApp service" -ForegroundColor Red
    Write-Host "Check logs at: $env:TEMP\zaptui-service-error.log" -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ Connected. launching TUI..." -ForegroundColor Green
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
Write-Host ""

# Run TUI
try {
    & $binary $PassThruArgs
    $exitCode = $LASTEXITCODE
} catch {
    Write-Host "❌ Error: $_" -ForegroundColor Red
    $exitCode = 1
} finally {
    Cleanup
}

exit $exitCode
