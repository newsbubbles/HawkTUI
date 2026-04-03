# HawkTUI Design Document

> **"See everything. Control everything. Code like a hawk."**

## Vision

HawkTUI is a **premium terminal user interface** that wraps `pi_agent_rust`, transforming the already-powerful AI coding agent into an immersive, visually stunning experience. Think of it as the cockpit of a fighter jet - every piece of information at your fingertips, beautifully organized, instantly accessible.

## Core Philosophy

### 🦅 The Hawk Metaphor
- **Sharp vision**: See your entire conversation, context, and tools at a glance
- **Swift action**: Keyboard-driven workflow with zero friction
- **Precision**: Surgical control over AI interactions
- **Altitude**: Bird's-eye view of sessions, branches, and history

### Design Principles
1. **Zero-compromise performance**: Match or exceed pi_agent_rust's speed
2. **Information density**: Maximum signal, minimum noise
3. **Keyboard-first**: Every action accessible without mouse
4. **Beautiful by default**: Stunning out of the box, customizable for power users
5. **Rust-native**: No subprocess wrapping - direct library integration

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              HawkTUI                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐  │
│  │   UI Layer      │  │   Core Layer    │  │   Integration Layer         │  │
│  │                 │  │                 │  │                             │  │
│  │  • Panels       │  │  • State Mgmt   │  │  • pi_agent_rust (lib)      │  │
│  │  • Widgets      │  │  • Event Loop   │  │  • Provider Adapters        │  │
│  │  • Themes       │  │  • Keybindings  │  │  • Session Bridge           │  │
│  │  • Animations   │  │  • Commands     │  │  • Tool Execution           │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

## UI Layout Concepts

