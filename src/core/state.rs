//! Application state management for HawkTUI.
//!
//! The state follows a unidirectional data flow pattern:
//! Events -> Update State -> Render UI

use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use uuid::Uuid;

/// The main application state.
#[derive(Debug)]
pub struct AppState {
    /// Current mode the app is in.
    pub mode: AppMode,

    /// Current layout configuration.
    pub layout: LayoutMode,

    /// Active panel (for keyboard focus).
    pub active_panel: Panel,

    /// Current conversation.
    pub conversation: Conversation,

    /// List of sessions.
    pub sessions: Vec<SessionInfo>,

    /// Selected session index in the list (for UI navigation).
    pub selected_session_index: Option<usize>,

    /// Current session ID.
    pub current_session_id: Option<Uuid>,

    /// Input buffer for the message editor.
    pub input: InputState,

    /// Status bar information.
    pub status: StatusInfo,

    /// Tool execution state.
    pub tools: ToolsState,

    /// Context/attachments state.
    pub context: ContextState,

    /// Whether the app should quit.
    pub should_quit: bool,

    /// Overlay/modal state.
    pub overlay: Option<Overlay>,

    /// Streaming state for assistant responses.
    pub streaming: StreamingState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: AppMode::Normal,
            layout: LayoutMode::CommandCenter,
            active_panel: Panel::Input,
            conversation: Conversation::default(),
            sessions: Vec::new(),
            selected_session_index: None,
            current_session_id: None,
            input: InputState::default(),
            status: StatusInfo::default(),
            tools: ToolsState::default(),
            context: ContextState::default(),
            should_quit: false,
            overlay: None,
            streaming: StreamingState::default(),
        }
    }
}

impl AppState {
    /// Create a new app state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if we're currently streaming a response.
    pub const fn is_streaming(&self) -> bool {
        self.streaming.is_active
    }

    /// Get the current model name.
    pub fn current_model(&self) -> &str {
        &self.status.model
    }

    /// Get total tokens used in current session.
    pub const fn total_tokens(&self) -> u64 {
        self.status.total_tokens
    }
}

/// Application mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    /// Normal mode - ready for input.
    #[default]
    Normal,
    /// Insert mode - typing in editor.
    Insert,
    /// Command mode - entering slash command.
    Command,
    /// Streaming mode - receiving response.
    Streaming,
    /// Waiting mode - waiting for tool execution.
    Waiting,
}

/// Layout modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutMode {
    /// Full command center with all panels.
    #[default]
    CommandCenter,
    /// Focus mode - conversation only.
    Focus,
    /// Split mode - code review style.
    Split,
}

impl LayoutMode {
    /// Parse layout mode from string.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "focus" => Self::Focus,
            "split" => Self::Split,
            _ => Self::CommandCenter,
        }
    }
}

/// UI panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Panel {
    /// Conversation/chat panel.
    Conversation,
    /// Session list panel.
    Sessions,
    /// Tools panel.
    Tools,
    /// Context/attachments panel.
    Context,
    /// Input/editor panel.
    #[default]
    Input,
}

/// Conversation state.
#[derive(Debug, Default)]
pub struct Conversation {
    /// Messages in the conversation.
    pub messages: Vec<Message>,

    /// Scroll position in the viewport.
    pub scroll_offset: u16,

    /// Whether auto-scroll is enabled.
    pub auto_scroll: bool,
}

/// A message in the conversation.
#[derive(Debug, Clone)]
pub struct Message {
    /// Unique message ID.
    pub id: Uuid,

    /// Message role.
    pub role: MessageRole,

    /// Message content.
    pub content: String,

    /// Timestamp.
    pub timestamp: DateTime<Utc>,

    /// Whether this message is still being streamed.
    pub is_streaming: bool,

    /// Tool calls in this message (for assistant messages).
    pub tool_calls: Vec<ToolCall>,

    /// Thinking content (for models with thinking).
    pub thinking: Option<String>,
}

