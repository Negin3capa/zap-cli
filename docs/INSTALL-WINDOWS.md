# ZapTUI Windows Installation Guide

## Prerequisites

### Install Rust

1. Download and run [rustup-init.exe](https://rustup.rs/)
2. Follow the installer prompts
3. Restart your terminal after installation
4. Verify: `cargo --version`

### Install Node.js

1. Download the LTS version from [nodejs.org](https://nodejs.org/)
2. Run the installer (includes npm)
3. Verify: `node --version` and `npm --version`

## Installation

### Option 1: PowerShell (Recommended)

1. Open PowerShell
2. Clone the repository:

   ```powershell
   git clone https://github.com/Negin3capa/zaptui.git
   cd zaptui
   ```

3. Enable script execution (if needed):

   ```powershell
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   ```

4. Run the installer:

   ```powershell
   .\install.ps1
   ```

5. Choose installation option (Global recommended)

6. When prompted, add to PATH for easy access

### Option 2: Windows Terminal

Use the same steps as PowerShell but in Windows Terminal for a better experience.

## Running ZapTUI

### If installed globally and added to PATH:

```powershell
zaptui
```

### From project directory:

```powershell
.\scripts\zaptui.bat
```

## Troubleshooting

### PowerShell Execution Policy

If you see "running scripts is disabled", run:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Port Already in Use

If port 8080 is already in use, run in PowerShell as Administrator:

```powershell
Get-NetTCPConnection -LocalPort 8080 | ForEach-Object { Stop-Process -Id $_.OwningProcess -Force }
```

### Windows Defender / Firewall

Windows Firewall may prompt for Node.js to access the network. Click "Allow access" to enable the WhatsApp service.

### Missing Visual C++ Build Tools

If Rust compilation fails, you may need:

- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
- Or install [Visual Studio Community](https://visualstudio.microsoft.com/vs/community/)
- Select "Desktop development with C++" workload

## File Locations

- **Installation**: `%LOCALAPPDATA%\zaptui` (usually `C:\Users\<YourName>\AppData\Local\zaptui`)
- **Config**: `%APPDATA%\zaptui\config.toml` (not yet implemented)
- **WhatsApp Auth**: `%LOCALAPPDATA%\zaptui\.wwebjs_auth`
- **Launcher**: `%LOCALAPPDATA%\zaptui\bin\zaptui.bat`

## Uninstalling

Run the uninstaller:

```powershell
.\scripts\uninstall.ps1
```

Or manually remove:

- `%LOCALAPPDATA%\zaptui`
- `%APPDATA%\zaptui`
- Remove from PATH if added

## Tips

- Use **Windows Terminal** for the best TUI experience
- The traditional Command Prompt (cmd.exe) works but has limited Unicode support
- For QR code display, ensure your terminal font supports Unicode characters
- Recommended fonts: Cascadia Code, Consolas, or Fira Code
