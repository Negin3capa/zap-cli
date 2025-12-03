# ZAP-CLI (Terminal User Interface)

Cliente WhatsApp minimalista e otimizado para rodar diretamente no terminal. Desenvolvido em Node.js, utilizando `whatsapp-web.js` para conexão e `blessed` para a interface gráfica.

## 🚀 Funcionalidades

- **Interface Gráfica no Terminal (TUI):** Navegação por mouse e teclado.
- **Leve e Rápido:** Carregamento otimizado de contatos e mensagens (Lazy Loading).
- **Tempo Real:** Recebimento e envio de mensagens com atualização instantânea.
- **Suporte a Grupos:** Identificação colorida de participantes em grupos.
- **Modo Seguro:** Tratamento de erros para atualizações recentes do WhatsApp Web (evita crash em `getIsMyContact`).

## 🛠️ Instalação

1. **Pré-requisitos:** Node.js instalado (v14 ou superior).
2. **Clone/Baixe o projeto.**
3. **Instale as dependências:**

```bash
npm install whatsapp-web.js qrcode-terminal blessed chalk
```

## 💻 Como Usar

Para iniciar a aplicação:

```bash
node index.js
```

1. Na primeira execução, um **QR Code** será exibido no terminal.
2. Abra seu WhatsApp no celular > Aparelhos Conectados > Conectar Aparelho.
3. Escaneie o código.
4. Aguarde a interface carregar.

### Comandos da Interface

| Ação | Teclado | Mouse |
|Data | --- | --- |
| **Navegar na Lista** | Setas `↑` `↓` | Rolar Scroll / Clicar |
| **Selecionar Chat** | `Enter` | Clique Esquerdo |
| **Enviar Mensagem** | `Enter` (no input) | Botão "Enviar" (se houver) |
| **Sair** | `Ctrl + C` | - |

## 📂 Estrutura do Projeto

O projeto foi modularizado para facilitar manutenção:

- **`index.js`**: Controlador principal. Gerencia o fluxo entre o Cliente WhatsApp e a Interface.
- **`src/config.js`**: Configurações do Puppeteer e constantes.
- **`src/client.js`**: Lógica de inicialização do bot (separada se necessário).
- **`src/ui.js`**: Construção da interface visual (Blessed). Define caixas, listas e inputs.
- **`src/utils.js`**: Funções auxiliares, como formatação de data, cores de mensagens e cache de nomes de contatos.

## ⚠️ Solução de Problemas Comuns

**Erro: `TypeError: window.Store.ContactMethods.getIsMyContact is not a function`**
Este erro ocorre devido a atualizações do WhatsApp Web. Este cliente possui um tratamento `try/catch` no arquivo `src/utils.js` para contornar isso automaticamente, exibindo o número do contato caso o nome falhe ao carregar.

**Sessão desconectando:**
O arquivo de sessão é salvo na pasta `.wwebjs_auth`. Se tiver problemas de login, apague esta pasta e escaneie o QR Code novamente.

---
*Desenvolvido para fins educacionais. Use com responsabilidade.*
