# 📱 ZapTUI - WhatsApp for the Terminal

> A fast, lightweight WhatsApp TUI client built with Rust and Ratatui

![License](https://img.shields.io/badge/license-MIT-blue)
![Build](https://img.shields.io/badge/build-passing-green)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)

## ✨ Features

- 💬 **Full Messaging** - Send and receive text messages
- 🔒 **QR Authentication** - Secure login via QR code
- 📜 **Chat History** - Browse and scroll through message history
- ⚡ **Instant Navigation** - Lightning-fast chat switching
- 🎨 **Theme Support** - Adapts to your terminal colors (Kitty, Alacritty, etc.)
- ⌨️ **Keyboard-Driven** - Navigate with vim-style keys or arrows
- 🔍 **Focus System** - Tab through Chat List, Messages, and Input
- 🚀 **Lightweight** - Minimal resource usage
- 🌍 **Cross-Platform** - Linux, macOS, WSL2

## 📸 Screenshots

_Coming soon - help us by contributing screenshots!_

## 📦 Quick Install

### Prerequisites

- **Node.js** 18+ (for WhatsApp Web.js service)
- **Rust** 1.70+ (for building)

### One-Command Install

```bash
cd zaptui-rust
./scripts/install.sh
```

This will:

1. Install Node.js dependencies
2. Build the optimized Rust binary
3. Copy `zaptui` to `~/.cargo/bin/` (must be in your PATH)
4. Create config at `~/.config/zaptui/config.toml`

### Verify Installation

```bash
zaptui --version
```

## 🚀 Usage

Just run from anywhere:

```bash
zaptui
```

### First Run

1. **Scan QR Code** - Open WhatsApp on your phone → Settings → Linked Devices → Link a Device
2. **Scan the code** displayed in the terminal
3. **Start chatting!**

The WhatsApp service starts automatically in the background.

## ⌨️ Keyboard Shortcuts

| Key        | Action                                     |
| ---------- | ------------------------------------------ |
| **Tab**    | Cycle focus (Chat List → Messages → Input) |
| **j / ↓**  | Navigate down                              |
| **k / ↑**  | Navigate up                                |
| **Enter**  | Send message (when typing)                 |
| **Esc**    | Clear input                                |
| **Ctrl+C** | Quit                                       |

### Focus States

- **Chat List** (default) - Navigate chats with j/k or arrows
- **Message View** - Scroll through messages with arrows
- **Input** - Type and send messages

_Any typing automatically focuses the input box_

## ⚙️ Configuration

Config location: `~/.config/zaptui/config.toml`

```toml
[ui]
theme = "terminal"  # Uses your terminal's color scheme

[whatsapp]
service_url = "ws://localhost:8080"
```

See [docs/configuration.md](docs/configuration.md) for all options.

## 🛠️ Advanced

### Manual Installation

See [docs/installation.md](docs/installation.md)

### Troubleshooting

See [docs/troubleshooting.md](docs/troubleshooting.md)

### Cleanup

If you encounter issues, use:

```bash
./scripts/cleanup.sh
```

## 🗺️ Roadmap

### Priority 1 ✅ COMPLETE

- [x] QR authentication
- [x] Send/receive messages
- [x] Chat list
- [x] Message history
- [x] Session persistence
- [x] Terminal theme support
- [x] Focus system

### Priority 2 (Planned)

- [ ] Multi-line messages (Shift+Enter)
- [ ] Media viewing (images/videos)
- [ ] Search in chats
- [ ] Desktop notifications
- [ ] Last message previews
- [ ] Unread message counts

### Priority 3 (Future)

- [ ] Group chat management
- [ ] Contact management
- [ ] Emoji picker
- [ ] Package for AUR, Homebrew, Apt

## 🏗️ Architecture

ZapTUI uses a hybrid architecture:

- **Rust TUI** (`zaptui`) - Fast, responsive terminal interface
- **Node.js Service** - WhatsApp Web.js wrapper with WebSocket API
- **Communication** - Async WebSocket (ws://localhost:8080)

This combines Rust's performance with the mature WhatsApp Web.js library.

## 🤝 Contributing

Contributions are welcome! Areas that need help:

- 📸 Screenshots and demo GIFs
- 🐛 Bug reports and fixes
- ✨ New features from the roadmap
- 📚 Documentation improvements
- 🎨 Theme customization

## 📄 License

MIT License - see [LICENSE](LICENSE) for details

## 🙏 Acknowledgments

Built with:

- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [WhatsApp Web.js](https://github.com/pedroslopez/whatsapp-web.js) - WhatsApp API
- Inspired by [Discordo](https://github.com/ayn2op/discordo) and [spotify-player](https://github.com/aome510/spotify-player)

---

<p align="center">
  Made with ❤️ and ☕
</p>
