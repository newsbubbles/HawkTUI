# Implementation Plan: MCP Server Manager UI

**Date**: 2026-04-03  
**Status**: Ready for Implementation  
**Priority**: Phase 4 (Revised Plugin System)  
**Estimated LoC**: 450-750  

---

## Overview

Build a TUI panel for managing MCP (Model Context Protocol) servers in HawkTUI, leveraging pi_agent_rust's existing MCP support instead of building a custom plugin system.

### Goals

- ✅ List all configured MCP servers with status
- ✅ Show tools and resources from each server
- ✅ Add/edit/remove MCP servers via UI
- ✅ Restart servers without restarting HawkTUI
- ✅ View server logs for debugging

### Non-Goals

- Building a custom plugin system (use MCP instead!)
- Supporting MCP servers that pi doesn't support
- Managing MCP server lifecycle outside of pi

---

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        HawkTUI Architecture                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │  AppState    │───▶│   McpState   │◀───│  PiBridge    │       │
│  │              │    │              │    │              │       │
│  │ - mcp_state  │    │ - servers    │    │ - handle     │       │
│  └──────────────┘    │ - selected   │    │              │       │
│         │            │ - logs       │    │ MCP Methods: │       │
│         │            └──────────────┘    │ - list()     │       │
│         │                   │            │ - add()      │       │
│         ▼                   │            │ - remove()   │       │
│  ┌──────────────┐           │            │ - restart()  │       │
│  │   AppMode    │           │            └──────────────┘       │
│  │              │           │                    ▲              │
│  │ - Chat       │           │                    │              │
│  │ - McpManager │◀──────────┘                    │              │
│  │ - ...        │                                │              │
│  └──────────────┘                                │              │
│         │                                        │              │
│         ▼                                        │              │
│  ┌──────────────┐                                │              │
│  │   McpPanel   │────────────────────────────────┘              │
│  │              │                                                │
│  │ Renders MCP  │                                                │
│  │ server list  │                                                │
│  └──────────────┘                                                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### File Structure

```
src/
├── core/
│   ├── state.rs          # Add McpState
│   ├── events.rs         # Add McpEvent variants
│   └── mode.rs           # Add McpManager mode
├── providers/
│   └── pi_bridge.rs      # Add MCP methods
├── ui/
│   ├── panels/
│   │   └── mcp_panel.rs  # NEW: MCP server list panel
│   ├── widgets/
│   │   └── mcp_server_card.rs  # NEW: Individual server widget
│   └── mod.rs            # Update exports
└── app.rs                # Add McpManager mode handling
```

---

## Implementation Steps

### Phase 1: Core Infrastructure (100-150 LoC)

#### Step 1.1: Add MCP Types to Core

**File**: `src/core/state.rs`

```rust
/// MCP server management state.
#[derive(Debug, Default)]
pub struct McpState {
    /// Known MCP servers.
    pub servers: Vec<McpServerInfo>,
    
    /// Currently selected server index.
    pub selected_index: Option<usize>,
    
    /// Scroll offset for server list.
    pub scroll_offset: usize,
    
    /// Server logs (server_name -> recent logs).
    pub logs: HashMap<String, VecDeque<String>>,
    
    /// Loading state.
    pub loading: bool,
    
    /// Error message (if any).
    pub error: Option<String>,
}

/// Information about an MCP server.
#[derive(Debug, Clone)]
pub struct McpServerInfo {
    /// Server name (unique identifier).
    pub name: String,
    
    /// Current status.
    pub status: McpServerStatus,
    
    /// Number of tools exposed.
    pub tool_count: usize,
    
    /// Number of resources exposed.
    pub resource_count: usize,
    
    /// Command to run the server.
    pub command: String,
    
    /// Arguments for the command.
    pub args: Vec<String>,
    
    /// Environment variables.
    pub env: HashMap<String, String>,
    
    /// Whether the server is disabled.
    pub disabled: bool,
}

/// MCP server status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpServerStatus {
    /// Server is starting up.
    Starting,
    
    /// Server is running and healthy.
    Running,
    
    /// Server is stopped.
    Stopped,
    
    /// Server encountered an error.
    Error,
}

impl McpState {
    /// Create new MCP state.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Select next server.
    pub fn select_next(&mut self) {
        if self.servers.is_empty() {
            self.selected_index = None;
        } else {
            let next = self.selected_index
                .map(|i| (i + 1) % self.servers.len())
                .unwrap_or(0);
            self.selected_index = Some(next);
        }
    }
    
    /// Select previous server.
    pub fn select_prev(&mut self) {
        if self.servers.is_empty() {
            self.selected_index = None;
        } else {
            let prev = self.selected_index
                .map(|i| i.saturating_sub(1))
                .unwrap_or(self.servers.len() - 1);
            self.selected_index = Some(prev);
        }
    }
    
    /// Get selected server.
    pub fn selected_server(&self) -> Option<&McpServerInfo> {
        self.selected_index.and_then(|i| self.servers.get(i))
    }
}
```

