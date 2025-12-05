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

### Prerequisites

- **Rust** (1.70+)
- **Node.js** (18+)
- **npm**

### Installation

```bash
# 1. Clone the repository
git clone https://github.com/Negin3capa/zaptui.git
cd zaptui

# 2. Run the installer
./install.sh
```

The installer will:

1. Check dependencies
2. Build the Rust binary
3. Install Node.js backend dependencies
4. Offer to add `zaptui` to your PATH

### Usage

If you added it to your PATH:

```bash
zaptui
```

Otherwise:

```bash
./zaptui
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

**"Port 8080 is already in use"**
Running `zaptui` again after a crash might show this. Run:

```bash
sudo lsof -ti:8080 | xargs kill -9
```

**"Binary not found"**
Run `./install.sh` to rebuild.

**Uninstall**
Run `make uninstall` (if you used Make) or:

```bash
rm -rf ~/.config/zaptui
rm ~/.cargo/bin/zaptui
```

## 🏗️ Architecture

ZapTUI uses a hybrid architecture:

- **Rust TUI**: Renders the interface and handles user input (Ratatui).
- **Node.js Service**: Wraps `whatsapp-web.js` to provide the WhatsApp API via WebSocket.

## 🤝 Contributing

Contributions are welcome! Pull requests and issues are appreciated.

## 📄 License

MIT License.
