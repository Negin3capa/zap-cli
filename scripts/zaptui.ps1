# ZapTUI Windows PowerShell Launcher
# Manages the WhatsApp service and launches the TUI

param(
    [Parameter(ValueFromRemainingArguments=$true)]
    [string[]]$PassThruArgs
)

$ErrorActionPreference = "Stop"

# Installation paths
$installDir = "$env:LOCALAPPDATA\zaptui"
$serviceDir = "$installDir\whatsapp-service"
$authDir = "$installDir\.wwebjs_auth"
$binary = "$installDir\zaptui.exe"

# Handle info flags directly without starting service
if ($PassThruArgs -contains "--version" -or $PassThruArgs -contains "--help" -or 
    $PassThruArgs -contains "-h" -or $PassThruArgs -contains "-V") {
    if (Test-Path $binary) {
        & $binary $PassThruArgs
        exit $LASTEXITCODE
    } else {
        Write-Host "[ERROR] ZapTUI not properly installed. Run 'install.ps1' from the project directory." -ForegroundColor Red
        exit 1
    }
}

# Check if installation exists
if (-not (Test-Path $installDir)) {
    Write-Host "[ERROR] ZapTUI not properly installed." -ForegroundColor Red
    Write-Host "Run 'install.ps1' from the project directory to install." -ForegroundColor Yellow
    exit 1
}

if (-not (Test-Path $binary)) {
    Write-Host "[ERROR] ZapTUI binary not found at $binary" -ForegroundColor Red
    Write-Host "Run 'install.ps1' from the project directory to reinstall." -ForegroundColor Yellow
    exit 1
}

Write-Host "[START] ZapTUI - Starting..." -ForegroundColor Cyan
Write-Host ""

# Check Node.js
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] Error: Node.js is not installed" -ForegroundColor Red
    exit 1
}

# Check if service dependencies are installed
if (-not (Test-Path "$serviceDir\node_modules")) {
    Write-Host "[SETUP] WhatsApp service dependencies missing. Installing..." -ForegroundColor Yellow
    Push-Location $serviceDir
    npm install --silent
    Pop-Location
}

# Check if service is already running on port 8080
$serviceAlreadyRunning = $false
$connection = Get-NetTCPConnection -LocalPort 8080 -State Listen -ErrorAction SilentlyContinue
if ($connection) {
    Write-Host "[OK] WhatsApp service already running (shared)" -ForegroundColor Green
    $serviceAlreadyRunning = $true
}

# Cleanup function
function Stop-Service {
    Write-Host ""
    if (-not $serviceAlreadyRunning) {
        Write-Host "[STOP] Stopping WhatsApp service..." -ForegroundColor Yellow
        
        # Kill Node.js processes running server.js
        Get-Process -Name node -ErrorAction SilentlyContinue | Where-Object {
            $_.Path -and $_.CommandLine -like "*server.js*"
        } | Stop-Process -Force -ErrorAction SilentlyContinue
        
        # Force kill anything on port 8080
        Start-Sleep -Milliseconds 500
        $conn = Get-NetTCPConnection -LocalPort 8080 -State Listen -ErrorAction SilentlyContinue
        if ($conn) {
            $processPid = $conn.OwningProcess
            Stop-Process -Id $processPid -Force -ErrorAction SilentlyContinue
        }
    } else {
        Write-Host "[INFO] Leaving shared service running for other instances" -ForegroundColor Cyan
    }
    Write-Host "[OK] Goodbye!" -ForegroundColor Green
}

# Register cleanup on exit
Register-EngineEvent PowerShell.Exiting -Action { Stop-Service } | Out-Null

# Start WhatsApp Service if not already running
if (-not $serviceAlreadyRunning) {
    Write-Host "[CONNECT] Starting WhatsApp service..." -ForegroundColor Cyan
    
    # Set auth path via environment variable
    $env:ZAPTUI_AUTH_PATH = $authDir
    
    # Start Node.js service in background
    Push-Location $serviceDir
    $serviceProcess = Start-Process -FilePath "node" -ArgumentList "server.js" -NoNewWindow -PassThru -RedirectStandardOutput "$env:TEMP\zaptui-service.log" -RedirectStandardError "$env:TEMP\zaptui-service-error.log"
    Pop-Location
    
    # Wait for service to start
    Start-Sleep -Seconds 2
    
    if ($serviceProcess.HasExited) {
        Write-Host "[ERROR] Failed to start WhatsApp service" -ForegroundColor Red
        Write-Host "Check logs at: $env:TEMP\zaptui-service-error.log" -ForegroundColor Yellow
        exit 1
    }
    
    Write-Host "[OK] Connected. launching TUI..." -ForegroundColor Green
} else {
    Write-Host "[START] Launching TUI..." -ForegroundColor Cyan
}

Write-Host "================================================" -ForegroundColor Blue
Write-Host ""

# Run TUI
try {
    & $binary $PassThruArgs
    $exitCode = $LASTEXITCODE
} catch {
    Write-Host "[ERROR] Error running ZapTUI: $_" -ForegroundColor Red
    $exitCode = 1
} finally {
    Stop-Service
}

exit $exitCode
