const { Client, LocalAuth } = require('whatsapp-web.js');
const EventEmitter = require('events');
const qrcode = require('qrcode-terminal');

class WhatsAppClient extends EventEmitter {
    constructor(config) {
        super();
        this.config = config;
        this.client = new Client({
            authStrategy: new LocalAuth({ dataPath: config.sessionPath }),
            puppeteer: {
                headless: true,
                args: ['--no-sandbox', '--disable-setuid-sandbox']
            }
        });

        this.initializeEvents();
    }

    initializeEvents() {
        this.client.on('qr', (qr) => {
            this.emit('qr', qr);
        });

        this.client.on('ready', () => {
            this.emit('ready');
        });

        this.client.on('authenticated', () => {
            this.emit('authenticated');
        });

        this.client.on('auth_failure', (msg) => {
            this.emit('auth_failure', msg);
        });

        this.client.on('message', async (msg) => {
            this.emit('message', msg);
        });
        
        this.client.on('message_create', async (msg) => {
             if (msg.fromMe) {
                 this.emit('message', msg);
             }
        });
    }

    initialize() {
        this.client.initialize();
    }

    async getChats() {
        return await this.client.getChats();
    }

    async getChatById(chatId) {
        return await this.client.getChatById(chatId);
    }

    async sendMessage(chatId, content) {
        return await this.client.sendMessage(chatId, content);
    }
    
    async downloadMedia(msg) {
        if (msg.hasMedia) {
            return await msg.downloadMedia();
        }
        return null;
    }
}

module.exports = WhatsAppClient;
