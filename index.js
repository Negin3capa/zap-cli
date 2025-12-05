const fs = require("fs");
const WhatsAppClient = require("./src/client");

// Main async function to support dynamic imports for Ink
async function main() {
  // Load config
  let config = {};
  try {
    config = JSON.parse(fs.readFileSync("./config.json", "utf8"));
  } catch (e) {
    console.error("Failed to load config.json");
    process.exit(1);
  }

  // Initialize Client
  const client = new WhatsAppClient(config);

  // Initialize UI - support both neo-blessed and Ink
  const USE_INK = process.env.USE_INK === "true";
  let ui;

  if (USE_INK) {
    console.log("🚀 Starting ZapTUI with Ink UI...");
    // Dynamic import for Ink (ESM module)
    const { default: InkUI } = await import("./src/ui-ink/index.mjs");
    ui = new InkUI(client);
  } else {
    console.log("📺 Starting ZapTUI with neo-blessed UI...");
    const TUI = require("./src/ui");
    ui = new TUI(client);
  }

  // Connect Events
  client.on("qr", (qr) => {
    ui.showQR(qr);
    ui.log("Scan QR Code to login");
  });

  client.on("ready", async () => {
    ui.log("WhatsApp Client is Ready!");
    ui.log("Loading chats...");
    const chats = await client.getChats();
    ui.setChats(chats);
    ui.log("Chats loaded.");
  });

  client.on("authenticated", () => {
    ui.log("Authenticated successfully!");
  });

  client.on("auth_failure", (msg) => {
    ui.log("Auth failure: " + msg);
  });

  client.on("message", (msg) => {
    ui.appendMessage(msg);
    // If we receive a message and it's not the current chat,
    // we might want to update the chat list sorting or show a badge.
    // For now, simple append if it matches current chat is handled in ui.appendMessage
  });

  // Start
  ui.log("Initializing WhatsApp");

  let dotCount = 0;
  const loadingInterval = setInterval(() => {
    dotCount++;
    const dots = ".".repeat(dotCount);
    ui.log("Initializing WhatsApp" + dots);
  }, 9000);

  const initTimeout = setTimeout(() => {
    ui.log("{yellow-fg}Initialization is taking longer than expected...{/}");
    ui.log("If stuck, stop the app and try deleting the .wwebjs_auth folder.");
  }, 30000);

  const cleanupInit = () => {
    clearInterval(loadingInterval);
    clearTimeout(initTimeout);
  };

  client.on("ready", () => cleanupInit());
  client.on("qr", () => cleanupInit());

  client.initialize().catch((err) => {
    cleanupInit();
    ui.log("{red-fg}Error initializing client: " + err.message + "{/}");
    if (err.message.includes("session")) {
      ui.log("Try deleting the .wwebjs_auth folder and restarting.");
    }
  });
}

// Start the application
main().catch((err) => {
  console.error("Failed to start application:", err);
  process.exit(1);
});
