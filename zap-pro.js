const blessed = require('blessed');
const { Client, LocalAuth } = require('whatsapp-web.js');
const qrcode = require('qrcode-terminal');

// --- CONFIGURAÇÃO ---
const client = new Client({
    authStrategy: new LocalAuth(),
    puppeteer: { 
        args: ['--no-sandbox', '--disable-setuid-sandbox'],
        headless: true
    }
});

// Cache simples
const contactCache = {}; 
let screen, contactsList, chatBox, inputBox, chatTitle;
let currentChatId = null;
let allChats = [];
let isInterfaceReady = false;

console.log('--- ZAP CLI PRO (Modo Seguro) ---');
console.log('Inicializando...');

// 1. QR Code
client.on('qr', (qr) => {
    console.clear();
    console.log('Escaneie o QR Code:');
    qrcode.generate(qr, { small: true });
});

// 2. Ready - REMOVIDO O getContacts() QUE CAUSAVA O ERRO
client.on('ready', async () => {
    console.log('Conectado! Carregando interface...');
    setTimeout(() => {
        iniciarInterface();
        carregarChats();
    }, 1000);
});

// EVENTOS DE MENSAGEM
client.on('message', async (msg) => {
    await processarMensagemRecebida(msg);
});

client.on('message_create', async (msg) => {
    if (msg.fromMe) {
        await processarMensagemRecebida(msg);
    }
});

client.initialize();

// --- LÓGICA DE MENSAGENS E NOMES ---

async function obterNomeContato(idContato) {
    // 1. Verifica Cache
    if (contactCache[idContato]) return contactCache[idContato];

    // 2. Tenta buscar nome (BLINDADO COM TRY/CATCH)
    try {
        const contact = await client.getContactById(idContato);
        // Prioriza pushname (nome que a pessoa definiu) ou name (nome na sua agenda)
        const nome = contact.pushname || contact.name || contact.number;
        
        if (nome) {
            contactCache[idContato] = nome;
            return nome;
        }
    } catch (e) {
        // Se der erro interno do WhatsApp, retorna apenas o número formatado
        const numero = idContato.split('@')[0];
        contactCache[idContato] = numero;
        return numero;
    }
    return idContato.split('@')[0];
}

async function processarMensagemRecebida(msg) {
    if (!isInterfaceReady || !chatBox) return;

    // Lógica para saber se a mensagem pertence ao chat aberto
    const isActiveChat = currentChatId && (msg.from === currentChatId || msg.to === currentChatId);

    if (isActiveChat) {
        const linhaFormatada = await formatarMensagem(msg);
        chatBox.pushLine(linhaFormatada);
        chatBox.setScrollPerc(100);
        screen.render();
    }
}

async function formatarMensagem(msg) {
    let nomeExibicao = "";
    let corNome = "";
    
    if (msg.fromMe) {
        nomeExibicao = "Eu";
        corNome = "{cyan-fg}";
    } else {
        // Pega quem mandou (se for grupo é author, se for privado é from)
        const autorRealId = msg.author || msg.from;
        nomeExibicao = await obterNomeContato(autorRealId);
        
        // Cores baseadas no nome
        const colors = ['{red-fg}', '{yellow-fg}', '{magenta-fg}', '{green-fg}'];
        corNome = colors[nomeExibicao.length % colors.length];
    }

    // Tratamento para mensagens sem texto (imagens, áudios)
    let conteudo = msg.body;
    if (!conteudo) {
         if (msg.hasMedia) conteudo = '[Mídia/Arquivo]';
         else conteudo = '[Mensagem do Sistema]';
    }

    return `${corNome}${blessed.escape(nomeExibicao)}:{/} ${conteudo}`;
}

// --- INTERFACE GRÁFICA ---

function iniciarInterface() {
    screen = blessed.screen({
        smartCSR: true,
        title: 'WhatsApp CLI',
        mouse: true,
        fullUnicode: true
    });

    // Layout Esquerdo (Lista)
    const listContainer = blessed.box({
        top: 0, left: 0, width: '30%', height: '100%',
        label: ' Conversas ',
        border: { type: 'line', fg: 'blue' }
    });

    contactsList = blessed.list({
        parent: listContainer,
        width: '95%', height: '95%',
        keys: true, mouse: true, vi: true,
        style: { selected: { bg: 'blue', fg: 'white' } },
        scrollbar: { ch: ' ', inverse: true }
    });

    // Layout Direito (Chat)
    const chatContainer = blessed.box({
        top: 0, left: '30%', width: '70%', height: '85%',
        label: ' Chat ',
        border: { type: 'line', fg: 'white' }
    });

    chatTitle = blessed.text({
        parent: chatContainer, top: 0, left: 'center',
        content: 'Selecione uma conversa'
    });

    chatBox = blessed.box({
        parent: chatContainer, top: 1, left: 0, width: '100%', height: '100%-2',
        tags: true, scrollable: true, alwaysScroll: true, mouse: true,
        scrollbar: { ch: '|', style: { fg: 'blue' } }
    });

    // Layout Input
    const inputContainer = blessed.box({
        bottom: 0, left: '30%', width: '70%', height: '15%',
        label: ' Mensagem ',
        border: { type: 'line', fg: 'green' }
    });

    inputBox = blessed.textarea({
        parent: inputContainer, width: '98%', height: '80%',
        keys: true, inputOnFocus: true,
        style: { fg: 'white' }
    });

    // --- EVENTOS DE UI ---
    
    screen.key(['C-c'], () => process.exit(0));

    contactsList.on('select', (item, index) => {
        const chat = allChats[index];
        if (chat) abrirChat(chat);
    });

    inputBox.key('enter', async () => {
        const text = inputBox.getValue().replace(/\n/g, '').trim();
        inputBox.clearValue(); 
        screen.render();

        if (text && currentChatId) {
            try {
                await client.sendMessage(currentChatId, text);
                inputBox.focus();
            } catch (err) {
                chatBox.pushLine(`{red-fg}Erro ao enviar: ${err.message}{/}`);
                screen.render();
            }
        }
    });

    inputBox.on('click', () => inputBox.focus());

    screen.append(listContainer);
    screen.append(chatContainer);
    screen.append(inputContainer);
    
    contactsList.focus();
    screen.render();
    isInterfaceReady = true;
}

async function carregarChats() {
    try {
        const chats = await client.getChats();
        // Limita a 40 chats para não pesar
        allChats = chats.slice(0, 40); 
        
        const nomes = allChats.map(c => c.name || c.id.user);
        contactsList.setItems(nomes);
        screen.render();
    } catch (error) {
        // Se der erro ao carregar lista, não crasha
        console.error("Erro ao carregar lista de chats");
    }
}

async function abrirChat(chat) {
    currentChatId = chat.id._serialized;
    chatTitle.setContent(`Chat: ${chat.name || chat.id.user}`);
    chatBox.setContent('{grey-fg}Carregando...{/}');
    screen.render();

    try {
        // Carrega 20 mensagens
        const messages = await chat.fetchMessages({ limit: 20 });
        chatBox.setContent('');

        for (const msg of messages) {
            const linha = await formatarMensagem(msg);
            chatBox.pushLine(linha);
        }
    } catch (e) {
        chatBox.pushLine(`{red-fg}Erro ao ler histórico.{/}`);
    }

    chatBox.setScrollPerc(100);
    inputBox.focus();
    screen.render();
}
