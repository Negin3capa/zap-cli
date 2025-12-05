# Configuration Guide

ZapTUI uses TOML for configuration.

## Configuration File Location

**Default**: `~/.config/zaptui/config.toml`

On first run, ZapTUI will create this directory and a default config file.

## Configuration Options

### UI Settings

```toml
[ui]
theme = "terminal"  # Uses your terminal's color scheme
```

### WhatsApp Service

```toml
[whatsapp]
service_url = "ws://localhost:8080"
```

## Example Configuration

See [`config.example.toml`](../config.example.toml) for a full example with all available options.

## Customization

- **Terminal Colors**: ZapTUI automatically adapts to your terminal theme (Kitty, Alacritty, etc.)
- **Keyboard Shortcuts**: Currently hardcoded, will be customizable in future versions
