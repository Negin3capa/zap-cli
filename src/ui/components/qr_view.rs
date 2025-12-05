use qrcode::QrCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use super::super::Theme;

pub struct QRView<'a> {
    qr_data: &'a str,
    status: &'a str,
    theme: &'a Theme,
}

impl<'a> QRView<'a> {
    pub fn new(qr_data: &'a str, status: &'a str, theme: &'a Theme) -> Self {
        Self {
            qr_data,
            status,
            theme,
        }
    }
    
    fn generate_qr_text(&self) -> Vec<Line<'a>> {
        match QrCode::new(self.qr_data) {
            Ok(qr) => {
                // Use unicode half-block characters to make QR codes more square
                // Terminal cells are roughly 1:2 (width:height), so we use half blocks
                let colors = qr.to_colors();
                let width = (colors.len() as f64).sqrt() as usize;
                
                let mut lines = Vec::new();
                
                // Process two rows at a time using half-blocks
                for y in (0..width).step_by(2) {
                    let mut line_str = String::new();
                    line_str.push_str("  "); // Small padding
                    
                    for x in 0..width {
                        use qrcode::Color;
                        let top = match colors[x + y * width] {
                            Color::Dark => true,
                            Color::Light => false,
                        };
                        let bottom = if y + 1 < width {
                            match colors[x + (y + 1) * width] {
                                Color::Dark => true,
                                Color::Light => false,
                            }
                        } else {
                            false
                        };
                        
                        // Use Unicode half-block characters
                        let ch = match (top, bottom) {
                            (false, false) => ' ',  // Both white
                            (true, true) => '█',    // Both black  
                            (true, false) => '▀',   // Top black, bottom white
                            (false, true) => '▄',   // Top white, bottom black
                        };
                        line_str.push(ch);
                    }
                    
                    line_str.push_str("  "); // Small padding
                    lines.push(Line::from(line_str));
                }
                
                lines
            }
            Err(_) => {
                vec![Line::from("Failed to generate QR code")]
            }
        }
    }
}

impl<'a> Widget for QRView<'a> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        // Calculate proper layout based on terminal size
        let title_height = 3;
        let instructions_height = 12;
        let min_qr_height = 25;  // Minimum height for QR code
        
        // Ensure we have enough space for QR code
        let qr_height = area.height.saturating_sub(title_height + instructions_height).max(min_qr_height);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(title_height),
                Constraint::Length(qr_height),
                Constraint::Min(instructions_height),
            ])
            .split(area);
        
        // Title
        let title = Paragraph::new("WhatsApp Authentication")
            .alignment(Alignment::Center)
            .style(Style::default()
                .fg(self.theme.primary)
                .add_modifier(Modifier::BOLD));
        title.render(chunks[0], buf);
        
        // QR Code - centered and properly sized
        let qr_lines = self.generate_qr_text();
        let qr = Paragraph::new(qr_lines)
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Scan this code ")
                .border_style(Style::default().fg(self.theme.border)));
        qr.render(chunks[1], buf);
        
        // Instructions - more compact
        let instructions = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("📱 ", Style::default().fg(self.theme.highlight)),
                Span::styled("Open WhatsApp → Linked Devices → Link a Device → Scan", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(self.status, Style::default().fg(self.theme.system)),
            ]),
        ];
        
        let instructions_widget = Paragraph::new(instructions)
            .alignment(Alignment::Center);
        instructions_widget.render(chunks[2], buf);
    }
}
