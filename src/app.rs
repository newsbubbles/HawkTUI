//! Main application for HawkTUI.
//!
//! The App struct orchestrates the entire TUI, managing:
//! - Terminal setup and teardown
//! - Event loop
//! - State management
//! - Rendering

use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout as RatatuiLayout, Rect},
};

use crate::core::{
    error::{Error, Result},
    events::{Action, map_key_to_action},
    state::{
        AppMode, AppState, ConnectionStatus, LayoutMode, Message, Panel, SessionInfo, ToolInfo,
    },
};
use crate::providers::PiBridge;
use crate::ui::{
    layout::LayoutManager,
    panels::{ConversationPanel, HeaderPanel, InputPanel, SessionsPanel, ToolsPanel},
    themes::Theme,
};

/// Application version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Tick rate for animations (in milliseconds).
const TICK_RATE_MS: u64 = 100;

/// The main HawkTUI application.
pub struct App {
    /// Application state.
    state: AppState,

    /// Layout manager.
    layout_manager: LayoutManager,

    /// Current theme.
    theme: Theme,

    /// Pi agent bridge.
    bridge: PiBridge,

    /// Animation frame counter.
    frame: usize,

    /// Initial message to send.
    initial_message: Option<String>,
}

impl App {
    /// Create a new App builder.
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    /// Create a new App with default settings.
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    /// Run the application.
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        let mut terminal = self.setup_terminal()?;

        // Connect to agent
        self.bridge.connect().await?;
        self.state.status.connection = ConnectionStatus::Connected;
        self.state.status.model = self.bridge.model().await;
        self.state.status.provider = self.bridge.provider().await;

        // Load tools
        self.state.tools.available = self
            .bridge
            .available_tools()
            .into_iter()
            .map(|t| ToolInfo {
                name: t.name,
                description: t.description,
                enabled: t.enabled,
            })
            .collect();

        // Send initial message if provided
        if let Some(msg) = self.initial_message.take() {
            self.send_message(&msg).await?;
        }

        // Main event loop
        let result = self.event_loop(&mut terminal).await;

        // Restore terminal
        self.restore_terminal(&mut terminal)?;

