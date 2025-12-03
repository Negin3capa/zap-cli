# Zap CLI - WhatsApp TUI

Zap CLI is a Terminal User Interface (TUI) for WhatsApp, built with Node.js. It features a mouse-interactive interface, real-time messaging, and inline image viewing support for Kitty terminals.

## Features

- **TUI Interface**: Interactive terminal interface with mouse support using `blessed`.
- **Message History**: View and scroll through chat history.
- **Send/Receive**: Real-time messaging.
- **Image Viewing**: 
    - **Kitty Terminal**: Inline high-quality image viewing using Kitty's graphics protocol.
    - **Other Terminals**: Images are downloaded and saved to disk.
- **Contacts**: Browse and search (basic list) contacts/chats.

## Prerequisites

- **Node.js**: Version 14 or higher.
- **Google Chrome / Chromium**: Required for `puppeteer` (installed automatically in most cases, but system libraries might be needed).
- **Kitty Terminal** (Optional): For inline image viewing.

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/Negin3capa/zap-cli.git
   cd zap-cli
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. **(Linux Only)** Install system dependencies if Puppeteer fails to launch:

   **Debian/Ubuntu**:
   ```bash
   sudo apt-get install -y ca-certificates fonts-liberation libasound2 libatk-bridge2.0-0 libatk1.0-0 libc6 libcairo2 libcups2 libdbus-1-3 libexpat1 libfontconfig1 libgbm1 libgcc1 libglib2.0-0 libgtk-3-0 libnspr4 libnss3 libpango-1.0-0 libpangocairo-1.0-0 libstdc++6 libx11-6 libx11-xcb1 libxcb1 libxcomposite1 libxcursor1 libxdamage1 libxext6 libxfixes3 libxi6 libxrandr2 libxrender1 libxss1 libxtst6 lsb-release wget xdg-utils
   ```

   **Fedora**:
   ```bash
   sudo dnf install -y alsa-lib atk cups-libs gtk3 libXcomposite libXcursor libXdamage libXext libXi libXrandr libXScrnSaver libXtst pango xorg-x11-fonts-100dpi xorg-x11-fonts-75dpi xorg-x11-fonts-cyrillic xorg-x11-fonts-misc xorg-x11-fonts-Type1 xorg-x11-utils
   ```

   **Arch Linux**:
   ```bash
   sudo pacman -S --noconfirm alsa-lib atk cups gtk3 libxcomposite libxcursor libxdamage libxext libxi libxrandr libxss libxtst nss pango xdg-utils
   ```

## Configuration

Edit `config.json` to customize settings:

```json
{
  "sessionPath": "./.wwebjs_auth",  // Path to store session (do not delete to keep login)
  "downloadMedia": true,            // Auto-download media (not yet fully implemented auto-download, currently on-demand)
  "downloadPath": "./media",        // Path to save media
  "notifications": true
}
```

## Usage

Start the application:

```bash
npm start
```

or

```bash
node index.js
```

### First Time Login
1. On first run, a QR Code will be displayed in the terminal.
2. Open WhatsApp on your phone.
3. Go to **Linked Devices** -> **Link a Device**.
4. Scan the QR code.

### Controls
- **Mouse**: Click on chats to select them. Click on input box to type.
- **Arrow Keys**: Navigate lists.
- **Enter**: Send message.
- **Tab**: Switch focus between Chat List, Messages, and Input.
- **Ctrl+C**: Quit.
- **Right Click** (on Message Log): View the last received image in the chat (if using Kitty).

## Image Viewing
If you are using the **Kitty** terminal:
- When an image is received, right-click anywhere in the message log area to view the last media image.
- The image will clear the screen and display. Press any key to return to the chat.

If you are NOT using Kitty:
- Images will be saved to the current directory (or configured path) when you try to view them.

## Troubleshooting

- **Puppeteer Error**: If you see errors related to Chrome/Chromium launch, ensure you have necessary system libraries installed. On Linux, you might need libraries like `libnss3`, `libatk1.0-0`, etc.
- **QR Code formatting**: If the QR code looks broken, try resizing your terminal or ensuring a monospaced font is used.

## License
MIT
