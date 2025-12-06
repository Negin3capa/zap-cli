# ZapTUI Windows Uninstaller
# Removes ZapTUI installation from user profile

# Colors for output
function Write-Success { Write-Host "✅ $args" -ForegroundColor Green }
function Write-Error-Custom { Write-Host "❌ $args" -ForegroundColor Red }
function Write-Info { Write-Host "ℹ️  $args" -ForegroundColor Cyan }
function Write-Warning-Custom { Write-Host "⚠️  $args" -ForegroundColor Yellow }

Write-Host "🗑️  ZapTUI Uninstaller" -ForegroundColor Blue
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue

$installDir = "$env:LOCALAPPDATA\zaptui"
$binPath = "$env:LOCALAPPDATA\zaptui\bin\zaptui.bat"
$configDir = "$env:APPDATA\zaptui"

# Check if installed
if (-not (Test-Path $installDir) -and -not (Test-Path $binPath)) {
    Write-Warning-Custom "ZapTUI is not installed."
    exit 0
}

Write-Host "`nThis will remove:" -ForegroundColor Yellow
if (Test-Path $binPath) { Write-Host "  - $binPath" }
if (Test-Path $installDir) { Write-Host "  - $installDir" }
Write-Host ""

# Kill any running service
Write-Info "Checking for running services..."
$connection = Get-NetTCPConnection -LocalPort 8080 -State Listen -ErrorAction SilentlyContinue
if ($connection) {
    Write-Warning-Custom "Stopping running WhatsApp service..."
    Stop-Process -Id $connection.OwningProcess -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1
}

# Remove installation directory
if (Test-Path $installDir) {
    Write-Host ""
    Write-Warning-Custom "The installation directory contains:"
    if (Test-Path "$installDir\.wwebjs_auth") {
        Write-Host "  - WhatsApp authentication data (you'd need to re-scan QR code)"
    }
    Write-Host ""

    $remove = Read-Host "Remove installation directory? [y/N]"

    if ($remove -eq 'y' -or $remove -eq 'Y') {
        Write-Info "Removing installation directory..."
        Remove-Item -Recurse -Force $installDir
        Write-Success "Removed $installDir"
    } else {
        Write-Warning-Custom "Keeping $installDir"
    }
}

# Remove from PATH if present
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$binDir = "$env:LOCALAPPDATA\zaptui\bin"
if ($userPath -like "*$binDir*") {
    Write-Host ""
    $removePath = Read-Host "Remove from PATH? [Y/n]"

    if ($removePath -ne 'n' -and $removePath -ne 'N') {
        $newPath = $userPath.Replace(";$binDir", "").Replace("$binDir;", "").Replace("$binDir", "")
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Success "Removed from PATH"
    }
}

# Ask about config
if (Test-Path $configDir) {
    Write-Host ""
    $removeConfig = Read-Host "Remove configuration directory ($configDir)? [y/N]"

    if ($removeConfig -eq 'y' -or $removeConfig -eq 'Y') {
        Write-Info "Removing config directory..."
        Remove-Item -Recurse -Force $configDir
        Write-Success "Removed $configDir"
    } else {
        Write-Warning-Custom "Keeping $configDir"
    }
}

Write-Host ""
Write-Success "🎉 Uninstall complete!"
Write-Host ""

# Check if we are in the source repo
$scriptPath = $MyInvocation.MyCommand.Path
$parentDir = Split-Path $scriptPath -Parent
$repoDir = Split-Path $parentDir -Parent

if (Test-Path "$repoDir\.git") {
    Write-Host ""
    Write-Warning-Custom "It seems you are running this from the cloned repository ($repoDir)"
    $removeRepo = Read-Host "Remove this source repository as well? [y/N]"

    if ($removeRepo -eq 'y' -or $removeRepo -eq 'Y') {
        Write-Info "Removing source repository..."

        # Change to parent directory to allow deletion
        Set-Location (Split-Path $repoDir -Parent)

        # Wait a moment to ensure no file locks
        Start-Sleep -Milliseconds 500

        try {
            Remove-Item -Recurse -Force $repoDir -ErrorAction Stop
            Write-Success "Removed source repository"
        } catch {
            Write-Error-Custom "Failed to remove source repository: $_"
            Write-Host "You may need to close other programs using this folder or remove it manually." -ForegroundColor Yellow
        }
    } else {
        Write-Info "Keeping source repository"
    }
}
