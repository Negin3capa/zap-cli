const blessed = require('blessed');
const qrcode = require('qrcode-terminal');
const imageViewer = require('./image-viewer');
const qrcodeText = require('qrcode');

const THEME = {
    primary: 'white',
    secondary: 'green',
    highlight: 'yellow',
    text: 'white',
    timestamp: '#aaaaaa',
    me: 'white',
    other: 'green',
    system: 'blue'
};

class TUI {
    constructor(client) {
        this.client = client;
        this.screen = blessed.screen({
            smartCSR: true,
            title: 'Zap CLI',
            fullUnicode: true,
            dockBorders: true
        });

        this.currentChat = null;
        this.currentMessages = [];
        this.chats = [];
        
        this.setupLayout();
        this.setupEvents();
    }

    setupLayout() {
        // Sidebar for Chats
        this.chatList = blessed.list({
            parent: this.screen,
            top: 0,
            left: 0,
            width: '30%',
            height: '100%-3',
            label: ' {bold}Chats{/bold} ',
            tags: true,
            border: { type: 'line', fg: THEME.secondary },
            style: {
                selected: { bg: THEME.secondary, fg: 'black', bold: true },
                item: { fg: THEME.timestamp },
                border: { fg: THEME.secondary },
                label: { fg: THEME.secondary }
            },
            keys: true,
            mouse: true,
            vi: true,
            scrollbar: {
                style: { bg: THEME.secondary },
                track: { bg: 'black' }
            }
        });

        // Status Bar (Bottom Left)
        this.statusBar = blessed.box({
            parent: this.screen,
            bottom: 0,
            left: 0,
            width: '30%',
            height: 3,
            content: '{bold}Initializing...{/bold}',
            tags: true,
            border: { type: 'line', fg: THEME.system },
            style: {
                fg: 'white',
                border: { fg: THEME.system }
            }
        });

        // Main Chat Area
        this.chatBox = blessed.log({
            parent: this.screen,
            top: 0,
            left: '30%',
            width: '70%',
            height: '85%',
            label: ' {bold}Messages{/bold} ',
            tags: true,
            border: { type: 'line', fg: 'white' },
            style: {
                border: { fg: 'white' },
                label: { fg: 'white' }
            },
            scrollable: true,
            mouse: true,
            scrollbar: {
                style: { bg: 'white' },
                track: { bg: 'black' }
            }
        });

        // Input Area
        this.inputBox = blessed.textarea({
            parent: this.screen,
            bottom: 0,
            left: '30%',
            width: '70%',
            height: '15%',
            label: ' {bold}Type a message{/bold} ',
            tags: true,
            border: { type: 'line', fg: THEME.highlight },
            style: {
                border: { fg: THEME.highlight },
                label: { fg: THEME.highlight },
                focus: { border: { fg: 'white' } }
            },
            inputOnFocus: true,
            keys: true,
            mouse: true
        });
    }

    setupEvents() {
        // Quit on C-c
        const exitApp = () => {
            this.screen.destroy();
            process.exit(0);
        };

        this.screen.key(['C-c'], exitApp);
        this.inputBox.key(['C-c'], exitApp);
        this.chatList.key(['C-c'], exitApp);

        // Chat selection
        this.chatList.on('select', async (item, index) => {
            const chat = this.chats[index];
            if (chat) {
                await this.selectChat(chat);
            }
        });

        // Message Input
        this.inputBox.key('enter', async () => {
            const text = this.inputBox.getValue().trim();
            if (text && this.currentChat) {
                await this.client.sendMessage(this.currentChat.id._serialized, text);
                this.inputBox.clearValue();
                this.inputBox.focus(); // Keep focus
                this.screen.render();
            }
        });
        
        // Handle Tab to switch focus
        this.screen.key(['tab'], () => {
            this.screen.focusNext();
        });

        // Handle ChatBox click (Right click to view last image)
        this.chatBox.on('element mouseup', (data) => {
            if (data.button === 'right') {
                if (this.currentMessages) {
                    // Find the last media message
                    const lastMedia = [...this.currentMessages].reverse().find(m => m.hasMedia);
                    if (lastMedia) {
                        this.viewMedia(lastMedia);
                    } else {
                        this.log('No media to view in this chat history.');
                    }
                }
            }
        });
    }

