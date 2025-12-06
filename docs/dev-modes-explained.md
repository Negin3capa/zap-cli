# Development Modes Explained

## Why Can't TUI Apps Auto-Run?

**Important:** Terminal User Interface (TUI) applications like ZapTUI **cannot be automatically restarted** by build watchers like cargo-watch. Here's why:

- TUIs need **direct terminal control** (raw mode) to capture keyboard input and render the interface
- When run through cargo-watch's shell execution, the terminal is already controlled by cargo-watch
- stdin/stdout are redirected/piped, preventing proper TUI operation
- The TUI can't enter raw mode and fails to display

This is a fundamental limitation, not a bug. **You must run TUI apps manually** after builds complete.

---

## The 2 Development Modes

### 1. `make dev` - Watch & Build Only

**What it does:**

- ✅ Starts WhatsApp service
- ✅ Watches your Rust files
- ✅ Auto-rebuilds when you save
- ❌ Does NOT run the app (you run manually)

**You need TWO terminals:**

```bash
# Terminal 1
make dev

# Terminal 2
./target/release/zaptui
# (restart manually after each build)
```

**Best for:**

- When you want control over when the app starts
- Testing different command-line arguments
- Checking build output before running

---

### 2. `make dev-tmux` - Multi-Pane Pro Mode ⭐ **RECOMMENDED**

**What it does:**

- ✅ Creates 3-pane tmux layout
- ✅ Top pane: WhatsApp service (auto-running)
- ✅ Middle pane: Build watcher (auto-rebuilding)
- ✅ Bottom pane: Your terminal (run app here)

**Layout:**

```
┌─────────────────────────────────┐
│  WhatsApp Service Logs          │
├─────────────────────────────────┤
│  Cargo Watch (Rebuilding)       │
├─────────────────────────────────┤
│  Run: ./target/release/zaptui   │
└─────────────────────────────────┘
```

**Only need ONE terminal (tmux session):**

```bash
make dev-tmux
# Navigate panes: Ctrl+B then arrow keys
# Run app manually in bottom pane when ready
```

**Best for:**

- Complex debugging
- Monitoring service logs and build output simultaneously
- Professional development setup
- When you want to see everything at once

---

## Quick Comparison

| Feature              | `make dev`     | `make dev-tmux` |
| -------------------- | -------------- | --------------- |
| Auto-rebuild         | ✅ Yes         | ✅ Yes          |
| Auto-run app         | ❌ No (manual) | ❌ No (manual)  |
| Terminals needed     | 2              | 1 (tmux)        |
| Service logs visible | ⚠️ Background  | ✅ Top pane     |
| Build output visible | ✅ Yes         | ✅ Middle pane  |
| App restarts         | Manual         | Manual          |
| Complexity           | Low            | Medium          |
| Best for             | Simple setup   | Full visibility |

---

## Which Should You Use?

### For Most Development → `make dev-tmux` ⭐ **RECOMMENDED**

```bash
make dev-tmux
# See everything: service logs, builds, run app
# All in one organized view!
```

### For Simple 2-Terminal Workflow → `make dev`

```bash
# Terminal 1
make dev

# Terminal 2
./target/release/zaptui
# Run when YOU decide
```

---

## Examples

### Example 1: Typical Development Session (dev-tmux)

```bash
make dev-tmux

# Edit code in your editor
# Watch middle pane for build completion
# Run in bottom pane: ./target/release/zaptui
# View service logs in top pane if needed
# Ctrl+C to stop, edit, run again!
```

### Example 2: Testing Different Scenarios (dev)

```bash
# Terminal 1
make dev

# Terminal 2 - Test scenario 1
./target/release/zaptui

# Close app, test scenario 2
./target/release/zaptui --help

# Close app, test scenario 3
RUST_LOG=debug ./target/release/zaptui
```

---

## FAQ

### Q: Why can't TUI apps auto-restart?

**A:** TUIs need direct terminal control (raw mode). When run through cargo-watch, the terminal is already controlled, so the TUI can't capture input or render properly. This is a fundamental limitation of how TUI apps work.

### Q: Why does `make dev` need two terminals?

**A:** It only watches and rebuilds - it doesn't run the app. This gives you complete control over when to start/restart the app.

### Q: Which is best for daily development?

**A:** `make dev-tmux` - you get service logs, build output, and run the app all in one organized view.

### Q: I don't have tmux installed, what should I use?

**A:** Use `make dev` with two terminals. Or install tmux - it's worth it! (`sudo apt install tmux` on Linux)

### Q: Can I see service logs with `make dev`?

**A:** The service runs in the background. To see logs, either:

- Use `make dev-tmux` (recommended)
- Run service manually in a separate terminal

---

## Pro Tips

### Quick Iteration (dev-tmux)

```bash
make dev-tmux

# Bottom pane: Run app
./target/release/zaptui

# Edit code in your editor
# Middle pane shows: "✅ BUILD COMPLETE!"
# Bottom pane: Ctrl+C, run again
./target/release/zaptui
```

### Maximum Visibility (dev-tmux)

```bash
make dev-tmux

# Ctrl+B → ↑  = View service logs
# Ctrl+B → ↓  = View build output
# Ctrl+B → ↓  = Run app
```

### Simple Workflow (dev)

```bash
# Terminal 1: Watch mode
make dev

# Terminal 2: Run when ready
./target/release/zaptui

# Stop app: Ctrl+C
# Edit code
# Wait for: "✅ BUILD COMPLETE!" in terminal 1
# Start app again in terminal 2
```

---

## Summary

**TL;DR:**

- 🔍 **Best overall** → `make dev-tmux` ⭐ (RECOMMENDED)
- 🎮 **Simple 2-terminal** → `make dev`
- ❌ **Auto-run?** → Not possible for TUI apps

**Most people should use: `make dev-tmux`** - it provides the best development experience!
