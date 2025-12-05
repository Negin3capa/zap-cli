@echo off
REM ZapTUI Windows Batch Launcher (Fallback)
REM This is a simple launcher that calls the PowerShell version

REM Check if running from installation directory
if exist "%LOCALAPPDATA%\zaptui\zaptui.exe" (
    powershell.exe -ExecutionPolicy Bypass -File "%LOCALAPPDATA%\zaptui\bin\zaptui-launcher.ps1" %*
) else if exist ".\target\release\zaptui.exe" (
    REM Running from source directory
    powershell.exe -ExecutionPolicy Bypass -File ".\scripts\zaptui-local.ps1" %*
) else (
    echo Error: ZapTUI not found. Please run install.ps1 first.
    exit /b 1
)
