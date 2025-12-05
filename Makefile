
# ZapTUI Makefile

.PHONY: all build install clean run

all: build

build:
	@echo "Building ZapTUI..."
	cargo build --release
	cd whatsapp-service && npm install --silent

install:
	@./install.sh

clean:
	cargo clean
	rm -rf whatsapp-service/node_modules

run:
	./zaptui
