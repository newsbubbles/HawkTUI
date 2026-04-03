# HawkTUI Code Review - Round 2: UI Panel Deep Dive

**Date**: 2026-04-03  
**Reviewer**: Rusty 🦀  
**Status**: ✅ Complete

---

## Overview

This round audits each UI panel for real implementations vs placeholders, checking:
- Data binding to state
- Event handling
- Edge case handling
- Rendering completeness

---

## 1. Header Panel (`src/ui/panels/header.rs`)

### Purpose
Status bar showing model name, token count, cost, connection status, and session name.

### Implementation Completeness: **95%** ✅

### What's Real ✅

| Feature | Status | Evidence |
|---------|--------|----------|
| Model name display | ✅ Real | `&self.status.model` from state |
| Token count with K/M formatting | ✅ Real | `format_tokens()` function |
| Cost display | ✅ Real | `format!("${:.3}", self.status.cost)` |
| Connection status icons | ✅ Real | 5 states: Disconnected, Connecting, Connected, Streaming, Error |
| Session name (right-aligned) | ✅ Real | `self.status.session_name` from state |
| Theme integration | ✅ Real | Uses `self.theme.*` colors throughout |
| Version display | ✅ Real | Passed via `VERSION` constant |

### Issues Found

#### 🟡 Minor: Session name truncation not handled

**Location**: `header.rs:90-101`

```rust
if let Some(ref session_name) = self.status.session_name {
    let used: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    let remaining = area.width as usize - used - session_name.len() - 5;
    if remaining > 0 {
        // ... render session name
    }
}
```

**Problem**: If `session_name.len()` is very long, the calculation can overflow (usize subtraction). The session name is simply not shown if it doesn't fit, rather than being truncated.

**Risk**: Low - just won't display long session names.

#### 🟢 Good: Token formatting is solid

```rust
fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}k", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}
```

### Edge Cases

| Case | Handled? |
|------|----------|
| Empty model name | 🟡 Displays empty, no fallback |
| Zero tokens | ✅ Shows "0" |
| Very small terminal width | 🟡 Session name hidden, rest may overflow |
| Unicode in session name | 🟡 Uses `.len()` not `.width()` - could misalign |

---

## 2. Conversation Panel (`src/ui/panels/conversation.rs`)

### Purpose
Displays chat messages with scrolling, timestamps, tool calls, and thinking blocks.

### Implementation Completeness: **85%** ✅

### What's Real ✅

| Feature | Status | Evidence |
|---------|--------|----------|
| Message rendering | ✅ Real | `render_message()` builds lines from `Message` |
| Role icons & colors | ✅ Real | User 👤, Assistant 🤖, System ⚙️, Tool 🔧 |
| Time ago formatting | ✅ Real | `format_time_ago()` with mins/hours/days |
| Streaming indicator | ✅ Real | Green dot when `message.is_streaming` |
| Thinking block display | ✅ Real | Shows 💭 with first 3 lines truncated |
| Tool call status | ✅ Real | Pending ⏳, Running ⚡, Success ✓, Error ✗ |
| Text wrapping | ✅ Real | Uses `textwrap::wrap()` |
| Scrollbar | ✅ Real | `Scrollbar` with ↑↓ symbols |
| Auto-scroll | ✅ Real | Scrolls to bottom when `auto_scroll` is true |
| Empty state | ✅ Real | Welcome message when no messages |
| Focus border | ✅ Real | Different border color when focused |

### Issues Found

#### 🟡 Medium: No real Markdown rendering

**Location**: `conversation.rs:90-109`