        result
    }

    /// Setup the terminal for TUI rendering.
    fn setup_terminal(&self) -> Result<Terminal<CrosstermBackend<Stdout>>> {
        enable_raw_mode().map_err(|e| Error::terminal(e.to_string()))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .map_err(|e| Error::terminal(e.to_string()))?;
        let backend = CrosstermBackend::new(stdout);
        Terminal::new(backend).map_err(|e| Error::terminal(e.to_string()))
    }

    /// Restore the terminal to normal state.
    fn restore_terminal(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        disable_raw_mode().map_err(|e| Error::terminal(e.to_string()))?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .map_err(|e| Error::terminal(e.to_string()))?;
        terminal
            .show_cursor()
            .map_err(|e| Error::terminal(e.to_string()))?;
        Ok(())
    }

    /// Main event loop.
    async fn event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
        let tick_rate = Duration::from_millis(TICK_RATE_MS);

        loop {
            // Render
            terminal
                .draw(|frame| self.render(frame))
                .map_err(|e| Error::terminal(e.to_string()))?;

            // Handle events
            if event::poll(tick_rate).map_err(|e| Error::terminal(e.to_string()))? {
                let event = event::read().map_err(|e| Error::terminal(e.to_string()))?;
                self.handle_crossterm_event(event).await?;
            } else {
                // Tick for animations
                self.tick();
            }

            // Check if we should quit
            if self.state.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Handle a crossterm event.
    async fn handle_crossterm_event(&mut self, event: CrosstermEvent) -> Result<()> {
        match event {
            CrosstermEvent::Key(key) => {
                let action = map_key_to_action(
                    key,
                    self.state.mode,
                    self.state.overlay.as_ref(),
                    Some(self.state.active_panel),
                );
                self.handle_action(action).await?;
            }
            CrosstermEvent::Resize(width, height) => {
                // Terminal resized - ratatui handles this automatically
                tracing::debug!("Terminal resized to {width}x{height}");
            }
            CrosstermEvent::Mouse(_mouse) => {
                // Mouse events are intentionally ignored in v0.2.0.
                // Future versions may add click-to-focus panels, scrolling, etc.
            }
            CrosstermEvent::Paste(text) => {
                // Handle paste - insert at char position
                let byte_pos =
                    char_index_to_byte_index(&self.state.input.text, self.state.input.cursor);
                self.state.input.text.insert_str(byte_pos, &text);
                self.state.input.cursor += text.chars().count();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle an action.
    async fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                self.state.should_quit = true;
            }
            Action::SendMessage => {
                if !self.state.input.text.is_empty() {
                    let msg = std::mem::take(&mut self.state.input.text);
                    self.state.input.cursor = 0;

                    // Add to history (avoid duplicates of last entry)
                    if self.state.input.history.front() != Some(&msg) {
                        self.state.input.history.push_front(msg.clone());
                        // Limit history to 100 entries
                        if self.state.input.history.len() > 100 {
                            self.state.input.history.pop_back();
                        }
                    }
                    // Reset history navigation
                    self.state.input.history_index = None;

                    self.send_message(&msg).await?;
                }
            }
            Action::Cancel => {
                if self.state.is_streaming() {
                    self.bridge.cancel();
                    self.state.streaming.is_active = false;
                    self.state.mode = AppMode::Normal;
                    self.state.status.connection = ConnectionStatus::Connected;
                }
            }
            Action::ClearScreen => {
                self.state.conversation.messages.clear();
            }
            Action::ToggleHelp => {
                self.state.overlay = if self.state.overlay.is_some() {
                    None
                } else {
                    Some(crate::core::state::Overlay::Help)
                };
            }
            Action::CloseOverlay => {
                self.state.overlay = None;
            }
            Action::OpenCommandPalette => {
                self.state.overlay = Some(crate::core::state::Overlay::CommandPalette {
                    query: String::new(),
                    selected: 0,
                });
            }
            Action::OpenSessionPicker => {
                self.state.overlay =
                    Some(crate::core::state::Overlay::SessionPicker { selected: 0 });
            }
            Action::ToggleLayout => {
                self.layout_manager.toggle_mode();
            }
            Action::NextPanel => {
                self.state.active_panel = match self.state.active_panel {
                    Panel::Input => Panel::Conversation,
                    Panel::Conversation => Panel::Sessions,
                    Panel::Sessions => Panel::Tools,
                    Panel::Tools => Panel::Context,
                    Panel::Context => Panel::Input,
                };
            }
            Action::PrevPanel => {
                self.state.active_panel = match self.state.active_panel {
                    Panel::Input => Panel::Context,
                    Panel::Conversation => Panel::Input,
                    Panel::Sessions => Panel::Conversation,
                    Panel::Tools => Panel::Sessions,
                    Panel::Context => Panel::Tools,
                };
            }
            Action::ScrollUp(n) => {
                self.state.conversation.scroll_offset =
                    self.state.conversation.scroll_offset.saturating_sub(n);
                self.state.conversation.auto_scroll = false;
            }
            Action::ScrollDown(n) => {
                self.state.conversation.scroll_offset =
                    self.state.conversation.scroll_offset.saturating_add(n);
            }
            Action::ScrollToTop => {
                self.state.conversation.scroll_offset = 0;
                self.state.conversation.auto_scroll = false;
            }
            Action::ScrollToBottom => {
                self.state.conversation.auto_scroll = true;
            }
            Action::InsertChar(c) => {
                // Check for slash command
                if self.state.input.text.is_empty() && c == '/' {
                    self.state.mode = AppMode::Command;
                }
                // Convert char index to byte index for insertion
                let byte_pos =
                    char_index_to_byte_index(&self.state.input.text, self.state.input.cursor);
                self.state.input.text.insert(byte_pos, c);
                self.state.input.cursor += 1;
            }
            Action::Backspace => {
                if self.state.input.cursor > 0 {
                    self.state.input.cursor -= 1;
                    // Convert char index to byte index for removal
                    let byte_pos =
                        char_index_to_byte_index(&self.state.input.text, self.state.input.cursor);
                    self.state.input.text.remove(byte_pos);
                }
                // Exit command mode if we deleted the slash
                if self.state.input.text.is_empty() {
                    self.state.mode = AppMode::Normal;
                }
            }
            Action::DeleteChar => {
                let char_count = self.state.input.text.chars().count();
                if self.state.input.cursor < char_count {
                    // Convert char index to byte index for removal
                    let byte_pos =
                        char_index_to_byte_index(&self.state.input.text, self.state.input.cursor);
                    self.state.input.text.remove(byte_pos);
                }
            }
            Action::CursorLeft => {
                self.state.input.cursor = self.state.input.cursor.saturating_sub(1);
            }
            Action::CursorRight => {
                let char_count = self.state.input.text.chars().count();
                self.state.input.cursor = (self.state.input.cursor + 1).min(char_count);
            }
            Action::CursorHome => {
                self.state.input.cursor = 0;
            }
            Action::CursorEnd => {
                self.state.input.cursor = self.state.input.text.chars().count();
            }
            Action::HistoryPrev => {
                if !self.state.input.history.is_empty() {
                    let new_index = match self.state.input.history_index {
                        None => {
                            // First press: save current input and show most recent history
                            if !self.state.input.text.is_empty() {
                                // Store current input temporarily at the end
                                self.state
                                    .input
                                    .history
                                    .push_back(self.state.input.text.clone());
                            }
                            Some(0)
                        }
                        Some(idx) => {
                            // Move to older entry if available
                            let max_idx = self.state.input.history.len().saturating_sub(1);
                            Some((idx + 1).min(max_idx))
                        }
                    };
                    self.state.input.history_index = new_index;
                    if let Some(idx) = new_index {
                        if let Some(entry) = self.state.input.history.get(idx) {
                            self.state.input.text = entry.clone();
                            self.state.input.cursor = self.state.input.text.chars().count();
                        }
                    }
                }
            }
            Action::HistoryNext => {
                if let Some(idx) = self.state.input.history_index {
                    if idx == 0 {
                        // At most recent entry, clear and exit history mode
                        // Check if we saved current input at the back
                        if self.state.input.history.len() > 100 {
                            if let Some(saved) = self.state.input.history.pop_back() {
                                self.state.input.text = saved;
                            }
                        } else {
                            self.state.input.text.clear();
                        }
                        self.state.input.history_index = None;
                        self.state.input.cursor = self.state.input.text.chars().count();
                    } else {
                        // Move to newer entry
                        let new_index = idx.saturating_sub(1);
                        self.state.input.history_index = Some(new_index);
                        if let Some(entry) = self.state.input.history.get(new_index) {
                            self.state.input.text = entry.clone();
                            self.state.input.cursor = self.state.input.text.chars().count();
                        }
                    }
                }
            }
            Action::SelectNextSession => {
                if !self.state.sessions.is_empty() {
                    let new_idx = match self.state.selected_session_index {
                        None => 0,
                        Some(idx) => (idx + 1).min(self.state.sessions.len() - 1),
                    };
                    self.state.selected_session_index = Some(new_idx);
                }
            }
            Action::SelectPrevSession => {
                if !self.state.sessions.is_empty() {
                    let new_idx = match self.state.selected_session_index {
                        None => 0,
                        Some(idx) => idx.saturating_sub(1),
                    };
                    self.state.selected_session_index = Some(new_idx);
                }
            }
            Action::SwitchToSelectedSession => {
                if let Some(idx) = self.state.selected_session_index {
                    if let Some(session) = self.state.sessions.get(idx) {
                        let session_id = session.id;
                        self.switch_to_session(session_id).await?;
                    }
                }
            }
            Action::RefreshSessions => {
                self.refresh_sessions().await?;
            }
            Action::SelectNextTool => {
                if !self.state.tools.available.is_empty() {
                    let new_idx = self
                        .state
                        .tools
                        .selected_index
                        .map_or(0, |idx| (idx + 1).min(self.state.tools.available.len() - 1));
                    self.state.tools.selected_index = Some(new_idx);
                }
            }
            Action::SelectPrevTool => {
                if !self.state.tools.available.is_empty() {
                    let new_idx = self
                        .state
                        .tools
                        .selected_index
                        .map_or(0, |idx| idx.saturating_sub(1));
                    self.state.tools.selected_index = Some(new_idx);
                }
            }
            Action::ToggleSelectedTool => {
                if let Some(idx) = self.state.tools.selected_index {
                    if let Some(tool) = self.state.tools.available.get_mut(idx) {
                        tool.enabled = !tool.enabled;
                    }
                }
            }
            Action::RefreshTools => {
                self.refresh_tools();
            }
            _ => {}
        }
        Ok(())
    }

    /// Send a message to the agent.
    async fn send_message(&mut self, message: &str) -> Result<()> {
        // Check for slash command
        if message.starts_with('/') {
            return self.handle_command(message).await;
        }

        // Add user message to conversation
        self.state
            .conversation
            .messages
            .push(Message::user(message));
        self.state.conversation.auto_scroll = true;

        // Start streaming mode
        self.state.mode = AppMode::Streaming;
        self.state.status.connection = ConnectionStatus::Streaming;
        self.state.streaming.is_active = true;

        // Add placeholder assistant message
        let assistant_msg = Message::assistant_streaming();
        self.state.streaming.message_id = Some(assistant_msg.id);
        self.state.conversation.messages.push(assistant_msg);

        // Pi agent integration pending - simulate response for now.
        // When pi_agent_rust is available, this will call bridge.send_message()
        // and stream chunks back via the streaming state.
        self.simulate_response(message).await;

        Ok(())
    }

    /// Handle a slash command.
    async fn handle_command(&mut self, command: &str) -> Result<()> {
        use crate::core::commands::{find_command, parse_command};

        let parsed = match parse_command(command) {
            Some(p) => p,
            None => return Ok(()),
        };

        let cmd = match find_command(&parsed.name) {
            Some(c) => c,
            None => {
                // Unknown command
                tracing::warn!("Unknown command: {}", parsed.name);
                return Ok(());
            }
        };

        match cmd.name {
            "help" => {
                self.state.overlay = Some(crate::core::state::Overlay::Help);
            }
            "clear" => {
                self.state.conversation.messages.clear();
            }
            "exit" | "quit" => {
                self.state.should_quit = true;
            }
            "model" => {
                if let Some(model) = parsed.args.first() {
                    // Parse as "provider/model" or just "model"
                    let (provider, model_id) = if model.contains('/') {
                        let parts: Vec<&str> = model.splitn(2, '/').collect();
                        (parts[0].to_string(), parts[1].to_string())
                    } else {
                        (self.state.status.provider.clone(), model.clone())
                    };
                    if let Err(e) = self.bridge.set_model(&provider, &model_id).await {
                        self.state
                            .conversation
                            .messages
                            .push(Message::system(format!("Failed to set model: {e}")));
                    } else {
                        self.state.status.model = model_id;
                        self.state.status.provider = provider;
                    }
                }
            }
            "layout" => {
                if let Some(layout) = parsed.args.first() {
                    self.layout_manager.set_mode(LayoutMode::from_str(layout));
                }
            }
            "theme" => {
                if let Some(theme_name) = parsed.args.first() {
                    if let Some(new_theme) = Theme::by_name(theme_name) {
                        self.theme = new_theme;
                        self.state
                            .conversation
                            .messages
                            .push(Message::system(format!(
                                "Switched to theme: {}",
                                self.theme.meta.name
                            )));
                    } else {
                        let available = Theme::available_themes().join(", ");
                        self.state
                            .conversation
                            .messages
                            .push(Message::system(format!(
                                "Unknown theme '{theme_name}'. Available themes: {available}"
                            )));
                    }
                } else {
                    let available = Theme::available_themes().join(", ");
                    self.state
                        .conversation
                        .messages
                        .push(Message::system(format!(
                            "Current theme: {}. Available: {available}",
                            self.theme.meta.name
                        )));
                }
            }
            "vim" => {
                self.state.input.vim_mode = !self.state.input.vim_mode;
                let status = if self.state.input.vim_mode {
                    "enabled"
                } else {
                    "disabled"
                };
                self.state
                    .conversation
                    .messages
                    .push(Message::system(format!("Vim mode {status}")));
            }
            "shortcuts" => {
                // Show help overlay (same as /help for now - contains keybindings)
                self.state.overlay = Some(crate::core::state::Overlay::Help);
            }
            "session" => match parsed.args.first().map(String::as_str) {
                Some("new") => {
                    let name = parsed.args.get(1).map(String::as_str).unwrap_or("unnamed");
                    self.state.conversation.messages.push(
                            Message::system(format!("Session created: {name} (simulated - pi_agent_rust integration pending)"))
                        );
                }
                Some("list") => {
                    self.state.conversation.messages.push(
                            Message::system(format!(
                                "Sessions:\n  • current (active)\n\n(Session management requires pi_agent_rust integration)"
                            ))
                        );
                }
                Some("switch") => {
                    if let Some(id) = parsed.args.get(1) {
                        self.state.conversation.messages.push(
                                Message::system(format!("Switched to session: {id} (simulated - pi_agent_rust integration pending)"))
                            );
                    } else {
                        self.state.conversation.messages.push(Message::system(
                            "Usage: /session switch <session-id>".to_string(),
                        ));
                    }
                }
                Some("delete") => {
                    if let Some(id) = parsed.args.get(1) {
                        self.state.conversation.messages.push(
                                Message::system(format!("Session deleted: {id} (simulated - pi_agent_rust integration pending)"))
                            );
                    } else {
                        self.state.conversation.messages.push(Message::system(
                            "Usage: /session delete <session-id>".to_string(),
                        ));
                    }
                }
                _ => {
                    self.state.conversation.messages.push(Message::system(
                        "Usage: /session [new|list|switch|delete] [name/id]".to_string(),
                    ));
                }
            },
            "context" => match parsed.args.first().map(String::as_str) {
                Some("add") => {
                    if let Some(file) = parsed.args.get(1) {
                        self.state
                            .conversation
                            .messages
                            .push(Message::system(format!("Added to context: {file}")));
                    } else {
                        self.state.conversation.messages.push(Message::system(
                            "Usage: /context add <file-path>".to_string(),
                        ));
                    }
                }
                Some("remove") => {
                    if let Some(file) = parsed.args.get(1) {
                        self.state
                            .conversation
                            .messages
                            .push(Message::system(format!("Removed from context: {file}")));
                    } else {
                        self.state.conversation.messages.push(Message::system(
                            "Usage: /context remove <file-path>".to_string(),
                        ));
                    }
                }
                Some("clear") => {
                    self.state
                        .conversation
                        .messages
                        .push(Message::system("Context cleared".to_string()));
                }
                Some("list") | None => {
                    self.state.conversation.messages.push(Message::system(
                        "Context: (empty)\n\nUsage: /context [add|remove|clear|list] [file-path]"
                            .to_string(),
                    ));
                }
                Some(subcmd) => {
                    self.state.conversation.messages.push(
                            Message::system(format!("Unknown context subcommand: {subcmd}\n\nUsage: /context [add|remove|clear|list] [file-path]"))
                        );
                }
            },
            "export" => {
                let format = parsed
                    .args
                    .first()
                    .map(String::as_str)
                    .unwrap_or("markdown");
                self.state.conversation.messages.push(
                    Message::system(format!("Conversation exported as {format} (simulated - export functionality pending)"))
                );
            }
            "tools" => match parsed.args.first().map(String::as_str) {
                Some("list") | None => {
                    self.state.conversation.messages.push(
                            Message::system("Available tools:\n  • file_read\n  • file_write\n  • shell_exec\n  • web_search\n\n(Tool management requires pi_agent_rust integration)".to_string())
                        );
                }
                Some("enable") => {
                    if let Some(tool) = parsed.args.get(1) {
                        self.state
                            .conversation
                            .messages
                            .push(Message::system(format!("Tool enabled: {tool} (simulated)")));
                    } else {
                        self.state.conversation.messages.push(Message::system(
                            "Usage: /tools enable <tool-name>".to_string(),
                        ));
                    }
                }
                Some("disable") => {
                    if let Some(tool) = parsed.args.get(1) {
                        self.state
                            .conversation
                            .messages
                            .push(Message::system(format!(
                                "Tool disabled: {tool} (simulated)"
                            )));
                    } else {
                        self.state.conversation.messages.push(Message::system(
                            "Usage: /tools disable <tool-name>".to_string(),
                        ));
                    }
                }
                Some(subcmd) => {
                    self.state.conversation.messages.push(
                            Message::system(format!("Unknown tools subcommand: {subcmd}\n\nUsage: /tools [list|enable|disable] [tool-name]"))
                        );
                }
            },
            "branch" => {
                self.state.conversation.messages.push(
                    Message::system("Conversation branching is not yet available.\n\nThis feature will allow you to create alternate conversation paths and explore different directions.".to_string())
                );
            }
            "provider" => {
                if let Some(provider) = parsed.args.first() {
                    self.state.conversation.messages.push(
                        Message::system(format!("Provider set to: {provider} (simulated - pi_agent_rust integration pending)"))
                    );
                } else {
                    self.state.conversation.messages.push(Message::system(
                        "Usage: /provider <provider-name>\n\nAvailable: anthropic, openai, local"
                            .to_string(),
                    ));
                }
            }
            "system" => {
                if !parsed.raw_args.is_empty() {
                    self.state
                        .conversation
                        .messages
                        .push(Message::system(format!(
                            "System prompt updated (simulated - pi_agent_rust integration pending)"
                        )));
                } else {
                    self.state.conversation.messages.push(Message::system(
                        "Usage: /system <prompt>\n\nSets the system prompt for the conversation."
                            .to_string(),
                    ));
                }
            }
            "compact" => {
                self.state.conversation.messages.push(
                    Message::system("Conversation compacted (simulated - reduces token usage by summarizing older messages)".to_string())
                );
            }
            "tokens" => {
                let msg_count = self.state.conversation.messages.len();
                // Rough estimate: ~4 chars per token on average
                let estimated_tokens: usize = self
                    .state
                    .conversation
                    .messages
                    .iter()
                    .map(|m| m.content.len() / 4)
                    .sum();
                self.state.conversation.messages.push(
                    Message::system(format!(
                        "Token usage (estimated):\n  Messages: {msg_count}\n  Tokens: ~{estimated_tokens}\n\n(Accurate token counting requires pi_agent_rust integration)"
                    ))
                );
            }
            "cost" => {
                self.state.conversation.messages.push(
                    Message::system("Cost tracking is not yet available.\n\nThis feature will show estimated costs based on token usage and model pricing.".to_string())
                );
            }
            _ => {
                // Catch-all for any commands we missed - give user feedback
                self.state
                    .conversation
                    .messages
                    .push(Message::system(format!(
                        "Command '{}' is not yet implemented.",
                        cmd.name
                    )));
                tracing::info!("Command not yet implemented: {}", cmd.name);
            }
        }

        self.state.mode = AppMode::Normal;
        Ok(())
    }

    /// Simulate a streaming response for demonstration purposes.
    ///
    /// This method demonstrates the streaming UI behavior by progressively
    /// building a response with delays between chunks. When pi_agent_rust
    /// integration is complete, this will be replaced by actual agent calls.
    async fn simulate_response(&mut self, _message: &str) {
        let response_chunks = [
            "I'm HawkTUI, your AI coding assistant! 🦅\n\n",
            "This is a **simulated streaming response** demonstrating the UI.\n\n",
            "When pi_agent_rust integration is complete, I'll help you with:\n\n",
            "- **Code analysis** and optimization\n",
            "- **Debugging** assistance\n",
            "- **File operations** (read, write, search)\n",
            "- **Shell commands** execution\n\n",
            "Try `/help` to see available commands!",
        ];

        let mut accumulated = String::new();

        for chunk in response_chunks {
            // Simulate network/processing delay between chunks
            tokio::time::sleep(Duration::from_millis(80)).await;

            accumulated.push_str(chunk);

            // Update the streaming message content
            if let Some(msg) = self.state.conversation.messages.last_mut() {
                msg.content = accumulated.clone();
            }
        }

        // Finalize the message
        if let Some(msg) = self.state.conversation.messages.last_mut() {
            msg.is_streaming = false;
        }

        // End streaming state
        self.state.streaming.is_active = false;
        self.state.mode = AppMode::Normal;
        self.state.status.connection = ConnectionStatus::Connected;
        self.state.status.total_tokens += 150; // Simulated token count
        self.state.status.cost += 0.001; // Simulated cost
    }

    /// Refresh the sessions list from the bridge.
    async fn refresh_sessions(&mut self) -> Result<()> {
        let summaries = self.bridge.list_sessions().await?;

        // Convert SessionSummary to SessionInfo
        self.state.sessions = summaries
            .into_iter()
            .map(|s| {
                let is_active = self
                    .state
                    .current_session_id
                    .map(|id| id.to_string() == s.id)
                    .unwrap_or(false);
                SessionInfo {
                    id: uuid::Uuid::parse_str(&s.id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
                    name: s.name,
                    created_at: s.created_at,
                    updated_at: s.updated_at,
                    message_count: s.message_count,
                    is_active,
                }
            })
            .collect();

        // Update selected index if needed
        if self.state.selected_session_index.is_none() && !self.state.sessions.is_empty() {
            self.state.selected_session_index = Some(0);
        }

        Ok(())
    }

    /// Switch to a different session.
    async fn switch_to_session(&mut self, session_id: uuid::Uuid) -> Result<()> {
        // Mark old session as inactive
        for session in &mut self.state.sessions {
            session.is_active = false;
        }

        // Mark new session as active
        for session in &mut self.state.sessions {
            if session.id == session_id {
                session.is_active = true;
                self.state.status.session_name = Some(session.name.clone());
                break;
            }
        }

        self.state.current_session_id = Some(session_id);

        // Clear conversation for new session context.
        // When pi_agent_rust integration is complete, this will load
        // the session's conversation history from the bridge.
        self.state.conversation.messages.clear();
        self.state
            .conversation
            .messages
            .push(Message::system(format!(
                "Switched to session: {session_id}"
            )));

        Ok(())
    }

    /// Refresh the tools list from the bridge.
    fn refresh_tools(&mut self) {
        self.state.tools.available = self
            .bridge
            .available_tools()
            .into_iter()
            .map(|t| ToolInfo {
                name: t.name,
                description: t.description,
                enabled: t.enabled,
            })
            .collect();

        // Update selected index if needed
        if self.state.tools.selected_index.is_none() && !self.state.tools.available.is_empty() {
            self.state.tools.selected_index = Some(0);
        }
    }

    /// Tick for animations.
    fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    /// Render the application.
    fn render(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();
        let layout = self.layout_manager.calculate(area);

        // Header
        let header = HeaderPanel::new(&self.state.status, &self.theme, VERSION);
        frame.render_widget(header, layout.header);

        // Sidebar (if visible)
        if let Some(sidebar_area) = layout.sidebar {
            // Split sidebar into sessions and tools
            let sidebar_chunks = RatatuiLayout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(sidebar_area);

            // Sessions panel
            let sessions = SessionsPanel::new(
                &self.state.sessions,
                self.state.selected_session_index,
                &self.theme,
                self.state.active_panel == Panel::Sessions,
            );
            frame.render_widget(sessions, sidebar_chunks[0]);

            // Tools panel
            let tools = ToolsPanel::new(
                &self.state.tools,
                self.state.tools.selected_index,
                &self.theme,
                self.state.active_panel == Panel::Tools,
                self.frame,
            );
            frame.render_widget(tools, sidebar_chunks[1]);
        }

        // Conversation panel
        let conversation = ConversationPanel::new(
            &self.state.conversation,
            &self.state.streaming,
            &self.theme,
            self.state.active_panel == Panel::Conversation,
            self.frame,
        );
        frame.render_widget(conversation, layout.conversation);

        // Input panel
        let input = InputPanel::new(
            &self.state.input,
            self.state.mode,
            &self.theme,
            self.state.active_panel == Panel::Input,
        );
        frame.render_widget(input, layout.input);

        // Footer
        self.render_footer(frame, layout.footer);

        // Overlay (if any)
        if let Some(ref overlay) = self.state.overlay {
            self.render_overlay(frame, overlay, area);
        }
    }

    /// Render the footer.
    fn render_footer(&self, frame: &mut ratatui::Frame, area: Rect) {
        use ratatui::{
            style::Style,
            text::{Line, Span},
            widgets::Paragraph,
        };

        let hints = [
            ("Ctrl+Enter", "send"),
            ("Ctrl+P", "palette"),
            ("F1", "help"),
            ("F2", "layout"),
            ("Ctrl+C", "quit"),
        ];

        let spans: Vec<Span> = hints
            .iter()
            .flat_map(|(key, action)| {
                vec![
                    Span::styled(
                        format!(" {key} "),
                        Style::default().fg(self.theme.bg()).bg(self.theme.muted()),
                    ),
                    Span::styled(
                        format!(" {action} "),
                        Style::default().fg(self.theme.muted()),
                    ),
                ]
            })
            .collect();

        let line = Line::from(spans);
        let footer = Paragraph::new(line)
            .style(Style::default().bg(Theme::parse_color(&self.theme.panels.status_bg)));

        frame.render_widget(footer, area);
    }

    /// Render an overlay.
    fn render_overlay(
        &self,
        frame: &mut ratatui::Frame,
        overlay: &crate::core::state::Overlay,
        area: Rect,
    ) {
        use ratatui::{
            style::{Modifier, Style},
            text::{Line, Span},
            widgets::{Block, Borders, Clear, Paragraph, Wrap},
        };

        // Calculate overlay area (centered, 60% width, 60% height)
        let overlay_width = (area.width * 60 / 100).max(40);
        let overlay_height = (area.height * 60 / 100).max(10);
        let overlay_x = (area.width - overlay_width) / 2;
        let overlay_y = (area.height - overlay_height) / 2;

        let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

        // Clear the area
        frame.render_widget(Clear, overlay_area);

        match overlay {
            crate::core::state::Overlay::Help => {
                let help_text = vec![
                    Line::from(vec![Span::styled(
                        "🦅 HawkTUI Help",
                        Style::default()
                            .fg(self.theme.accent())
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::raw(""),
                    Line::from(vec![Span::styled(
                        "Keyboard Shortcuts:",
                        Style::default().add_modifier(Modifier::BOLD),
                    )]),
                    Line::raw("  Ctrl+Enter  Send message"),
                    Line::raw("  Ctrl+C      Cancel/Quit"),
                    Line::raw("  Ctrl+L      Clear screen"),
                    Line::raw("  Ctrl+P      Command palette"),
                    Line::raw("  Ctrl+S      Session picker"),
                    Line::raw("  F1          Toggle help"),
                    Line::raw("  F2          Toggle layout"),
                    Line::raw("  Tab         Next panel"),
                    Line::raw("  Esc         Close overlay"),
                    Line::raw(""),
                    Line::from(vec![Span::styled(
                        "Slash Commands:",
                        Style::default().add_modifier(Modifier::BOLD),
                    )]),
                    Line::raw("  /help       Show help"),
                    Line::raw("  /clear      Clear conversation"),
                    Line::raw("  /model      Switch model"),
                    Line::raw("  /layout     Switch layout"),
                    Line::raw("  /session    Manage sessions"),
                    Line::raw("  /vim        Toggle vim mode"),
                    Line::raw("  /exit       Exit HawkTUI"),
                    Line::raw(""),
                    Line::from(vec![Span::styled(
                        "Press Esc to close",
                        Style::default().fg(self.theme.muted()),
                    )]),
                ];

                let block = Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_type(self.theme.borders.style.to_ratatui())
                    .border_style(Style::default().fg(self.theme.border_focused()))
                    .style(Style::default().bg(Theme::parse_color(&self.theme.panels.sidebar_bg)));

                let para = Paragraph::new(help_text)
                    .block(block)
                    .wrap(Wrap { trim: false });

                frame.render_widget(para, overlay_area);
            }
            _ => {
                // Other overlays - placeholder
                let block = Block::default()
                    .title(" Overlay ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.border_focused()))
                    .style(Style::default().bg(Theme::parse_color(&self.theme.panels.sidebar_bg)));

                frame.render_widget(block, overlay_area);
            }
        }
    }
}

/// Builder for App.
#[derive(Default)]
pub struct AppBuilder {
    theme: Option<String>,
    layout: Option<String>,
    session: Option<String>,
    model: Option<String>,
    provider: Option<String>,
    continue_last: bool,
    initial_message: Option<String>,
}

impl AppBuilder {
    /// Set the theme.
    pub fn theme(mut self, theme: &str) -> Self {
        self.theme = Some(theme.to_string());
        self
    }

    /// Set the layout.
    pub fn layout(mut self, layout: &str) -> Self {
        self.layout = Some(layout.to_string());
        self
    }

    /// Set the session to load.
    pub fn session(mut self, session: Option<String>) -> Self {
        self.session = session;
        self
    }

    /// Set the model.
    pub fn model(mut self, model: Option<String>) -> Self {
        self.model = model;
        self
    }

    /// Set the provider.
    pub fn provider(mut self, provider: Option<String>) -> Self {
        self.provider = provider;
        self
    }

    /// Continue the last session.
    pub fn continue_last(mut self, continue_last: bool) -> Self {
        self.continue_last = continue_last;
        self
    }

    /// Set an initial message to send.
    pub fn initial_message(mut self, message: Option<String>) -> Self {
        self.initial_message = message;
        self
    }

    /// Build the App.
    pub fn build(self) -> Result<App> {
        let theme = Theme::by_name_or_default(self.theme.as_deref().unwrap_or("hawk-dark"));
        let layout_mode = LayoutMode::from_str(self.layout.as_deref().unwrap_or("command-center"));
        let layout_manager = LayoutManager::new(layout_mode);
        let bridge = PiBridge::new(self.model, self.provider);

        let mut state = AppState::new();
        state.conversation.auto_scroll = true;

        Ok(App {
            state,
            layout_manager,
            theme,
            bridge,
            frame: 0,
            initial_message: self.initial_message,
        })
    }
}

/// Convert a character index to a byte index in a string.
/// Returns the byte position where the nth character starts.
#[inline]
fn char_index_to_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(s.len())
}
