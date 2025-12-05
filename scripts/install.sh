#!/bin/bash
set -e

echo "🚀 Installing ZapTUI..."
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed"
    echo ""
    echo "Install Rust from: https://rustup.rs/"
    echo "Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed"
    echo ""
    echo "Install Node.js from: https://nodejs.org/"
    exit 1
fi

echo "✅ Dependencies check passed"
echo ""

# Build Rust binary
echo "📦 Building Rust binary (this may take a few minutes)..."
cargo build --release

# Install WhatsApp service
echo ""
echo "📦 Installing WhatsApp service..."
cd whatsapp-service
npm install --silent
cd ..

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Installation complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "To start ZapTUI, run:"
echo ""
echo "    ./zaptui"
echo ""
