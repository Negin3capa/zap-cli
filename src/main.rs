use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use tokio::sync::mpsc;

mod config;
mod ui;
mod whatsapp;

use config::Config;
use ui::App;
use whatsapp::{WhatsAppClient, WhatsAppEvent};

/// ZapTUI - WhatsApp Terminal User Interface
#[derive(Parser, Debug)]
#[command(name = "zaptui")]
#[command(version, about = "A fast and beautiful TUI for WhatsApp", long_about = None)]
struct Cli {
    // No additional arguments for now, but --version and --help are automatic
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments (this handles --version and --help automatically)
    let _cli = Cli::parse();

    // Initialize logging
    env_logger::init();
    log::info!("Starting ZapTUI v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = Config::load()?;
    log::info!("Configuration loaded from: {}", config.config_path.display());

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let result = run_app(&mut terminal, config).await;

    // Flush pending input events before cleanup to avoid escape codes leaking to terminal
    while event::poll(std::time::Duration::from_millis(0))? {
        let _ = event::read()?;
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableMouseCapture, LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Print any errors
    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, config: Config) -> Result<()> {
    // Create channels for WhatsApp events
    let (event_tx, mut event_rx) = mpsc::channel::<WhatsAppEvent>(100);

    // Connect to WhatsApp service
    log::info!("Connecting to WhatsApp service at {}", config.whatsapp.service_url);
    let whatsapp_client = WhatsAppClient::connect(&config.whatsapp.service_url, event_tx.clone()).await?;
    
    // Create app state
    let mut app = App::new(config.clone(), whatsapp_client, event_tx.clone());

    // Create a periodic sync timer (every 30 seconds for current chat)
    let mut sync_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

    // Background sync for chats with updates (every 2 minutes)
    let mut background_sync_interval = tokio::time::interval(tokio::time::Duration::from_secs(120));

    // Dirty flag for conditional rendering
    let mut needs_render = true;

    // Main event loop
    loop {
        // Only render if something changed
        if needs_render {
            terminal.draw(|frame| {
                app.render(frame);
            })?;
            needs_render = false;
        }

        // Handle events with priority on user input
        tokio::select! {
            biased;  // Process branches in order for input priority
            
            // Terminal events (keyboard/mouse) - HIGHEST PRIORITY
            result = tokio::task::spawn_blocking(|| {
                // Poll with minimal timeout for near-instant response
                event::poll(std::time::Duration::from_millis(10))
                    .and_then(|has_event| {
                        if has_event {
                            event::read().map(Some)
                        } else {
                            Ok(None)
                        }
                    })
            }) => {
                if let Ok(Ok(Some(terminal_event))) = result {
                    // Handle Ctrl+C to quit
                    if let Event::Key(key) = &terminal_event {
                        if key.kind == KeyEventKind::Press {
                            if key.code == KeyCode::Char('c') 
                                && key.modifiers.contains(event::KeyModifiers::CONTROL) {
                                log::info!("User requested quit via Ctrl+C");
                                break;
                            }
                        }
                    }
                    
                    // Let app handle other events
                    if app.handle_event(terminal_event).await? {
                        break; // App requested quit
                    }
                    
                    // Mark for re-render after input
                    needs_render = true;
                }
            }
            
            // WhatsApp events from service
            Some(wa_event) = event_rx.recv() => {
                app.handle_whatsapp_event(wa_event).await?;
                needs_render = true;
            }
            
            // Periodic sync (lowest priority)
            _ = sync_interval.tick() => {
                // Only sync if we're in Ready state and have a current chat
                if let Err(e) = app.refresh_current_chat_messages().await {
                    log::warn!("Periodic sync failed: {}", e);
                }
                needs_render = true;
            }

            // Background sync for chats with updates (every 2 minutes)
            _ = background_sync_interval.tick() => {
                app.background_sync_updated_chats();
                needs_render = true;
            }
        }
    }

    Ok(())
}
