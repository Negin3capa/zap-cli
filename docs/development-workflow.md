# Development Workflow Guide

## Quick Start

### Restart with Updates

```bash
# Quick restart with latest changes
./scripts/restart.sh

# Then run the app
./zaptui
```

### Development Mode (Auto-Reload)

```bash
# Basic dev mode with auto-rebuild
./scripts/dev.sh

# Advanced dev mode with tmux (recommended)
./scripts/dev-tmux.sh
```

---

## Production: Restart with Updates

### Step-by-Step

1. **Stop running instances:**

   ```bash
   pkill -f zaptui
   pkill -f whatsapp-service
   ```

2. **Rebuild the application:**

   ```bash
   cargo build --release
   ```

3. **Start the application:**
   ```bash
   ./zaptui
   ```

### Or Use the Script

```bash
./scripts/restart.sh  # Stops, rebuilds, ready to run
./zaptui             # Start the updated app
```

---

## Development Mode Options

### Option 1: Basic Dev Mode (Simple)

**What it does:**

- Auto-rebuilds when Rust files change
- Runs WhatsApp service in background
- Simple single-terminal workflow

**Usage:**

```bash
./scripts/dev.sh
```

**Features:**

- ✅ Auto-detects and installs `cargo-watch`
- ✅ Starts WhatsApp service automatically
- ✅ Watches for file changes
- ✅ Rebuilds on save
- ✅ Clean exit with Ctrl+C

**Workflow:**

1. Edit code in your editor
2. Save file
3. cargo-watch rebuilds automatically
4. Run `./target/release/zaptui` to test
5. Repeat

**First-time setup:**

```bash
# cargo-watch will be installed automatically
# But you can install it manually:
cargo install cargo-watch

# Optional: Install nodemon for service auto-reload
npm install -g nodemon
```

---

### Option 2: Advanced Tmux Mode (Pro)

**What it does:**

- Creates 3-pane tmux layout
- Top: WhatsApp service logs
- Middle: Cargo watch auto-rebuild
- Bottom: Run your app
- All visible at once!

**Usage:**

```bash
./scripts/dev-tmux.sh
```

**Layout:**

```bash
┌─────────────────────────────────┐
│  WhatsApp Service Logs          │
│  (npm start output)             │
├─────────────────────────────────┤
│  Cargo Watch                    │
│  (Auto-rebuild on file change)  │
├─────────────────────────────────┤
│  ZapTUI Application             │
│  (Run ./target/release/zaptui)  │
└─────────────────────────────────┘
```

**Tmux Controls:**

- `Ctrl+B` then arrow keys = Switch between panes
- `Ctrl+B` then `d` = Detach (session keeps running)
- `tmux attach -t zaptui-dev` = Reattach to session
- `Ctrl+C` in any pane = Stop that process
- `Ctrl+D` or `exit` = Close pane

**First-time setup:**

```bash
# Install tmux
sudo apt install tmux      # Debian/Ubuntu
sudo dnf install tmux      # Fedora
brew install tmux          # macOS

# Install cargo-watch (done automatically)
cargo install cargo-watch
```

**Workflow:**

1. Start tmux dev mode: `./scripts/dev-tmux.sh`
2. Top pane shows service logs (auto-running)
3. Middle pane shows build output (auto-building)
4. Bottom pane is where you run the app
5. Edit code → Auto-rebuild → Test → Repeat
6. All logs visible at once!

---

## Comparison

| Feature            | restart.sh    | dev.sh    | dev-tmux.sh |
| ------------------ | ------------- | --------- | ----------- |
| Auto-rebuild       | ❌ No         | ✅ Yes    | ✅ Yes      |
| Service auto-start | ❌ No         | ✅ Yes    | ✅ Yes      |
| Split-pane view    | ❌ No         | ❌ No     | ✅ Yes      |
| Visible logs       | ❌ No         | ⚠️ Some   | ✅ All      |
| Complexity         | Low           | Medium    | High        |
| Best for           | Quick restart | Daily dev | Power users |

---

## Development Workflow Recommendations

### For Quick Fixes

```bash
# Edit code
vim src/ui/app.rs

# Restart and test
./scripts/restart.sh
./zaptui
```

### For Active Development

```bash
# Start dev mode
./scripts/dev.sh

# In another terminal, test after each rebuild
watch -n 2 './target/release/zaptui --version'
```

### For Intense Development Sessions

```bash
# Start tmux dev environment
./scripts/dev-tmux.sh

# Edit code (cargo-watch rebuilds automatically)
# View service logs in top pane
# See build output in middle pane
# Test app in bottom pane
# All visible at once!
```

---

## Tips & Tricks

### Faster Rebuilds

```bash
# Use cargo check instead of build for faster feedback
cargo watch -x check

# Or watch specific files
cargo watch -w src/ui -x 'build --release'
```

### Debug Mode for Faster Compilation

```bash
# Development builds are faster but larger
cargo watch -x build  # No --release flag

# Run with:
./target/debug/zaptui
```

### Watch Node.js Service Too

```bash
# Install nodemon globally
npm install -g nodemon

# Run service with auto-reload
cd whatsapp-service
nodemon server.js
```

### Log Everything

```bash
# Enable debug logging
RUST_LOG=debug ./target/release/zaptui

# Or specific modules
RUST_LOG=zaptui::ui=trace ./target/release/zaptui
```

### Keep Session Running

```bash
# Start tmux dev mode
./scripts/dev-tmux.sh

# Detach with Ctrl+B, d
# Session keeps running in background

# Later, reattach
tmux attach -t zaptui-dev
```

---

## Troubleshooting

### "cargo-watch not found"

```bash
cargo install cargo-watch
```

### "tmux not found"

```bash
# Debian/Ubuntu
sudo apt install tmux

# macOS
brew install tmux
```

### "Port already in use"

```bash
# Kill all zaptui processes
pkill -f zaptui
pkill -f whatsapp-service

# Or specific port
lsof -ti:8080 | xargs kill -9
```

### "Build is slow"

```bash
# Use debug builds (faster)
cargo watch -x build

# Or use cargo check
cargo watch -x check
```

### "Service won't start"

```bash
# Check if port 8080 is free
lsof -i :8080

# Install service dependencies
cd whatsapp-service
npm install
```

---

## Real-World Examples

### Example 1: Quick Bug Fix

```bash
# 1. Stop current app
pkill -f zaptui

# 2. Fix the bug
vim src/ui/app.rs

# 3. Rebuild and restart
./scripts/restart.sh
./zaptui
```

### Example 2: Feature Development

```bash
# 1. Start tmux dev mode
./scripts/dev-tmux.sh

# 2. Edit feature in your editor
# 3. Watch middle pane for build completion
# 4. Test in bottom pane: ./target/release/zaptui
# 5. Check service logs in top pane if needed
# 6. Repeat steps 2-5
```

### Example 3: Testing Multiple Scenarios

```bash
# Terminal 1: Dev mode
./scripts/dev.sh

# Terminal 2: Test different configs
./target/release/zaptui
# Close and test again
./target/release/zaptui --help
```

---

## Summary

### Quick Reference

```bash
# Production restart
./scripts/restart.sh && ./zaptui

# Simple dev mode
./scripts/dev.sh

# Pro dev mode
./scripts/dev-tmux.sh

# Manual rebuild
cargo build --release
```

**Choose your workflow:**

- 🚀 **Quick fixes** → `restart.sh`
- ⚙️ **Daily development** → `dev.sh`
- 💪 **Power user** → `dev-tmux.sh`

Happy coding! 🎉
