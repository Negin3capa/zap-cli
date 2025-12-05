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

## 📦 Installation

### Prerequisites

Before installing ZapTUI, ensure you have the following dependencies:

#### 1. **Rust** (version 1.70 or higher)

Install via `rustup` (recommended):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify installation:

```bash
rustc --version
cargo --version
```

#### 2. **Node.js** (version 18 or higher)

Install via your package manager:

**Ubuntu/Debian:**

```bash
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```

**macOS (Homebrew):**

```bash
brew install node
```

**Arch Linux:**

```bash
sudo pacman -S nodejs npm
```

Or download from [nodejs.org](https://nodejs.org/)

Verify installation:

```bash
node --version
npm --version
```

#### 3. **Build Tools**

**Linux:**

```bash
# Debian/Ubuntu
sudo apt-get install build-essential

# Arch Linux
sudo pacman -S base-devel
```

**macOS:**

```bash
xcode-select --install
```

### Installing ZapTUI

#### Step 1: Clone the Repository

```bash
git clone https://github.com/Negin3capa/zaptui.git
cd zaptui
```

#### Step 2: Install Node.js Dependencies

```bash
npm install
```

This installs the WhatsApp Web.js library and other Node.js dependencies required for the backend service.

#### Step 3: Build the Rust TUI

```bash
cd zaptui-rust
cargo build --release
```

This will compile the Rust terminal interface. The first build may take several minutes as it downloads and compiles all dependencies.

#### Step 4: Install Globally (Optional but Recommended)

To make `zaptui` available from anywhere:

```bash
# Install the binary to your PATH
cp target/release/zaptui ~/.cargo/bin/

# Or copy to system-wide location
sudo cp target/release/zaptui /usr/local/bin/
```

**Note:** Make sure `~/.cargo/bin` is in your PATH. Add this to your `~/.bashrc` or `~/.zshrc` if needed:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

#### Step 5: Create Configuration Directory

```bash
mkdir -p ~/.config/zaptui
```

The config file will be created automatically on first run, or you can copy the example:

```bash
cp config.example.toml ~/.config/zaptui/config.toml
```

### Quick Install Script

Alternatively, use the automated install script:

```bash
# From the project root directory
cd zaptui-rust
./scripts/install.sh
```

**Important:** The script must be run from inside the `zaptui-rust` directory, not the project root.

### Verify Installation

Check that ZapTUI is installed correctly:

```bash
zaptui --version
```

You should see output like: `zaptui 2.0.0`

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

## � Troubleshooting

### "file not found" when running install script

**Problem:** Running `./scripts/install.sh` from the project root gives "No such file or directory"

**Solution:** The install script is located in `zaptui-rust/scripts/`, not at the project root. Run:

```bash
cd zaptui-rust
./scripts/install.sh
```

### Command not found: zaptui

**Problem:** After installation, running `zaptui` gives "command not found"

**Solutions:**

1. Ensure `~/.cargo/bin` is in your PATH:
   ```bash
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

2. Or copy to a system-wide location:
   ```bash
   sudo cp zaptui-rust/target/release/zaptui /usr/local/bin/
   ```

### Node.js dependencies fail to install

**Problem:** `npm install` fails with permission errors

**Solution:** Don't use `sudo` with npm. If you have permission issues:

```bash
npm config set prefix ~/.npm-global
echo 'export PATH=$HOME/.npm-global/bin:$PATH' >> ~/.bashrc
source ~/.bashrc
```

### Rust compilation errors

**Problem:** `cargo build` fails

**Solutions:**

1. Update Rust to the latest version:
   ```bash
   rustup update
   ```

2. Ensure you have build tools installed (see Prerequisites section above)

### WhatsApp service won't connect

**Problem:** TUI starts but shows "Connecting..." indefinitely

**Solution:** The Node.js backend service should auto-start, but if it doesn't:

1. Check if the service is running:
   ```bash
   ps aux | grep node
   ```

2. Manually test the service:
   ```bash
   node index.js
   ```

3. Check logs for errors

### QR code not displaying properly

**Problem:** QR code appears garbled or unreadable

**Solutions:**

1. Ensure your terminal supports Unicode
2. Try a different terminal emulator (Kitty, Alacritty, or WezTerm work well)
3. Increase terminal window size

### Clean installation

If you encounter persistent issues, clean and reinstall:

```bash
# From project root
cd zaptui-rust
./scripts/cleanup.sh

# Then reinstall
rm -rf target/
cargo clean
cargo build --release

# Reinstall Node dependencies
cd ..
rm -rf node_modules/
npm install
```

For more troubleshooting help, see [docs/troubleshooting.md](docs/troubleshooting.md) or [open an issue](https://github.com/Negin3capa/zaptui/issues).

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
