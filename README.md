# 📱 ZapTUI - WhatsApp for the Terminal

> A fast, lightweight WhatsApp TUI client built with Rust and Ratatui.

![License](https://img.shields.io/badge/license-MIT-blue)
![Build](https://img.shields.io/badge/build-passing-green)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)

## ✨ Features

- 💬 **Full Messaging** - Send and receive text messages
- 🔒 **QR Authentication** - Secure login via QR code
- 📜 **Chat History** - Browse and scroll through message history
- ⚡ **Instant Navigation** - Lightning-fast chat switching
- 🎨 **Theme Support** - Adapts to your terminal colors
- ⌨️ **Keyboard-Driven** - Vim-style keys or arrows
- 🚀 **Lightweight** - Minimal resource usage

## 🚀 Quick Start

### Supported Platforms

- **Linux** (Ubuntu, Debian, Fedora, Arch, openSUSE, and more)
- **macOS** (10.15+)
- **Windows** (10/11)

### Prerequisites

All platforms require:

- **Rust** (1.70+) - [Install from rustup.rs](https://rustup.rs/)
- **Node.js** (18+) - [Install from nodejs.org](https://nodejs.org/)
- **npm** (comes with Node.js)

### Installation

#### Linux

```bash
# 1. Clone the repository
git clone https://github.com/Negin3capa/zaptui.git
cd zaptui

# 2. Run the installer
chmod +x install.sh
./install.sh
```

The installer will:

1. Detect your Linux distribution
2. Provide distro-specific dependency installation commands if needed
3. Build the Rust binary
4. Install Node.js backend dependencies
5. Offer to add `zaptui` to your PATH

**Supported distros:** Ubuntu, Debian, Fedora, RHEL, CentOS, Arch, Manjaro, openSUSE, Gentoo, and more.

#### macOS

```bash
# 1. Clone the repository
git clone https://github.com/Negin3capa/zaptui.git
cd zaptui

# 2. Run the macOS installer
chmod +x install-macos.sh
./install-macos.sh
```

The installer follows macOS conventions and installs to `~/Library/Application Support/zaptui`.

#### Windows

```powershell
# 1. Clone the repository
git clone https://github.com/Negin3capa/zaptui.git
cd zaptui

# 2. Run the PowerShell installer
.\install.ps1
```

**Note:** You may need to allow script execution:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

The installer will place files in `%LOCALAPPDATA%\zaptui` and optionally add to PATH.

### Usage

After installation, run from anywhere:

```bash
zaptui        # On all platforms (if added to PATH)
```

Or from the project directory:

```bash
./zaptui              # Linux
./scripts/zaptui-macos    # macOS
.\scripts\zaptui.bat      # Windows
```

**First Run:**

1. Open WhatsApp on your phone
2. Go to **Settings** -> **Linked Devices** -> **Link a Device**
3. Scan the QR code displayed in the terminal

## ⌨️ Controls

| Key                | Action                                    |
| ------------------ | ----------------------------------------- |
| `Tab`              | Switch focus (Chats -> Messages -> Input) |
| `j`/`k` or `↓`/`↑` | Navigate lists                            |
| `Enter`            | Send message (in input) / Select chat     |
| `Esc`              | Clear input / Unfocus                     |
| `Ctrl+C`           | Quit                                      |

## ⚙️ Configuration

Configuration is stored at `~/.config/zaptui/config.toml`. It is automatically created on first run.

```toml
[ui]
theme = "terminal"

[whatsapp]
service_url = "ws://localhost:8080"
```

## 🛠️ Troubleshooting

### Linux & macOS

**"Port 8080 is already in use"**

Running `zaptui` again after a crash might show this. Run:

```bash
# Linux
sudo lsof -ti:8080 | xargs kill -9

# macOS
lsof -ti:8080 | xargs kill -9
```

**"Binary not found"**

Run the installer again:

```bash
./install.sh         # Linux
./install-macos.sh   # macOS
```

### Windows

**"Port 8080 is already in use"**

In PowerShell as Administrator:

```powershell
Get-NetTCPConnection -LocalPort 8080 | ForEach-Object { Stop-Process -Id $_.OwningProcess -Force }
```

**"Running scripts is disabled"**

Enable PowerShell script execution:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

**"Binary not found"**

Run the installer again:

```powershell
.\install.ps1
```

### All Platforms

**Uninstall**

```bash
# Linux
./scripts/uninstall.sh

# macOS
./scripts/uninstall-macos.sh
```

```powershell
# Windows
.\scripts\uninstall.ps1
```

Or manually:

- Linux: `rm -rf ~/.local/share/zaptui ~/.config/zaptui ~/.cargo/bin/zaptui`
- macOS: `rm -rf ~/Library/Application\ Support/zaptui ~/.config/zaptui /usr/local/bin/zaptui`
- Windows: Remove `%LOCALAPPDATA%\zaptui` and `%APPDATA%\zaptui`

## 🏗️ Architecture

ZapTUI uses a hybrid architecture:

- **Rust TUI**: Renders the interface and handles user input (Ratatui).
- **Node.js Service**: Wraps `whatsapp-web.js` to provide the WhatsApp API via WebSocket.

## 🤝 Contributing

Contributions are welcome! Pull requests and issues are appreciated.

## 📄 License

MIT License.
