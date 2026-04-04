# Implementation Plan: Extension Manager UI

**Date**: 2026-04-03  
**Status**: Ready for Implementation  
**Priority**: Phase 4 (Revised)  
**Estimated LoC**: 300-500  

---

## Overview

Build a TUI panel for managing **extensions** in HawkTUI, leveraging pi_agent_rust's `ExtensionManager` API.

### Goals

- ✅ List all loaded extensions
- ✅ Show tools from each extension
- ✅ Show commands from each extension
- ✅ Show extension count and status
- ✅ Provide visibility into what extensions provide

### Non-Goals

- Direct MCP server management (extensions handle this)
- Adding/removing extensions at runtime (Phase 4.1+)
- Extension configuration editing

---

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        HawkTUI Architecture                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────────┐    ┌──────────────┐   │
│  │  AppState    │───▶│  ExtensionState  │◀───│  PiBridge    │   │
│  │              │    │                  │    │              │   │
│  │ - extensions │    │ - count          │    │ - handle     │   │
│  └──────────────┘    │ - tools          │    │              │   │
│         │            │ - commands       │    │ Ext Methods: │   │
│         │            │ - selected       │    │ - list_ext() │   │
│         ▼            └──────────────────┘    │ - list_tools()│   │
│  ┌──────────────┐             │              └──────────────┘   │
│  │   AppMode    │             │                    ▲            │
│  │              │             │                    │            │
│  │ - Chat       │             │                    │            │
│  │ - Extensions │◀────────────┘                    │            │
│  │ - ...        │                                  │            │
│  └──────────────┘                                  │            │
│         │                                          │            │
│         ▼                                          │            │
│  ┌──────────────────┐                              │            │
│  │ ExtensionPanel   │──────────────────────────────┘            │
│  │                  │                                           │
│  │ Renders ext     │                                           │
│  │ list + tools    │                                           │
│  └──────────────────┘                                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### File Structure

```
src/
├── core/
│   ├── state.rs          # Add ExtensionState
│   ├── events.rs         # Add ExtensionEvent variants
│   └── mode.rs           # Add Extensions mode
├── providers/
│   └── pi_bridge.rs      # Add extension methods
├── ui/
│   ├── panels/
│   │   └── extension_panel.rs  # NEW: Extension list panel
│   └── mod.rs            # Update exports
└── app.rs                # Add Extensions mode handling
```

---

## Implementation Steps

### Phase 1: Core Infrastructure (50-75 LoC)

#### Step 1.1: Add Extension Types

**File**: `src/core/state.rs`

```rust
/// Extension management state.
#[derive(Debug, Default)]
pub struct ExtensionState {
    /// Number of loaded extensions.
    pub count: usize,
    
    /// All tools from extensions.
    pub tools: Vec<ExtensionTool>,
    
    /// All commands from extensions.
    pub commands: Vec<ExtensionCommand>,
    
    /// Currently selected item index.
    pub selected_index: Option<usize>,
    
    /// Scroll offset for long lists.
    pub scroll_offset: usize,
    
    /// Loading state.
    pub loading: bool,
    
    /// Error message (if any).
    pub error: Option<String>,
}

/// Information about an extension tool.
#[derive(Debug, Clone)]
pub struct ExtensionTool {
    /// Tool name.
    pub name: String,
    
    /// Tool description.
    pub description: String,
    
    /// Source extension (if known).
    pub source: Option<String>,
}

/// Information about an extension command.
#[derive(Debug, Clone)]
pub struct ExtensionCommand {
    /// Command name.
    pub name: String,
    
    /// Command description.
    pub description: String,
}

impl ExtensionState {
    /// Create new extension state.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Select next item.
    pub fn select_next(&mut self) {
        let total = self.tools.len() + self.commands.len();
        if total == 0 {
            self.selected_index = None;
        } else {
            let next = self.selected_index
                .map(|i| (i + 1) % total)
                .unwrap_or(0);
            self.selected_index = Some(next);
        }
    }
    
    /// Select previous item.
    pub fn select_prev(&mut self) {
        let total = self.tools.len() + self.commands.len();
        if total == 0 {
            self.selected_index = None;
        } else {
            let prev = self.selected_index
                .map(|i| i.saturating_sub(1))
                .unwrap_or(total - 1);
            self.selected_index = Some(prev);
        }
    }
}
```

