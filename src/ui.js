const blessed = require('blessed');

let screen, contactsList, chatBox, inputBox, chatTitle;

function initScreen(callbacks) {
    screen = blessed.screen({
        smartCSR: true,
        title: 'WhatsApp CLI',
        mouse: true,
        fullUnicode: true
    });

    // --- PAINEL ESQUERDO (LISTA) ---
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

    // --- PAINEL DIREITO (CHAT) ---
    const chatContainer = blessed.box({
        top: 0, left: '30%', width: '70%', height: '85%',
        label: ' Chat ',
        border: { type: 'line', fg: 'white' }
    });

    chatTitle = blessed.text({
        parent: chatContainer, top: 0, left: 'center',
        content: 'Aguardando seleção...'
    });

    chatBox = blessed.box({
        parent: chatContainer, top: 1, left: 0, width: '100%', height: '100%-2',
        tags: true, scrollable: true, alwaysScroll: true, mouse: true,
        scrollbar: { ch: '|', style: { fg: 'blue' } }
    });

    // --- PAINEL INPUT ---
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
        callbacks.onChatSelect(index);
    });

    inputBox.key('enter', () => {
        const text = inputBox.getValue().replace(/\n/g, '').trim();
        inputBox.clearValue();
        screen.render();
        if (text) callbacks.onMessageSend(text);
        inputBox.focus(); // Mantém o foco
    });
    
    inputBox.on('click', () => inputBox.focus());

    screen.append(listContainer);
    screen.append(chatContainer);
    screen.append(inputContainer);
    contactsList.focus();
    screen.render();

    return { screen, contactsList, chatBox, inputBox, chatTitle };
}

module.exports = { initScreen };
