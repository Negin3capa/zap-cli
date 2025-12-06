const { Client, LocalAuth } = require("whatsapp-web.js");
const WebSocket = require("ws");

class WhatsAppService {
  constructor() {
    this.client = null;
    this.wss = null;
    this.clients = new Set();
    this.initializeClient();
    this.initializeWebSocket();
  }

  initializeClient() {
    console.log("Initializing WhatsApp client...");

    this.client = new Client({
      authStrategy: new LocalAuth({
        dataPath: process.env.ZAPTUI_AUTH_PATH || "../.wwebjs_auth",
      }),
      puppeteer: {
        headless: true,
        args: ["--no-sandbox", "--disable-setuid-sandbox"],
      },
    });

    // Forward WhatsApp events to all connected clients
    this.client.on("qr", (qr) => {
      console.log("QR Code received");
      this.broadcast({
        event: "qr",
        data: qr,
      });
    });

    this.client.on("ready", () => {
      console.log("WhatsApp client is ready!");
      this.broadcast({
        event: "ready",
      });
    });

    this.client.on("authenticated", () => {
      console.log("Authenticated");
      this.broadcast({
        event: "authenticated",
      });
    });

    this.client.on("auth_failure", (msg) => {
      console.error("Auth failure:", msg);
    });

    this.client.on("message", async (msg) => {
      const serialized = await this.serializeMessage(msg);
      this.broadcast({
        event: "message",
        data: serialized,
      });
    });

    this.client.on("message_create", async (msg) => {
      if (msg.fromMe) {
        const serialized = await this.serializeMessage(msg);
        this.broadcast({
          event: "message",
          data: serialized,
        });
      }
    });

    this.client.on("disconnected", (reason) => {
      console.log("Client disconnected:", reason);
      this.broadcast({
        event: "disconnected",
      });
    });

    this.client.initialize().catch((err) => {
      console.error("Failed to initialize client:", err);
    });
  }

  initializeWebSocket() {
    this.wss = new WebSocket.Server({ port: 8080 });
    console.log("WebSocket server listening on port 8080");

    this.wss.on("connection", (ws) => {
      console.log("New client connected");
      this.clients.add(ws);

      // Send initial state to new client
      if (this.client) {
        // If already ready/authenticated, tell the new client
        if (this.client.info) {
          console.log("Sending initial ready/auth state to new client");
          ws.send(JSON.stringify({ event: "ready" }));
          ws.send(JSON.stringify({ event: "authenticated" }));
        }
        // If unauthenticated, we don't need to do anything,
        // the client will wait for QR event which comes from event listener
      }

      ws.on("message", async (data) => {
        try {
          const request = JSON.parse(data);
          const response = await this.handleRequest(request);
          ws.send(JSON.stringify(response));
        } catch (err) {
          console.error("Error handling request:", err);
          ws.send(
            JSON.stringify({
              id: request?.id || "unknown",
              error: err.message,
            }),
          );
        }
      });

      ws.on("close", () => {
        console.log("Client disconnected");
        this.clients.delete(ws);
      });

      ws.on("error", (err) => {
        console.error("WebSocket error:", err);
        this.clients.delete(ws);
      });
    });
  }

  async handleRequest(req) {
    const { id, method, params } = req;

    try {
      let result;

      switch (method) {
        case "getChats":
          result = await this.getChats();
          break;

        case "getMessages":
          result = await this.getMessages(params.chatId, params.limit);
          break;

        case "sendMessage":
          result = await this.sendMessage(params.chatId, params.text);
          break;

        case "downloadMedia":
          result = await this.downloadMedia(params.messageId);
          break;

        default:
          throw new Error(`Unknown method: ${method}`);
      }

      return { id, result };
    } catch (err) {
      return { id, error: err.message };
    }
  }

  async getChats() {
    console.log("getChats called - starting chat sync...");
    const startTime = Date.now();

    const chats = await this.client.getChats();

    const elapsedSeconds = ((Date.now() - startTime) / 1000).toFixed(1);
    console.log(`Found ${chats.length} chats (took ${elapsedSeconds}s)`);

    const serialized = chats.map((chat) => this.serializeChat(chat));
    console.log("Sample chat:", JSON.stringify(serialized[0], null, 2));
    return serialized;
  }

  async getMessages(chatId, limit = 50) {
    const chat = await this.client.getChatById(chatId);
    const messages = await chat.fetchMessages({ limit });
    return Promise.all(messages.map((msg) => this.serializeMessage(msg)));
  }

  async sendMessage(chatId, text) {
    await this.client.sendMessage(chatId, text);
    return { success: true };
  }

  async downloadMedia(messageId) {
    // This would need to be implemented with message lookup
    // For now, return placeholder
    return null;
  }

  serializeChat(chat) {
    const serialized = {
      id: chat.id._serialized,
      name: chat.name || chat.id.user || "Unknown",
      is_group: chat.isGroup || false,
      unread_count: chat.unreadCount || 0,
      archived: chat.archived || false,
      timestamp: chat.timestamp || Math.floor(Date.now() / 1000),
      last_message: chat.lastMessage?.body || null,
    };
    return serialized;
  }

  async serializeMessage(msg) {
    let sender = null;
    if (!msg.fromMe) {
      try {
        const contact = await msg.getContact();
        // Prioritize: pushname > saved name > WhatsApp name > number
        sender =
          contact.pushname ||
          contact.name ||
          contact.verifiedName ||
          contact.number;
      } catch (e) {
        // Fallback to number if contact fetch fails
        sender =
          msg._data?.notifyName ||
          msg.author?.split("@")[0] ||
          msg.from?.split("@")[0] ||
          "Unknown";
      }
    }

    return {
      id: msg.id._serialized,
      chat_id: msg.from || msg.to,
      body: msg.body || "",
      timestamp: msg.timestamp,
      from_me: msg.fromMe,
      has_media: msg.hasMedia,
      media_type: msg.type,
      sender,
    };
  }

  broadcast(message) {
    const data = JSON.stringify(message);
    this.clients.forEach((client) => {
      if (client.readyState === WebSocket.OPEN) {
        client.send(data);
      }
    });
  }
}

// Start the service
const service = new WhatsAppService();

// Handle graceful shutdown
process.on("SIGINT", async () => {
  console.log("Shutting down...");
  if (service.client) {
    await service.client.destroy();
  }
  if (service.wss) {
    service.wss.close();
  }
  process.exit(0);
});