**Add to AppState**:
```rust
pub struct AppState {
    // ... existing fields
    
    /// Extension management state.
    pub extensions: ExtensionState,
}
```

#### Step 1.2: Add Extension Events

**File**: `src/core/events.rs`

```rust
/// Extension-related events.
pub enum ExtensionEvent {
    /// Refresh extension data.
    Refresh,
    
    /// Extension data updated.
    Updated {
        count: usize,
        tools: Vec<ExtensionTool>,
        commands: Vec<ExtensionCommand>,
    },
    
    /// Error occurred.
    Error(String),
}
```

#### Step 1.3: Add Extensions Mode

**File**: `src/core/mode.rs`

```rust
pub enum AppMode {
    Chat,
    ToolReview,
    Help,
    Extensions,  // NEW
}

impl AppMode {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Chat => "Chat",
            Self::ToolReview => "Tool Review",
            Self::Help => "Help",
            Self::Extensions => "Extensions",
        }
    }
}
```

---

### Phase 2: PiBridge Extension Integration (75-100 LoC)

#### Step 2.1: Add Extension Methods to PiBridge

**File**: `src/providers/pi_bridge.rs`

```rust
use pi::sdk::ExtensionManager;

impl PiBridge {
    /// Refresh extension data from the agent session.
    pub async fn refresh_extensions(&self) -> Result<ExtensionData> {
        let handle = self.handle.as_ref()
            .ok_or_else(|| Error::NotConnected)?;
        
        // Get extension manager
        let manager = handle.extension_manager()
            .ok_or_else(|| Error::ExtensionsNotLoaded)?;
        
        // Get count
        let count = manager.extension_count();
        
        // Get tools
        let tools = manager.list_tools()
            .into_iter()
            .map(|t| ExtensionTool {
                name: t.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                description: t.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                source: None, // pi doesn't expose this directly
            })
            .collect();
        
        // Get commands
        let commands = manager.list_commands()
            .into_iter()
            .map(|c| ExtensionCommand {
                name: c.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                description: c.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect();
        
        
        Ok(ExtensionData {
            count,
            tools,
            commands,
        })
    }
    
    /// Check if extensions are loaded.
    pub fn has_extensions(&self) -> bool {
        self.handle
            .as_ref()
            .map(|h| h.has_extensions())
            .unwrap_or(false)
    }
}

/// Data about extensions.
#[derive(Debug, Clone)]
pub struct ExtensionData {
    pub count: usize,
    pub tools: Vec<ExtensionTool>,
    pub commands: Vec<ExtensionCommand>,
}
```

---

### Phase 3: Extension Panel UI (150-200 LoC)

#### Step 3.1: Create Extension Panel

**File**: `src/ui/panels/extension_panel.rs`

