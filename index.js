const { Client } = require('whatsapp-web.js');
const qrcode = require('qrcode-terminal');
const { clientConfig } = require('./src/config');
const { initScreen } = require('./src/ui');
const { formatarMensagem } = require('./src/utils');

// Variáveis de Estado
const client = new Client(clientConfig);
let ui = null; // Guardará a referência dos elementos da tela
let allChats = [];
let currentChatId = null;

// --- CALLBACKS DA UI ---
// Estas funções são passadas para o ui.js para serem chamadas lá
const uiCallbacks = {
    onChatSelect: async (index) => {
        const chat = allChats[index];
        if (!chat) return;

        currentChatId = chat.id._serialized;
        ui.chatTitle.setContent(`Chat: ${chat.name || chat.id.user}`);
        ui.chatBox.setContent('{grey-fg}Carregando histórico...{/}');
        ui.screen.render();

        try {
            const messages = await chat.fetchMessages({ limit: 20 });
            ui.chatBox.setContent('');
            for (const msg of messages) {
                const linha = await formatarMensagem(client, msg);
                ui.chatBox.pushLine(linha);
            }
        } catch (e) {
            ui.chatBox.pushLine('{red-fg}Erro ao carregar.{/}');
        }
        ui.chatBox.setScrollPerc(100);
        ui.screen.render();
    },

    onMessageSend: async (text) => {
        if (!currentChatId) return;
        try {
            await client.sendMessage(currentChatId, text);
        } catch (err) {
            ui.chatBox.pushLine(`{red-fg}Erro envio: ${err.message}{/}`);
            ui.screen.render();
        }
    }
};

// --- EVENTOS DO WHATSAPP ---

console.log('--- ZAP CLI (Modular) ---');

client.on('qr', (qr) => {
    console.clear();
    console.log('Escaneie o QR Code:');
    qrcode.generate(qr, { small: true });
});

client.on('ready', async () => {
    console.log('Conectado! Iniciando UI...');
    setTimeout(async () => {
        // 1. Inicia a Tela
        ui = initScreen(uiCallbacks);
        
        // 2. Carrega Lista de Chats
        const chats = await client.getChats();
        allChats = chats.slice(0, 40);
        const nomes = allChats.map(c => c.name || c.id.user);
        
        ui.contactsList.setItems(nomes);
        ui.screen.render();
    }, 1000);
});

// Receber Mensagem (Realtime)
client.on('message', async (msg) => processarMsg(msg));
client.on('message_create', async (msg) => {
    if (msg.fromMe) processarMsg(msg);
});

async function processarMsg(msg) {
    if (!ui) return;
    
    // Verifica se a msg pertence ao chat aberto
    if (currentChatId && (msg.from === currentChatId || msg.to === currentChatId)) {
        const linha = await formatarMensagem(client, msg);
        ui.chatBox.pushLine(linha);
        ui.chatBox.setScrollPerc(100);
        ui.screen.render();
    }
}

// Inicializa
client.initialize();
