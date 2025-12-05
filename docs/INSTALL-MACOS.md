# ZapTUI macOS Installation Guide

## Prerequisites

### Install Homebrew (Recommended)

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### Install Rust

**Option 1: Using rustup (Recommended)**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Option 2: Using Homebrew**

```bash
brew install rust
```

Verify: `cargo --version`

### Install Node.js

**Option 1: Using Homebrew (Recommended)**

```bash
brew install node
```

**Option 2: From nodejs.org**
Download the macOS installer from [nodejs.org](https://nodejs.org/)

Verify: `node --version` and `npm --version`

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/Negin3capa/zaptui.git
   cd zaptui
   ```

2. Run the macOS installer:

   ```bash
   chmod +x install-macos.sh
   ./install-macos.sh
   ```

3. Choose installation option:
   - **Option 2** (Recommended): Installs to `~/Library/Application Support/zaptui`
   - **Option 3**: Uses Homebrew location if you prefer

## Running ZapTUI

### If installed globally:

```bash
zaptui
```

### From project directory:

```bash
./scripts/zaptui-macos
```

## Troubleshooting

### Port Already in Use

If port 8080 is already in use:

```bash
lsof -ti:8080 | xargs kill -9
```

### Permission Denied

If you get permission errors during installation, ensure the scripts are executable:

```bash
chmod +x install-macos.sh
chmod +x scripts/zaptui-macos
```

### Gatekeeper Security

macOS may warn about running downloaded software:

- Right-click the binary and select "Open"
- Or run: `xattr -d com.apple.quarantine zaptui` in the installation directory

### Terminal Compatibility

ZapTUI works best with:

- **Terminal.app** (built-in)
- **iTerm2** (recommended for advanced features)
- **Alacritty**
- **Kitty**

### Command Not Found

If `zaptui` is not found after installation, ensure `/usr/local/bin` is in your PATH:

```bash
echo $PATH
```

If not, add to your shell config (`~/.zshrc` or `~/.bash_profile`):

```bash
export PATH="/usr/local/bin:$PATH"
```

## File Locations

- **Installation**: `~/Library/Application Support/zaptui`
- **Config**: `~/.config/zaptui/config.toml`
- **WhatsApp Auth**: `~/Library/Application Support/zaptui/.wwebjs_auth`
- **Launcher**: `/usr/local/bin/zaptui` (symlink)

## Uninstalling

Run the uninstaller:

```bash
./scripts/uninstall-macos.sh
```

Or manually remove:

```bash
rm -rf ~/Library/Application\ Support/zaptui
rm -rf ~/.config/zaptui
sudo rm /usr/local/bin/zaptui
```

## Tips

- Use iTerm2 for better Unicode and emoji support
- Ensure your terminal font supports glyphs (SF Mono, Menlo, or Fira Code)
- For best QR code display, adjust terminal font size
- macOS Terminal.app might require enabling "Use Option as Meta key" for some keyboard shortcuts
