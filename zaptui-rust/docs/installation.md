# Installation Guide

## Prerequisites

- **Node.js** 18+ (for WhatsApp service)
- **Rust** 1.70+ (for building)
- Linux, macOS, or WSL2

## Quick Install (Recommended)

```bash
cd zaptui-rust
./scripts/install.sh
```

This will:

1. Install Node.js dependencies
2. Build the Rust binary
3. Copy binary to `~/.cargo/bin/` (should be in your PATH)
4. Create default config at `~/.config/zaptui/`

## Manual Installation

### 1. Install Dependencies

```bash
cd whatsapp-service
npm install
cd ..
```

### 2. Build Rust Binary

```bash
cargo build --release
```

### 3. Install Binary

```bash
# Copy to a directory in your PATH
cp target/release/zaptui ~/.cargo/bin/
# or
sudo cp target/release/zaptui /usr/local/bin/
```

### 4. Create Config

```bash
mkdir -p ~/.config/zaptui
cp config.example.toml ~/.config/zaptui/config.toml
```

## Verify Installation

```bash
zaptui --version
```

## Running

Simply run from anywhere:

```bash
zaptui
```

The WhatsApp service will start automatically in the background.

## Troubleshooting

See [troubleshooting.md](troubleshooting.md) for common issues.
