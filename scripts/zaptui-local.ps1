# ZapTUI Local Windows Launcher (Development)
# Manages the WhatsApp service and launches the TUI from source
# Run from the project directory

param(
    [Parameter(ValueFromRemainingArguments=$true)]
    [string[]]$PassThruArgs
)

$ErrorActionPreference = "Stop"

# Change to script directory's parent (project root)
Set-Location $PSScriptRoot\..
$projectRoot = Get-Location

# Paths
$serviceDir = "$projectRoot\whatsapp-service"
$authDir = "$projectRoot\.wwebjs_auth"
$binary = "$projectRoot\target\release\zaptui.exe"
$serviceLog = "$env:TEMP\zaptui-service.log"
$serviceErr = "$env:TEMP\zaptui-service-error.log"

# Handle info flags directly without starting service
if ($PassThruArgs -contains "--version" -or $PassThruArgs -contains "--help" -or
    $PassThruArgs -contains "-h" -or $PassThruArgs -contains "-V") {
    if (Test-Path $binary) {
        & $binary $PassThruArgs
        exit $LASTEXITCODE
    } else {
        Write-Host "❌ ZapTUI binary not found at $binary" -ForegroundColor Red
        Write-Host "Run 'cargo build --release' first." -ForegroundColor Yellow
        exit 1
    }
}

# Check if binary exists
if (-not (Test-Path $binary)) {
    Write-Host "❌ ZapTUI binary not found at $binary" -ForegroundColor Red
    Write-Host "Run 'cargo build --release' first." -ForegroundColor Yellow
    exit 1
}

Write-Host "🚀 ZapTUI - Starting..." -ForegroundColor Cyan
Write-Host ""

# Check Node.js
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Error: Node.js is not installed" -ForegroundColor Red
    exit 1
}

# Check if service dependencies are installed
if (-not (Test-Path "$serviceDir\node_modules")) {
    Write-Host "📦 WhatsApp service dependencies missing. Installing..." -ForegroundColor Yellow
    Push-Location $serviceDir
    npm install --silent
    Pop-Location
}

# Check if service is already running on port 8080
$serviceAlreadyRunning = $false
$connection = Get-NetTCPConnection -LocalPort 8080 -State Listen -ErrorAction SilentlyContinue
if ($connection) {
    Write-Host "✅ WhatsApp service already running (shared)" -ForegroundColor Green
    $serviceAlreadyRunning = $true
}

# Cleanup function
function Stop-Service {
    Write-Host ""
    if (-not $serviceAlreadyRunning) {
        Write-Host "🛑 Stopping WhatsApp service..." -ForegroundColor Yellow

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
        Write-Host "ℹ️  Leaving shared service running for other instances" -ForegroundColor Cyan
    }
    Write-Host "✅ Goodbye!" -ForegroundColor Green
}

# Register cleanup on exit
Register-EngineEvent PowerShell.Exiting -Action { Stop-Service } | Out-Null

# Start WhatsApp Service if not already running
if (-not $serviceAlreadyRunning) {
    Write-Host "🔌 Starting WhatsApp service..." -ForegroundColor Cyan

    # Set auth path via environment variable (optional for local, but good for consistency)
    $env:ZAPTUI_AUTH_PATH = $authDir

    # Start Node.js service in background
    Push-Location $serviceDir
    $serviceProcess = Start-Process -FilePath "node" -ArgumentList "server.js" -NoNewWindow -PassThru -RedirectStandardOutput $serviceLog -RedirectStandardError $serviceErr
    Pop-Location

    # Wait for service to start
    Start-Sleep -Seconds 2

    if ($serviceProcess.HasExited) {
        Write-Host "❌ Failed to start WhatsApp service" -ForegroundColor Red
        Write-Host "Check logs at: $serviceErr" -ForegroundColor Yellow
        Get-Content $serviceErr -Tail 10 | Write-Host -ForegroundColor Gray
        exit 1
    }

    Write-Host "✅ Connected. launching TUI..." -ForegroundColor Green
} else {
    Write-Host "🚀 Launching TUI..." -ForegroundColor Cyan
}

Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
Write-Host ""

# Run TUI
try {
    & $binary $PassThruArgs
    $exitCode = $LASTEXITCODE
} catch {
    Write-Host "❌ Error running ZapTUI: $_" -ForegroundColor Red
    $exitCode = 1
} finally {
    Stop-Service
}

exit $exitCode
