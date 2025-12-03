const fs = require('fs');
const WhatsAppClient = require('./src/client');
const TUI = require('./src/ui');

// Load config
let config = {};
try {
    config = JSON.parse(fs.readFileSync('./config.json', 'utf8'));
} catch (e) {
    console.error('Failed to load config.json');
    process.exit(1);
}

// Initialize Client and UI
const client = new WhatsAppClient(config);
const ui = new TUI(client);

// Connect Events
client.on('qr', (qr) => {
    ui.showQR(qr);
    ui.log('Scan QR Code to login');
});

client.on('ready', async () => {
    ui.log('WhatsApp Client is Ready!');
    ui.log('Loading chats...');
    const chats = await client.getChats();
    ui.setChats(chats);
    ui.log('Chats loaded.');
});

client.on('authenticated', () => {
    ui.log('Authenticated successfully!');
});

client.on('auth_failure', (msg) => {
    ui.log('Auth failure: ' + msg);
});

client.on('message', (msg) => {
    ui.appendMessage(msg);
    // If we receive a message and it's not the current chat, 
    // we might want to update the chat list sorting or show a badge.
    // For now, simple append if it matches current chat is handled in ui.appendMessage
});

// Start
ui.log('Initializing WhatsApp Client...');
client.initialize();
