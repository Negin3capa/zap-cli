# Development Workflow - Complete Setup Summary

## ✅ What's Been Created

### Scripts

1. **`scripts/restart.sh`** - Quick restart with rebuild
2. **`scripts/dev.sh`** - Auto-rebuild development mode
3. **`scripts/dev-tmux.sh`** - Advanced tmux development environment

### Makefile Targets

- `make restart` - Stop, rebuild, ready to run
- `make dev` - Auto-rebuild on file changes
- `make dev-tmux` - Multi-pane dev environment
- `make watch-check` - Fast syntax checking

### Documentation

1. **`docs/development-workflow.md`** - Complete workflow guide
2. **`docs/dev-quick-ref.md`** - Quick reference card
3. **`docs/real-time-sync.md`** - Real-time sync features
4. **`docs/sync-before-after.md`** - Before/after comparison
5. **`docs/chat-loading-timeout-fix.md`** - Timeout fix details

---

## 🚀 How to Use

### Quickest Way to Restart

```bash
make restart
./zaptui
```

### Development Mode (Recommended)

```bash
make dev
# Edit code in your editor
# Automatically rebuilds on save
# Test with: ./target/release/zaptui
```

### Pro Development Mode

```bash
make dev-tmux
# Multi-pane layout:
#   Top: WhatsApp service logs
#   Middle: Auto-rebuild output
#   Bottom: Run/test your app
```

---

## 📋 Quick Reference

### Restart Commands

| Command           | Description              |
| ----------------- | ------------------------ |
| `make restart`    | Stop all, rebuild, ready |
| `./zaptui`        | Run the updated app      |
| `pkill -f zaptui` | Emergency stop           |

### Development Commands

| Command            | Description      | Best For    |
| ------------------ | ---------------- | ----------- |
| `make dev`         | Auto-rebuild     | Daily dev   |
| `make dev-tmux`    | Tmux environment | Debugging   |
| `make watch-check` | Fast checking    | Syntax only |

### Tmux Controls

| Keys                        | Action         |
| --------------------------- | -------------- |
| `Ctrl+B` → arrows           | Switch panes   |
| `Ctrl+B` → `d`              | Detach session |
| `Ctrl+C`                    | Stop current   |
| `tmux attach -t zaptui-dev` | Reattach       |

---

## 🎯 Common Scenarios

### Scenario 1: Quick Code Change

```bash
# Edit file
vim src/ui/app.rs

# Restart and test
make restart && ./zaptui
```

### Scenario 2: Working on a Feature

```bash
# Terminal 1
make dev

# Terminal 2 (after builds complete)
./target/release/zaptui
# Test, stop, repeat
```

### Scenario 3: Debugging Complex Issues

```bash
make dev-tmux
# See everything at once:
# - Service logs (top)
# - Build output (middle)
# - App runtime (bottom)
```

---

## 🔧 Setup Requirements

### Already Have

- ✅ Rust/Cargo
- ✅ Node.js/npm
- ✅ Scripts are executable

### Optional Installs

```bash
# For auto-rebuild (installed automatically by dev.sh)
cargo install cargo-watch

# For tmux mode
sudo apt install tmux    # Linux
brew install tmux        # macOS

# For service auto-reload (optional)
npm install -g nodemon
```

---

## 📖 Full Documentation

- **Quick Start**: See `docs/dev-quick-ref.md`
- **Complete Guide**: See `docs/development-workflow.md`
- **Makefile Help**: Run `make help`

---

## 💡 Pro Tips

1. **Faster Iteration**: Use `make watch-check` while coding, `make build` when ready to test

2. **Keep Tmux Running**: Detach with `Ctrl+B d`, reattach anytime with `tmux attach -t zaptui-dev`

3. **Multiple Workspaces**: Run `make dev` in one terminal, test in another

4. **Debug Logging**: `RUST_LOG=debug ./target/release/zaptui`

---

## 🎉 You're All Set!

You now have a professional development workflow with:

- ✅ One-command restart
- ✅ Auto-rebuild on save
- ✅ Multi-pane development environment
- ✅ Full documentation

**Start coding with:**

```bash
make dev-tmux
```

Happy developing! 🚀