```rust
//! Extension Manager Panel.
//!
//! Displays list of loaded extensions with their tools and commands.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::core::state::{ExtensionState, ExtensionTool, ExtensionCommand};

/// Render the extension manager panel.
pub fn render_extension_panel(f: &mut Frame, area: Rect, state: &ExtensionState) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Tools
            Constraint::Length(3),  // Footer
        ])
        .split(area);
    
    // Render header
    render_header(f, chunks[0], state);
    
    // Render tools/commands list
    render_items_list(f, chunks[1], state);
    
    // Render footer
    render_footer(f, chunks[2]);
}

/// Render header with title and stats.
fn render_header(f: &mut Frame, area: Rect, state: &ExtensionState) {
    let title = Line::from(vec![
        Span::styled(" Extensions ", Style::default().fg(Color::Cyan).bold()),
        Span::raw("│"),
        Span::styled(
            format!(" {} loaded ", state.count),
            Style::default().fg(Color::Green)
        ),
        Span::raw("│"),
        Span::styled(
            format!(" {} tools ", state.tools.len()),
            Style::default().fg(Color::Blue)
        ),
        Span::raw("│"),
        Span::styled(
            format!(" {} commands ", state.commands.len()),
            Style::default().fg(Color::Yellow)
        ),
    ]);
    
    let header = Paragraph::new(title)
        .block(Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray)));
    
    f.render_widget(header, area);
}

/// Render tools and commands list.
fn render_items_list(f: &mut Frame, area: Rect, state: &ExtensionState) {
    if state.tools.is_empty() && state.commands.is_empty() {
        render_empty_state(f, area);
        return;
    }
    
    // Build combined list: tools first, then commands
    let mut items: Vec<ListItem> = Vec::new();
    
    // Add tools section header
    if !state.tools.is_empty() {
        items.push(ListItem::new(
            Line::styled("── Tools ──", Style::default().fg(Color::Blue))
        ));
        
        for (i, tool) in state.tools.iter().enumerate() {
            let selected = state.selected_index == Some(i);
            items.push(render_tool_item(tool, selected));
        }
    }
    
    // Add commands section header
    if !state.commands.is_empty() {
        let offset = state.tools.len();
        items.push(ListItem::new(
            Line::styled("── Commands ──", Style::default().fg(Color::Yellow))
        ));
        
        for (i, cmd) in state.commands.iter().enumerate() {
            let selected = state.selected_index == Some(offset + i);
            items.push(render_command_item(cmd, selected));
        }
    }
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Items "));
    
    f.render_widget(list, area);
}

/// Render a single tool item.
fn render_tool_item(tool: &ExtensionTool, selected: bool) -> ListItem {
    let style = if selected {
        Style::default()
            .fg(Color::Green)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let content = Line::from(vec![
        Span::styled("  🔧 ", style),
        Span::styled(&tool.name, style),
        Span::raw("  "),
        Span::styled(
            &tool.description,
            Style::default().fg(Color::DarkGray)
        ),
    ]);
    
    ListItem::new(content)
}

/// Render a single command item.
fn render_command_item(cmd: &ExtensionCommand, selected: bool) -> ListItem {
    let style = if selected {
        Style::default()
            .fg(Color::Yellow)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let content = Line::from(vec![
        Span::styled("  ⚡ ", style),
        Span::styled(&cmd.name, style),
        Span::raw("  "),
        Span::styled(
            &cmd.description,
            Style::default().fg(Color::DarkGray)
        ),
    ]);
    
    ListItem::new(content)
}

/// Render empty state.
fn render_empty_state(f: &mut Frame, area: Rect) {
    let text = Paragraph::new(
        "No extensions loaded.\n\n\
         Extensions provide tools and commands.\n\
         Use --extension flag to load extensions."
    )
    .style(Style::default().fg(Color::DarkGray))
    .alignment(ratatui::layout::Alignment::Center);
    
    f.render_widget(text, area);
}

/// Render footer with keybindings.
fn render_footer(f: &mut Frame, area: Rect) {
    let keybindings = Line::from(vec![
        Span::styled("[j/k]", Style::default().fg(Color::Cyan)),
        Span::raw(" Navigate  "),
        Span::styled("[r]", Style::default().fg(Color::Cyan)),
        Span::raw(" Refresh  "),
        Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
        Span::raw(" Back  "),
    ]);
    
    let footer = Paragraph::new(keybindings)
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(footer, area);
}
```

#### Step 3.2: Register Panel in App

**File**: `src/app.rs`

```rust
// Add to render method
match self.mode {
    AppMode::Chat => render_chat_panel(f, area, &self.state),
    AppMode::ToolReview => render_tool_review_panel(f, area, &self.state),
    AppMode::Help => render_help_panel(f, area),
    AppMode::Extensions => render_extension_panel(f, area, &self.state.extensions),
}
```

---

### Phase 4: Input Handling (25-50 LoC)

#### Step 4.1: Add Extension Mode Keybindings

**File**: `src/app.rs`

```rust
// In handle_key_event method
AppMode::Extensions => {
    match key_event.code {
        KeyCode::Char('j') | KeyCode::Down => {
            self.state.extensions.select_next();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            self.state.extensions.select_prev();
        }
        KeyCode::Char('r') => {
            // Refresh extension data
            self.refresh_extensions();
        }
        KeyCode::Esc => {
            self.mode = AppMode::Chat;
        }
        _ => {}
    }
}
```

