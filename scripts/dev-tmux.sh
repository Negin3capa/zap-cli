#!/bin/bash
# Advanced development mode using tmux for split-pane workflow
# Runs WhatsApp service and cargo watch in separate panes

SESSION_NAME="zaptui-dev"

echo "🔧 Starting ZapTUI Advanced Development Mode"
echo "=============================================="

# Check if tmux is installed
if ! command -v tmux &> /dev/null; then
    echo "❌ tmux not installed. Install with:"
    echo "   sudo apt install tmux    # Debian/Ubuntu"
    echo "   sudo dnf install tmux    # Fedora"
    echo "   brew install tmux        # macOS"
    echo ""
    echo "💡 Alternatively, use ./scripts/dev.sh for basic dev mode"
    exit 1
fi

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "📦 Installing cargo-watch..."
    cargo install cargo-watch
fi

# Kill existing session if it exists
tmux kill-session -t $SESSION_NAME 2>/dev/null

echo "✨ Creating tmux session: $SESSION_NAME"
echo ""
echo "Layout:"
echo "  ┌─────────────────────────────────┐"
echo "  │  WhatsApp Service Logs          │"
echo "  ├─────────────────────────────────┤"
echo "  │  Cargo Watch (Auto-rebuild)     │"
echo "  ├─────────────────────────────────┤"
echo "  │  ZapTUI Application             │"
echo "  └─────────────────────────────────┘"
echo ""
echo "Controls:"
echo "  • Ctrl+B then arrow keys = Switch panes"
echo "  • Ctrl+B then d = Detach session"
echo "  • tmux attach -t $SESSION_NAME = Reattach"
echo "  • Ctrl+C in any pane = Stop that process"
echo ""

# Create new session
tmux new-session -d -s $SESSION_NAME

# Rename window
tmux rename-window -t $SESSION_NAME:0 'ZapTUI Dev'

# Pane 0: WhatsApp service
tmux send-keys -t $SESSION_NAME:0.0 'cd whatsapp-service && echo "🚀 WhatsApp Service" && npm start' C-m

# Split horizontally for cargo watch
tmux split-window -h -t $SESSION_NAME:0
tmux send-keys -t $SESSION_NAME:0.1 'echo "⚙️  Cargo Watch - Auto-rebuilding on file changes..." && sleep 3 && cargo watch -x "build --release" -s "echo \"\" && echo \"✅ Build complete! Restart ZapTUI to see changes\" && echo \"\""' C-m

# Split bottom pane for running the app
tmux split-window -v -t $SESSION_NAME:0.1
tmux send-keys -t $SESSION_NAME:0.2 'echo "" && echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" && echo "🎯 RECOMMENDED WORKFLOW:" && echo "" && echo "1. Edit code in your editor" && echo "2. Watch middle pane for build completion" && echo "3. Run the app here:" && echo "   ./target/release/zaptui" && echo "" && echo "4. Stop with Ctrl+C, edit, repeat!" && echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" && echo "" && bash' C-m

# Adjust layout
tmux select-layout -t $SESSION_NAME:0 main-horizontal

# Attach to session
tmux attach-session -t $SESSION_NAME