**Add to AppState**:
```rust
pub struct AppState {
    // ... existing fields
    
    /// MCP server management state.
    pub mcp: McpState,
}
```

#### Step 1.2: Add MCP Events

**File**: `src/core/events.rs`

```rust
/// MCP-related events.
pub enum McpEvent {
    /// Refresh server list.
    Refresh,
    
    /// Server list updated.
    ServersUpdated(Vec<McpServerInfo>),
    
    /// Add a new server.
    AddServer {
        name: String,
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
    },
    
    /// Remove a server.
    RemoveServer(String),
    
    /// Restart a server.
    RestartServer(String),
    
    /// Toggle server enabled/disabled.
    ToggleServer(String),
    
    /// Server status changed.
    ServerStatusChanged {
        name: String,
        status: McpServerStatus,
    },
    
    /// Error occurred.
    Error(String),
}
```

#### Step 1.3: Add McpManager Mode

**File**: `src/core/mode.rs`

```rust
pub enum AppMode {
    Chat,
    ToolReview,
    Help,
    McpManager,  // NEW
}

impl AppMode {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Chat => "Chat",
            Self::ToolReview => "Tool Review",
            Self::Help => "Help",
            Self::McpManager => "MCP Servers",
        }
    }
}
```

---

### Phase 2: PiBridge MCP Integration (100-150 LoC)

#### Step 2.1: Add MCP Methods to PiBridge

**File**: `src/providers/pi_bridge.rs`

