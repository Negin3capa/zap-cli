#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🚀 ZapTUI Installation Wrapper${NC}"
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

check_cmd "cargo" || MISSING_DEPS=1
check_cmd "node" || MISSING_DEPS=1
check_cmd "npm" || MISSING_DEPS=1

if [ $MISSING_DEPS -eq 1 ]; then
    echo -e "\n${RED}Please install missing dependencies and try again.${NC}"
    echo "Rust: https://rustup.rs/"
    echo "Node.js: https://nodejs.org/"
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
# We need to make sure the 'zaptui' script works and is executable
chmod +x zaptui

echo -e "\n${BLUE}🔗 Installation Options${NC}"
echo "1) Local Install (Recommended for dev) - Just run './zaptui'"
echo "2) Add to User PATH (~/.cargo/bin) - Create a symlink"
echo "3) System Install (/usr/local/bin) - Create a symlink (Requires sudo)"

# Interactive limit check - if non-interactive, default to just build
if [ -t 0 ]; then
    read -p "Choose an option [1-3]: " CHOICE
else
    echo "Non-interactive mode detected. Skipping PATH integration."
    CHOICE=1
fi

case $CHOICE in
    2)
        BIN_DIR="$HOME/.cargo/bin"
        mkdir -p "$BIN_DIR"
        # We need an absolute path for the symlink
        ABS_PATH="$(pwd)/zaptui"
        ln -sf "$ABS_PATH" "$BIN_DIR/zaptui"
        echo -e "${GREEN}✅ Symlinked $ABS_PATH to $BIN_DIR/zaptui${NC}"
        echo "Make sure $BIN_DIR is in your PATH."
        ;;
    3)
        ABS_PATH="$(pwd)/zaptui"
        echo "Running sudo to create symlink..."
        sudo ln -sf "$ABS_PATH" "/usr/local/bin/zaptui"
        echo -e "${GREEN}✅ Symlinked $ABS_PATH to /usr/local/bin/zaptui${NC}"
        ;;
    *)
        echo "Skipping PATH integration."
        ;;
esac

echo -e "\n${GREEN}🎉 Setup Complete!${NC}"
echo "To start ZapTUI, run:"
echo -e "  ${BLUE}./zaptui${NC}  (or 'zaptui' if installed to PATH)"
echo ""