```rust
// Check if this is a code block
if line.starts_with("```") {
    // ... style as keyword
} else if line.starts_with("  ") || line.starts_with("\t") {
    // Code content
    // ... style as string
} else {
    // Regular text
}
```

**Problem**: This is a naive heuristic, not actual markdown parsing. The `pulldown-cmark` dependency exists but is **not used**.

**Missing**:
- Bold/italic text
- Headers
- Lists
- Links
- Inline code
- Proper code block language detection

**Risk**: Medium - AI responses with markdown won't render properly.

#### 🟡 Medium: No syntax highlighting

**Location**: Code blocks just use a single color.

```rust
Span::styled(
    line.to_string(),
    Style::default().fg(Theme::parse_color(&self.theme.syntax.string)),
)
```

**Problem**: The `syntect` dependency exists but is **not used**. Code blocks are all one color.

**Risk**: Medium - reduces readability for code-heavy responses.

#### 🟢 Good: Scrolling implementation

```rust
let scroll = if self.conversation.auto_scroll && total_lines > visible_lines {
    (total_lines - visible_lines) as u16
} else {
    self.conversation.scroll_offset
};
```

Auto-scroll works correctly, and manual scroll offset is preserved.

#### 🟡 Minor: `truncate_str` returns slice, not truncated with "..."

**Location**: `conversation.rs:234-249`

```rust
fn truncate_str(s: &str, max_width: usize) -> &str {
    // ... returns &s[..end] without "..."
}
```

**Problem**: Function calculates space for "..." (`max_width.saturating_sub(3)`) but doesn't actually append it.

### Edge Cases

| Case | Handled? |
|------|----------|
| Empty conversation | ✅ Welcome message |
| Very long message | ✅ Text wrapping |
| Unicode/emoji in messages | ✅ Uses `unicode_width` |
| Hundreds of messages | 🟡 All rendered (no virtualization) |
| Message with no content | ✅ Just shows header |
| Deeply nested code blocks | 🟡 Naive detection may fail |

---

## 3. Sessions Panel (`src/ui/panels/sessions.rs`)

### Purpose
Lists sessions with selection highlighting and active status.

### Implementation Completeness: **80%** ✅

### What's Real ✅

| Feature | Status | Evidence |
|---------|--------|----------|
| Session list rendering | ✅ Real | Iterates `self.sessions` |
| Active indicator | ✅ Real | ● for active, ○ for inactive |
| Selection highlighting | ✅ Real | Uses `ListState` with `highlight_style` |
| Empty state | ✅ Real | "No sessions" message |
| Focus border | ✅ Real | Different border when focused |

### Issues Found

#### 🔴 Critical: Sessions list is always empty!

**Location**: `app.rs` - sessions are never populated.

```rust
// In AppState::default()
pub sessions: Vec<SessionInfo>,  // Always empty!
```

**Evidence from `app.rs:456-461`**:
```rust
let sessions = SessionsPanel::new(
    &self.state.sessions,  // This is always empty
    None,                  // Selection is always None
    &self.theme,
    self.state.active_panel == Panel::Sessions,
);
```

**Problem**: 
1. `self.state.sessions` is never populated from `PiBridge`
2. Selection is hardcoded to `None`
3. No session loading/saving logic exists

**Risk**: High - panel is essentially non-functional.

#### 🟡 Medium: No session management actions

**Missing**:
- Create new session
- Delete session
- Rename session
- Switch session (loads conversation)

### Edge Cases

| Case | Handled? |
|------|----------|
| Empty sessions | ✅ Shows message |
| Many sessions (scrolling) | 🟡 No explicit scroll handling |
| Long session names | 🟡 Not truncated |
| Session selection | 🔴 Not connected to state |

---

## 4. Tools Panel (`src/ui/panels/tools.rs`)

### Purpose
Shows available tools and currently executing tools with progress.

### Implementation Completeness: **75%** ⚠️

### What's Real ✅

| Feature | Status | Evidence |
|---------|--------|----------|
| Available tools list | ✅ Real | Iterates `self.tools_state.available` |
| Executing tools list | ✅ Real | Iterates `self.tools_state.executing` |
| Tool enabled/disabled | ✅ Real | ✓ for enabled, ○ for disabled |
| Progress percentage | ✅ Real | Displays `tool.progress` if present |
| Empty state | ✅ Real | "No tools loaded" message |
| Focus border | ✅ Real | Different border when focused |

### Issues Found

#### 🔴 Critical: Tools come from stubbed PiBridge!

**Location**: `app.rs:81-89`

```rust
self.state.tools.available = self
    .bridge
    .available_tools()  // This returns fake data!
    .into_iter()
    .map(|t| ToolInfo { ... })
    .collect();
