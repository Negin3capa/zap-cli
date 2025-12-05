#!/bin/bash
# Comprehensive cleanup script that handles all cases

echo "🧹 Comprehensive cleanup..."

# 1. Kill any zaptui processes
pkill -f "zaptui" 2>/dev/null && echo "✅ Killed zaptui processes" || echo "ℹ️  No zaptui processes"

# 2. Kill node server.js processes
pkill -f "node.*server.js" 2>/dev/null && echo "✅ Killed node server.js" || echo "ℹ️  No node server.js running"

# 3. Force kill anything on port 8080
if lsof -Pi :8080 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "⚠️  Killing process on port 8080..."
    lsof -ti:8080 | xargs kill -9 2>/dev/null
    echo "✅ Port 8080 freed"
else
    echo "✅ Port 8080 already free"
fi

# 4. Wait a moment
sleep 1

# 5. Verify port is free
if lsof -Pi :8080 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "❌ WARNING: Port 8080 still in use!"
    lsof -i :8080
else
    echo "✅ Verified: Port 8080 is free"
fi

echo ""
echo "✨ Cleanup complete! You can now run ./zaptui"
