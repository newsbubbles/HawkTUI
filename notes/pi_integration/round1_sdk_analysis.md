# Investigation Round 1: pi::sdk API Analysis

**Date**: 2026-04-04  
**Investigator**: Rusty 🦀  
**Ticket**: `dbd0950e-56f8-499c-9556-c8ef59a121f0`

---

## Summary

The `pi::sdk` module is a **comprehensive, stable API surface** for embedding Pi as a library. It provides everything HawkTUI needs.

---

## Key Exports from `pi::sdk`

### Agent & Session
```rust
pub use crate::agent::{
    AbortHandle, AbortSignal, Agent, AgentConfig, AgentEvent, AgentSession, QueueMode,
};
pub use crate::session::Session;
```

### Configuration
```rust
pub use crate::config::Config;
pub use crate::error::{Error, Result};
```

### Messages & Models
```rust
pub use crate::model::{
    AssistantMessage, ContentBlock, Cost, CustomMessage, ImageContent, Message, StopReason,
    StreamEvent, TextContent, ThinkingContent, ToolCall, ToolResultMessage, Usage, UserContent,
    UserMessage,
};
pub use crate::models::{ModelEntry, ModelRegistry};
```

### Providers
```rust
pub use crate::provider::{
    Context as ProviderContext, InputType, Model, ModelCost, Provider, StreamOptions,
    ThinkingBudgets as ProviderThinkingBudgets, ToolDef,
};
```

### Tools
```rust
pub use crate::tools::{Tool, ToolOutput, ToolRegistry, ToolUpdate};

// Built-in tools
pub const BUILTIN_TOOL_NAMES: &[&str] = &[
    "read", "bash", "edit", "write", "grep", "find", "ls", "hashline_edit",
];

// Factory functions
pub fn create_all_tools(cwd: &Path) -> Vec<Box<dyn Tool>>
pub fn all_tool_definitions(cwd: &Path) -> Vec<ToolDefinition>
```

---

## Main Entry Point: `create_agent_session`

```rust
pub async fn create_agent_session(options: SessionOptions) -> Result<AgentSessionHandle>
```

This is THE function to use. It:
1. Loads config from disk
2. Resolves provider/model
3. Creates tool registry
4. Sets up session persistence
5. Returns an `AgentSessionHandle`

---

## SessionOptions (Configuration)

```rust
pub struct SessionOptions {
    pub provider: Option<String>,           // e.g., "anthropic", "openai"
    pub model: Option<String>,              // e.g., "claude-sonnet-4-20250514"
    pub api_key: Option<String>,            // API key override
    pub thinking: Option<ThinkingLevel>,    // Thinking level
    pub system_prompt: Option<String>,      // Custom system prompt
    pub enabled_tools: Option<Vec<String>>, // Tool filter
    pub working_directory: Option<PathBuf>, // CWD for tools
    pub no_session: bool,                   // Ephemeral mode
    pub session_path: Option<PathBuf>,      // Load specific session
    pub session_dir: Option<PathBuf>,       // Session storage directory
    pub max_tool_iterations: usize,         // Max tool loop iterations (default: 50)
    
    // Callbacks
    pub on_event: Option<Arc<dyn Fn(AgentEvent) + Send + Sync>>,
    pub on_tool_start: Option<OnToolStart>,
    pub on_tool_end: Option<OnToolEnd>,
    pub on_stream_event: Option<OnStreamEvent>,
}
```

---

## AgentSessionHandle (Main Interface)

### Prompting
```rust
// Send a prompt and get streaming events
pub async fn prompt(
    &mut self,
    input: impl Into<String>,
    on_event: impl Fn(AgentEvent) + Send + Sync + 'static,
) -> Result<AssistantMessage>

// With abort support
pub async fn prompt_with_abort(
    &mut self,
    input: impl Into<String>,
    abort_signal: AbortSignal,
    on_event: impl Fn(AgentEvent) + Send + Sync + 'static,
) -> Result<AssistantMessage>
```

### Abort/Cancel
```rust
pub fn new_abort_handle() -> (AbortHandle, AbortSignal)
```

### State Access
```rust
pub async fn state(&self) -> Result<AgentSessionState>
pub fn model(&self) -> (String, String)  // (provider, model_id)
pub async fn messages(&self) -> Result<Vec<Message>>
```

### Model/Provider Control
```rust
pub async fn set_model(&mut self, provider: &str, model_id: &str) -> Result<()>
pub async fn set_thinking_level(&mut self, level: ThinkingLevel) -> Result<()>
```

### Event Subscriptions
```rust
pub fn subscribe(&self, listener: impl Fn(AgentEvent) + Send + Sync + 'static) -> SubscriptionId
pub fn unsubscribe(&self, id: SubscriptionId) -> bool
```

---

## AgentEvent (Streaming Events)

```rust
pub enum AgentEvent {
    AgentStart { session_id },
    AgentEnd { session_id, messages, error },
    TurnStart { session_id, turn_index, timestamp },
    TurnEnd { session_id, turn_index, message, tool_results },
    MessageStart { message },
    MessageUpdate { message, assistant_message_event },  // STREAMING TEXT HERE
    MessageEnd { message },
    ToolExecutionStart { tool_call_id, tool_name, args },
    ToolExecutionUpdate { tool_call_id, tool_name, args, partial_result },
    ToolExecutionEnd { tool_call_id, tool_name, result, is_error },
    AutoCompactionStart { reason },
    AutoCompactionEnd { result, aborted, will_retry, error_message },
    AutoRetryStart { attempt, max_attempts, delay_ms, error_message },
    AutoRetryEnd { success, attempt, final_error },
    ExtensionError { extension_id, event, error },
}
```

### AssistantMessageEvent (Nested in MessageUpdate)

```rust
pub enum AssistantMessageEvent {
    Start { .. },
    TextStart { content_index },
    TextDelta { content_index, delta },  // CHARACTER-BY-CHARACTER STREAMING
    TextEnd { content_index, content },
    ThinkingStart { content_index },
    ThinkingDelta { content_index, delta },
    ThinkingEnd { content_index, content },
    ToolCallStart { content_index },
    ToolCallDelta { content_index, delta },
    ToolCallEnd { content_index, tool_call },
    Done { reason, message },
    Error { reason, error },
}
```

---

## Callback Types

```rust
pub type OnToolStart = Arc<dyn Fn(&str, &Value) + Send + Sync>;
pub type OnToolEnd = Arc<dyn Fn(&str, &ToolOutput, bool) + Send + Sync>;
pub type OnStreamEvent = Arc<dyn Fn(&StreamEvent) + Send + Sync>;
pub type EventSubscriber = Arc<dyn Fn(AgentEvent) + Send + Sync>;
```

---

## Confidence Level: HIGH

The SDK is well-documented, has tests, and provides a clean API surface.

---

## Next Steps

1. **Round 2**: Analyze current HawkTUI stubs to understand what needs replacing
2. **Round 3**: Map AgentEvent → HawkTUI's AgentEvent
3. **Round 4**: Session storage analysis
4. **Round 5**: Tool registry analysis
