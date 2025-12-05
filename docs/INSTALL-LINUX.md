# ZapTUI Linux Installation Guide

ZapTUI supports a wide range of Linux distributions. This guide provides installation instructions for the most popular ones.

## Supported Distributions

- Ubuntu / Debian / Linux Mint / Pop!\_OS
- Fedora / RHEL / CentOS
- Arch Linux / Manjaro
- openSUSE / SUSE
- Gentoo
- And any other distribution with Rust and Node.js support

## Prerequisites

### Ubuntu / Debian / Pop!\_OS / Linux Mint

```bash
# Update package list
sudo apt update

# Install build essentials
sudo apt install curl build-essential

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Node.js (LTS)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install nodejs

# Verify
cargo --version
node --version
npm --version
```

### Fedora / RHEL / CentOS

```bash
# Install development tools
sudo dnf groupinstall 'Development Tools'

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Node.js
sudo dnf install nodejs npm

# Verify
cargo --version
node --version
npm --version
```

### Arch Linux / Manjaro

```bash
# Install all dependencies
sudo pacman -S base-devel rust nodejs npm

# Verify
cargo --version
node --version
npm --version
```

### openSUSE / SUSE

```bash
# Install development tools
sudo zypper install -t pattern devel_basis

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Node.js
sudo zypper install nodejs npm

# Verify
cargo --version
node --version
npm --version
```

### Gentoo

```bash
# Install Rust and Node.js
sudo emerge --ask dev-lang/rust net-libs/nodejs

# Verify
cargo --version
node --version
npm --version
```

## Installation

The installation process is the same for all distributions:

1. Clone the repository:

   ```bash
   git clone https://github.com/Negin3capa/zaptui.git
   cd zaptui
   ```

2. Run the installer:

   ```bash
   chmod +x install.sh
   ./install.sh
   ```

3. The installer will:
   - Detect your Linux distribution
   - Check for dependencies
   - Build the Rust binary
   - Install Node.js service dependencies
   - Offer installation options

4. Choose installation option:
   - **Option 3** (Recommended): Global install to `~/.local/share/zaptui`

## Running ZapTUI

### If installed globally:

```bash
zaptui
```

Ensure `~/.cargo/bin` is in your PATH. Add to `~/.bashrc` or `~/.zshrc` if needed:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### From project directory:

```bash
./zaptui
```

## Troubleshooting

### Port Already in Use

If port 8080 is already in use:

```bash
sudo lsof -ti:8080 | xargs kill -9
```

If `lsof` is not installed:

```bash
# Ubuntu/Debian
sudo apt install lsof

# Fedora/RHEL
sudo dnf install lsof

# Arch
sudo pacman -S lsof
```

### Missing Dependencies

If the installer reports missing dependencies, refer to the Prerequisites section for your distribution.

### Permission Denied

Ensure the scripts are executable:

```bash
chmod +x install.sh
chmod +x zaptui
```

### Command Not Found After Install

Ensure `~/.cargo/bin` is in your PATH:

```bash
echo $PATH | grep cargo
```

If not found, add to your shell config (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Then reload:

```bash
source ~/.bashrc  # or ~/.zshrc
```

## File Locations

- **Installation**: `~/.local/share/zaptui`
- **Config**: `~/.config/zaptui/config.toml`
- **WhatsApp Auth**: `~/.local/share/zaptui/.wwebjs_auth`
- **Launcher**: `~/.cargo/bin/zaptui`

## Uninstalling

Run the uninstaller:

```bash
./scripts/uninstall.sh
```

Or manually remove:

```bash
rm -rf ~/.local/share/zaptui
rm -rf ~/.config/zaptui
rm ~/.cargo/bin/zaptui
```

## Tips

- Use a modern terminal emulator for best results (Alacritty, Kitty, GNOME Terminal, Konsole, etc.)
- Ensure your terminal supports Unicode for proper QR code display
- Recommended fonts: Fira Code, JetBrains Mono, Cascadia Code
- Some distributions may require additional packages for full Unicode support