```

**From `pi_bridge.rs:90-102`**:
```rust
pub fn available_tools(&self) -> Vec<PiToolInfo> {
    // Return some default tools for now
    vec![
        PiToolInfo { name: "read_file".to_string(), description: "Read file contents".to_string(), enabled: true },
        PiToolInfo { name: "write_file".to_string(), ... },
        // ... hardcoded list
    ]
}
```

**Problem**: Tool list is hardcoded, not from actual agent capabilities.

**Risk**: High - shows fake tool availability.

#### 🟡 Medium: Executing tools never populated

**Location**: `self.tools_state.executing` is never updated during "streaming".

```rust
// In ToolsState::default()
pub executing: Vec<ExecutingTool>,  // Always empty
```

**Problem**: Even during simulated response, no tools are shown as executing.

#### 🟡 Minor: No tool interaction

**Missing**:
- Enable/disable tools
- View tool details
- Cancel executing tool

### Edge Cases

| Case | Handled? |
|------|----------|
| No tools | ✅ Shows message |
| Many tools (scrolling) | 🟡 No explicit scroll |
| Long tool names | 🟡 Not truncated |
| Progress update | 🟡 Would work if data provided |

---

## 5. Input Panel (`src/ui/panels/input.rs`)

### Purpose
Message composition with cursor, mode indicator, vim state, and character count.

### Implementation Completeness: **90%** ✅

### What's Real ✅

| Feature | Status | Evidence |
|---------|--------|----------|
| Text input display | ✅ Real | `&self.input.text` from state |
| Cursor rendering | ✅ Real | Splits text at cursor position |
| Mode indicator | ✅ Real | NORMAL, INSERT, COMMAND, STREAMING, WAITING |
| Vim state indicator | ✅ Real | [vim:n], [vim:i], [vim:v] |
| Placeholder text | ✅ Real | "Type your message..." when empty |
| Character count | ✅ Real | Shows count on right side |
| Focus border | ✅ Real | Different border when focused |
| Prompt symbol | ✅ Real | "> " prefix |

### Issues Found

#### 🟢 Good: Cursor rendering is solid

```rust
let (before_cursor, at_cursor, after_cursor) = if cursor_pos < text.len() {
    let (before, rest) = text.split_at(cursor_pos);
    let mut chars = rest.chars();
    let cursor_char = chars.next().unwrap_or(' ');
    (before, cursor_char, chars.as_str())
} else {
    (text.as_str(), ' ', "")
};
```

Handles cursor at end of text correctly.

#### 🟡 Medium: Multi-line input not supported

**Problem**: Input is rendered as single `Paragraph` with one `Line`. No support for:
- Multi-line messages
- Newline insertion
- Vertical scrolling within input

**Risk**: Medium - limits message composition.

#### 🟡 Minor: Attachment indicator always shows

**Location**: `input.rs:143`

```rust
spans.push(Span::styled("📎", Style::default().fg(theme.muted())));
```

**Problem**: Paperclip icon always shows, but there's no attachment functionality.

#### 🟡 Minor: Unicode cursor position issues

**Location**: `input.rs:91`

```rust
let (before, rest) = text.split_at(cursor_pos);
```

**Problem**: `split_at` uses byte position, but `cursor` is likely intended as char position. Could panic on multi-byte UTF-8.

**Evidence from `app.rs:282-284`**:
```rust
Action::InsertChar(c) => {
    self.state.input.text.insert(self.state.input.cursor, c);
    self.state.input.cursor += 1;
}
```

`String::insert` uses byte index, but `cursor += 1` treats it as char index. This will break with emoji/unicode.

### Edge Cases

| Case | Handled? |
|------|----------|
| Empty input | ✅ Shows placeholder |
| Very long input | 🟡 Overflows, no horizontal scroll |
| Unicode characters | 🔴 Cursor math is broken |
| Paste multi-line | 🟡 Flattens to single line |
| Vim mode toggle | ✅ Works via `/vim` command |

---

## 6. Cross-Panel Analysis

### State Binding Summary

| Panel | State Source | Real Data? |
|-------|-------------|------------|
| Header | `StatusInfo` | 🟡 Partial (model/tokens simulated) |
| Conversation | `Conversation` | 🟡 Partial (messages work, responses simulated) |
| Sessions | `Vec<SessionInfo>` | 🔴 Always empty |
| Tools | `ToolsState` | 🔴 Hardcoded list |
| Input | `InputState` | ✅ Real |

### Event Flow Analysis

```
User Input → CrosstermEvent → map_key_to_action() → handle_action() → State Update → Render
```

| Event | Handler | Working? |
|-------|---------|----------|
| Key press | ✅ `handle_action()` | Yes |
| Paste | ✅ `CrosstermEvent::Paste` | Yes |
| Resize | ✅ Logged, ratatui handles | Yes |
| Mouse | 🔴 TODO comment | No |
| Scroll | ✅ `ScrollUp/Down` actions | Yes |

### Missing Integrations

1. **Sessions ↔ Conversation**: Switching sessions should load different conversations
2. **Tools ↔ Streaming**: Tool execution should update `executing` list
3. **Header ↔ Session**: Session name should update when switching
4. **Input ↔ History**: History navigation is TODO

---

## 7. Round 2 Findings Summary

### Panel Completeness Scores

| Panel | Score | Verdict |
|-------|-------|--------|
| Header | 95% | ✅ Production-ready with minor fixes |
| Conversation | 85% | ⚠️ Needs markdown/syntax highlighting |
| Sessions | 80% | 🔴 Non-functional (empty data) |
| Tools | 75% | 🔴 Shows fake data |
| Input | 90% | ⚠️ Unicode bug, no multi-line |

### 🔴 Critical Issues

1. **Sessions panel always empty** - No session management
2. **Tools panel shows hardcoded data** - Not from real agent
3. **Unicode cursor bug** - Will panic on emoji input
4. **No markdown rendering** - `pulldown-cmark` unused
5. **No syntax highlighting** - `syntect` unused

### 🟡 Medium Issues

1. **Multi-line input not supported**
2. **History navigation unimplemented**
3. **Mouse events not handled**
4. **No tool execution visualization**
5. **Session switching not connected**

### 🟢 Strengths

1. **Clean Widget implementations** - All panels use proper ratatui patterns
2. **Theme integration** - Consistent theming throughout
3. **Focus handling** - Panels show focus state correctly
4. **Empty states** - All panels handle empty data gracefully
5. **Scrolling** - Conversation scrolling works well
6. **Time formatting** - Relative timestamps are nice

---

## 8. Recommendations for Round 3

### Priority Fixes

1. **Fix Unicode cursor handling** - Use `char_indices()` instead of byte positions
2. **Implement session loading** - Connect `PiBridge.list_sessions()` to state
3. **Add markdown parsing** - Use `pulldown-cmark` in conversation panel
4. **Connect tool execution** - Update `executing` during streaming

### Suggested Next Rounds

- **Round 3**: Error Handling & Edge Cases
- **Round 4**: State Management & Data Flow
- **Round 5**: Theme System & Styling
- **Round 6**: Commands & Keybindings

---

*End of Round 2 Investigation*

---
---

# Round 3: Core Functionality Audit

**Date**: 2026-04-03  
**Reviewer**: Rusty 🦀  
**Status**: ✅ Complete

---

## 1. State Management (`src/core/state.rs`)

### Implementation Completeness: **90%** ✅

### Struct Field Usage Analysis

#### `AppState` - Main State Container

| Field | Type | Used? | Where |
|-------|------|-------|-------|
| `mode` | `AppMode` | ✅ Yes | `app.rs:170,196-322`, `events.rs:280` |
| `layout` | `LayoutMode` | ⚠️ Partial | Set in default, but `layout_manager` used instead |
| `active_panel` | `Panel` | ✅ Yes | `app.rs:243-259`, panel rendering |
| `conversation` | `Conversation` | ✅ Yes | Throughout `app.rs` |
| `sessions` | `Vec<SessionInfo>` | 🔴 Never populated | Always empty |
| `current_session_id` | `Option<Uuid>` | 🔴 Never set | Always `None` |
| `input` | `InputState` | ✅ Yes | Throughout `app.rs` |
| `status` | `StatusInfo` | ✅ Yes | `app.rs:79-82`, header panel |
| `tools` | `ToolsState` | ⚠️ Partial | Populated with fake data |
| `context` | `ContextState` | 🔴 Never used | Defined but not rendered |
| `should_quit` | `bool` | ✅ Yes | `app.rs:152,199` |
| `overlay` | `Option<Overlay>` | ✅ Yes | `app.rs:220-238`, overlay rendering |
| `streaming` | `StreamingState` | ✅ Yes | `app.rs:209-215,340-347` |

#### Dead/Unused State Fields

1. **`AppState.layout`** - Redundant with `LayoutManager.mode`
2. **`AppState.sessions`** - Never populated
3. **`AppState.current_session_id`** - Never set
4. **`AppState.context`** - Entire `ContextState` unused
5. **`InputState.history`** - Defined but history navigation is TODO
6. **`InputState.history_index`** - Defined but never used

#### `ContextState` - Completely Unused

```rust
pub struct ContextState {
    pub files: Vec<AttachedFile>,      // Never populated
    pub total_tokens: u64,             // Never used
    pub window_usage: f32,             // Never used
}

