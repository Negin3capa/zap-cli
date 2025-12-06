use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind, MouseButton};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::collections::HashMap;
use tokio::sync::mpsc;

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
    event_tx: mpsc::Sender<WhatsAppEvent>,
    state: AppState,
    
    // Data
    chats: Vec<Chat>,
    current_chat_id: Option<String>,
    messages: HashMap<String, Vec<Message>>,
    
    // UI State
    focused: FocusedWidget,
    chat_list_state: ListState,
    chat_list_scroll: usize,  // Scroll offset for chat list
    chat_list_area: Rect,  // Store chat list area for mouse click detection
    message_scroll: u16,  // Scroll offset for message view
    input_buffer: String,
    
    // Authentication
    qr_code: Option<String>,
    status_message: String,
}

impl App {
    pub fn new(_config: Config, client: WhatsAppClient, event_tx: mpsc::Sender<WhatsAppEvent>) -> Self {
        let theme = Theme::terminal();  // Always use terminal theme
        
        Self {
            theme,
            client,
            event_tx,
            state: AppState::Authenticating,
            chats: Vec::new(),
            current_chat_id: None,
            messages: HashMap::new(),
            focused: FocusedWidget::ChatList,
            chat_list_state: ListState::default(),
            chat_list_scroll: 0,
            chat_list_area: Rect::default(),
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
                self.qr_code = None; // clear QR code immediately
                self.status_message = "Authenticated! Loading chats...".to_string();
            }
            
            WhatsAppEvent::Ready => {
                log::info!("Ready");
                self.state = AppState::Ready;
                self.qr_code = None;
                self.status_message = "Syncing chats... First sync after login can take 3-5 minutes with many contacts".to_string();

                // Load chats in background to avoid blocking the event loop
                let client = self.client.clone(); // WhatsAppClient is cheap to clone (Arc internal)
                let event_tx = self.event_tx.clone();

                tokio::spawn(async move {
                    log::info!("Starting background chat load...");
                    match client.get_chats().await {
                        Ok(chats) => {
                            log::info!("Loaded {} chats in background", chats.len());
                            let _ = event_tx.send(WhatsAppEvent::ChatsLoaded(chats)).await;
                        }
                        Err(e) => {
                            log::error!("Failed to load chats: {}", e);
                            // Send error event to update UI
                            let error_msg = format!("Failed to load chats: {}. Try restarting the app.", e);
                            let _ = event_tx.send(WhatsAppEvent::Error(error_msg)).await;
                        }
                    }
                });
            }
            
            WhatsAppEvent::ChatsLoaded(mut chats) => {
                 // Sort chats by timestamp (most recent first)
                 chats.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

                 self.chats = chats;
                 if !self.chats.is_empty() && self.chat_list_state.selected().is_none() {
                     self.chat_list_state.select(Some(0));
                 }
                 self.status_message = format!("Ready - {} chats loaded", self.chats.len());
                 // Ensure we are in Ready state
                 self.state = AppState::Ready;
                 // Force UI refresh if needed by triggering a render (handled by loop)
            }
            
            WhatsAppEvent::MessageReceived(msg) => {
                log::debug!("Received message in chat {}", msg.chat_id);
                
                // Add to messages
                self.messages.entry(msg.chat_id.clone())
                    .or_insert_with(Vec::new)
                    .push(msg.clone());
                
                // Update chat's last message and timestamp
                if let Some(chat) = self.chats.iter_mut().find(|c| c.id == msg.chat_id) {
                    chat.last_message = Some(msg.body.clone());
                    chat.timestamp = msg.timestamp;
                    if !msg.from_me {
                        chat.unread_count += 1;
                    }
                }

                // Re-sort chats to bring the updated chat to the top
                self.chats.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

                // Preserve selection by finding the currently selected chat's new position
                if let Some(current_index) = self.chat_list_state.selected() {
                    if let Some(current_chat) = self.chats.get(current_index).map(|c| c.id.clone()) {
                        // Find where this chat moved to after sorting
                        if let Some(new_index) = self.chats.iter().position(|c| c.id == current_chat) {
                            self.chat_list_state.select(Some(new_index));
                        }
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

            WhatsAppEvent::MessagesLoaded(chat_id, messages) => {
                log::debug!("Messages loaded for chat {}: {} messages", chat_id, messages.len());
                
                // Update messages cache
                self.messages.insert(chat_id.clone(), messages.clone());
                
                // Update status message
                if let Some(chat) = self.chats.iter().find(|c| c.id == chat_id) {
                    let count = messages.len();
                    self.status_message = format!("{} - {} messages", chat.name, count);
                }
            }
            
            WhatsAppEvent::Disconnected => {
                log::warn!("Disconnected from WhatsApp");
                self.state = AppState::Disconnected;
                self.status_message = "Disconnected. Reconnecting...".to_string();
            }

            WhatsAppEvent::Error(error_msg) => {
                log::error!("Error event received: {}", error_msg);
                self.status_message = format!("⚠️  {}", error_msg);
            }
        }
        
        Ok(())
    }
    
    /// Handle terminal events
    pub async fn handle_event(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return Ok(false);
                }
                self.handle_key(key).await
            }
            Event::Mouse(mouse) => {
                self.handle_mouse(mouse).await
            }
            _ => Ok(false),
        }
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
    
    async fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<bool> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Check if click is within chat list area
                let x = mouse.column;
                let y = mouse.row;
                