```rust
use pi::sdk::{
    // ... existing imports
    McpServerConfig as PiMcpConfig,
    McpServerStatus as PiMcpStatus,
};

impl PiBridge {
    /// List all MCP servers.
    ///
    /// Returns information about each configured MCP server.
    pub async fn list_mcp_servers(&self) -> Result<Vec<McpServerInfo>> {
        let handle = self.handle.as_ref()
            .ok_or_else(|| Error::NotConnected)?;
        
        let guard = handle.lock().await;
        
        // Call pi's MCP server listing API
        let pi_servers = guard.mcp_servers().await
            .map_err(|e| Error::McpError(e.to_string()))?;
        
        // Convert to our types
        let servers = pi_servers
            .into_iter()
            .map(|s| McpServerInfo {
                name: s.name,
                status: convert_status(s.status),
                tool_count: s.tool_count,
                resource_count: s.resource_count,
                command: s.command,
                args: s.args,
                env: s.env,
                disabled: s.disabled,
            })
            .collect();
        
        Ok(servers)
    }
    
    /// Add a new MCP server.
    pub async fn add_mcp_server(
        &self,
        name: String,
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
    ) -> Result<()> {
        let handle = self.handle.as_ref()
            .ok_or_else(|| Error::NotConnected)?;
        
        let mut guard = handle.lock().await;
        
        let config = PiMcpConfig {
            name,
            command,
            args,
            env,
            disabled: false,
        };
        
        guard.add_mcp_server(config).await
            .map_err(|e| Error::McpError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Remove an MCP server.
    pub async fn remove_mcp_server(&self, name: &str) -> Result<()> {
        let handle = self.handle.as_ref()
            .ok_or_else(|| Error::NotConnected)?;
        
        let mut guard = handle.lock().await;
        
        guard.remove_mcp_server(name).await
            .map_err(|e| Error::McpError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Restart an MCP server.
    pub async fn restart_mcp_server(&self, name: &str) -> Result<()> {
        let handle = self.handle.as_ref()
            .ok_or_else(|| Error::NotConnected)?;
        
        let mut guard = handle.lock().await;
        
        guard.restart_mcp_server(name).await
            .map_err(|e| Error::McpError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Toggle an MCP server enabled/disabled.
    pub async fn toggle_mcp_server(&self, name: &str) -> Result<()> {
        let handle = self.handle.as_ref()
            .ok_or_else(|| Error::NotConnected)?;
        
        let mut guard = handle.lock().await;
        guard.toggle_mcp_server(name).await
            .map_err(|e| Error::McpError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get tools from an MCP server.
    pub async fn get_mcp_tools(&self, server_name: &str) -> Result<Vec<ToolSummary>> {
        let handle = self.handle.as_ref()
            .ok_or_else(|| Error::NotConnected)?;
        
        let guard = handle.lock().await;
        
        let tools = guard.mcp_server_tools(server_name).await
            .map_err(|e| Error::McpError(e.to_string()))?;
        
        Ok(tools.into_iter().map(|t| ToolSummary::from(t)).collect())
    }
}

/// Convert pi's MCP status to our status.
fn convert_status(status: PiMcpStatus) -> McpServerStatus {
    match status {
        PiMcpStatus::Starting => McpServerStatus::Starting,
        PiMcpStatus::Running => McpServerStatus::Running,
        PiMcpStatus::Stopped => McpServerStatus::Stopped,
        PiMcpStatus::Error => McpServerStatus::Error,
    }
}
```

---

### Phase 3: MCP Panel UI (150-250 LoC)

#### Step 3.1: Create MCP Panel

**File**: `src/ui/panels/mcp_panel.rs`

