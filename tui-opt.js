const blessed = require('blessed');
const { Client, LocalAuth } = require('whatsapp-web.js');
const qrcode = require('qrcode-terminal');

// --- CONFIGURAÇÃO DO CLIENTE ---
const client = new Client({
    authStrategy: new LocalAuth(),
    puppeteer: { 
        args: ['--no-sandbox', '--disable-setuid-sandbox'],
        headless: true
    }
});

// Variáveis de Estado
let screen, contactsList, chatBox, inputBox, chatTitle, chatContainer;
let currentChatId = null;
let allChats = [];
let isInterfaceReady = false;

console.log('--- ZAP CLI OTIMIZADO ---');
console.log('Inicializando... (Se for a 1ª vez, aguarde o QR Code)');

// 1. QR Code
client.on('qr', (qr) => {
    console.clear();
    console.log('Escaneie o QR Code abaixo:');
    qrcode.generate(qr, { small: true });
});

// 2. Ready
client.on('ready', async () => {
    console.log('Conectado! Carregando conversas...');
    // Aguarda um momento para garantir sincronia
    setTimeout(() => {
        iniciarInterface();
        carregarChatsOtimizado();
    }, 1000);
});

// 3. Recebimento de Mensagem (BLINDADO CONTRA CRASH)
client.on('message', async (msg) => {
    // Se a interface não estiver pronta, ignora renderização visual
    if (!isInterfaceReady || !chatBox) return;

    try {
        // Só renderiza se a mensagem for do chat aberto
        if (currentChatId && msg.from === currentChatId) {
            const contact = await msg.getContact();
            const nome = contact.pushname || contact.number || "Desconhecido";
            
            chatBox.pushLine(`{red-fg}${nome}:{/} ${msg.body}`);
            chatBox.setScrollPerc(100);
            screen.render();
        }
        
        // Opcional: Tocar um 'beep'
        screen.program.bell(); 
    } catch (err) {
        // Silencia erro para não quebrar a UI, apenas loga internamente se necessário
    }
});

client.initialize();

// --- INTERFACE GRÁFICA (TUI) ---

function iniciarInterface() {
    screen = blessed.screen({
        smartCSR: true,
        title: 'WhatsApp TUI v2',
        mouse: true,
        fullUnicode: true // Melhor suporte a emojis
    });

    // Layout: Contatos (Esquerda)
    const listContainer = blessed.box({
        top: 0,
        left: 0,
        width: '30%',
        height: '100%',
        label: ' Recentes (Top 30) ',
        border: { type: 'line', fg: 'green' }
    });

    contactsList = blessed.list({
        parent: listContainer,
        top: 0,
        left: 0,
        width: '95%',
        height: '95%',
        keys: true,
        mouse: true,
        vi: true,
        style: {
            selected: { bg: 'green', fg: 'white', bold: true },
            item: { fg: 'white' }
        },
        scrollbar: { ch: ' ', inverse: true }
    });

    // Layout: Chat (Direita Superior)
    chatContainer = blessed.box({
        top: 0,
        left: '30%',
        width: '70%',
        height: '85%',
        label: ' Conversa ',
        border: { type: 'line', fg: 'white' }
    });

    chatTitle = blessed.text({
        parent: chatContainer,
        top: 0,
        left: 'center',
        content: 'Selecione um contato'
    });

    chatBox = blessed.box({
        parent: chatContainer,
        top: 1,
        left: 0,
        width: '100%',
        height: '100%-2', // Ajuste fino
        tags: true,
        scrollable: true,
        alwaysScroll: true,
        mouse: true,
        scrollbar: { ch: '|', style: { fg: 'cyan' } }
    });

    // Layout: Input (Direita Inferior)
    const inputContainer = blessed.box({
        bottom: 0,
        left: '30%',
        width: '70%',
        height: '15%',
        label: ' Digite sua mensagem ',
        border: { type: 'line', fg: 'cyan' }
    });

    inputBox = blessed.textarea({
        parent: inputContainer,
        width: '98%',
        height: '80%',
        keys: true,
        inputOnFocus: true,
        style: { fg: 'white' }
    });

    // --- EVENTOS ---

    screen.key(['C-c'], () => process.exit(0));

    contactsList.on('select', (item, index) => {
        const chat = allChats[index];
        if (chat) abrirChat(chat);
    });

    // Enviar com Enter (hack para textarea não criar nova linha)
    inputBox.key('enter', async () => {
        const text = inputBox.getValue().replace(/\n/g, '').trim(); // Remove quebra de linha
        
        if (text && currentChatId) {
            try {
                // Limpa input imediatamente para UX rápida
                inputBox.setValue(''); 
                screen.render();
                
                // Envia
                await client.sendMessage(currentChatId, text);
                
                // Adiciona na tela localmente
                chatBox.pushLine(`{cyan-fg}Eu:{/} ${text}`);
                chatBox.setScrollPerc(100);
                inputBox.focus();
                screen.render();
            } catch (err) {
                chatBox.pushLine(`{red-fg}[Erro ao enviar]{/}`);
                screen.render();
            }
        } else {
            inputBox.setValue(''); // Reseta se for vazio
        }
    });
    
    // Foco no input ao clicar
    inputBox.on('click', () => inputBox.focus());

    screen.append(listContainer);
    screen.append(chatContainer);
    screen.append(inputContainer);
    
    contactsList.focus();
    screen.render();
    isInterfaceReady = true;
}

// --- LÓGICA DE DADOS OTIMIZADA ---

async function carregarChatsOtimizado() {
    try {
        // Carrega conversas
        const chats = await client.getChats();
        
        // OTIMIZAÇÃO: Pega apenas os 30 primeiros chats ativos
        // Isso evita travar o loop em contas com 5000 chats
        allChats = chats.slice(0, 30);

        const nomes = allChats.map(c => c.name || c.id.user || "Sem Nome");
        
        contactsList.setItems(nomes);
        screen.render();
    } catch (e) {
        // Se der erro, não crasha, só avisa
    }
}

async function abrirChat(chat) {
    try {
        currentChatId = chat.id._serialized;
        chatTitle.setContent(`Chat: ${chat.name || chat.id.user}`);
        chatBox.setContent('{grey-fg}Carregando histórico...{/}');
        screen.render();

        // Carrega apenas 15 msgs para ser rápido
        const messages = await chat.fetchMessages({ limit: 15 });
        chatBox.setContent('');

        messages.forEach(msg => {
            const isMe = msg.fromMe;
            const prefix = isMe ? '{cyan-fg}Eu' : '{red-fg}Contato';
            // Trata msg.body undefined (ex: sticker ou imagem)
            const conteudo = msg.body || '[Mídia/Sticker]';
            chatBox.pushLine(`${prefix}:{/} ${conteudo}`);
        });

        chatBox.setScrollPerc(100);
        inputBox.focus();
        screen.render();
    } catch (err) {
        chatBox.pushLine(`{red-fg}Erro ao abrir chat.{/}`);
    }
}
