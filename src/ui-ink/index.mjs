/**
 * Ink UI entry point (ESM module)
 * Provides the same API as the neo-blessed TUI class for compatibility
 *
 * Note: This is an ESM module to work with Ink v4+
 */

import React from "react";
import { render } from "ink";
import App from "./components/App.mjs";

class InkUI {
  constructor(client) {
    this.client = client;
    this.inkInstance = null;
    this.initialized = false;

    // Render the Ink app immediately
    this.init();
  }

  init() {
    try {
      // Render the Ink app
      const { waitUntilExit } = render(
        React.createElement(App, { client: this.client }),
      );

      this.initialized = true;

      // Store the exit promise for cleanup
      this.exitPromise = waitUntilExit();
    } catch (error) {
      console.error("Failed to initialize Ink UI:", error);
      process.exit(1);
    }
  }

  /**
   * Legacy API compatibility methods
   * These are called from index.js
   */

  log(msg) {
    // Status updates are handled by the App component via events
    // This method exists for API compatibility
  }

  showQR(qr) {
    // QR display will be handled by QRCodeView component
    // For now, emit the qr event which App listens to
  }

  setChats(chats) {
    // Will be handled by ChatList component
    // For now, this is a no-op
  }

  appendMessage(msg) {
    // Real-time message updates will be handled by MessageList component
    // For now, this is a no-op for compatibility
  }
}

export default InkUI;
