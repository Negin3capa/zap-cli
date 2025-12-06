
# ZapTUI Makefile
# Note: For Windows, use build.ps1 instead

# Detect OS
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
    PLATFORM := linux
endif
ifeq ($(UNAME_S),Darwin)
    PLATFORM := macos
endif

.PHONY: all build install install-global uninstall clean run help restart dev dev-tmux watch-check

all: build

help:
	@echo "ZapTUI Build System"
	@echo ""
	@echo "Targets:"
	@echo "  make build           - Build the project"
	@echo "  make install         - Run installer (interactive)"
	@echo "  make install-global  - Install globally"
	@echo "  make uninstall       - Uninstall ZapTUI"
	@echo "  make clean           - Clean build artifacts"
	@echo "  make run             - Build and run"
	@echo ""
	@echo "Development:"
	@echo "  make restart         - Stop, rebuild, ready to run"
	@echo "  make dev             - Watch + auto-rebuild (run manually in 2nd terminal)"
	@echo "  make dev-tmux        - Multi-pane dev environment (RECOMMENDED)"
	@echo "  make watch-check     - Fast syntax checking only"
	@echo ""
	@echo "Platform: $(PLATFORM)"
	@echo ""
	@echo "Windows users: Use build.ps1 instead of make"

build:
	@echo "Building ZapTUI..."
	cargo build --release
	cd whatsapp-service && npm install --silent

install:
ifeq ($(PLATFORM),macos)
	@./install-macos.sh
else
	@./install.sh
endif

install-global:
ifeq ($(PLATFORM),macos)
	@./install-macos.sh --global
else
	@./install.sh --global
endif

uninstall:
ifeq ($(PLATFORM),macos)
	@./scripts/uninstall-macos.sh
else
	@./scripts/uninstall.sh
endif

clean:
	cargo clean
	rm -rf whatsapp-service/node_modules

run:
ifeq ($(PLATFORM),macos)
	./scripts/zaptui-macos
else
	./zaptui
endif

# Development workflow targets
restart:
	@./scripts/restart.sh

dev:
	@./scripts/dev.sh

dev-tmux:
	@./scripts/dev-tmux.sh

watch-check:
	@echo "Starting fast check mode (no build)..."
	@cargo watch -x check
