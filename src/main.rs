use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
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
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let result = run_app(&mut terminal, config).await;

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
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
    let whatsapp_client = WhatsAppClient::connect(&config.whatsapp.service_url, event_tx).await?;
    
    // Create app state
    let mut app = App::new(config.clone(), whatsapp_client);

    // Main event loop
    loop {
        // Render UI
        terminal.draw(|frame| {
            app.render(frame);
        })?;

        // Handle events
        tokio::select! {
            // WhatsApp events from service
            Some(wa_event) = event_rx.recv() => {
                app.handle_whatsapp_event(wa_event).await?;
            }
            
            // Terminal events (keyboard/mouse)
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                if event::poll(tokio::time::Duration::from_millis(0))? {
                    let terminal_event = event::read()?;
                    
                    // Handle Ctrl+C to quit (check this FIRST before app.handle_event)
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
                }
            }
        }
    }

    Ok(())
}
