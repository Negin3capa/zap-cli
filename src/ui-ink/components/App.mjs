import React from "react";
import { Box, Text } from "ink";

/**
 * Main Ink application component
 * Replaces the neo-blessed TUI class
 *
 * Note: Using React.createElement instead of JSX to avoid needing a transpiler
 */
const App = ({ client }) => {
  const [status, setStatus] = React.useState("Initializing...");

  React.useEffect(() => {
    // Basic event listener setup
    const handleQR = (qr) => {
      setStatus("QR Code received - scan to login");
    };

    const handleReady = () => {
      setStatus("WhatsApp Client is Ready!");
    };

    const handleAuthenticated = () => {
      setStatus("Authenticated successfully!");
    };

    const handleAuthFailure = (msg) => {
      setStatus(`Auth failure: ${msg}`);
    };

    client.on("qr", handleQR);
    client.on("ready", handleReady);
    client.on("authenticated", handleAuthenticated);
    client.on("auth_failure", handleAuthFailure);

    return () => {
      // Cleanup listeners
      client.off("qr", handleQR);
      client.off("ready", handleReady);
      client.off("authenticated", handleAuthenticated);
      client.off("auth_failure", handleAuthFailure);
    };
  }, [client]);

  // Using React.createElement instead of JSX
  return React.createElement(
    Box,
    { flexDirection: "column", height: "100%" },
    React.createElement(
      Box,
      {
        borderStyle: "single",
        borderColor: "green",
        padding: 1,
        flexGrow: 1,
        justifyContent: "center",
        alignItems: "center",
      },
      React.createElement(
        Text,
        { color: "green", bold: true },
        "ZapTUI - Ink Version (Phase 1)",
      ),
    ),
    React.createElement(
      Box,
      { borderStyle: "single", borderColor: "blue", padding: 1 },
      React.createElement(Text, { color: "white" }, status),
    ),
  );
};

export default App;
