const blessed = require('blessed');
const qrcode = require('qrcode-terminal');
const imageViewer = require('./image-viewer');
const qrcodeText = require('qrcode');
const stringWidth = (...args) => import('string-width').then(({default: stringWidth}) => stringWidth(...args));
const stripAnsi = (...args) => import('strip-ansi').then(({default: stripAnsi}) => stripAnsi(...args));

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
        this.contactCache = new Map();
        
        this.setupLayout();
        this.setupEvents();
        this.screen.render();
    }

    setupLayout() {
        this.mainGrid = blessed.box({
            parent: this.screen,
            width: '100%',
            height: '100%'
        });

        this.sidebar = blessed.box({
            parent: this.mainGrid,
            left: 0,
            top: 0,
            height: '100%',
            width: '30%'
        });

        this.mainContent = blessed.box({
            parent: this.mainGrid,
            left: '30%',
            top: 0,
            height: '100%',
            width: '70%'
        });
        
        // Sidebar for Chats
        this.chatList = blessed.list({
            parent: this.sidebar,
            top: 0,
            left: 0,
            right: 0,
            bottom: 3,
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
            vi: true
        });

        // Status Bar (Bottom Left)
        this.statusBar = blessed.box({
            parent: this.sidebar,
            bottom: 0,
            left: 0,
            right: 0,
            height: 3,
            content: '{bold}Initializing...{/bold}',
            tags: true,
            border: { type: 'line', fg: THEME.system },
            style: {
                fg: 'white',
                border: { fg: THEME.system }
            }
        });

        // Main Chat Area - Refactored to listtable for stability
        this.chatBox = blessed.listtable({
            parent: this.mainContent,
            top: 0,
            left: 0,
            right: 0,
            bottom: 5,
            label: ' {bold}Messages{/bold} ',
            tags: true,
            border: { type: 'line', fg: 'white' },
            style: {
                header: { bold: true, fg: 'blue' },
                cell: { selected: { bg: 'blue' }, fg: 'white' },
                border: { fg: 'white' },
                label: { fg: 'white' }
            },
            align: 'left',
            mouse: true,
            keys: true,
            vi: true,
            scrollable: true,
            scrollbar: {
                style: { bg: 'white' },
                track: { bg: 'black' }
            },
            noCellBorders: true,
            data: null
        });

        // Input Area
        this.inputBox = blessed.textarea({
            parent: this.mainContent,
            bottom: 0,
            left: 0,
            right: 0,
            height: 5,
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

        this.screen.on('resize', () => {
            this.setChats(this.chats);
            this.screen.render();
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
        const maxNameLength = this.chatList.width - this.chatList.iwidth - 2; // Buffer for padding/borders
        const items = chats.map(c => {
            let name = c.name || c.id.user;
            const nameWidth = stringWidth(name);

            if (nameWidth > maxNameLength) {
                let truncated = '';
                let currentWidth = 0;
                for (const char of name) {
                    const charWidth = stringWidth(char);
                    if (currentWidth + charWidth + 3 > maxNameLength) {
                        break;
                    }
                    truncated += char;
                    currentWidth += charWidth;
                }
                name = truncated + '...';
            }
            
            const cleanName = stripAnsi(name);
            const visibleWidth = stringWidth(cleanName);
            const padding = ' '.repeat(Math.max(0, maxNameLength - visibleWidth));
            return name + padding;
        });
        this.chatList.setItems(items);
        this.screen.render();
    }

    async selectChat(chat) {
        if (this.messageLoader) {
            clearInterval(this.messageLoader);
            this.messageLoader = null;
        }

        this.currentChat = chat;
        this.currentMessages = [];
        this.messageLimit = 20;
        this.isLoadingMessages = false;
        
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

        const messages = await chat.fetchMessages({ limit: this.messageLimit });
        
        // Pre-format messages in parallel
        await Promise.all(messages.map(msg => this.formatMessage(msg)));

        // Merge with any real-time messages that arrived during fetch
        const currentIds = new Set(this.currentMessages.map(m => m.id._serialized));
        const newMessages = messages.filter(m => !currentIds.has(m.id._serialized));
        
        this.currentMessages = [...newMessages, ...this.currentMessages];
        
        // Batch update UI using setRows for listtable
        const tableData = this.currentMessages.map(m => m._displayData);
        this.chatBox.setRows([['Sender', 'Message'], ...tableData]);

        this.screen.render();
        this.inputBox.focus();

        // Periodic fetching disabled to prevent rendering artifacts
        // this.messageLoader = setInterval(() => this.loadOlderMessages(), 5000);
    }

    async formatMessage(msg) {
        if (msg._displayString) return msg._displayString;

        let sender = 'User';
        let senderColor = THEME.other;

        if (msg.fromMe) {
            sender = 'Me';
            senderColor = THEME.me;
        } else {
            const senderId = msg.author || msg.from;
            if (this.contactCache.has(senderId)) {
                sender = this.contactCache.get(senderId);
            } else {
                try {
                    const contact = await msg.getContact();
                    sender = contact.pushname || contact.name || contact.number || 'User';
                    this.contactCache.set(senderId, sender);
                } catch (e) {
                    // Fallback if getContact fails (common issue with wwebjs updates)
                    sender = msg._data.notifyName || msg.author || msg.from.split('@')[0];
                }
            }
        }

        let content = msg.body;
        
        // Escape content to prevent tag parsing issues and strip specific zero-width chars
        if (content) {
            content = content.replace(/[\u200B-\u200D\uFEFF]/g, ''); // Strip zero-width spaces and joiners
            content = content.replace(/\{/g, '｛').replace(/\}/g, '｝');
        }

        if (msg.hasMedia) {
             content = `{${THEME.highlight}-fg}[MEDIA: ${msg.type}] (Right-click to view){/}`;
        }

        let time = new Date(msg.timestamp * 1000).toLocaleTimeString([], { hour: 'numeric', minute: '2-digit', hour12: true });
        time = time.replace(/\s+/g, '');
        
        const formatted = `{${THEME.timestamp}-fg}${time}{/} {bold}{${senderColor}-fg}${sender}{/}`;
        msg._displayString = formatted; // Keep for compatibility if needed elsewhere
        
        // Ensure content is a string before stripping ANSI codes
        const cleanContent = typeof content === 'string' ? await stripAnsi(content) : '';
        msg._displayData = [formatted, cleanContent];
        return msg._displayData;
    }

    async appendMessage(msg) {
        if (!this.currentChat || msg.id.remote !== this.currentChat.id._serialized) {
             if (msg.fromMe && this.currentChat && msg.to === this.currentChat.id._serialized) {
                 // proceed
             } else {
                 return;
             }
        }

        const formatted = await this.formatMessage(msg);
        this.currentMessages.push(msg);

        // Atomic update for listtable
        const tableData = this.currentMessages.map(m => m._displayData);
        this.chatBox.setRows([['Sender', 'Message'], ...tableData]);
        
        // Auto-scroll to bottom for new incoming messages
        this.chatBox.scrollTo(this.chatBox.items.length);
        
        this.screen.render();
    }

    async loadOlderMessages() {
        if (!this.currentChat || this.isLoadingMessages) return;
        this.isLoadingMessages = true;

        try {
            this.messageLimit += 10;
            const messages = await this.currentChat.fetchMessages({ limit: this.messageLimit });
            
            const currentIds = new Set(this.currentMessages.map(m => m.id._serialized));
            const newMessages = [];
            
            for (const msg of messages) {
                if (!currentIds.has(msg.id._serialized)) {
                    newMessages.push(msg);
                }
            }
            
            if (newMessages.length > 0) {
                // Pre-format messages in parallel
                await Promise.all(newMessages.map(msg => this.formatMessage(msg)));
                
                this.currentMessages = [...newMessages, ...this.currentMessages];
                this.refreshChatBox();
            }
        } catch (e) {
            // Error fetching messages
        } finally {
            this.isLoadingMessages = false;
        }
    }

    refreshChatBox() {
        const oldScroll = this.chatBox.getScroll();
        const oldLines = this.chatBox.getScrollHeight();
        
        const tableData = this.currentMessages.map(m => m._displayData);
        this.chatBox.setRows([['Sender', 'Message'], ...tableData]);
        
        const newLines = this.chatBox.items.length;
        const addedLines = newLines - oldLines;
        
        if (addedLines > 0) {
             this.chatBox.scrollTo(oldScroll + addedLines);
        }
        
        this.screen.render();
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