                if x >= self.chat_list_area.x && x < self.chat_list_area.x + self.chat_list_area.width
                    && y >= self.chat_list_area.y && y < self.chat_list_area.y + self.chat_list_area.height
                {
                    // Calculate which chat was clicked
                    // Subtract 1 for the border at the top
                    let relative_y = y.saturating_sub(self.chat_list_area.y + 1);
                    let clicked_index = self.chat_list_scroll + relative_y as usize;
                    
                    if clicked_index < self.chats.len() {
                        // Select the clicked chat
                        self.chat_list_state.select(Some(clicked_index));
                        self.focused = FocusedWidget::ChatList;
                        
                        // Clear input and reset message scroll
                        self.input_buffer.clear();
                        self.message_scroll = 0;
                        
                        // Load messages for the clicked chat
                        self.load_chat_messages_background(clicked_index).await?;
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                // Scroll the chat list down (show later items)
                let max_scroll = self.chats.len().saturating_sub(1);
                self.chat_list_scroll = (self.chat_list_scroll + 1).min(max_scroll);
            }
            MouseEventKind::ScrollUp => {
                // Scroll the chat list up (show earlier items)
                self.chat_list_scroll = self.chat_list_scroll.saturating_sub(1);
            }
            _ => {}
        }
        
        Ok(false)
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
                
                // Auto-adjust scroll to keep selection visible
                let visible_height = self.chat_list_area.height.saturating_sub(2) as usize; // -2 for borders
                if i >= self.chat_list_scroll + visible_height {
                    self.chat_list_scroll = i.saturating_sub(visible_height - 1);
                }
                
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
                
                // Auto-adjust scroll to keep selection visible
                if i < self.chat_list_scroll {
                    self.chat_list_scroll = i;
                }
                
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
                if self.input_buffer.is_empty() {
                    self.focused = FocusedWidget::ChatList;
                } else {
                    self.input_buffer.pop();
                }
            }
            
            KeyCode::Esc => {
                // Clear input buffer
                // Clear input buffer
                if self.input_buffer.is_empty() {
                    self.focused = FocusedWidget::ChatList;
                } else {
                    self.input_buffer.clear();
                }
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

                // Spawn non-blocking task to load messages
                let client = self.client.clone();
                let event_tx = self.event_tx.clone();

                tokio::spawn(async move {
                    match client.get_messages(&chat_id, 50).await {
                        Ok(messages) => {
                            let _ = event_tx.send(WhatsAppEvent::MessagesLoaded(chat_id, messages)).await;
                        }
                        Err(e) => {
                            log::error!("Failed to load messages: {}", e);
                            let error_msg = format!("Failed to load messages: {}", e);
                            let _ = event_tx.send(WhatsAppEvent::Error(error_msg)).await;
                        }
                    }
                });
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

    /// Refresh messages for the current chat (useful for periodic sync)
    pub async fn refresh_current_chat_messages(&mut self) -> Result<()> {
        if let Some(chat_id) = &self.current_chat_id {
            log::debug!("Refreshing messages for current chat: {}", chat_id);

            match self.client.get_messages(chat_id, 50).await {
                Ok(messages) => {
                    let old_count = self.messages.get(chat_id).map(|m| m.len()).unwrap_or(0);
                    let new_count = messages.len();

                    self.messages.insert(chat_id.clone(), messages);

                    if new_count > old_count {
                        log::info!("Refreshed {} new messages", new_count - old_count);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to refresh messages: {}", e);
                }
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
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(self.theme.border)));
        
        frame.render_widget(loading, frame.area());
    }
    
    fn render_qr(&mut self, frame: &mut Frame) {
        let status_text = format!("{} (Ctrl+C to Quit)", self.status_message);
        let qr_view = QRView::new(
            self.qr_code.as_ref().unwrap(),
            &status_text,
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
        // Store the area for mouse click detection
        self.chat_list_area = area;

        // Calculate visible range based on scroll offset
        let visible_height = area.height.saturating_sub(2) as usize; // -2 for borders
        let max_scroll = self.chats.len().saturating_sub(visible_height);

        // Clamp scroll to valid range
        if self.chat_list_scroll > max_scroll {
            self.chat_list_scroll = max_scroll;
        }

        // Get visible slice of chats
        let end_index = (self.chat_list_scroll + visible_height).min(self.chats.len());
        let visible_chats = &self.chats[self.chat_list_scroll..end_index];

        // Determine which item is selected in the visible window
        let selected_in_window = if let Some(selected) = self.chat_list_state.selected() {
            if selected >= self.chat_list_scroll && selected < end_index {
                Some(selected - self.chat_list_scroll)
            } else {
                None
            }
        } else {
            None
        };

        let items: Vec<ListItem> = visible_chats.iter().enumerate().map(|(idx, chat)| {
            let name = &chat.name;
            let unread = if chat.unread_count > 0 {
                format!(" ({})", chat.unread_count)
            } else {
                String::new()
            };

            // Apply highlight style only to the text if this item is selected
            let is_selected = selected_in_window == Some(idx);
            let text_style = if is_selected {
                Style::default()
                    .bg(self.theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = format!("{}{}", name, unread);
            let line = Line::from(Span::styled(content, text_style));
            ListItem::new(line)
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
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)))
            .highlight_symbol("► ");

        // Use a state with no selection since we're manually handling highlighting
        let mut display_state = ListState::default();
        display_state.select(selected_in_window);

        frame.render_stateful_widget(list, area, &mut display_state);
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
                .border_type(BorderType::Rounded)
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
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)));
        
        frame.render_widget(input, area);
    }
}
