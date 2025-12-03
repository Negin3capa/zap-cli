const blessed = require('blessed');
const { Client, LocalAuth } = require('whatsapp-web.js');
const qrcode = require('qrcode-terminal');

// --- CONFIGURAÇÃO DO WHATSAPP ---
const client = new Client({
    authStrategy: new LocalAuth(),
    puppeteer: { args: ['--no-sandbox'] }
});

console.log('Inicializando cliente... Aguarde o QR Code se necessário.');

// Variáveis globais para controle
let screen, contactsList, chatBox, inputBox, chatTitle;
let currentChatId = null;
let allChats = [];

// 1. Gera QR Code no terminal padrão (antes de carregar a UI)
client.on('qr', (qr) => {
    console.log('Escaneie o QR Code abaixo:');
    qrcode.generate(qr, { small: true });
});

// 2. Quando estiver pronto, inicia a interface gráfica (TUI)
client.on('ready', async () => {
    console.log('Conectado! Carregando interface...');
    // Pequeno delay para garantir que logs anteriores não quebrem a tela
    setTimeout(() => {
        iniciarInterface();
        carregarContatos();
    }, 1000);
});

// 3. Ao receber mensagem
client.on('message', async (msg) => {
    // Se a mensagem for do chat aberto atualmente, adiciona à tela
    if (currentChatId && msg.from === currentChatId) {
        const contact = await msg.getContact();
        const nome = contact.pushname || contact.number;
        chatBox.pushLine(`{red-fg}${nome}:{/} ${msg.body}`);
        chatBox.setScrollPerc(100); // Rola para o fim
        screen.render();
    }
    // Opcional: Atualizar a lista de contatos para subir o chat recente (complexo para este exemplo)
});

client.initialize();

// --- FUNÇÕES DA INTERFACE (BLESSED) ---

function iniciarInterface() {
    // Cria a tela
    screen = blessed.screen({
        smartCSR: true,
        title: 'WhatsApp TUI',
        mouse: true // HABILITA O MOUSE
    });

    // --- PAINEL ESQUERDO: LISTA DE CONTATOS ---
    const contactsBox = blessed.box({
        top: 0,
        left: 0,
        width: '30%',
        height: '100%',
        label: ' Contatos ',
        border: { type: 'line' },
        style: { border: { fg: 'green' } }
    });

    contactsList = blessed.list({
        parent: contactsBox,
        top: 0,
        left: 0,
        width: '95%',
        height: '95%',
        keys: true, // Permite navegar com setas
        mouse: true, // Permite clicar
        vi: true,
        style: {
            selected: { bg: 'green', fg: 'white', bold: true },
            item: { fg: 'white' }
        },
        scrollbar: {
            ch: ' ',
            track: { bg: 'grey' },
            style: { inverse: true }
        }
    });

    // --- PAINEL DIREITO: CONVERSA ---
    const chatContainer = blessed.box({
        top: 0,
        left: '30%',
        width: '70%',
        height: '85%',
        label: ' Conversa ',
        border: { type: 'line' },
        style: { border: { fg: 'white' } }
    });

    chatTitle = blessed.text({
        parent: chatContainer,
        top: 0,
        left: 'center',
        content: 'Selecione um contato'
    });

    chatBox = blessed.box({
        parent: chatContainer,
        top: 2,
        left: 1,
        width: '95%',
        height: '90%',
        tags: true, // Permite cores no texto {red-fg}
        scrollable: true,
        alwaysScroll: true,
        mouse: true,
        scrollbar: {
            ch: ' ',
            inverse: true
        }
    });

    // --- PAINEL DIREITO INFERIOR: INPUT ---
    const inputBoxContainer = blessed.box({
        bottom: 0,
        left: '30%',
        width: '70%',
        height: '15%',
        label: ' Enviar Mensagem (Enter para enviar) ',
        border: { type: 'line' },
        style: { border: { fg: 'cyan' } }
    });

    inputBox = blessed.textarea({
        parent: inputBoxContainer,
        top: 0,
        left: 0,
        width: '98%',
        height: '80%',
        inputOnFocus: true,
        style: { fg: 'white' }
    });

    // --- EVENTOS DA UI ---

    // Sair com Ctrl+C
    screen.key(['C-c'], () => process.exit(0));

    // Selecionar contato na lista
    contactsList.on('select', async (item, index) => {
        const chat = allChats[index];
        if (chat) {
            await abrirChat(chat);
        }
    });

    // Enviar mensagem ao pressionar Enter
    inputBox.key('enter', async () => {
        const text = inputBox.getValue().trim();
        if (text && currentChatId) {
            try {
                await client.sendMessage(currentChatId, text);
                chatBox.pushLine(`{cyan-fg}Eu:{/} ${text}`); // Mostra sua msg
                chatBox.setScrollPerc(100);
                inputBox.clearValue(); // Limpa o input
                inputBox.focus(); // Mantém o foco
                screen.render();
            } catch (err) {
                chatBox.pushLine(`{red-fg}Erro ao enviar: ${err.message}{/}`);
            }
        } else {
            // Remove a quebra de linha do Enter se não enviou
            inputBox.setValue(''); 
        }
    });

    // Focar no input ao clicar nele
    inputBox.on('click', () => inputBox.focus());

    screen.append(contactsBox);
    screen.append(chatContainer);
    screen.append(inputBoxContainer);
    
    contactsList.focus(); // Foco inicial na lista
    screen.render();
}

// --- LÓGICA DE DADOS ---

async function carregarContatos() {
    // Carrega chats
    const chats = await client.getChats();
    allChats = chats; // Salva na global

    // Prepara nomes para a lista
    const nomes = chats.map(chat => chat.name || chat.id.user);
    
    contactsList.setItems(nomes);
    screen.render();
}

async function abrirChat(chat) {
    currentChatId = chat.id._serialized;
    
    // Atualiza título
    chatTitle.setContent(`Chat: ${chat.name || chat.id.user}`);
    
    // Limpa chat anterior
    chatBox.setContent('{grey-fg}Carregando mensagens...{/}');
    screen.render();

    // Busca mensagens
    const messages = await chat.fetchMessages({ limit: 20 });
    
    chatBox.setContent(''); // Limpa o "Carregando"

    for (const msg of messages) {
        const isMe = msg.fromMe;
        const color = isMe ? '{cyan-fg}Eu' : '{red-fg}Contato';
        
        // Formata a mensagem
        chatBox.pushLine(`${color}:{/} ${msg.body}`);
    }

    chatBox.setScrollPerc(100); // Rola para o fim
    inputBox.focus(); // Já joga o foco para digitar
    screen.render();
}