pub struct AttachedFile {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub tokens: u64,
}
```

**Verdict**: The entire context/attachment system is scaffolded but not implemented.

#### `Overlay` Variants

| Variant | Rendered? | Functional? |
|---------|-----------|-------------|
| `Help` | ✅ Yes | ✅ Yes |
| `CommandPalette` | 🔴 Placeholder | 🔴 No |
| `SessionPicker` | 🔴 Placeholder | 🔴 No |
| `ModelPicker` | 🔴 Placeholder | 🔴 No |
| `Confirm` | 🔴 Placeholder | 🔴 No |

**Evidence** (`app.rs:565-577`):
```rust
_ => {
    // Other overlays - placeholder
    let block = Block::default()
        .title(" Overlay ")
        .borders(Borders::ALL)
        // ... just an empty box
}
```

### State Initialization

| State | Initialization | Correct? |
|-------|---------------|----------|
| `mode` | `AppMode::Normal` | ✅ |
| `layout` | `LayoutMode::CommandCenter` | ✅ |
| `active_panel` | `Panel::Input` | ✅ |
| `conversation.auto_scroll` | `true` (via builder) | ✅ |
| `status.model` | `"claude-sonnet-4-20250514"` | ✅ |
| `status.connection` | `Disconnected` → `Connected` | ✅ |
| `tools.available` | Hardcoded list | ⚠️ Fake |

---

## 2. Event System (`src/core/events.rs`)

### Implementation Completeness: **75%** ⚠️

### Event Enum Analysis

#### `Event` - Top Level

| Variant | Handled? | Where |
|---------|----------|-------|
| `Terminal(TerminalEvent)` | ✅ Yes | `app.rs:169-189` |
| `Agent(AgentEvent)` | 🔴 No | Not used anywhere |
| `Internal(InternalEvent)` | 🔴 No | Not used anywhere |
| `Tick` | ⚠️ Implicit | `app.rs:156` calls `tick()` |

**Critical**: `AgentEvent` and `InternalEvent` are **never processed**!

#### `TerminalEvent` Variants

| Variant | Handled? | Where |
|---------|----------|-------|
| `Key(KeyEvent)` | ✅ Yes | `app.rs:170-173` |
| `Mouse(MouseEvent)` | 🔴 TODO | `app.rs:178-180` |
| `Resize` | ✅ Logged | `app.rs:175-177` |
| `FocusGained` | 🔴 No | Not handled |
| `FocusLost` | 🔴 No | Not handled |
| `Paste(String)` | ✅ Yes | `app.rs:181-186` |

#### `AgentEvent` Variants - NONE HANDLED!

```rust
pub enum AgentEvent {
    Connected,              // 🔴 Not handled
    Disconnected,           // 🔴 Not handled
    StreamStart,            // 🔴 Not handled
    TextDelta,              // 🔴 Not handled
    ThinkingDelta,          // 🔴 Not handled
    ThinkingStart,          // 🔴 Not handled
    ThinkingEnd,            // 🔴 Not handled
    ToolStart,              // 🔴 Not handled
    ToolProgress,           // 🔴 Not handled
    ToolEnd,                // 🔴 Not handled
    StreamEnd,              // 🔴 Not handled
    Usage,                  // 🔴 Not handled
    Error,                  // 🔴 Not handled
}
```

**Problem**: These events are beautifully designed but the event loop never receives them. The `PiBridge` doesn't emit events - it's a stub.

#### `InternalEvent` Variants - NONE HANDLED!

```rust
pub enum InternalEvent {
    SessionLoaded,          // 🔴 Not handled
    SessionCreated,         // 🔴 Not handled
    SessionsUpdated,        // 🔴 Not handled
    ThemeChanged,           // 🔴 Not handled
    LayoutChanged,          // 🔴 Not handled
    Notification,           // 🔴 Not handled
    CommandExecuted,        // 🔴 Not handled
    FileAttached,           // 🔴 Not handled
    FileDetached,           // 🔴 Not handled
}
```

### Action Handling Analysis

#### Actions Defined in `events.rs`

| Action | Defined | Handled in `app.rs`? |
|--------|---------|---------------------|
| `Quit` | ✅ | ✅ Yes |
| `SendMessage` | ✅ | ✅ Yes |
| `Cancel` | ✅ | ✅ Yes |
| `ClearScreen` | ✅ | ✅ Yes |
| `ToggleHelp` | ✅ | ✅ Yes |
| `OpenCommandPalette` | ✅ | ✅ Yes (opens empty overlay) |
| `CloseOverlay` | ✅ | ✅ Yes |
| `FocusPanel(Panel)` | ✅ | 🔴 Not handled |
| `NextPanel` | ✅ | ✅ Yes |
| `PrevPanel` | ✅ | ✅ Yes |
| `ScrollUp(u16)` | ✅ | ✅ Yes |
| `ScrollDown(u16)` | ✅ | ✅ Yes |
| `ScrollToTop` | ✅ | ✅ Yes |
| `ScrollToBottom` | ✅ | ✅ Yes |
| `ToggleLayout` | ✅ | ✅ Yes |
| `OpenSessionPicker` | ✅ | ✅ Yes (opens empty overlay) |
| `OpenModelPicker` | ✅ | 🔴 Not handled |
| `NewSession` | ✅ | 🔴 Not handled |
| `ContinueSession` | ✅ | 🔴 Not handled |
| `Copy` | ✅ | 🔴 Not handled |
| `Paste` | ✅ | 🔴 Not handled |
| `Undo` | ✅ | 🔴 Not handled |
| `Redo` | ✅ | 🔴 Not handled |
| `InsertChar(char)` | ✅ | ✅ Yes |
| `DeleteChar` | ✅ | ✅ Yes |
| `Backspace` | ✅ | ✅ Yes |
| `CursorLeft` | ✅ | ✅ Yes |
| `CursorRight` | ✅ | ✅ Yes |
| `CursorHome` | ✅ | ✅ Yes |
| `CursorEnd` | ✅ | ✅ Yes |
| `HistoryPrev` | ✅ | ⚠️ TODO |
| `HistoryNext` | ✅ | ⚠️ TODO |
| `ToggleVimMode` | ✅ | 🔴 Not handled |
| `ExecuteCommand(String)` | ✅ | 🔴 Not handled |
| `AttachFile(String)` | ✅ | 🔴 Not handled |
| `None` | ✅ | ✅ Implicit (catch-all) |

**Summary**: 21 handled, 15 unhandled actions.

---

## 3. Keybindings (`src/core/keybindings.rs`)

### Implementation Completeness: **60%** ⚠️

### Default Bindings vs Actual Behavior

#### Global Bindings

| Binding | Defined Action | Actually Works? |
|---------|---------------|----------------|
| `ctrl+c` | `quit` | ✅ Yes |
| `ctrl+q` | `quit` | ✅ Yes |
| `ctrl+l` | `clear` | ✅ Yes |
| `ctrl+p` | `command_palette` | ⚠️ Opens empty overlay |
| `ctrl+s` | `session_picker` | ⚠️ Opens empty overlay |
| `ctrl+h` | `help` | ✅ Yes |
| `ctrl+enter` | `send` | ✅ Yes |
| `f1` | `help` | ✅ Yes |
| `f2` | `toggle_layout` | ✅ Yes |

#### Normal Mode Bindings

| Binding | Defined Action | Actually Works? |
|---------|---------------|----------------|
| `i` | `insert_mode` | 🔴 Action not handled |
| `:` | `command_mode` | 🔴 Action not handled |
| `/` | `search` | 🔴 Action not defined |
| `j` | `scroll_down` | 🔴 Not mapped in `map_key_to_action` |
| `k` | `scroll_up` | 🔴 Not mapped in `map_key_to_action` |
| `g` | `scroll_top` | 🔴 Not mapped in `map_key_to_action` |
| `G` | `scroll_bottom` | 🔴 Not mapped in `map_key_to_action` |
| `tab` | `next_panel` | ✅ Yes |
| `shift+tab` | `prev_panel` | ✅ Yes |

### Critical Issue: Keybindings Not Used!

**The `KeyBindings` struct is defined but NEVER USED in the application!**

```rust
// keybindings.rs defines:
pub struct KeyBindings { ... }

