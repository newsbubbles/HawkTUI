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
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout as RatatuiLayout, Rect},
    Terminal,
};

use crate::core::{
    error::{Error, Result},
    events::{Action, map_key_to_action},
    state::{AppMode, AppState, ConnectionStatus, LayoutMode, Message, Panel, ToolInfo},
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
        self.state.status.model = self.bridge.model().to_string();
        self.state.status.provider = self.bridge.provider().to_string();

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
    fn restore_terminal(
        &self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
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
                let action = map_key_to_action(key, self.state.mode, self.state.overlay.as_ref());
                self.handle_action(action).await?;
            }
            CrosstermEvent::Resize(width, height) => {
                // Terminal resized - ratatui handles this automatically
                tracing::debug!("Terminal resized to {width}x{height}");
            }
            CrosstermEvent::Mouse(_mouse) => {
                // TODO: Handle mouse events
            }
            CrosstermEvent::Paste(text) => {
                // Handle paste
                for c in text.chars() {
                    self.state.input.text.insert(self.state.input.cursor, c);
                    self.state.input.cursor += 1;
                }
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
                self.state.overlay = Some(crate::core::state::Overlay::SessionPicker {
                    selected: 0,
                });
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
                self.state.input.text.insert(self.state.input.cursor, c);
                self.state.input.cursor += 1;
            }
            Action::Backspace => {
                if self.state.input.cursor > 0 {
                    self.state.input.cursor -= 1;
                    self.state.input.text.remove(self.state.input.cursor);
                }
                // Exit command mode if we deleted the slash
                if self.state.input.text.is_empty() {
                    self.state.mode = AppMode::Normal;
                }
            }
            Action::DeleteChar => {
                if self.state.input.cursor < self.state.input.text.len() {
                    self.state.input.text.remove(self.state.input.cursor);
                }
            }
            Action::CursorLeft => {
                self.state.input.cursor = self.state.input.cursor.saturating_sub(1);
            }
            Action::CursorRight => {
                self.state.input.cursor =
                    (self.state.input.cursor + 1).min(self.state.input.text.len());
            }
            Action::CursorHome => {
                self.state.input.cursor = 0;
            }
            Action::CursorEnd => {
                self.state.input.cursor = self.state.input.text.len();
            }
            Action::HistoryPrev => {
                // TODO: Implement history navigation
            }
            Action::HistoryNext => {
                // TODO: Implement history navigation
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
        self.state.conversation.messages.push(Message::user(message));
        self.state.conversation.auto_scroll = true;

        // Start streaming mode
        self.state.mode = AppMode::Streaming;
        self.state.status.connection = ConnectionStatus::Streaming;
        self.state.streaming.is_active = true;

        // Add placeholder assistant message
        let assistant_msg = Message::assistant_streaming();
        self.state.streaming.message_id = Some(assistant_msg.id);
        self.state.conversation.messages.push(assistant_msg);

        // TODO: Actually send to agent and handle streaming response
        // For now, simulate a response
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
                    self.bridge.set_model(model);
                    self.state.status.model = model.clone();
                }
            }
            "layout" => {
                if let Some(layout) = parsed.args.first() {
                    self.layout_manager.set_mode(LayoutMode::from_str(layout));
                }
            }
            "vim" => {
                self.state.input.vim_mode = !self.state.input.vim_mode;
            }
            _ => {
                tracing::info!("Command not yet implemented: {}", cmd.name);
            }
        }

        self.state.mode = AppMode::Normal;
        Ok(())
    }

    /// Simulate a response (placeholder until pi integration is complete).
    async fn simulate_response(&mut self, _message: &str) {
        // Simulate streaming delay
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Update the assistant message
        if let Some(msg) = self.state.conversation.messages.last_mut() {
            msg.content = "I'm HawkTUI, your AI coding assistant! 🦅\n\n\
                This is a placeholder response. Once pi_agent_rust integration is complete, \
                I'll be able to help you with:\n\n\
                - **Code analysis** and optimization\n\
                - **Debugging** assistance\n\
                - **File operations** (read, write, search)\n\
                - **Shell commands** execution\n\n\
                Try `/help` to see available commands!"
                .to_string();
            msg.is_streaming = false;
        }

        // End streaming
        self.state.streaming.is_active = false;
        self.state.mode = AppMode::Normal;
        self.state.status.connection = ConnectionStatus::Connected;
        self.state.status.total_tokens += 150; // Simulated
        self.state.status.cost += 0.001; // Simulated
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
                None,
                &self.theme,
                self.state.active_panel == Panel::Sessions,
            );
            frame.render_widget(sessions, sidebar_chunks[0]);

            // Tools panel
            let tools = ToolsPanel::new(
                &self.state.tools,
                &self.theme,
                self.state.active_panel == Panel::Tools,
            );
            frame.render_widget(tools, sidebar_chunks[1]);
        }

        // Conversation panel
        let conversation = ConversationPanel::new(
            &self.state.conversation,
            &self.theme,
            self.state.active_panel == Panel::Conversation,
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

        let hints = vec![
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
                        Style::default()
                            .fg(self.theme.bg())
                            .bg(self.theme.muted()),
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

                let para = Paragraph::new(help_text).block(block).wrap(Wrap { trim: false });

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
        let theme = Theme::by_name(self.theme.as_deref().unwrap_or("hawk-dark"));
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