```rust
//! MCP Server Manager Panel.
//!
//! Displays list of MCP servers with status, tools, and controls.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::core::state::{McpServerInfo, McpServerStatus, McpState};
use crate::core::mode::AppMode;

/// Render the MCP manager panel.
pub fn render_mcp_panel(f: &mut Frame, area: Rect, state: &McpState) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(5),     // Server list
            Constraint::Length(3),  // Footer/help
        ])
        .split(area);
    
    // Render header
    render_header(f, chunks[0], state);
    
    // Render server list
    render_server_list(f, chunks[1], state);
    
    // Render footer
    render_footer(f, chunks[2]);
}

/// Render header with title and stats.
fn render_header(f: &mut Frame, area: Rect, state: &McpState) {
    let running = state.servers.iter()
        .filter(|s| s.status == McpServerStatus::Running)
        .count();
    
    let stopped = state.servers.iter()
        .filter(|s| s.status == McpServerStatus::Stopped)
        .count();
    
    let title = Line::from(vec![
        Span::styled(" MCP Servers ", Style::default().fg(Color::Cyan).bold()),
        Span::raw("│"),
        Span::styled(
            format!(" {} running ", running),
            Style::default().fg(Color::Green)
        ),
        Span::raw("│"),
        Span::styled(
            format!(" {} stopped ", stopped),
            Style::default().fg(Color::Yellow)
        ),
    ]);
    
    let header = Paragraph::new(title)
        .block(Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray)));
    
    f.render_widget(header, area);
}

/// Render the server list.
fn render_server_list(f: &mut Frame, area: Rect, state: &McpState) {
    if state.servers.is_empty() {
        render_empty_state(f, area);
        return;
    }
    
    let items: Vec<ListItem> = state.servers
        .iter()
        .enumerate()
        .map(|(i, server)| render_server_item(server, i == state.selected_index.unwrap_or(usize::MAX)))
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Servers "));
    
    f.render_widget(list, area);
}

/// Render a single server item.
fn render_server_item(server: &McpServerInfo, selected: bool) -> ListItem {
    let status_icon = match server.status {
        McpServerStatus::Starting => "◐",
        McpServerStatus::Running => "●",
        McpServerStatus::Stopped => "○",
        McpServerStatus::Error => "✗",
    };
    
    let status_color = match server.status {
        McpServerStatus::Starting => Color::Yellow,
        McpServerStatus::Running => Color::Green,
        McpServerStatus::Stopped => Color::Gray,
        McpServerStatus::Error => Color::Red,
    };
    
    let style = if selected {
        Style::default()
            .fg(status_color)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(status_color)
    };
    
    let content = Line::from(vec![
        Span::styled(format!("{} ", status_icon), style),
        Span::styled(
            format!("{:<15}", server.name),
            if selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }
        ),
        Span::raw("  "),
        Span::styled(
            format!("{:<10}", format!("{:?}", server.status)),
            style
        ),
        Span::raw("  "),
        Span::styled(
            format!("{} tools", server.tool_count),
            Style::default().fg(Color::Blue)
        ),
        Span::raw("  "),
        Span::styled(
            server.command.as_str(),
            Style::default().fg(Color::DarkGray)
        ),
    ]);
    
    ListItem::new(content)
}

/// Render empty state.
fn render_empty_state(f: &mut Frame, area: Rect) {
    let text = Paragraph::new(
        "No MCP servers configured.\n\nPress [a] to add a server."
    )
    .style(Style::default().fg(Color::DarkGray))
    .alignment(ratatui::layout::Alignment::Center);
    
    f.render_widget(text, area);
}

/// Render footer with keybindings.
fn render_footer(f: &mut Frame, area: Rect) {
    let keybindings = Line::from(vec![
        Span::styled("[a]", Style::default().fg(Color::Cyan)),
        Span::raw(" Add  "),
        Span::styled("[e]", Style::default().fg(Color::Cyan)),
        Span::raw(" Edit  "),
        Span::styled("[d]", Style::default().fg(Color::Cyan)),
        Span::raw(" Delete  "),
        Span::styled("[r]", Style::default().fg(Color::Cyan)),
        Span::raw(" Restart  "),
        Span::styled("[t]", Style::default().fg(Color::Cyan)),
        Span::raw(" Tools  "),
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
    AppMode::McpManager => render_mcp_panel(f, area, &self.state.mcp),
}
```

---

### Phase 4: Input Handling (50-100 LoC)

#### Step 4.1: Add MCP Mode Keybindings

**File**: `src/app.rs`

```rust
// In handle_key_event method
AppStateMode::McpManager => {
    match key_event.code {
        KeyCode::Char('a') => {
            // Open add server form
            self.show_add_server_form = true;
        }
        KeyCode::Char('e') => {
            // Edit selected server
            if let Some(server) = self.state.mcp.selected_server() {
                self.show_edit_server_form(server.clone());
            }
        }
        KeyCode::Char('d') => {
            // Delete selected server
            if let Some(server) = self.state.mcp.selected_server() {
                self.delete_server(server.name.clone());
            }
        }
        KeyCode::Char('r') => {
            // Restart selected server
            if let Some(server) = self.state.mcp.selected_server() {
                self.restart_server(server.name.clone());
            }
        }
        KeyCode::Char('t') => {
            // Show tools from selected server
            if let Some(server) = self.state.mcp.selected_server() {
                self.show_server_tools(server.name.clone());
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            self.state.mcp.select_prev();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            self.state.mcp.select_next();
        }
        KeyCode::Esc => {
            self.mode = AppMode::Chat;
        }
        _ => {}
    }
}
```

---

### Phase 5: Add/Edit Server Form (100-150 LoC)

#### Step 5.1: Create Add Server Form

**File**: `src/ui/widgets/mcp_server_form.rs`