impl Message {
    /// Create a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: MessageRole::User,
            content: content.into(),
            timestamp: Utc::now(),
            is_streaming: false,
            tool_calls: Vec::new(),
            thinking: None,
        }
    }

    /// Create a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: MessageRole::Assistant,
            content: content.into(),
            timestamp: Utc::now(),
            is_streaming: false,
            tool_calls: Vec::new(),
            thinking: None,
        }
    }

    /// Create a new streaming assistant message.
    pub fn assistant_streaming() -> Self {
        Self {
            id: Uuid::new_v4(),
            role: MessageRole::Assistant,
            content: String::new(),
            timestamp: Utc::now(),
            is_streaming: true,
            tool_calls: Vec::new(),
            thinking: None,
        }
    }

    /// Create a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: MessageRole::System,
            content: content.into(),
            timestamp: Utc::now(),
            is_streaming: false,
            tool_calls: Vec::new(),
            thinking: None,
        }
    }
}

/// Message role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// Tool call information.
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: String,
    pub output: Option<String>,
    pub status: ToolCallStatus,
}

/// Tool call status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCallStatus {
    Pending,
    Running,
    Success,
    Error,
}

/// Session information.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
    pub is_active: bool,
}

/// Input/editor state.
#[derive(Debug, Default)]
pub struct InputState {
    /// Current input text.
    pub text: String,

    /// Cursor position (character index, not byte index).
    pub cursor: usize,

    /// Input history.
    pub history: VecDeque<String>,

    /// Current position in history.
    pub history_index: Option<usize>,

    /// Whether in vim mode.
    pub vim_mode: bool,

    /// Vim mode state.
    pub vim_state: VimState,
}

/// Vim mode state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimState {
    #[default]
    Normal,
    Insert,
    Visual,
}

/// Status bar information.
#[derive(Debug)]
pub struct StatusInfo {
    /// Current model name.
    pub model: String,

    /// Current provider.
    pub provider: String,

    /// Total tokens used.
    pub total_tokens: u64,

    /// Estimated cost.
    pub cost: f64,

    /// Connection status.
    pub connection: ConnectionStatus,

    /// Current session name.
    pub session_name: Option<String>,
}

impl Default for StatusInfo {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-20250514".to_string(),
            provider: "anthropic".to_string(),
            total_tokens: 0,
            cost: 0.0,
            connection: ConnectionStatus::Disconnected,
            session_name: None,
        }
    }
}

/// Connection status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Streaming,
    Error,
}

/// Tools panel state.
#[derive(Debug, Default)]
pub struct ToolsState {
    /// Available tools.
    pub available: Vec<ToolInfo>,

    /// Currently executing tools.
    pub executing: Vec<ExecutingTool>,

    /// Selected tool index (for UI navigation).
    pub selected_index: Option<usize>,

    /// Scroll position.
    pub scroll_offset: u16,
}

/// Tool information.
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

/// Currently executing tool.
#[derive(Debug, Clone)]
pub struct ExecutingTool {
    pub id: String,
    pub name: String,
    pub started_at: DateTime<Utc>,
    pub progress: Option<f32>,
}

/// Context/attachments state.
#[derive(Debug, Default)]
pub struct ContextState {
    /// Attached files.
    pub files: Vec<AttachedFile>,

    /// Total context tokens.
    pub total_tokens: u64,

    /// Context window usage (0.0 - 1.0).
    pub window_usage: f32,
}

/// Attached file.
#[derive(Debug, Clone)]
pub struct AttachedFile {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub tokens: u64,
}

/// Overlay/modal state.
#[derive(Debug, Clone)]
pub enum Overlay {
    /// Help overlay.
    Help,
    /// Command palette.
    CommandPalette { query: String, selected: usize },
    /// Session picker.
    SessionPicker { selected: usize },
    /// Model picker.
    ModelPicker { selected: usize },
    /// Confirmation dialog.
    Confirm { title: String, message: String },
}

/// Streaming state.
#[derive(Debug, Default)]
pub struct StreamingState {
    /// Whether currently streaming.
    pub is_active: bool,

    /// Current streaming message ID.
    pub message_id: Option<Uuid>,

    /// Tokens streamed so far.
    pub tokens_streamed: u64,

    /// Whether thinking is visible.
    pub thinking_visible: bool,

    /// Current thinking content.
    pub thinking_buffer: String,
}
