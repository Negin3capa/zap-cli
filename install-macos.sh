#!/bin/bash
set -e

# ZapTUI macOS Installer
# Follows macOS conventions for installation paths

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🚀 ZapTUI macOS Installer${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Function to check command existence
check_cmd() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}❌ $1 is not installed.${NC}"
        return 1
    else
        echo -e "${GREEN}✅ $1 found.${NC}"
        return 0
    fi
}

# 1. Check Dependencies
echo -e "\n${BLUE}📦 Checking dependencies...${NC}"
MISSING_DEPS=0

check_cmd "cargo" || {
    echo "Install via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    MISSING_DEPS=1
}

check_cmd "node" || {
    echo "Install via Homebrew: brew install node"
    echo "Or download from: https://nodejs.org/"
    MISSING_DEPS=1
}

check_cmd "npm" || MISSING_DEPS=1

if [ $MISSING_DEPS -eq 1 ]; then
    echo -e "\n${RED}Please install missing dependencies and try again.${NC}"
    exit 1
fi

# 2. Build Rust Binary
echo -e "\n${BLUE}🦀 Building Rust binary...${NC}"
if cargo build --release; then
    echo -e "${GREEN}✅ Rust build successful.${NC}"
else
    echo -e "${RED}❌ Rust build failed.${NC}"
    exit 1
fi

# 3. Install Node Dependencies
echo -e "\n${BLUE}🟢 Installing WhatsApp service dependencies...${NC}"
cd whatsapp-service
if npm install --silent; then
    echo -e "${GREEN}✅ Node dependencies installed.${NC}"
else
    echo -e "${RED}❌ Failed to install Node dependencies.${NC}"
    cd ..
    exit 1
fi
cd ..

# 4. Installation Verification
echo -e "\n${BLUE}🧪 Verifying build...${NC}"
if [ -f "./target/release/zaptui" ]; then
    echo -e "${GREEN}✅ Binary created at target/release/zaptui${NC}"
else
    echo -e "${RED}❌ Binary not found.${NC}"
    exit 1
fi

# 5. Setup 'zaptui' command (Launcher)
chmod +x scripts/zaptui-macos

echo -e "\n${BLUE}🔗 Installation Options${NC}"
echo "1) Local Install - Run from current directory with './scripts/zaptui-macos'"
echo "2) User Install (Recommended) - Install to ~/Library/Application Support/zaptui"
echo "3) Homebrew Install - Install to /usr/local/bin (requires Homebrew)"

# Check for --global flag for non-interactive mode
if [[ "$1" == "--global" ]]; then
    CHOICE=2
elif [ -t 0 ]; then
    read -p "Choose an option [1-3]: " CHOICE
else
    echo "Non-interactive mode detected. Skipping installation."
    CHOICE=1
fi

case $CHOICE in
    2)
        echo -e "\n${BLUE}📦 Installing to ~/Library/Application Support/zaptui${NC}"

        INSTALL_DIR="$HOME/Library/Application Support/zaptui"
        BIN_DIR="/usr/local/bin"

        # Create installation directory
        mkdir -p "$INSTALL_DIR"

        # Copy and setup WhatsApp service
        echo "📋 Installing WhatsApp service..."
        rm -rf "$INSTALL_DIR/whatsapp-service"
        cp -r whatsapp-service "$INSTALL_DIR/"

        # Install Node dependencies at new location
        echo "📦 Installing service dependencies..."
        cd "$INSTALL_DIR/whatsapp-service"
        npm install --silent
        cd - > /dev/null

        # Install binary
        echo "📋 Installing binary..."
        cp ./target/release/zaptui "$INSTALL_DIR/zaptui"
        chmod +x "$INSTALL_DIR/zaptui"

        # Install launcher script
        echo "📋 Installing launcher..."
        cp scripts/zaptui-macos "$INSTALL_DIR/zaptui-launcher"
        chmod +x "$INSTALL_DIR/zaptui-launcher"

        # Create symlink in /usr/local/bin
        if [ -w "$BIN_DIR" ]; then
            ln -sf "$INSTALL_DIR/zaptui-launcher" "$BIN_DIR/zaptui"
            echo -e "${GREEN}✅ Created symlink in $BIN_DIR${NC}"
        else
            echo -e "${YELLOW}⚠️  Creating symlink requires sudo...${NC}"
            sudo ln -sf "$INSTALL_DIR/zaptui-launcher" "$BIN_DIR/zaptui"
            echo -e "${GREEN}✅ Created symlink in $BIN_DIR${NC}"
        fi

        # Migrate auth data if it exists
        if [ -d "./.wwebjs_auth" ]; then
            echo -e "\n${YELLOW}⚠️  Found existing WhatsApp authentication data${NC}"
            read -p "Migrate to global installation? [Y/n]: " -n 1 -r
            echo

            if [[ ! $REPLY =~ ^[Nn]$ ]]; then
                echo "📋 Migrating authentication data..."
                rm -rf "$INSTALL_DIR/.wwebjs_auth"
                cp -r ./.wwebjs_auth "$INSTALL_DIR/"
                echo -e "${GREEN}✅ Authentication data migrated${NC}"
            fi
        fi

        echo -e "${GREEN}✅ Installation complete!${NC}"
        echo ""
        echo "You can now run 'zaptui' from anywhere."
        ;;
    3)
        if ! command -v brew &> /dev/null; then
            echo -e "${RED}❌ Homebrew not found. Install from https://brew.sh/${NC}"
            exit 1
        fi

        echo -e "\n${BLUE}📦 Installing via Homebrew location...${NC}"
        BIN_DIR="$(brew --prefix)/bin"
        ABS_PATH="$(pwd)/scripts/zaptui-macos"
        
        ln -sf "$ABS_PATH" "$BIN_DIR/zaptui"
        echo -e "${GREEN}✅ Symlinked to $BIN_DIR/zaptui${NC}"
        ;;
    *)
        echo "Skipping installation."
        echo "Run './scripts/zaptui-macos' to launch."
        ;;
esac

echo -e "\n${GREEN}🎉 Setup Complete!${NC}"
if [ "$CHOICE" == "2" ]; then
    echo -e "To start ZapTUI, run: ${BLUE}zaptui${NC}"
elif [ "$CHOICE" == "3" ]; then
    echo -e "To start ZapTUI, run: ${BLUE}zaptui${NC}"
else
    echo -e "To start ZapTUI, run: ${BLUE}./scripts/zaptui-macos${NC}"
fi
echo ""
