const blessed = require('blessed');

// Cache para armazenar nomes e evitar chamadas repetidas à API
const contactCache = {};

// Função segura para obter nome (evita crash do Puppeteer)
async function obterNomeContato(client, idContato) {
    if (contactCache[idContato]) return contactCache[idContato];

    try {
        const contact = await client.getContactById(idContato);
        const nome = contact.pushname || contact.name || contact.number;
        
        if (nome) {
            contactCache[idContato] = nome;
            return nome;
        }
    } catch (e) {
        // Fallback silencioso em caso de erro da API
    }
    
    const fallback = idContato.split('@')[0];
    contactCache[idContato] = fallback;
    return fallback;
}

async function formatarMensagem(client, msg) {
    let nomeExibicao = "";
    let corNome = "";
    
    if (msg.fromMe) {
        nomeExibicao = "Eu";
        corNome = "{cyan-fg}";
    } else {
        const autorRealId = msg.author || msg.from;
        nomeExibicao = await obterNomeContato(client, autorRealId);
        
        const colors = ['{red-fg}', '{yellow-fg}', '{magenta-fg}', '{green-fg}'];
        corNome = colors[nomeExibicao.length % colors.length];
    }

    let conteudo = msg.body;
    if (!conteudo) {
         if (msg.hasMedia) conteudo = '[Mídia/Arquivo]';
         else conteudo = '[Mensagem do Sistema]';
    }

    return `${corNome}${blessed.escape(nomeExibicao)}:{/} ${conteudo}`;
}

module.exports = { obterNomeContato, formatarMensagem };
