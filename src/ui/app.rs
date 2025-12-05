use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::collections::HashMap;

use crate::config::Config;
use crate::whatsapp::{Chat, Message, WhatsAppClient, WhatsAppEvent};
use super::theme::Theme;
use super::components::qr_view::QRView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppState {
    Authenticating,
    Ready,
    Disconnected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedWidget {
    ChatList,
    MessageView,
    Input,
}

pub struct App {
    theme: Theme,
    client: WhatsAppClient,
    state: AppState,
    
    // Data
    chats: Vec<Chat>,
    current_chat_id: Option<String>,
    messages: HashMap<String, Vec<Message>>,
    
    // UI State
    focused: FocusedWidget,
    chat_list_state: ListState,
    message_scroll: u16,  // Scroll offset for message view
    input_buffer: String,
    
    // Authentication
    qr_code: Option<String>,
    status_message: String,
}

impl App {
    pub fn new(_config: Config, client: WhatsAppClient) -> Self {
        let theme = Theme::terminal();  // Always use terminal theme
        
        Self {
            theme,
            client,
            state: AppState::Authenticating,
            chats: Vec::new(),
            current_chat_id: None,
            messages: HashMap::new(),
            focused: FocusedWidget::ChatList,
            chat_list_state: ListState::default(),
            message_scroll: 0,
            input_buffer: String::new(),
            qr_code: None,
            status_message: "Connecting to WhatsApp...".to_string(),
        }
    }
    
    /// Handle WhatsApp events
    pub async fn handle_whatsapp_event(&mut self, event: WhatsAppEvent) -> Result<()> {
        match event {
            WhatsAppEvent::QRCode(qr) => {
                log::info!("Received QR code");
                self.qr_code = Some(qr);
                self.status_message = "Scan QR code to authenticate".to_string();
            }
            
            WhatsAppEvent::Authenticated => {
                log::info!("Authenticated");
                self.status_message = "Authenticated! Loading chats...".to_string();
            }
            
            WhatsAppEvent::Ready => {
                log::info!("Ready");
                self.state = AppState::Ready;
                self.qr_code = None;
                self.status_message = "Loading chats (this may take a minute)...".to_string();
                
                // Load chats with retry
                let mut attempts = 0;
                let max_attempts = 3;
                
                while attempts < max_attempts {
                    attempts += 1;
                    log::info!("Loading chats (attempt {}/{})", attempts, max_attempts);
                    
                    match self.client.get_chats().await {
                        Ok(chats) => {
                            log::info!("Loaded {} chats", chats.len());
                            self.chats = chats;
                            if !self.chats.is_empty() && self.chat_list_state.selected().is_none() {
                                self.chat_list_state.select(Some(0));
                            }
                            self.status_message = format!("Ready - {} chats loaded", self.chats.len());
                            break;
                        }
                        Err(e) => {
                            log::error!("Failed to load chats (attempt {}): {}", attempts, e);
                            log::error!("Error details: {:?}", e);
                            if attempts < max_attempts {
                                self.status_message = format!("Retrying... (attempt {}/{})", attempts + 1, max_attempts);
                                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                            } else {
                                self.status_message = format!("Error: Check logs. Press 'q' to quit.");
                            }
                        }
                    }
                }
            }
            
            WhatsAppEvent::MessageReceived(msg) => {
                log::debug!("Received message in chat {}", msg.chat_id);
                
                // Add to messages
                self.messages.entry(msg.chat_id.clone())
                    .or_insert_with(Vec::new)
                    .push(msg.clone());
                
                // Update chat's last message
                if let Some(chat) = self.chats.iter_mut().find(|c| c.id == msg.chat_id) {
                    chat.last_message = Some(msg.body.clone());
                    chat.timestamp = msg.timestamp;
                    if !msg.from_me {
                        chat.unread_count += 1;
                    }
                }
            }
            
            WhatsAppEvent::ChatUpdated(updated_chat) => {
                if let Some(chat) = self.chats.iter_mut().find(|c| c.id == updated_chat.id) {
                    *chat = updated_chat;
                } else {
                    self.chats.push(updated_chat);
                }
            }
            
            WhatsAppEvent::Disconnected => {
                log::warn!("Disconnected from WhatsApp");
                self.state = AppState::Disconnected;
                self.status_message = "Disconnected. Reconnecting...".to_string();
            }
        }
        
        Ok(())
    }
    
    /// Handle terminal events
    pub async fn handle_event(&mut self, event: Event) -> Result<bool> {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return Ok(false);
            }
            return self.handle_key(key).await;
        }
        
        Ok(false)
    }
    
    async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Tab cycles focus
        if key.code == KeyCode::Tab {
            self.focused = match self.focused {
                FocusedWidget::ChatList => FocusedWidget::MessageView,
                FocusedWidget::MessageView => FocusedWidget::Input,
                FocusedWidget::Input => FocusedWidget::ChatList,
            };
            return Ok(false);
        }
        
        // Route based on current focus
        match self.focused {
            FocusedWidget::ChatList => {
                // Only j/k/up/down navigate chat list
                if matches!(key.code, KeyCode::Up | KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('k')) {
                    return self.handle_chat_list_key(key).await;
                }
                // All other keys go to input (auto-focus on typing)
                self.focused = FocusedWidget::Input;
                self.handle_input_key(key).await
            }
            FocusedWidget::MessageView => {
                // Arrow keys scroll messages
                if matches!(key.code, KeyCode::Up | KeyCode::Down) {
                    self.handle_message_scroll(key);
                    return Ok(false);
                }
                // Any other key goes to input (auto-focus on typing)
                self.focused = FocusedWidget::Input;
                self.handle_input_key(key).await
            }
            FocusedWidget::Input => {
                // All keys handled by input
                self.handle_input_key(key).await
            }
        }
    }
    
    fn handle_message_scroll(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Down => {
                self.message_scroll = self.message_scroll.saturating_add(1);
            }
            KeyCode::Up => {
                self.message_scroll = self.message_scroll.saturating_sub(1);
            }
            _ => {}
        }
    }
    
    async fn handle_chat_list_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                let i = match self.chat_list_state.selected() {
                    Some(i) => {
                        if i >= self.chats.len().saturating_sub(1) {
                            i
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.chat_list_state.select(Some(i));
                
                // Clear input and reset scroll when changing chats
                self.input_buffer.clear();
                self.message_scroll = 0;
                
                // Load messages in background immediately
                self.load_chat_messages_background(i).await?;
            }
            
            KeyCode::Up | KeyCode::Char('k') => {
                let i = match self.chat_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            0
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.chat_list_state.select(Some(i));
                
                // Clear input and reset scroll when changing chats
                self.input_buffer.clear();
                self.message_scroll = 0;
                
                // Load messages in background immediately
                self.load_chat_messages_background(i).await?;
            }
            
            _ => {}
        }
        
        Ok(false)
    }
    
    async fn handle_input_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Enter => {
                // Enter ONLY sends messages (or will handle multi-line in future)
                if !self.input_buffer.is_empty() {
                    return self.send_current_message().await;
                }
            }
            
            KeyCode::Char(c) => {
                // All typing goes to input - input is always active
                self.input_buffer.push(c);
            }
            
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            
            KeyCode::Esc => {
                // Clear input buffer
                self.input_buffer.clear();
            }
            
            _ => {}
        }
        
        Ok(false)
    }
    
    async fn load_chat_messages_background(&mut self, chat_index: usize) -> Result<()> {
        if let Some(chat) = self.chats.get(chat_index) {
            let chat_id = chat.id.clone();
            let chat_name = chat.name.clone();
            
            // Set as current chat immediately
            self.current_chat_id = Some(chat_id.clone());
            
            // Load messages if not cached
            if !self.messages.contains_key(&chat_id) {
                self.status_message = format!("Loading {} messages...", chat_name);
                log::info!("Loading messages for chat: {}", chat_name);
                
                match self.client.get_messages(&chat_id, 50).await {
                    Ok(messages) => {
                        let count = messages.len();
                        self.messages.insert(chat_id.clone(), messages);
                        self.status_message = format!("{} - {} messages", chat_name, count);
                    }
                    Err(e) => {
                        log::error!("Failed to load messages: {}", e);
                        self.status_message = format!("Error: {}", e);
                    }
                }
            } else {
                // Already cached - instant!
                let count = self.messages.get(&chat_id).map(|m| m.len()).unwrap_or(0);
                self.status_message = format!("{} - {} messages (cached)", chat_name, count);
            }
            
            // Mark as read
            if let Some(chat) = self.chats.get_mut(chat_index) {
                chat.unread_count = 0;
            }
        }
        
        Ok(())
    }
    
    async fn send_current_message(&mut self) -> Result<bool> {
        if let Some(chat_id) = &self.current_chat_id {
            let text = self.input_buffer.clone();
            log::info!("Sending message to {}: {}", chat_id, text);
            
            // Add message to UI immediately for instant feedback
            let sent_msg = Message {
                id: format!("temp_{}", chrono::Utc::now().timestamp()),
                chat_id: chat_id.clone(),
                body: text.clone(),
                timestamp: chrono::Utc::now().timestamp(),
                from_me: true,
                has_media: false,
                media_type: None,
                sender: None,
            };
            
            self.messages.entry(chat_id.clone())
                .or_insert_with(Vec::new)
                .push(sent_msg);
            
            // Actually send via API
            if let Err(e) = self.client.send_message(chat_id, &text).await {
                log::error!("Failed to send message: {}", e);
                self.status_message = format!("Error sending message: {}", e);
            } else {
                log::info!("Message sent successfully");
                self.input_buffer.clear();
            }
        }
        
        Ok(false)
    }
    
    /// Render the UI
    pub fn render(&mut self, frame: &mut Frame) {
        // Show QR code during authentication
        if self.state == AppState::Authenticating && self.qr_code.is_some() {
            self.render_qr(frame);
            return;
        }
        
        // Show loading screen if not ready or no chats loaded yet
        if self.state != AppState::Ready || self.chats.is_empty() {
            self.render_loading(frame);
            return;
        }
        
        // Main chat interface
        self.render_main(frame);
    }
    
    fn render_loading(&self, frame: &mut Frame) {
        let loading_text = vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(self.theme.highlight)),
                Span::styled(&self.status_message, Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(Span::styled("Please wait...", Style::default().fg(self.theme.system))),
        ];
        
        let loading = Paragraph::new(loading_text)
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" ZapTUI ")
                .border_style(Style::default().fg(self.theme.border)));
        
        frame.render_widget(loading, frame.area());
    }
    
    fn render_qr(&mut self, frame: &mut Frame) {
        let qr_view = QRView::new(
            self.qr_code.as_ref().unwrap(),
            &self.status_message,
            &self.theme,
        );
        frame.render_widget(qr_view, frame.area());
    }
    
    fn render_main(&mut self, frame: &mut Frame) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),  // Status bar
            ])
            .split(frame.area());
        
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(70),
            ])
            .split(main_chunks[0]);
        
        // Render chat list
        self.render_chat_list(frame, chunks[0]);
        
        // Render right pane
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(chunks[1]);
        
        self.render_messages(frame, right_chunks[0]);
        self.render_input(frame, right_chunks[1]);
        
        // Render status bar
        self.render_status_bar(frame, main_chunks[1]);
    }
    
    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let status = Paragraph::new(self.status_message.as_str())
            .style(Style::default().fg(self.theme.system));
        frame.render_widget(status, area);
    }
    
    fn render_chat_list(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.chats.iter().map(|chat| {
            let name = &chat.name;
            let unread = if chat.unread_count > 0 {
                format!(" ({})", chat.unread_count)
            } else {
                String::new()
            };
            
            let content = format!("{}{}", name, unread);
            ListItem::new(Line::from(content))
        }).collect();
        
        let border_color = if self.focused == FocusedWidget::ChatList {
            self.theme.border_focused
        } else {
            self.theme.border
        };
        
        let list = List::new(items)
            .block(Block::default()
                .title(" Chats ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)))
            .highlight_style(Style::default()
                .bg(self.theme.primary)
                .add_modifier(Modifier::BOLD))
            .highlight_symbol("► ");
        
        frame.render_stateful_widget(list, area, &mut self.chat_list_state);
    }
    
    fn render_messages(&self, frame: &mut Frame, area: Rect) {
        let title = if let Some(chat_id) = &self.current_chat_id {
            self.chats.iter()
                .find(|c| c.id == *chat_id)
                .map(|c| format!(" {} ", c.name))
                .unwrap_or_else(|| " Messages ".to_string())
        } else {
            " Messages ".to_string()
        };
        
        let messages_text = if let Some(chat_id) = &self.current_chat_id {
            if let Some(messages) = self.messages.get(chat_id) {
                let lines: Vec<Line> = messages.iter().map(|msg| {
                    let sender = if msg.from_me {
                        Span::styled("Me", Style::default().fg(self.theme.me))
                    } else {
                        Span::styled(
                            msg.sender.as_deref().unwrap_or("User"),
                            Style::default().fg(self.theme.other)
                        )
                    };
                    
                    let time = chrono::DateTime::from_timestamp(msg.timestamp, 0)
                        .map(|dt| dt.format("%H:%M").to_string())
                        .unwrap_or_default();
                    
                    let body = if msg.has_media {
                        format!("[Media: {}]", msg.media_type.as_deref().unwrap_or("unknown"))
                    } else {
                        msg.body.clone()
                    };
                    
                    Line::from(vec![
                        Span::styled(time, Style::default().fg(self.theme.system)),
                        Span::raw(" "),
                        sender,
                        Span::raw(": "),
                        Span::raw(body),
                    ])
                }).collect();
                
                lines
            } else {
                vec![Line::from("Loading messages...")]
            }
        } else {
            vec![Line::from("Select a chat to view messages")]
        };
        
        // Calculate scroll offset
        let num_lines = messages_text.len();
        let available_height = area.height.saturating_sub(2) as usize;
        
        // Use manual scroll if > 0, otherwise auto-scroll to bottom
        let scroll_offset = if self.message_scroll > 0 {
            self.message_scroll
        } else if num_lines > available_height {
            (num_lines - available_height) as u16
        } else {
            0
        };
        
        // Border color based on focus
        let border_color = if self.focused == FocusedWidget::MessageView {
            self.theme.border_focused
        } else {
            self.theme.border
        };
        
        let paragraph = Paragraph::new(messages_text)
            .block(Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)))
            .scroll((scroll_offset, 0));
        
        frame.render_widget(paragraph, area);
    }
    
    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let border_color = if self.focused == FocusedWidget::Input {
            self.theme.border_focused
        } else {
            self.theme.border
        };
        
        let input = Paragraph::new(self.input_buffer.as_str())
            .block(Block::default()
                .title(" Type message (Enter: send, Esc: clear, Tab: change focus) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)));
        
        frame.render_widget(input, area);
    }
}
