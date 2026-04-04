//! Event handling for HawkTUI.
//!
//! Events flow through the application:
//! 1. Terminal events (keyboard, mouse, resize)
//! 2. Agent events (streaming, tool calls)
//! 3. Internal events (timers, state updates)

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use uuid::Uuid;

/// Application events.
#[derive(Debug, Clone)]
pub enum Event {
    /// Terminal events.
    Terminal(TerminalEvent),

    /// Agent/AI events.
    Agent(AgentEvent),

    /// Internal application events.
    Internal(InternalEvent),

    /// Tick event for animations.
    Tick,
}

/// Terminal events from crossterm.
#[derive(Debug, Clone)]
pub enum TerminalEvent {
    /// Key press.
    Key(KeyEvent),

    /// Mouse event.
    Mouse(MouseEvent),

    /// Terminal resize.
    Resize { width: u16, height: u16 },

    /// Focus gained.
    FocusGained,

    /// Focus lost.
    FocusLost,

    /// Paste event.
    Paste(String),
}

/// Agent/AI events.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Connection established.
    Connected,

    /// Connection lost.
    Disconnected,

    /// Streaming started.
    StreamStart { message_id: Uuid },

    /// Text delta received.
    TextDelta { text: String },

    /// Thinking delta received.
    ThinkingDelta { text: String },

    /// Thinking started.
    ThinkingStart,

    /// Thinking ended.
    ThinkingEnd,

    /// Tool call started.
    ToolStart { id: String, name: String, input: String },

    /// Tool call progress.
    ToolProgress { id: String, progress: f32 },

    /// Tool call completed.
    ToolEnd { id: String, output: String, is_error: bool },

    /// Streaming completed.
    StreamEnd { stop_reason: StopReason },

    /// Usage update.
    Usage {
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: Option<u64>,
        cache_write_tokens: Option<u64>,
    },

    /// Error occurred.
    Error { message: String },
}

/// Stop reasons for streaming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    Cancelled,
}

/// Internal application events.
#[derive(Debug, Clone)]
pub enum InternalEvent {
    /// Session loaded.
    SessionLoaded { id: Uuid },

    /// Session created.
    SessionCreated { id: Uuid, name: String },

    /// Session list updated.
    SessionsUpdated,

    /// Theme changed.
    ThemeChanged { name: String },

    /// Layout changed.
    LayoutChanged { layout: String },

    /// Notification.
    Notification { message: String, level: NotificationLevel },

    /// Command executed.
    CommandExecuted { command: String },

    /// File attached.
    FileAttached { path: String },

    /// File detached.
    FileDetached { path: String },
}

/// Notification levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Action that can be triggered by events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Quit the application.
    Quit,

    /// Send the current input.
    SendMessage,

    /// Cancel current operation.
    Cancel,

    /// Clear the screen.
    ClearScreen,

    /// Toggle help overlay.
    ToggleHelp,

    /// Open command palette.
    OpenCommandPalette,

    /// Close overlay.
    CloseOverlay,

    /// Switch to panel.
    FocusPanel(super::state::Panel),

    /// Cycle to next panel.
    NextPanel,

    /// Cycle to previous panel.
    PrevPanel,

    /// Scroll up.
    ScrollUp(u16),

    /// Scroll down.
    ScrollDown(u16),

    /// Scroll to top.
    ScrollToTop,

    /// Scroll to bottom.
    ScrollToBottom,

    /// Toggle layout mode.
    ToggleLayout,

    /// Open session picker.
    OpenSessionPicker,

    /// Open model picker.
    OpenModelPicker,

    /// Create new session.
    NewSession,

    /// Continue last session.
    ContinueSession,

    /// Select next session in list.
    SelectNextSession,

    /// Select previous session in list.
    SelectPrevSession,

    /// Switch to selected session.
    SwitchToSelectedSession,

    /// Refresh sessions list.
    RefreshSessions,

    /// Select next tool in list.
    SelectNextTool,

    /// Select previous tool in list.
    SelectPrevTool,

    /// Toggle selected tool enabled/disabled.
    ToggleSelectedTool,

    /// Refresh tools list.
    RefreshTools,

    /// Copy selection.
    Copy,

    /// Paste.
    Paste,

    /// Undo.
    Undo,

    /// Redo.
    Redo,

    /// Insert character.
    InsertChar(char),

    /// Delete character.
    DeleteChar,

    /// Backspace.
    Backspace,

    /// Move cursor left.
    CursorLeft,

    /// Move cursor right.
    CursorRight,

    /// Move cursor to start.
    CursorHome,

    /// Move cursor to end.
    CursorEnd,

    /// History previous.
    HistoryPrev,

    /// History next.
    HistoryNext,

    /// Toggle vim mode.
    ToggleVimMode,

    /// Execute slash command.
    ExecuteCommand(String),

    /// Attach file.
    AttachFile(String),

    /// No action.
    None,
}