### Default Layout: "Command Center"

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ 🦅 HawkTUI v0.1.0 │ claude-sonnet-4-20250514 │ tokens: 12.4k │ $0.042 │ ⚡ streaming │
├────────────────────────────┬─────────────────────────────────────────────────┤
│                            │                                                 │
│  📁 Sessions               │  💬 Conversation                                │
│  ├─ refactor-auth (active) │                                                 │
│  ├─ debug-parser           │  You: Can you help me optimize this function?  │
│  ├─ new-feature-xyz        │                                                 │
│  └─ archived/              │  🤖 Assistant:                                  │
│     ├─ old-session-1       │  I'd be happy to help! Let me analyze the      │
│     └─ old-session-2       │  function and suggest optimizations...         │
│                            │                                                 │
│  🔧 Tools                  │  ```rust                                       │
│  ├─ read_file ✓            │  fn optimized_function() {                     │
│  ├─ write_file ✓           │      // Using iterators instead of loops       │
│  ├─ bash ⚠                 │      items.iter()                              │
│  └─ glob ✓                 │          .filter(|x| x.is_valid())             │
│                            │          .collect()                            │
│  📊 Context                │  }                                             │
│  └─ 3 files attached       │  ```                                           │
│                            │                                                 │
├────────────────────────────┴─────────────────────────────────────────────────┤
│ > Type your message... (Ctrl+Enter to send, /help for commands)    [vim] 📎 │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Alternate Layout: "Focus Mode"

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ 🦅 HawkTUI │ Focus Mode │ Session: refactor-auth                     [F1] ? │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│                           💬 Conversation                                    │
│                                                                              │
│   You ─────────────────────────────────────────────────── 2 minutes ago     │
│   Can you help me optimize this function?                                    │
│                                                                              │
│   ┌─ @src/parser.rs (lines 42-67)                                           │
│   │  fn parse_token(&mut self) -> Result<Token> { ... }                     │
│   └──────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   🤖 Assistant ──────────────────────────────────────────── just now        │
│   I'd be happy to help optimize this function! Here's my analysis:          │
│                                                                              │
│   **Performance Issues:**                                                    │
│   1. Unnecessary allocations in the loop                                     │
│   2. Could use `&str` instead of `String`                                    │
│                                                                              │
│   **Suggested Optimization:**                                                │
│   ▌                                                                          │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ > ▌                                                        [tokens: 2.1k] 📎│
└──────────────────────────────────────────────────────────────────────────────┘
```

### Split Layout: "Code Review"

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ 🦅 HawkTUI │ Code Review Mode │ Diff: src/main.rs                    [F2] ? │
├──────────────────────────────────┬───────────────────────────────────────────┤
│  📄 Original                     │  📝 Suggested                             │
│                                  │                                           │
│  fn main() {                     │  fn main() -> Result<()> {                │
│      let config = Config::new(); │      let config = Config::load()?;       │
│      let app = App::new(config); │      let app = App::builder()            │
│      app.run();                  │          .config(config)                  │
│  }                               │          .build()?;                       │
│                                  │      app.run()?;                          │
│                                  │      Ok(())                               │
│                                  │  }                                        │
├──────────────────────────────────┴───────────────────────────────────────────┤
│ 🤖 The suggested version adds proper error handling using Result<()>...      │
├──────────────────────────────────────────────────────────────────────────────┤
│ [a]ccept  [r]eject  [e]dit  [n]ext  [p]rev  [q]uit                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Key Features

### 1. **Streaming Visualization**
- Real-time token-by-token rendering with smooth animations
- Thinking indicator with expandable thought process
- Progress bars for tool execution
- Syntax highlighting as code streams in

### 2. **Session Management**
- Visual session tree with branches
- Quick session switching (Ctrl+1-9)
- Session search and filtering
- Branch visualization and navigation
- Session export/import

### 3. **Context Awareness**
- File attachment preview with syntax highlighting
- Token count visualization
- Cost estimation in real-time
- Context window usage indicator

### 4. **Tool Execution Dashboard**
- Live tool execution status
- Collapsible tool output
- Tool approval workflow (for dangerous operations)
- Execution history

### 5. **Themes & Customization**
- Multiple built-in themes (Hawk Dark, Hawk Light, Cyberpunk, etc.)
- Custom theme support via TOML
- Per-panel color schemes
- Font/unicode configuration

### 6. **Keyboard Shortcuts**
| Key | Action |
|-----|--------|
| `Ctrl+Enter` | Send message |
| `Ctrl+C` | Cancel/Abort |
| `Ctrl+L` | Clear screen |
| `Ctrl+P` | Command palette |
| `Ctrl+S` | Quick session switch |
| `Ctrl+T` | Toggle tools panel |
| `Ctrl+H` | Toggle history panel |
| `F1` | Help |
| `F2` | Toggle layout |
| `/` | Slash commands |
| `Esc` | Cancel/Close overlay |

## Technical Architecture

### Crate Structure

```
hawktui/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library exports
│   ├── app.rs               # Main application state
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── layout.rs        # Layout management
│   │   ├── panels/
│   │   │   ├── mod.rs
│   │   │   ├── conversation.rs
│   │   │   ├── sessions.rs
│   │   │   ├── tools.rs
│   │   │   ├── context.rs
│   │   │   └── input.rs
│   │   ├── widgets/
│   │   │   ├── mod.rs
│   │   │   ├── markdown.rs
│   │   │   ├── code_block.rs
│   │   │   ├── streaming.rs
│   │   │   └── status_bar.rs
│   │   └── themes/
│   │       ├── mod.rs
│   │       └── builtin.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── state.rs         # Application state
│   │   ├── events.rs        # Event handling
│   │   ├── commands.rs      # Slash commands
│   │   └── keybindings.rs   # Keyboard shortcuts
│   └── providers/
│       ├── mod.rs
│       └── pi_bridge.rs     # pi_agent_rust integration
├── themes/
│   ├── hawk_dark.toml
│   ├── hawk_light.toml
│   └── cyberpunk.toml
└── tests/
```

### Dependencies Strategy

**Core TUI Framework:**
- `ratatui` - Modern TUI framework (fork of tui-rs)
- `crossterm` - Cross-platform terminal manipulation

**From pi_agent_rust (as library):**
- `pi` crate - Direct library integration
- Reuse: Agent, Session, Provider, Tools, Models

**Additional:**
- `tokio` - Async runtime (for streaming)
- `syntect` - Syntax highlighting
- `pulldown-cmark` - Markdown parsing
- `unicode-width` - Proper text width calculation
- `directories` - XDG paths

### Integration Approach

Instead of wrapping `pi` as a subprocess, HawkTUI will:

1. **Import `pi` as a library crate**
   ```rust
   use pi::agent::{Agent, AgentConfig, AgentEvent};
   use pi::session::Session;
   use pi::providers;
   ```

2. **Subscribe to AgentEvents**
   ```rust
   agent.on_event(|event| match event {
       AgentEvent::TextDelta(text) => ui.stream_text(text),
       AgentEvent::ThinkingDelta(text) => ui.stream_thinking(text),
       AgentEvent::ToolStart { name, input } => ui.show_tool_start(name, input),
       AgentEvent::ToolEnd { name, output } => ui.show_tool_result(name, output),
       // ...
   });
   ```

3. **Share Session Storage**
   - Use the same SQLite session format
   - Sessions created in `pi` CLI visible in HawkTUI and vice versa

## Implementation Phases

### Phase 1: Foundation (MVP)
- [x] Project setup with Cargo.toml
- [ ] Basic TUI with ratatui
- [ ] Single conversation panel
- [ ] Text input with basic editing
- [ ] Integration with pi agent (send/receive)
- [ ] Basic streaming display

### Phase 2: Core Features
- [ ] Multi-panel layout
- [ ] Session list panel
- [ ] Tool execution panel
- [ ] Syntax highlighting for code
- [ ] Markdown rendering
- [ ] Basic theming

### Phase 3: Polish
- [ ] Smooth animations
- [ ] Multiple layout modes
- [ ] Command palette
- [ ] Advanced keybindings
- [ ] Theme customization
- [ ] Session branching UI

### Phase 4: Advanced
- [ ] Code review mode
- [ ] Diff visualization
- [ ] File browser integration
- [ ] Plugin system
- [ ] Remote session support

## Performance Targets

| Metric | Target |
|--------|--------|
| Startup time | < 50ms |
| Input latency | < 16ms (60fps) |
| Memory (idle) | < 30MB |
| Streaming render | 60fps smooth |
| Session load (1MB) | < 100ms |

## Theme System

```toml
# themes/hawk_dark.toml
[meta]
name = "Hawk Dark"
author = "HawkTUI Team"
version = "1.0.0"

[colors]
background = "#0d1117"
foreground = "#c9d1d9"
accent = "#58a6ff"
success = "#3fb950"
warning = "#d29922"
error = "#f85149"

[colors.syntax]
keyword = "#ff7b72"
string = "#a5d6ff"
comment = "#8b949e"
function = "#d2a8ff"
type = "#79c0ff"

[panels]
conversation_bg = "#0d1117"
sidebar_bg = "#161b22"
input_bg = "#21262d"
status_bg = "#30363d"

[borders]
style = "rounded"  # rounded, sharp, double, thick
color = "#30363d"
focused_color = "#58a6ff"
```

## Next Steps

1. Create `Cargo.toml` with dependencies
2. Implement basic app skeleton
3. Create main event loop
4. Build conversation panel
5. Integrate with pi_agent_rust library
6. Add streaming support
7. Iterate on UI/UX

---

*"From above, everything is clear."* 🦅
