use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub config_path: PathBuf,
    
    pub whatsapp: WhatsAppConfig,
    pub media: MediaConfig,
    pub ui: UiConfig,
    pub notifications: NotificationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConfig {
    #[serde(default = "default_service_url")]
    pub service_url: String,
    
    #[serde(default = "default_session_path")]
    pub session_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaConfig {
    #[serde(default = "default_download_path")]
    pub download_path: String,
    
    #[serde(default = "default_true")]
    pub auto_download: bool,
    
    #[serde(default = "default_true")]
    pub kitty_graphics: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    
    #[serde(default = "default_true")]
    pub show_avatars: bool,
    
    #[serde(default = "default_true")]
    pub relative_timestamps: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    #[serde(default)]
    pub muted_chats: Vec<String>,
}

// Default value functions
fn default_service_url() -> String {
    "ws://localhost:8080".to_string()
}

fn default_session_path() -> String {
    "./.wwebjs_auth".to_string()
}

fn default_download_path() -> String {
    "./media".to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_true() -> bool {
    true
}

impl Config {
    /// Load configuration from config.toml, or create default if not found
    pub fn load() -> Result<Self> {
        // Try XDG config directory first (~/.config/zaptui/)
        let config_dir = dirs::config_dir()
            .context("Could not find config directory")?
            .join("zaptui");
        
        let config_path = config_dir.join("config.toml");
        
        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
            log::info!("Created config directory: {:?}", config_dir);
        }
        
        // Create default config if it doesn't exist
        if !config_path.exists() {
            let default_config = Self::default();
            let toml_string = toml::to_string_pretty(&default_config)?;
            std::fs::write(&config_path, toml_string)?;
            log::info!("Created default config at: {:?}", config_path);
        }
        
        // Load config
        let config_str = std::fs::read_to_string(&config_path)?;
        let mut config: Config = toml::from_str(&config_str)?;
        
        config.config_path = config_path; // Set the path where it was loaded from
        
        log::info!("Loaded config from: {:?}", config.config_path);
        Ok(config)
    }
    
    /// Migrate from old JavaScript config.json format
    #[allow(dead_code)]
    fn migrate_from_json(json_path: &Path) -> Result<Self> {
        #[derive(Deserialize)]
        struct OldConfig {
            #[serde(default)]
            session_path: Option<String>,
            #[serde(default)]
            download_media: Option<bool>,
            #[serde(default)]
            download_path: Option<String>,
            #[serde(default)]
            notifications: Option<bool>,
        }
        
        log::info!("Migrating from config.json to config.toml");
        let content = std::fs::read_to_string(json_path)?;
        let old: OldConfig = serde_json::from_str(&content)?;
        
        // Backup old config
        std::fs::copy(json_path, "config.json.bak")?;
        log::info!("Backed up old config to config.json.bak");
        
        Ok(Config {
            config_path: PathBuf::from("config.toml"),
            whatsapp: WhatsAppConfig {
                service_url: default_service_url(),
                session_path: old.session_path.unwrap_or_else(default_session_path),
            },
            media: MediaConfig {
                download_path: old.download_path.unwrap_or_else(default_download_path),
                auto_download: old.download_media.unwrap_or(true),
                kitty_graphics: true,
            },
            ui: UiConfig {
                theme: default_theme(),
                show_avatars: true,
                relative_timestamps: true,
            },
            notifications: NotificationConfig {
                enabled: old.notifications.unwrap_or(true),
                muted_chats: vec![],
            },
        })
    }
    
    /// Save configuration to config.toml
    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        std::fs::write(&self.config_path, content)
            .context("Failed to write config.toml")?;
        log::info!("Configuration saved to {}", self.config_path.display());
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from("config.toml"),
            whatsapp: WhatsAppConfig {
                service_url: default_service_url(),
                session_path: default_session_path(),
            },
            media: MediaConfig {
                download_path: default_download_path(),
                auto_download: true,
                kitty_graphics: true,
            },
            ui: UiConfig {
                theme: default_theme(),
                show_avatars: true,
                relative_timestamps: true,
            },
            notifications: NotificationConfig {
                enabled: true,
                muted_chats: vec![],
            },
        }
    }
}
