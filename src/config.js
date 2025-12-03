const { LocalAuth } = require('whatsapp-web.js');

module.exports = {
    clientConfig: {
        authStrategy: new LocalAuth(),
        puppeteer: { 
            args: ['--no-sandbox', '--disable-setuid-sandbox'],
            headless: true
        }
    },
    uiConfig: {
        title: 'WhatsApp CLI - TUI',
        colors: {
            border: 'blue',
            selected: 'green',
            selfMsg: 'cyan'
        }
    }
};
