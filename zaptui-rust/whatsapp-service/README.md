# ZapTUI WhatsApp Service

WebSocket server that wraps `whatsapp-web.js` to provide WhatsApp functionality to the Rust TUI client.

## Installation

```bash
npm install
```

## Usage

```bash
npm start
```

The service will:

1. Start a WebSocket server on `ws://localhost:8080`
2. Initialize the WhatsApp client (using Puppeteer)
3. Display QR code events for authentication
4. Handle RPC requests from the Rust client
5. Broadcast WhatsApp events to all connected clients

## API

### Events (Server → Client)

- `qr`: QR code for authentication
- `ready`: WhatsApp client is ready
- `authenticated`: Successfully authenticated
- `message`: New message received
- `disconnected`: Client disconnected

### Methods (Client → Server)

- `getChats()`: Get all chats
- `getMessages(chatId, limit)`: Get messages for a chat
- `sendMessage(chatId, text)`: Send a text message
- `downloadMedia(messageId)`: Download media from a message

## Development

```bash
npm run dev  # Uses nodemon for auto-reload
```