```rust
//! Form for adding/editing MCP servers.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::core::state::McpServerInfo;

/// Form state for adding/editing servers.
pub struct ServerForm {
    pub name: String,
    pub command: String,
    pub args: String,
    pub env: String,
    pub focused_field: FormField,
    pub is_edit: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum FormField {
    Name,
    Command,
    Args,
    Env,
}

impl ServerForm {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            command: String::new(),
            args: String::new(),
            env: String::new(),
            focused_field: FormField::Name,
            is_edit: false,
        }
    }
    
    pub fn from_server(server: &McpServerInfo) -> Self {
        Self {
            name: server.name.clone(),
            command: server.command.clone(),
            args: server.args.join(" "),
            env: server.env
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", "),
            focused_field: FormField::Name,
            is_edit: true,
        }
    }
    
    pub fn next_field(&mut self) {
        self.focused_field = match self.focused_field {
            FormField::Name => FormField::Command,
            FormField::Command => FormField::Args,
            FormField::Args => FormField::Env,
            FormField::Env => FormField::Name,
        };
    }
    
    pub fn prev_field(&mut self) {
        self.focused_field = match self.focused_field {
            FormField::Name => FormField::Env,
            FormField::Command => FormField::Name,
            FormField::Args => FormField::Command,
            FormField::Env => FormField::Args,
        };
    }
}

/// Render the add/edit server form.
pub fn render_server_form(f: &mut Frame, area: Rect, form: &ServerForm) {
    // Center the form
    let form_area = centered_rect(60, 50, area);
    
    // Clear background
    f.render_widget(Clear, form_area);
    
    // Create form layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Name
            Constraint::Length(3),  // Command
            Constraint::Length(3),  // Args
            Constraint::Length(3),  // Env
            Constraint::Length(3),  // Buttons
        ])
        .split(form_area);
    
    // Title
    let title = if form.is_edit { "Edit Server" } else { "Add Server" };
    let title_widget = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title_widget, chunks[0]);
    
    // Name field
    render_text_field(f, chunks[1], "Name", &form.name, form.focused_field == FormField::Name);
    
    // Command field
    render_text_field(f, chunks[2], "Command", &form.command, form.focused_field == FormField::Command);
    
    // Args field
    render_text_field(f, chunks[3], "Args", &form.args, form.focused_field == FormField::Args);
    
    // Env field
    render_text_field(f, chunks[4], "Env (KEY=value, ...)", &form.env, form.focused_field == FormField::Env);
    
    // Buttons
    let buttons = Paragraph::new("[Enter] Save  [Esc] Cancel")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(buttons, chunks[5]);
}

fn render_text_field(f: &mut Frame, area: Rect, label: &str, value: &str, focused: bool) {
    let style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    
    let field = Paragraph::new(value)
        .style(style)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(label));
    
    f.render_widget(field, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

---

### Phase 6: Async Operations (50-100 LoC)

#### Step 6.1: Add Async Handlers

**File**: `src/app.rs`

```rust
impl App {
    /// Refresh MCP server list.
    pub async fn refresh_mcp_servers(&mut self) {
        self.state.mcp.loading = true;
        self.state.mcp.error = None;
        
        match self.bridge.list_mcp_servers().await {
            Ok(servers) => {
                self.state.mcp.servers = servers;
                self.state.mcp.loading = false;
            }
            Err(e) => {
                self.state.mcp.error = Some(e.to_string());
                self.state.mcp.loading = false;
            }
        }
    }
    
    /// Add a new MCP server.
    pub async fn add_mcp_server(&mut self, form: ServerForm) {
        let env = parse_env_string(&form.env);
        let args = form.args.split_whitespace().map(String::from).collect();
        
        match self.bridge.add_mcp_server(
            form.name,
            form.command,
            args,
            env,
        ).await {
            Ok(()) => {
                self.refresh_mcp_servers().await;
            }
            Err(e) => {
                self.state.mcp.error = Some(e.to_string());
            }
        }
    }
    
    /// Restart an MCP server.
    pub async fn restart_mcp_server(&mut self, name: String) {
        match self.bridge.restart_mcp_server(&name).await {
            Ok(()) => {
                self.refresh_mcp_servers().await;
            }
            Err(e) => {
                self.state.mcp.error = Some(e.to_string());
            }
        }
    }
}

