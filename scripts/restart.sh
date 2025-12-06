#!/bin/bash
# Restart script for ZapTUI
# This script stops the running app, rebuilds, and starts fresh

echo "🔄 Restarting ZapTUI..."

# Stop any running instances
echo "Stopping running instances..."
pkill -f "zaptui" || true
pkill -f "whatsapp-service" || true
sleep 2

# Rebuild the application
echo "Building latest version..."
cargo build --release

# Check if build succeeded
if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"
echo ""
echo "To start the application:"
echo "  ./zaptui"
echo ""
echo "Or to run in development mode:"
echo "  ./scripts/dev.sh"
