# HawkTUI v0.2.0 - Deeper Investigation: pi_agent_rust Integration

**Date**: 2026-04-04  
**Investigator**: Rusty 🦀  
**Depth**: 2 (Deeper Investigation)

---

## Objective

Integrate HawkTUI with `pi_agent_rust` to enable real AI agent functionality.

---

## pi_agent_rust API Analysis

### Core Types

| Type | Location | Purpose |
|------|----------|--------|
| `Agent` | `src/agent.rs:411` | Main agent orchestrator |
| `AgentConfig` | `src/agent.rs:56` | Agent configuration |
| `AgentEvent` | `src/agent.rs:206` | Events emitted during execution |
| `Provider` | `src/provider.rs:28` | LLM backend trait |
| `Tool` | `src/tools.rs:34` | Tool execution trait |
| `ToolRegistry` | `src/tools.rs` | Collection of available tools |
| `Session` | `src/session.rs` | Session persistence |
| `AbortHandle` / `AbortSignal` | `src/agent.rs:315` | Cancellation support |

### Key Agent Methods

```rust
// Create agent
Agent::new(provider: Arc<dyn Provider>, tools: ToolRegistry, config: AgentConfig) -> Self

// Run with user input (streaming events via callback)
agent.run(
    user_input: impl Into<String>,
    on_event: impl Fn(AgentEvent) + Send + Sync + 'static,
) -> Result<AssistantMessage>

// Run with abort support
agent.run_with_abort(
    user_input: impl Into<String>,
    abort: Option<AbortSignal>,
    on_event: impl Fn(AgentEvent) + Send + Sync + 'static,
) -> Result<AssistantMessage>

// Message management
agent.messages() -> &[Message]
agent.add_message(message: Message)
agent.clear_messages()
agent.replace_messages(messages: Vec<Message>)

// Provider management
agent.set_provider(provider: Arc<dyn Provider>)
```

### AgentEvent Variants

```rust
pub enum AgentEvent {
    AgentStart { session_id },
    AgentEnd { session_id, messages, error },
    TurnStart { session_id, turn_index, timestamp },
    TurnEnd { session_id, turn_index, message, tool_results },
    MessageStart { message },
    MessageUpdate { message, assistant_message_event },
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

### AssistantMessageEvent (for streaming)

```rust
pub enum AssistantMessageEvent {
    TextDelta { text: String },
    ThinkingDelta { text: String },
    ToolCall { tool_call: ToolCall },
    // ... other variants
}
```

---

## Integration Strategy

### Option A: Direct Library Integration (Recommended)

Uncomment the Cargo.toml dependency and use pi_agent_rust as a library:

```toml
pi_agent_rust = { path = "../pi_agent_rust_source" }
```

**Pros:**
- Full access to all types
- Compile-time type safety
- Share sessions with pi CLI

**Cons:**
- Tightly coupled
- Need to handle pi's async runtime

### Option B: Process-based Integration

Spawn `pi` CLI as a subprocess and communicate via stdin/stdout.

**Pros:**
- Loose coupling
- Independent lifecycles

**Cons:**
- More complex IPC
- Less type safety
- Harder to share state

---

## Implementation Plan

### Phase 1: Enable Dependency

1. Uncomment `pi_agent_rust` in `Cargo.toml`
2. Add necessary re-exports to `pi_bridge.rs`
3. Verify compilation

### Phase 2: Provider Setup

1. Create provider based on user's config
2. Use `pi::providers::anthropic::AnthropicProvider` or similar
3. Handle API key from environment

### Phase 3: Agent Lifecycle

1. Create `Agent` on connect
2. Store in `PiBridge`
3. Implement `send_message` using `agent.run()`

### Phase 4: Event Translation

1. Map `pi::agent::AgentEvent` → `hawktui::core::events::AgentEvent`
2. Handle streaming via callback
3. Update UI state based on events

### Phase 5: Session Management

1. Use pi's `Session` type for persistence
2. Load/save sessions via pi's session storage
3. Wire to sessions panel

### Phase 6: Tool Integration

1. Get tools from pi's `ToolRegistry`
2. Display in tools panel
3. Show execution progress

---

## File Changes Required

| File | Changes |
|------|--------|
| `Cargo.toml` | Uncomment pi_agent_rust dependency |
| `src/providers/pi_bridge.rs` | Full implementation |
| `src/core/events.rs` | Ensure AgentEvent matches pi's events |
| `src/app.rs` | Wire event handling to UI updates |
| `src/ui/panels/sessions.rs` | Display real session data |
| `src/ui/panels/tools.rs` | Display real tool data |

---

## Confidence Level

**HIGH** - The pi_agent_rust API is well-documented and the integration path is clear.

---

## Next Steps

1. Enable the dependency and verify compilation
2. Implement provider creation
3. Wire up the agent lifecycle
4. Test with real AI responses

---

*Ready for implementation. Proceeding with Phase 1.*
