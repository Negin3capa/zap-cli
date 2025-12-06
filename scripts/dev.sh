#!/bin/bash
# Development mode for ZapTUI with auto-reload
# Watches for file changes and automatically rebuilds

echo "🔧 Starting ZapTUI in Development Mode"
echo "========================================"
echo ""

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "📦 Installing cargo-watch (one-time setup)..."
    cargo install cargo-watch
    echo ""
fi

# Check if nodemon is installed for the service
if ! command -v nodemon &> /dev/null; then
    echo "📦 nodemon not found. Install it for auto-reload of WhatsApp service:"
    echo "    npm install -g nodemon"
    echo ""
    echo "For now, starting service normally..."
    USE_NODEMON=false
else
    USE_NODEMON=true
fi

# Clean up port 8080 if needed
if lsof -ti:8080 >/dev/null; then
    echo "🧹 Cleaning up port 8080 (killing old process)..."
    lsof -ti:8080 | xargs kill -9 2>/dev/null || true
fi

# Start WhatsApp service in background
echo "🚀 Starting WhatsApp service..."
cd whatsapp-service

if [ "$USE_NODEMON" = true ]; then
    nodemon server.js &
else
    npm start &
fi

SERVICE_PID=$!
echo "   Service PID: $SERVICE_PID"
cd ..
sleep 3

echo ""
echo "👀 Watching for changes..."
echo "   Rust files: Auto-rebuild and restart"
echo "   Press Ctrl+C to stop"
echo ""
echo "========================================"
echo ""

# Function to cleanup on exit
cleanup() {
    echo ""
    echo "🛑 Stopping development mode..."
    kill $SERVICE_PID 2>/dev/null || true
    pkill -P $$ 2>/dev/null || true
    exit 0
}

trap cleanup SIGINT SIGTERM

# Watch Rust files and auto-rebuild
cargo watch -x 'build --release' -s 'echo "" && echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" && echo "✅ BUILD COMPLETE!" && echo "" && echo "Run the app in another terminal:" && echo "    ./target/release/zaptui" && echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" && echo ""'
