#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🗑️  ZapTUI Uninstaller${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

INSTALL_DIR="$HOME/.local/share/zaptui"
BIN_PATH="$HOME/.cargo/bin/zaptui"
CONFIG_DIR="$HOME/.config/zaptui"

# Check if globally installed
if [ ! -f "$BIN_PATH" ] && [ ! -d "$INSTALL_DIR" ]; then
    echo -e "${YELLOW}ℹ️  ZapTUI is not globally installed.${NC}"
    exit 0
fi

echo -e "\n${YELLOW}This will remove:${NC}"
[ -f "$BIN_PATH" ] && echo "  - $BIN_PATH"
[ -d "$INSTALL_DIR" ] && echo "  - $INSTALL_DIR"
echo ""

# Kill any running service
if lsof -Pi :8080 -sTCP:LISTEN -t > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Stopping running WhatsApp service...${NC}"
    lsof -ti:8080 | xargs kill -9 2>/dev/null || true
    sleep 1
fi

# Remove wrapper
if [ -f "$BIN_PATH" ]; then
    echo -e "${BLUE}Removing wrapper...${NC}"
    rm -f "$BIN_PATH"
    echo -e "${GREEN}✅ Removed $BIN_PATH${NC}"
fi

# Ask about data preservation
if [ -d "$INSTALL_DIR" ]; then
    echo ""
    echo -e "${YELLOW}⚠️  The installation directory contains:${NC}"
    [ -d "$INSTALL_DIR/.wwebjs_auth" ] && echo "  - WhatsApp authentication data (you'd need to re-scan QR code)"
    echo ""

    read -p "Remove installation directory? [y/N]: " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Removing installation directory...${NC}"
        rm -rf "$INSTALL_DIR"
        echo -e "${GREEN}✅ Removed $INSTALL_DIR${NC}"
    else
        echo -e "${YELLOW}ℹ️  Keeping $INSTALL_DIR${NC}"
    fi
fi

# Ask about config
if [ -d "$CONFIG_DIR" ]; then
    echo ""
    read -p "Remove configuration directory ($CONFIG_DIR)? [y/N]: " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Removing config directory...${NC}"
        rm -rf "$CONFIG_DIR"
        echo -e "${GREEN}✅ Removed $CONFIG_DIR${NC}"
    else
        echo -e "${YELLOW}ℹ️  Keeping $CONFIG_DIR${NC}"
    fi
fi

echo ""
echo -e "${GREEN}🎉 Uninstall complete!${NC}"
echo ""

# Check if we are in the source repo
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"

if [ -d "$REPO_DIR/.git" ] && [ -f "$REPO_DIR/Cargo.toml" ]; then
    echo -e "${YELLOW}⚠️  It seems you are running this from the cloned repository ($REPO_DIR).${NC}"
    read -p "Remove this source repository as well? [y/N]: " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Removing source repository...${NC}"

        # Create a self-destruct script since we can't delete the script while running it easily
        CLEANUP_SCRIPT=$(mktemp)
        cat <<EOF > "$CLEANUP_SCRIPT"
#!/bin/bash
sleep 1
echo -e "${BLUE}Deleting $REPO_DIR...${NC}"
rm -rf "$REPO_DIR"
echo -e "${GREEN}✅ Removed source repository${NC}"
echo -e "${GREEN}👋 Cleanup finished!${NC}"
rm "\$0"
EOF
        chmod +x "$CLEANUP_SCRIPT"

        # Run in background via nohup to survive shell exit
        nohup "$CLEANUP_SCRIPT" >/dev/null 2>&1 &
        exit 0
    else
        echo -e "${YELLOW}ℹ️  Keeping source repository${NC}"
    fi
fi
