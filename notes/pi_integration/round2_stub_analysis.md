# Investigation Round 2: Current Stub Analysis

**Date**: 2026-04-04  
**Investigator**: Rusty 🦀  
**Ticket**: `dbd0950e-56f8-499c-9556-c8ef59a121f0`

---

## Summary

The current `pi_bridge.rs` is entirely stubs. Every method returns hardcoded data or does nothing.

---

## Current PiBridge Structure

```rust
pub struct PiBridge {
    model: String,           // Just a string, not connected to pi
    provider: String,        // Just a string, not connected to pi
    connected: bool,         // Fake flag, always true after connect()
}
```

### What's Missing:
- No `AgentSessionHandle` from pi::sdk
- No `AbortHandle` for cancellation
- No event translation
- No real session storage
- No real tool registry

---

## Stub Methods to Replace

| Method | Current Behavior | Real Implementation |
|--------|------------------|--------------------|
| `new()` | Stores model/provider strings | Create `SessionOptions`, prepare for connection |
| `connect()` | Sets `connected = true` | Call `create_agent_session()`, store handle |
| `send_message()` | Logs message, returns Ok | Call `handle.prompt()` with event callback |
| `cancel()` | Logs "cancelling" | Call `abort_handle.abort()` |
| `load_session()` | Returns Ok | Use `SessionOptions::session_path` |
| `create_session()` | Returns random UUID | Create new session via pi |
| `list_sessions()` | Returns empty Vec | Query pi's session storage |
| `available_tools()` | Returns 4 hardcoded tools | Call `pi::sdk::all_tool_definitions()` |

---

## HawkTUI AgentEvent Mapping

| HawkTUI Event | pi::sdk Event |
|---------------|---------------|
| `Connected` | After `create_agent_session()` succeeds |
| `Disconnected` | On error or shutdown |
| `StreamStart` | `AgentEvent::TurnStart` |
| `TextDelta` | `AgentEvent::MessageUpdate` with `TextDelta` |
| `ThinkingDelta` | `AgentEvent::MessageUpdate` with `ThinkingDelta` |
| `ThinkingStart` | `AgentEvent::MessageUpdate` with `ThinkingStart` |
| `ThinkingEnd` | `AgentEvent::MessageUpdate` with `ThinkingEnd` |
| `ToolStart` | `AgentEvent::ToolExecutionStart` |
| `ToolProgress` | `AgentEvent::ToolExecutionUpdate` |
| `ToolEnd` | `AgentEvent::ToolExecutionEnd` |
| `StreamEnd` | `AgentEvent::TurnEnd` or `AgentEvent::AgentEnd` |
| `Usage` | From `AssistantMessage` after completion |
| `Error` | Any error event |

---

## New PiBridge Structure

```rust
use pi::sdk::{
    AgentSessionHandle, AbortHandle, AbortSignal, AgentEvent as PiAgentEvent,
    SessionOptions, create_agent_session, all_tool_definitions,
    AssistantMessageEvent, ToolOutput,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct PiBridge {
    // Session handle (None until connected)
    handle: Option<Arc<Mutex<AgentSessionHandle>>>,
    
    // Abort handle for current operation
    abort_handle: Option<AbortHandle>,
    
    // Configuration
    model: Option<String>,
    provider: Option<String>,
    working_directory: Option<PathBuf>,
    
    // Connection state
    connected: bool,
}
```

---

## Implementation Plan

1. **Update imports** - Use `pi::sdk::*`
2. **Add handle field** - `Option<Arc<Mutex<AgentSessionHandle>>>`
3. **Implement connect()** - Call `create_agent_session()`
4. **Implement send_message()** - Call `handle.prompt()` with event translation
5. **Implement cancel()** - Use `AbortHandle`
6. **Implement list_sessions()** - Query session storage
7. **Implement available_tools()** - Use `all_tool_definitions()`

---

## Confidence Level: HIGH

The mapping is straightforward. The SDK provides everything we need.

---

## Next Steps

Proceeding to implementation. Rounds 3-5 can be done during implementation as needed.