fn parse_env_string(s: &str) -> HashMap<String, String> {
    s.split(',')
        .filter_map(|pair| {
            let parts: Vec<&str> = pair.trim().splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
            } else {
                None
            }
        })
        .collect()
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
    fn test_mcp_state_selection() {
        let mut state = McpState::default();
        state.servers = vec![
            McpServerInfo { name: "a".into(), ..Default::default() },
            McpServerInfo { name: "b".into(), ..Default::default() },
        ];
        
        state.select_next();
        assert_eq!(state.selected_index, Some(0));
        
        state.select_next();
        assert_eq!(state.selected_index, Some(1));
        
        state.select_next();
        assert_eq!(state.selected_index, Some(0)); // Wrap around
    }
    
    #[test]
    fn test_parse_env_string() {
        let env = parse_env_string("KEY=value, FOO=bar");
        assert_eq!(env.get("KEY"), Some(&"value".to_string()));
        assert_eq!(env.get("FOO"), Some(&"bar".to_string()));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_mcp_server_lifecycle() {
    let bridge = PiBridge::new(None, None).await.unwrap();
    
    // List servers
    let servers = bridge.list_mcp_servers().await.unwrap();
    
    // Add server
    bridge.add_mcp_server(
        "test".into(),
        "node".into(),
        vec!["server.js".into()],
        HashMap::new(),
    ).await.unwrap();
    
    // Restart
    bridge.restart_mcp_server("test").await.unwrap();
    
    // Remove
    bridge.remove_mcp_server("test").await.unwrap();
}
```

---

## Rollout Plan

### Milestone 1: Core Infrastructure (v0.3.0)

- [ ] Add `McpState` to `AppState`
- [ ] Add `McpManager` mode
- [ ] Add MCP event types
- [ ] Test selection logic

### Milestone 2: PiBridge Integration (v0.3.0)

- [ ] Add MCP methods to `PiBridge`
- [ ] Test against pi's MCP API
- [ ] Handle errors gracefully

### Milestone 3: Basic UI (v0.3.0)

- [ ] Create `McpPanel` with server list
- [ ] Add selection navigation (up/down)
- [ ] Show server status and stats

### Milestone 4: Full UI (v0.4.0)

- [ ] Add/edit server form
- [ ] Delete server confirmation
- [ ] Server tools viewer
- [ ] Server logs viewer

### Milestone 5: Polish (v0.4.0+)

- [ ] Keyboard shortcuts for all actions
- [ ] Status bar integration
- [ ] Configuration persistence
- [ ] Documentation

---

## Success Criteria

- ✅ Can list all MCP servers with status
- ✅ Can add new MCP servers via UI
- ✅ Can remove MCP servers
- ✅ Can restart servers without restart HawkTUI
- ✅ Shows tools count for each server
- ✅ Handles errors gracefully
- ✅ All tests passing
- ✅ No clippy warnings
- ✅ Code coverage > 80%

---

## Dependencies

### pi_agent_rust API Requirements

Need to verify pi exposes:

```rust
// In pi::sdk
pub fn mcp_servers(&self) -> impl Future<Output = Result<Vec<McpServerInfo>>>;
pub fn add_mcp_server(&mut self, config: McpConfig) -> impl Future<Output = Result<()>>;
pub fn remove_mcp_server(&mut self, name: &str) -> impl Future<Output = Result<()>>;
pub fn restart_mcp_server(&mut self, name: &str) -> impl Future<Output = Result<()>>;
pub fn mcp_server_tools(&self, name: &str) -> impl Future<Output = Result<Vec<ToolDef>>>;
```

**Action**: Check pi_agent_rust SDK to confirm these APIs exist.

---

## Risk Mitigation

| Risk | Mitigation | Owner |
|------|------------|-------|
| pi's MCP API differs from expected | Check API first, adapt as needed | Rusty |
| Server restart fails | Show error in UI, suggest manual restart | Rusty |
| Form validation errors | Client-side validation + error display | Rusty |
| Async race conditions | Use proper locking in PiBridge | Rusty |

---

## Notes

- **Simpler than custom plugins!** Leverage pi's MCP support.
- **Test with real MCP servers** (filesystem, github, postgres)
- **Consider hot-reload**: Restart servers without restart HawkTUI
- **Configuration persistence**: Should be handled by pi

---

*"Good questions save months of work."* 🦅