impl Event {
    /// Create a key event.
    pub const fn key(event: KeyEvent) -> Self {
        Self::Terminal(TerminalEvent::Key(event))
    }

    /// Create a resize event.
    pub const fn resize(width: u16, height: u16) -> Self {
        Self::Terminal(TerminalEvent::Resize { width, height })
    }

    /// Create a text delta event.
    pub fn text_delta(text: impl Into<String>) -> Self {
        Self::Agent(AgentEvent::TextDelta { text: text.into() })
    }
}

/// Map a key event to an action.
pub fn map_key_to_action(
    key: KeyEvent,
    mode: super::state::AppMode,
    overlay: Option<&super::state::Overlay>,
    active_panel: Option<super::state::Panel>,
) -> Action {
    use super::state::AppMode;

    // Handle overlay-specific keys first
    if overlay.is_some() {
        return match key.code {
            KeyCode::Esc => Action::CloseOverlay,
            KeyCode::Enter => Action::None, // Handled by overlay
            _ => Action::None,
        };
    }

    // Global shortcuts (work in any mode)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => {
                if matches!(mode, AppMode::Streaming | AppMode::Waiting) {
                    Action::Cancel
                } else {
                    Action::Quit
                }
            }
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('l') => Action::ClearScreen,
            KeyCode::Char('p') => Action::OpenCommandPalette,
            KeyCode::Char('s') => Action::OpenSessionPicker,
            KeyCode::Char('h') => Action::ToggleHelp,
            KeyCode::Enter => Action::SendMessage,
            _ => Action::None,
        };
    }

    // Mode-specific handling
    match mode {
        AppMode::Normal | AppMode::Insert => {
            // Handle panel-specific keys first
            if matches!(active_panel, Some(super::state::Panel::Sessions)) {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => return Action::SelectPrevSession,
                    KeyCode::Down | KeyCode::Char('j') => return Action::SelectNextSession,
                    KeyCode::Enter => return Action::SwitchToSelectedSession,
                    KeyCode::Char('r') => return Action::RefreshSessions,
                    _ => {}
                }
            }

            if matches!(active_panel, Some(super::state::Panel::Tools)) {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => return Action::SelectPrevTool,
                    KeyCode::Down | KeyCode::Char('j') => return Action::SelectNextTool,
                    KeyCode::Enter | KeyCode::Char(' ') => return Action::ToggleSelectedTool,
                    KeyCode::Char('r') => return Action::RefreshTools,
                    _ => {}
                }
            }

            match key.code {
                KeyCode::Esc => Action::CloseOverlay,
                KeyCode::F(1) => Action::ToggleHelp,
                KeyCode::F(2) => Action::ToggleLayout,
                KeyCode::Tab => Action::NextPanel,
                KeyCode::BackTab => Action::PrevPanel,
                KeyCode::Char(c) => Action::InsertChar(c),
                KeyCode::Backspace => Action::Backspace,
                KeyCode::Delete => Action::DeleteChar,
                KeyCode::Left => Action::CursorLeft,
                KeyCode::Right => Action::CursorRight,
                KeyCode::Home => Action::CursorHome,
                KeyCode::End => Action::CursorEnd,
                KeyCode::Up => Action::HistoryPrev,
                KeyCode::Down => Action::HistoryNext,
                KeyCode::PageUp => Action::ScrollUp(10),
                KeyCode::PageDown => Action::ScrollDown(10),
                _ => Action::None,
            }
        }
        AppMode::Command => match key.code {
            KeyCode::Esc => Action::CloseOverlay,
            KeyCode::Enter => Action::None, // Command execution handled separately
            KeyCode::Char(c) => Action::InsertChar(c),
            KeyCode::Backspace => Action::Backspace,
            _ => Action::None,
        },
        AppMode::Streaming | AppMode::Waiting => match key.code {
            KeyCode::Esc => Action::Cancel,
            KeyCode::PageUp => Action::ScrollUp(10),
            KeyCode::PageDown => Action::ScrollDown(10),
            _ => Action::None,
        },
    }
}
