use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind, MouseButton};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, BorderType, List, ListItem, ListState, Paragraph, Wrap},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChatListView {
    Normal,    // Show non-archived chats + "Archived Messages"
    Archived,  // Show archived chats only
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
    chat_list_view: ChatListView,
    chat_list_state: ListState,
    chat_list_scroll: usize,  // Scroll offset for chat list
    chat_list_area: Rect,  // Store chat list area for mouse click detection
    message_view_area: Rect,  // Store message view area for mouse detection
    input_area: Rect,  // Store input area for mouse detection
    message_scroll: u16,  // Scroll offset for message view (0 = bottom/newest)
    input_buffer: String,
    loading_more_messages: HashMap<String, bool>,  // Track if loading older messages for a chat
    chats_needing_sync: std::collections::HashSet<String>,  // Track chats with new updates that need syncing
    
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
            chat_list_view: ChatListView::Normal,
            chat_list_state: ListState::default(),
            chat_list_scroll: 0,
            chat_list_area: Rect::default(),
            message_view_area: Rect::default(),
            input_area: Rect::default(),
            message_scroll: 0,
            input_buffer: String::new(),
            loading_more_messages: HashMap::new(),
            chats_needing_sync: std::collections::HashSet::new(),
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
                self.status_message = "Authenticated! Preparing interface...".to_string();
            }

            WhatsAppEvent::Ready => {
                log::info!("Ready");
                // Switch to Ready state immediately - show main interface
                self.state = AppState::Ready;
                self.qr_code = None;
                self.status_message = "Syncing chats... This may take a few minutes".to_string();

                // Load chats in background - UI is already usable
                let client = self.client.clone();
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

            // Add/update message with deduplication
            let chat_messages = self.messages.entry(msg.chat_id.clone())
                .or_insert_with(Vec::new);

            // Check if message already exists (prevents duplicates from event + sync)
            if !chat_messages.iter().any(|m| m.id == msg.id) {
                chat_messages.push(msg.clone());
                // Sort by timestamp to maintain chronological order
                chat_messages.sort_by_key(|m| m.timestamp);

                // Mark this chat as needing a background sync to get full context
                self.chats_needing_sync.insert(msg.chat_id.clone());
            }

            // Update chat's last message and timestamp
            if let Some(chat) = self.chats.iter_mut().find(|c| c.id == msg.chat_id) {
                chat.last_message = Some(msg.body.clone());
                chat.timestamp = msg.timestamp;
                if !msg.from_me {
                    chat.unread_count += 1;
                }
            }
                // Get the currently selected chat ID (if any) before re-sorting
                // Need to account for "Archived Messages" offset (index 0) and view filtering
                let selected_chat_id = if let Some(visual_index) = self.chat_list_state.selected() {
                    if visual_index == 0 {
                        // "Archived Messages" is selected, keep it selected
                        None
                    } else {
                        // Get the filtered chats for current view
                        let filtered_chats: Vec<&Chat> = match self.chat_list_view {
                            ChatListView::Normal => self.chats.iter().filter(|c| !c.archived).collect(),
                            ChatListView::Archived => self.chats.iter().filter(|c| c.archived).collect(),
                        };

                        // Visual index 1+ maps to filtered_chats index 0+
                        let filtered_index = visual_index.saturating_sub(1);
                        filtered_chats.get(filtered_index).map(|c| c.id.clone())
                    }
                } else {
                    None
                };

                // Re-sort chats to bring the updated chat to the top
                self.chats.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

                // Restore selection to the same chat (by ID)
                if let Some(chat_id) = selected_chat_id {
                    // Get filtered chats after re-sorting
                    let filtered_chats: Vec<&Chat> = match self.chat_list_view {
                        ChatListView::Normal => self.chats.iter().filter(|c| !c.archived).collect(),
                        ChatListView::Archived => self.chats.iter().filter(|c| c.archived).collect(),
                    };

                    // Find the chat's new position in the filtered list
                    if let Some(filtered_index) = filtered_chats.iter().position(|c| c.id == chat_id) {
                        // Convert to visual index (add 1 for "Archived Messages" offset)
                        let new_visual_index = filtered_index + 1;
                        self.chat_list_state.select(Some(new_visual_index));
                    }
                } else if self.chat_list_state.selected() == Some(0) {
                    // Keep "Archived Messages" selected
                    self.chat_list_state.select(Some(0));
                }
            }
            
            WhatsAppEvent::ChatUpdated(updated_chat) => {
                if let Some(chat) = self.chats.iter_mut().find(|c| c.id == updated_chat.id) {
                    *chat = updated_chat;
                } else {
                    self.chats.push(updated_chat);
                }
            }

            WhatsAppEvent::MessagesLoaded(chat_id, new_messages) => {
                log::debug!("Messages loaded for chat {}: {} messages", chat_id, new_messages.len());
                
                // Merge with existing messages (for pagination)
                if let Some(existing) = self.messages.get_mut(&chat_id) {
                    let old_count = existing.len();
                    log::info!("Merging {} new messages with {} existing messages", new_messages.len(), old_count);
                    
                    // Prepend new messages (older ones) and deduplicate
                    let mut all_messages = new_messages.clone();
                    all_messages.extend(existing.clone());
                    
                    // Deduplicate by message ID, keeping first occurrence
                    let mut seen = std::collections::HashSet::new();
                    all_messages.retain(|msg| seen.insert(msg.id.clone()));
                    
                    // Sort by timestamp (oldest first)
                    all_messages.sort_by_key(|m| m.timestamp);
                    
                    *existing = all_messages;
                    
                    let new_count = existing.len() - old_count;
                    log::info!("After merge: {} total messages ({} new)", existing.len(), new_count);
                } else {
                    // Initial load
                    log::info!("Initial load of {} messages for chat {}", new_messages.len(), chat_id);
                    self.messages.insert(chat_id.clone(), new_messages.clone());
                }
                
                // Clear loading flag
                self.loading_more_messages.insert(chat_id.clone(), false);
                
                // Update status message
                if let Some(chat) = self.chats.iter().find(|c| c.id == chat_id) {
                    let count = self.messages.get(&chat_id).map(|m| m.len()).unwrap_or(0);
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
                let x = mouse.column;
                let y = mouse.row;

                // Check which area was clicked (in priority order)
                
                // 1. Chat list area
                if x >= self.chat_list_area.x && x < self.chat_list_area.x + self.chat_list_area.width
                    && y >= self.chat_list_area.y && y < self.chat_list_area.y + self.chat_list_area.height
                {
                    // Calculate which item was clicked
                    let relative_y = y.saturating_sub(self.chat_list_area.y + 1);
                    let clicked_index = self.chat_list_scroll + relative_y as usize;

                    // First item (index 0) is always "Archived Messages" - clicking toggles view
                    if clicked_index == 0 {
                        self.chat_list_view = match self.chat_list_view {
                            ChatListView::Normal => ChatListView::Archived,
                            ChatListView::Archived => ChatListView::Normal,
                        };
                        self.chat_list_scroll = 0;
                        self.chat_list_state.select(Some(0));
                        self.focused = FocusedWidget::ChatList;
                        return Ok(false);
                    }

                    // Get filtered chats for the current view
                    let filtered_chats: Vec<_> = match self.chat_list_view {
                        ChatListView::Normal => self.chats.iter().filter(|c| !c.archived).collect(),
                        ChatListView::Archived => self.chats.iter().filter(|c| c.archived).collect(),
                    };

                    // Calculate actual chat index (subtract 1 for "Archived Messages" offset)
                    let actual_chat_index = clicked_index.saturating_sub(1);

                    // Find the chat in the full chats list
                    if let Some(clicked_chat) = filtered_chats.get(actual_chat_index) {
                        let chat_id = clicked_chat.id.clone();

                        // Find this chat's absolute index in self.chats
                        if let Some(abs_index) = self.chats.iter().position(|c| c.id == chat_id) {
                            self.chat_list_state.select(Some(clicked_index));
                            self.focused = FocusedWidget::ChatList;
                            self.input_buffer.clear();
                            self.message_scroll = 0;
                            self.load_chat_messages_background(abs_index).await?;
                        }
                    }
                }
                // 2. Message view area
                else if x >= self.message_view_area.x && x < self.message_view_area.x + self.message_view_area.width
                    && y >= self.message_view_area.y && y < self.message_view_area.y + self.message_view_area.height
                {
                    // Focus message view
                    self.focused = FocusedWidget::MessageView;
                }
                // 3. Input area
                else if x >= self.input_area.x && x < self.input_area.x + self.input_area.width
                    && y >= self.input_area.y && y < self.input_area.y + self.input_area.height
                {
                    // Focus input
                    self.focused = FocusedWidget::Input;
                }
            }
            MouseEventKind::ScrollDown => {
                // Scroll based on focused widget
                match self.focused {
                    FocusedWidget::ChatList => {
                        // Scroll the chat list down (show later items)
                        let max_scroll = self.chats.len().saturating_sub(1);
                        self.chat_list_scroll = (self.chat_list_scroll + 1).min(max_scroll);
                    }
                    FocusedWidget::MessageView => {
                        // Scroll down = show newer messages (decrease scroll toward 0)
                        self.message_scroll = self.message_scroll.saturating_sub(3);
                    }
                    _ => {}
                }
            }
            MouseEventKind::ScrollUp => {
                // Scroll based on focused widget
                match self.focused {
                    FocusedWidget::ChatList => {
                        // Scroll the chat list up (show earlier items)
                        self.chat_list_scroll = self.chat_list_scroll.saturating_sub(1);
                    }
                    FocusedWidget::MessageView => {
                        // Scroll up = show older messages (increase offset from bottom)
                        if let Some(chat_id) = &self.current_chat_id {
                            if let Some(messages) = self.messages.get(chat_id) {
                                let num_lines = messages.len();
                                let available_height = self.message_view_area.height.saturating_sub(2) as usize;
                                let max_scroll = num_lines.saturating_sub(available_height);
                                
                                if (self.message_scroll as usize) < max_scroll {
                                    self.message_scroll = self.message_scroll.saturating_add(3);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        
        Ok(false)
    }
    
    fn handle_message_scroll(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                // Scroll up = show older messages (increase offset from bottom)
                if let Some(chat_id) = &self.current_chat_id {
                    if let Some(messages) = self.messages.get(chat_id) {
                        let num_lines = messages.len();
                        let available_height = self.message_view_area.height.saturating_sub(2) as usize;
                        let max_scroll = num_lines.saturating_sub(available_height);

                        if (self.message_scroll as usize) < max_scroll {
                            self.message_scroll = self.message_scroll.saturating_add(1);
                            
                            // Pagination: Check if we're near the top (within 5 lines of oldest message)
                            if (self.message_scroll as usize) >= max_scroll.saturating_sub(5) {
                                let is_loading = self.loading_more_messages.get(chat_id).copied().unwrap_or(false);
                                if !is_loading && num_lines >= 100 {  // Only paginate if we have at least initial load (was 50)
                                    self.load_more_messages(chat_id.clone(), num_lines);
                                }
                            }
                        }
                    }
                }
            }
            KeyCode::Down => {
                // Scroll down = show newer messages (decrease offset, toward 0 = bottom)
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
                    match client.get_messages(&chat_id, 100).await {
                        Ok(messages) => {
                            log::info!("Fetched {} messages for chat {}", messages.len(), chat_id);
                            if !messages.is_empty() {
                                let oldest = messages.first().map(|m| m.timestamp).unwrap_or(0);
                                let newest = messages.last().map(|m| m.timestamp).unwrap_or(0);
                                log::info!("Message range: oldest={} newest={}", oldest, newest);
                            }
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

    fn load_more_messages(&mut self, chat_id: String, current_count: usize) {
        // Mark as loading
        self.loading_more_messages.insert(chat_id.clone(), true);
        self.status_message = "Loading older messages...".to_string();
        log::info!("Loading more messages for chat: {} (current: {})", chat_id, current_count);

        let client = self.client.clone();
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            // Request more messages - WhatsApp API will return oldest to newest
            match client.get_messages(&chat_id, current_count + 50).await {
                Ok(messages) => {
                    let _ = event_tx.send(WhatsAppEvent::MessagesLoaded(chat_id, messages)).await;
                }
                Err(e) => {
                    log::error!("Failed to load more messages: {}", e);
                    let error_msg = format!("Failed to load older messages: {}", e);
                    let _ = event_tx.send(WhatsAppEvent::Error(error_msg)).await;
                }
            }
        });
    }

    /// Background sync for chats that have received updates
    /// Only syncs chats that are marked in chats_needing_sync
    pub fn background_sync_updated_chats(&mut self) {
        if self.chats_needing_sync.is_empty() {
            return; // Nothing to sync
        }

        // Take ownership of the set to avoid borrow checker issues
        let chats_to_sync: Vec<String> = self.chats_needing_sync.drain().collect();

        log::info!("Background syncing {} chats with updates", chats_to_sync.len());

        for chat_id in chats_to_sync {
            // Skip current chat if it's being synced by the main interval
            if Some(&chat_id) == self.current_chat_id.as_ref() {
                continue;
            }

            let client = self.client.clone();
            let event_tx = self.event_tx.clone();
            let chat_id_clone = chat_id.clone();

            tokio::spawn(async move {
                // Get full context (100 messages) to ensure we have everything
                match client.get_messages(&chat_id_clone, 100).await {
                    Ok(messages) => {
                        log::debug!("Background sync loaded {} messages for {}", messages.len(), chat_id_clone);
                        let _ = event_tx.send(WhatsAppEvent::MessagesLoaded(chat_id_clone, messages)).await;
                    }
                    Err(e) => {
                        log::warn!("Background sync failed for {}: {}", chat_id_clone, e);
                    }
                }
            });
        }
    }

    /// Refresh messages for the current chat (useful for periodic sync)
    pub async fn refresh_current_chat_messages(&mut self) -> Result<()> {
        if let Some(chat_id) = &self.current_chat_id {
            log::debug!("Refreshing messages for current chat: {}", chat_id);

            match self.client.get_messages(chat_id, 100).await {
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
            
            if text.is_empty() {
                return Ok(false);
            }
            
            // Clear input immediately for responsiveness
            self.input_buffer.clear();
            
            // Send message
            match self.client.send_message(chat_id, &text).await {
                Ok(_) => {
                    log::info!("Message sent successfully");
                    self.status_message = "Message sent".to_string();
                    
                    // Reset scroll to auto-scroll to the new message
                    self.message_scroll = 0;
                    
                    // Refresh messages to show the sent message
                    // This will be near-instant since the message was just sent
                    if let Err(e) = self.refresh_current_chat_messages().await {
                        log::warn!("Failed to refresh after send: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Failed to send message: {}", e);
                    self.status_message = format!("Failed to send: {}", e);
                    // Put the text back in the input buffer
                    self.input_buffer = text;
                }
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

        // Filter chats based on current view
        let filtered_chats: Vec<&Chat> = match self.chat_list_view {
            ChatListView::Normal => {
                // Show non-archived chats
                self.chats.iter()
                    .filter(|c| !c.archived)
                    .collect()
            },
            ChatListView::Archived => {
                // Show only archived chats
                self.chats.iter()
                    .filter(|c| c.archived)
                    .collect()
            }
        };

        // Calculate total items (chats + "Archived Messages")
        let total_items = filtered_chats.len() + 1;

        // Calculate visible range based on scroll offset
        let visible_height = area.height.saturating_sub(2) as usize; // -2 for borders
        let max_scroll = total_items.saturating_sub(visible_height);

        // Clamp scroll to valid range
        if self.chat_list_scroll > max_scroll {
            self.chat_list_scroll = max_scroll;
        }

        // Get visible range
        let end_index = (self.chat_list_scroll + visible_height).min(total_items);

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

        // Build items to display
        let mut items: Vec<ListItem> = Vec::new();

        for i in self.chat_list_scroll..end_index {
            // First item is always "Archived Messages"
            if i == 0 {
                let is_selected = selected_in_window == Some(items.len());
                let text_style = if is_selected {
                    Style::default()
                        .bg(self.theme.primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                // Check if there are any archived chats
                let archived_count = self.chats.iter().filter(|c| c.archived).count();
                
                // Change indicator based on current view
                let content = if self.chat_list_view == ChatListView::Archived {
                    if archived_count > 0 {
                        format!("📂 Archived Messages ({}) - Viewing", archived_count)
                    } else {
                        "📂 Archived Messages - Viewing (empty)".to_string()
                    }
                } else {
                    if archived_count > 0 {
                        format!("📁 Archived Messages ({})", archived_count)
                    } else {
                        "📁 Archived Messages".to_string()
                    }
                };

                let line = Line::from(Span::styled(content, text_style));
                items.push(ListItem::new(line));
            } else {
                // Regular chat item
                let chat_idx = i - 1;  // Subtract 1 for "Archived Messages" offset
                
                if let Some(chat) = filtered_chats.get(chat_idx) {
                    let name = &chat.name;
                    let unread = if chat.unread_count > 0 {
                        format!(" ({})", chat.unread_count)
                    } else {
                        String::new()
                    };

                    let is_selected = selected_in_window == Some(items.len());
                    let text_style = if is_selected {
                        Style::default()
                            .bg(self.theme.primary)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let content = format!("{}{}", name, unread);
                    let line = Line::from(Span::styled(content, text_style));
                    items.push(ListItem::new(line));
                }
            }
        }

        let border_color = if self.focused == FocusedWidget::ChatList {
            self.theme.border_focused
        } else {
            self.theme.border
        };

        // Always use "Chats" title
        let list = List::new(items)
            .block(Block::default()
                .title(" Chats ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)))
            .highlight_symbol("► ");

        // Use a state with the selection
        let mut display_state = ListState::default();
        display_state.select(selected_in_window);

        frame.render_stateful_widget(list, area, &mut display_state);
    }
    
    fn render_messages(&mut self, frame: &mut Frame, area: Rect) {
        // Store area for mouse detection
        self.message_view_area = area;
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
            let mut text = Text::default();

            for msg in messages {
                let sender = if msg.from_me {
                    Span::styled("Me", Style::default().fg(self.theme.me))
                } else {
                    Span::styled(
                        msg.sender.as_deref().unwrap_or("User"),
                        Style::default().fg(self.theme.other)
                    )
                };

                let time = chrono::DateTime::from_timestamp(msg.timestamp, 0)
                    .map(|dt| dt.with_timezone(&chrono::Local).format("%H:%M").to_string())
                    .unwrap_or_default();

                let body = if msg.has_media {
                    format!("[Media: {}]", msg.media_type.as_deref().unwrap_or("unknown"))
                } else {
                    msg.body.clone()
                };

                // Create a single line with all components - Paragraph will wrap it
                let message_line = Line::from(vec![
                    Span::styled(time, Style::default().fg(self.theme.system)),
                    Span::raw(" "),
                    sender,
                    Span::raw(": "),
                    Span::raw(body),
                ]);

                text.lines.push(message_line);
            }

            text
        } else {
            Text::from("Loading messages...")
        }
    } else {
        Text::from("Select a chat to view messages")
    };
    
    // Calculate scroll offset
    // Inverted: scroll=0 means "at bottom" (newest messages)
    // Increasing message_scroll means scrolling UP (shows older messages)
    let num_lines = messages_text.lines.len();
    let available_height = area.height.saturating_sub(2) as usize;
    
    // Calculate scroll offset from bottom
    let scroll_offset = if num_lines > available_height {
        let max_scroll = (num_lines - available_height) as u16;
        // Invert: subtract message_scroll from max to scroll from bottom
        max_scroll.saturating_sub(self.message_scroll)
    } else {
        0  // Not enough messages to scroll
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
            .wrap(Wrap { trim: false })
            .scroll((scroll_offset, 0));
        
        frame.render_widget(paragraph, area);
    }
    
    fn render_input(&mut self, frame: &mut Frame, area: Rect) {
        // Store area for mouse detection
        self.input_area = area;
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