// But app.rs uses hardcoded:
fn map_key_to_action(key: KeyEvent, mode: AppMode, ...) -> Action {
    // Direct KeyCode matching, ignores KeyBindings entirely!
}
```

**Evidence**: Search for `KeyBindings` usage:
- Defined in `keybindings.rs`
- Exported in `core/mod.rs`
- **Never imported or used in `app.rs`**

### Vim Mode Bindings

The vim mode (`j`, `k`, `g`, `G`, `i`, `:`) bindings are defined in `KeyBindings` but:
1. `KeyBindings` is never loaded
2. `map_key_to_action` doesn't check vim mode
3. `VimState` in `InputState` is set but never affects key handling

---

## 4. Slash Commands (`src/core/commands.rs`)

### Implementation Completeness: **35%** 🔴

### Command Definition vs Implementation

| Command | Defined | Parsed | Executed | Effect |
|---------|---------|--------|----------|--------|
| `/help` | ✅ | ✅ | ✅ | Opens help overlay |
| `/clear` | ✅ | ✅ | ✅ | Clears messages |
| `/exit` | ✅ | ✅ | ✅ | Quits app |
| `/quit` | ✅ | ✅ | ✅ | Quits app |
| `/model` | ✅ | ✅ | ⚠️ | Sets model string only |
| `/provider` | ✅ | ✅ | 🔴 | Not implemented |
| `/session` | ✅ | ✅ | 🔴 | Not implemented |
| `/theme` | ✅ | ✅ | 🔴 | Not implemented |
| `/layout` | ✅ | ✅ | ✅ | Changes layout |
| `/export` | ✅ | ✅ | 🔴 | Not implemented |
| `/context` | ✅ | ✅ | 🔴 | Not implemented |
| `/tools` | ✅ | ✅ | 🔴 | Not implemented |
| `/system` | ✅ | ✅ | 🔴 | Not implemented |
| `/branch` | ✅ | ✅ | 🔴 | Not implemented |
| `/compact` | ✅ | ✅ | 🔴 | Not implemented |
| `/tokens` | ✅ | ✅ | 🔴 | Not implemented |
| `/cost` | ✅ | ✅ | 🔴 | Not implemented |
| `/vim` | ✅ | ✅ | ✅ | Toggles vim_mode flag |

**Summary**: 17 commands defined, only 6 actually do something.

### Command Execution Code (`app.rs:351-396`)

```rust
match cmd.name {
    "help" => { self.state.overlay = Some(Overlay::Help); }
    "clear" => { self.state.conversation.messages.clear(); }
    "exit" | "quit" => { self.state.should_quit = true; }
    "model" => {
        if let Some(model) = parsed.args.first() {
            self.bridge.set_model(model);  // Just sets a string
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
```

### `/model` Command - Partial Implementation

```rust
"model" => {
    if let Some(model) = parsed.args.first() {
        self.bridge.set_model(model);      // Sets PiBridge.model string
        self.state.status.model = model.clone();  // Updates status bar
    }
}
```

**Issues**:
1. No validation of model name
2. No reconnection to agent with new model
3. No feedback to user
4. Doesn't actually change the AI model (PiBridge is stubbed)

### `/vim` Command - Toggles Flag But No Effect

```rust
"vim" => {
    self.state.input.vim_mode = !self.state.input.vim_mode;
}
```

**Issues**:
1. Sets `vim_mode = true` but key handling ignores it
2. `VimState` is never transitioned
3. Vim keybindings (`j`, `k`, `i`, `:`) don't work

---

## 5. Pi Bridge (`src/providers/pi_bridge.rs`)

### Implementation Completeness: **10%** 🔴

### TODO Inventory

| Line | TODO | Severity | Description |
|------|------|----------|-------------|
| 23 | Uncomment pi_agent_rust import | 🔴 Critical | No dependency |
| 42 | Add actual pi integration | 🔴 Critical | No agent field |
| 59 | Initialize pi agent | 🔴 Critical | `connect()` does nothing |
| 108 | Send message through pi agent | 🔴 Critical | No actual sending |
| 115 | Placeholder simulate response | 🔴 Critical | Hardcoded response |
| 123 | Cancel pi agent operation | 🔴 Critical | Just logs |
| 129 | Load session from storage | 🟡 Medium | Returns `Ok(())` |
| 135 | Create session through pi | 🟡 Medium | Returns random UUID |
| 141 | List sessions from storage | 🟡 Medium | Returns empty vec |
| 153 | Get tools from registry | 🟡 Medium | Returns hardcoded list |
| 180 | Translate pi events | 🟡 Medium | Commented out function |

### What PiBridge Actually Does

```rust
impl PiBridge {
    pub fn new(...) -> Self { /* stores model/provider strings */ }
    pub async fn connect(&mut self) -> Result<()> { self.connected = true; Ok(()) }
    pub const fn is_connected(&self) -> bool { self.connected }
    pub fn model(&self) -> &str { &self.model }
    pub fn set_model(&mut self, model: impl Into<String>) { self.model = model.into(); }
    pub fn provider(&self) -> &str { &self.provider }
    pub fn set_provider(&mut self, provider: impl Into<String>) { self.provider = provider.into(); }
    pub async fn send_message(&mut self, message: &str, _on_event: impl Fn(AgentEvent)) -> Result<()> {
        tracing::info!("Sending message: {message}");
        Ok(())  // Does nothing!
    }
    pub fn cancel(&mut self) { tracing::info!("Cancelling operation"); }
    pub async fn load_session(&mut self, _session_id: &str) -> Result<()> { Ok(()) }
    pub async fn create_session(&mut self, _name: &str) -> Result<String> {
        Ok(uuid::Uuid::new_v4().to_string())
    }
    pub async fn list_sessions(&self) -> Result<Vec<SessionSummary>> { Ok(Vec::new()) }
    pub fn available_tools(&self) -> Vec<ToolSummary> { /* hardcoded list */ }
}
```

### Minimum Viable Implementation

To make HawkTUI functional, `PiBridge` needs:

1. **Add `pi_agent_rust` dependency** to `Cargo.toml`
2. **Store actual `Agent` instance**:
   ```rust
   pub struct PiBridge {
       agent: Option<pi::agent::Agent>,
       // ...
   }
   ```
3. **Implement `connect()`**:
   ```rust
   pub async fn connect(&mut self) -> Result<()> {
       let config = AgentConfig { model: self.model.clone(), ... };
       self.agent = Some(Agent::new(config).await?);
       self.connected = true;
       Ok(())
   }
   ```
4. **Implement `send_message()` with streaming**:
   ```rust
   pub async fn send_message(&mut self, message: &str, on_event: impl Fn(AgentEvent)) -> Result<()> {
       let agent = self.agent.as_mut().ok_or(Error::agent("Not connected"))?;
       agent.send(message, |event| {
           on_event(translate_event(event));
       }).await
   }
   ```
5. **Implement event translation**:
   ```rust
   fn translate_event(event: PiAgentEvent) -> AgentEvent {
       match event {
           PiAgentEvent::TextDelta(text) => AgentEvent::TextDelta { text },
           // ...
       }
   }
   ```
6. **Wire events into `App` event loop**

---

## 6. Round 3 Summary

### Module Completeness Scores

| Module | Score | Verdict |
|--------|-------|--------|
| `state.rs` | 90% | ✅ Well-designed, some dead fields |
| `events.rs` | 75% | ⚠️ Events defined but not processed |
| `keybindings.rs` | 60% | ⚠️ Defined but not used |
| `commands.rs` | 35% | 🔴 Most commands unimplemented |
| `pi_bridge.rs` | 10% | 🔴 Complete stub |

### Critical Findings

1. **`AgentEvent` and `InternalEvent` are never processed** - The event system is designed but not wired up

2. **`KeyBindings` is never used** - Vim mode and custom keybindings are dead code

3. **15 Actions are unhandled** including:
   - `FocusPanel`, `OpenModelPicker`, `NewSession`, `ContinueSession`
   - `Copy`, `Paste`, `Undo`, `Redo`
   - `ToggleVimMode`, `ExecuteCommand`, `AttachFile`

4. **11 of 17 slash commands do nothing** - They parse but have no implementation

5. **`PiBridge` is 90% TODO comments** - No actual AI integration

### Dead Code Summary

| Item | Location | Type |
|------|----------|------|
| `AppState.layout` | `state.rs:17` | Redundant field |
| `AppState.sessions` | `state.rs:23` | Never populated |
| `AppState.current_session_id` | `state.rs:26` | Never set |
| `AppState.context` | `state.rs:35` | Entire struct unused |
| `InputState.history` | `state.rs:226` | Never used |
| `InputState.history_index` | `state.rs:229` | Never used |
| `ContextState` | `state.rs:283-292` | Entire struct unused |
| `AttachedFile` | `state.rs:295-301` | Entire struct unused |
| `Overlay::CommandPalette` | `state.rs:305` | Placeholder render |
| `Overlay::SessionPicker` | `state.rs:307` | Placeholder render |
| `Overlay::ModelPicker` | `state.rs:309` | Placeholder render |
| `Overlay::Confirm` | `state.rs:311` | Placeholder render |
| `AgentEvent` variants | `events.rs:43-84` | Never processed |
| `InternalEvent` variants | `events.rs:97-120` | Never processed |
| `KeyBindings` struct | `keybindings.rs:10-24` | Never loaded |
| All vim bindings | `keybindings.rs:51-61` | Never used |

### Recommendations for Next Rounds

1. **Round 4**: Focus on wiring up the event system
2. **Round 5**: Implement remaining slash commands
3. **Round 6**: Add `pi_agent_rust` integration
4. **Round 7**: Remove dead code or implement missing features

---

*End of Round 3 Investigation*