    log(msg) {
        this.statusBar.setContent(`{bold}${msg}{/bold}`);
        this.screen.render();
    }

    async showQR(qrData) {
        try {
            const str = await qrcodeText.toString(qrData, { type: 'terminal', small: true });
            this.chatBox.setContent(`SCAN QR CODE:\n${str}`);
            this.screen.render();
        } catch (e) {
            this.chatBox.setContent(`QR Code received. Please check logs.`);
        }
    }

    setChats(chats) {
        this.chats = chats;
        const items = chats.map(c => c.name || c.id.user);
        this.chatList.setItems(items);
        this.screen.render();
    }

    async selectChat(chat) {
        this.currentChat = chat;
        this.currentMessages = [];
        
        let chatName = chat.name;
        try {
            const contact = await chat.getContact();
            chatName = contact.name || contact.pushname || chat.name || contact.number;
        } catch(e) {
            // keep chat.name
        }

        this.chatBox.setLabel(` {bold}${chatName}{/bold} `);
        this.chatBox.setContent('{center}Loading messages...{/center}');
        this.screen.render();

        const messages = await chat.fetchMessages({ limit: 50 });
        this.chatBox.setContent('');
        for (const msg of messages) {
             await this.appendMessage(msg);
        }
        this.screen.render();
        this.inputBox.focus();
    }

    async appendMessage(msg) {
        if (!this.currentChat || msg.id.remote !== this.currentChat.id._serialized) {
             if (msg.fromMe && this.currentChat && msg.to === this.currentChat.id._serialized) {
                 // proceed
             } else {
                 return;
             }
        }

        let sender = 'User';
        let senderColor = THEME.other;

        if (msg.fromMe) {
            sender = 'Me';
            senderColor = THEME.me;
        } else {
            try {
                const contact = await msg.getContact();
                sender = contact.pushname || contact.name || contact.number || 'User';
            } catch (e) {
                // Fallback if getContact fails (common issue with wwebjs updates)
                sender = msg._data.notifyName || msg.author || msg.from.split('@')[0];
            }
        }

        let content = msg.body;

        if (msg.hasMedia) {
             content = `{${THEME.highlight}-fg}[MEDIA: ${msg.type}] (Right-click to view){/}`;
        }

        let time = new Date(msg.timestamp * 1000).toLocaleTimeString([], { hour: 'numeric', minute: '2-digit', hour12: true });
        time = time.replace(/\s+/g, '');
        this.chatBox.add(`{${THEME.timestamp}-fg}${time}{/} {bold}{${senderColor}-fg}${sender}{/}: ${content}`);
        this.currentMessages.push(msg);
    }

    async viewMedia(msg) {
        if (!msg.hasMedia) return;
        
        this.log('Downloading media...');
        try {
            const media = await msg.downloadMedia();
            if (media) {
                const buffer = Buffer.from(media.data, 'base64');
                this.log('Displaying image...');
                
                if (imageViewer.isKitty) {
                   process.stdout.write('\x1b[2J\x1b[H'); // Clear screen
                   imageViewer.display(buffer);
                   
                   this.screen.lockKeys = true;
                   
                   const restore = () => {
                       process.stdin.setRawMode(true);
                       process.stdin.resume();
                       this.screen.alloc();
                       this.screen.render();
                       this.screen.lockKeys = false;
                       process.stdin.removeListener('data', onKey);
                   };

                   const onKey = (key) => {
                       restore();
                   };
                   
                   process.stdin.once('data', onKey);
                } else {
                    this.log('Terminal not supported for inline images. Saved to disk.');
                    const fs = require('fs');
                    const filename = `media_${msg.timestamp}.${media.mimetype.split('/')[1]}`;
                    fs.writeFileSync(filename, buffer);
                    this.log(`Saved to ${filename}`);
                }
            }
        } catch (e) {
            this.log('Error downloading media: ' + e.message);
        }
    }
}

module.exports = TUI;
