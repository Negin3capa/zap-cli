use ratatui::style::Color;

/// Color themuse ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub system: Color,
    
    // Borders
    pub border: Color,           // Unfocused: transparent white
    pub border_focused: Color,   // Focused: brighter/colored
    
    // Message colors
    pub me: Color,
    pub other: Color,
    pub highlight: Color,
    pub primary: Color,
}

impl Theme {
    pub fn terminal() -> Self {
        Self {
            system: Color::DarkGray,
            
            // Transparent white border when unfocused
            border: Color::Gray,
            
            // Green when focused
            border_focused: Color::Green,
            
            // Message colors use terminal palette
            me: Color::Cyan,
            other: Color::Green,
            highlight: Color::Yellow,
            primary: Color::Blue,
        }
    }
}
