# Development Quick Reference

## 🚀 Quick Commands

### Restart Application

```bash
make restart      # or ./scripts/restart.sh
./zaptui         # Start updated app
```

### Development Modes

```bash
make dev         # Auto-rebuild (basic)
make dev-tmux    # Auto-rebuild (tmux)
make watch-check # Fast feedback (no build)
```

---

## Common Scenarios

### "I changed some code, how do I test it?"

**Option A - Quick Test:**

```bash
make restart
./zaptui
```

**Option B - Keep Developing:**

```bash
make dev    # In one terminal
# Edit code, it rebuilds automatically
# Test in another terminal when ready
```

### "How do I see logs while developing?"

**Use tmux mode:**

```bash
make dev-tmux
```

You'll see:

- Top pane: WhatsApp service logs
- Middle pane: Build output
- Bottom pane: Run your app here

### "Build is too slow!"

**Use check instead of build:**

```bash
make watch-check
# Faster! Just checks for errors, doesn't build
```

---

## Development Workflow Examples

### Example 1: Quick Bug Fix

```bash
# 1. Edit the code
vim src/ui/app.rs

# 2. Test your fix
make restart && ./zaptui
```

### Example 2: Feature Development

```bash
# Start dev mode in terminal 1
make dev

# Edit code in your favorite editor
# Watch terminal 1 for build completion
# Test in terminal 2
./target/release/zaptui
```

### Example 3: Debug Session

```bash
# Use tmux for full visibility
make dev-tmux

# Layout:
# - Top: Service logs (troubleshoot API issues)
# - Middle: Build output (catch compile errors)
# - Bottom: Run app (test features)
```

---

## Keyboard Shortcuts

### Tmux Controls

```
Ctrl+B →  arrow     Switch panes
Ctrl+B →  d         Detach (keeps running)
Ctrl+B →  [         Scroll mode
Ctrl+C              Stop process in current pane
```

### Reattach to Tmux

```bash
tmux attach -t zaptui-dev
```

---

## Troubleshooting

### "Application won't stop"

```bash
pkill -f zaptui
```

### "Port 8080 already in use"

```bash
pkill -f whatsapp-service
# or
lsof -ti:8080 | xargs kill -9
```

### "cargo-watch not installed"

```bash
cargo install cargo-watch
```

### "Build failed"

```bash
# Check error in terminal
# Fix code
# It will auto-rebuild (if using dev mode)
```

---

## Cheat Sheet

| Command            | What It Does   | When To Use            |
| ------------------ | -------------- | ---------------------- |
| `make restart`     | Rebuild fresh  | After pulling changes  |
| `make dev`         | Auto-rebuild   | Daily development      |
| `make dev-tmux`    | Multi-pane dev | Complex debugging      |
| `make watch-check` | Fast checks    | Syntax checking        |
| `make build`       | Just build     | CI/CD or manual        |
| `make run`         | Run current    | Testing existing build |

---

## Makefile All Commands

```bash
make help          # Show all commands
make build         # Build release binary
make run           # Run the application
make install       # Install globally
make uninstall     # Remove installation
make clean         # Clean build artifacts
make restart       # Stop + rebuild
make dev           # Auto-rebuild mode
make dev-tmux      # Tmux dev environment
make watch-check   # Fast syntax checking
```

---

## Tips

💡 **Faster iteration:** Use `make watch-check` while coding, then `make build` when ready to test

💡 **Multiple terminals:** Run `make dev` in one, test in another

💡 **Best experience:** Use `make dev-tmux` for all-in-one development environment

💡 **Production builds:** Always use `make build` for final testing before release

---

## Pro Tips

### Auto-restart on successful build

```bash
cargo watch -x 'build --release' -s 'pkill zaptui; ./target/release/zaptui &'
# Builds and auto-restarts the app!
```

### Watch specific directory

```bash
cargo watch -w src/ui -x 'build --release'
# Only rebuilds when UI code changes
```

### Run tests on save

```bash
cargo watch -x test
# Auto-runs tests
```

### Format on save

```bash
cargo watch -x fmt -x check
# Auto-formats and checks
```

---

Remember: **`make help`** shows all available commands!