---

### Phase 5: Async Operations (25-50 LoC)

#### Step 5.1: Add Async Handler

**File**: `src/app.rs`

```rust
impl App {
    /// Refresh extension data from bridge.
    pub async fn refresh_extensions(&mut self) {
        self.state.extensions.loading = true;
        self.state.extensions.error = None;
        
        match self.bridge.refresh_extensions().await {
            Ok(data) => {
                self.state.extensions.count = data.count;
                self.state.extensions.tools = data.tools;
                self.state.extensions.commands = data.commands;
                self.state.extensions.loading = false;
            }
            Err(e) => {
                self.state.extensions.error = Some(e.to_string());
                self.state.extensions.loading = false;
            }
        }
    }
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extension_state_selection() {
        let mut state = ExtensionState::default();
        state.tools = vec![
            ExtensionTool { name: "tool1".into(), description: "".into(), source: None },
            ExtensionTool { name: "tool2".into(), description: "".into(), source: None },
        ];
        
        state.select_next();
        assert_eq!(state.selected_index, Some(0));
        
        state.select_next();
        assert_eq!(state.selected_index, Some(1));
        
        state.select_next();
        assert_eq!(state.selected_index, Some(0)); // Wrap around
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_extension_refresh() {
    let bridge = PiBridge::new(None, None).await.unwrap();
    
    // Should fail without extensions loaded
    let result = bridge.refresh_extensions().await;
    assert!(result.is_err());
    
    // Should work with session
    // ... setup session with extensions ...
}
```

---

## Rollout Plan

### Milestone 1: Core Infrastructure (v0.3.0)

- [ ] Add `ExtensionState` to `AppState`
- [ ] Add `Extensions` mode
- [ ] Add extension event types
- [ ] Test selection logic

### Milestone 2: PiBridge Integration (v0.3.0)

- [ ] Add `refresh_extensions()` to `PiBridge`
- [ ] Test against pi's ExtensionManager
- [ ] Handle errors gracefully

### Milestone 3: Basic UI (v0.3.0)

- [ ] Create `ExtensionPanel` with tools/commands list
- [ ] Add selection navigation (up/down)
- [ ] Show extension stats

### Milestone 4: Polish (v0.4.0)

- [ ] Add refresh keybinding
- [ ] Show loading state
- [ ] Show error messages
- [ ] Keyboard shortcuts help

---

## Success Criteria

- ✅ Can list loaded extensions
- ✅ Can show tools from extensions
- ✅ Can show commands from extensions
- ✅ Shows extension count
- ✅ Handles missing extensions gracefully
- ✅ All tests passing
- ✅ No clippy warnings

---

## Future Enhancements

### Phase 4.1: Extension Loading (v0.4.0)

- Load extensions by path
- Reload extensions at runtime
- Extension configuration UI

### Phase 4.2: MCP Manager Extension (v0.5.0)

- Create JavaScript extension for MCP server management
- Expose MCP lifecycle commands
- Integrate with HawkTUI extension panel

---

## Dependencies

### pi_agent_rust API Requirements

✅ **Confirmed available**:

```rust
// In pi::sdk
pub fn extension_manager(&self) -> Option<&ExtensionManager>;

// In ExtensionManager
pub fn extension_count(&self) -> usize;
pub fn list_tools(&self) -> Vec<Value>;
pub fn list_commands(&self) -> Vec<Value>;
pub fn has_tool(&self, name: &str) -> bool;
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| No extensions loaded | Show helpful empty state |
| Extension list is empty | Guide user to --extension flag |
| Tool JSON parsing fails | Use unwrap_or defaults |
| Async refresh fails | Show error in UI |

---

## Notes

- **Simpler than MCP management!** Works with pi's actual API.
- **Extension count** is available but not extension names (limitation)
- **Tools and commands** are the primary visibility features
- **Future**: Create MCP manager extension for full MCP control

---

*"Investigation reveals the path."* 🦅