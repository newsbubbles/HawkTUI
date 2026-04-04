# Phase 4 Advanced Features: Deep Investigation

**Date**: 2026-04-03  
**Author**: Rusty 🦀

---

## Executive Summary

Phase 4 features from DESIGN.md:

1. **Code review mode** with side-by-side diffs
2. **Diff visualization** 
3. **File browser integration**
4. **Plugin system**
5. **Remote session support**

This investigation assesses:
- Current architecture readiness
- Implementation complexity (LoC estimates)
- Refactoring requirements
- Dependency impact
- Risk assessment

---

## Feature 1: Code Review Mode

### Vision (from DESIGN.md)

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
├──────────────────────────────────┴───────────────────────────────────────┤
│ 🤖 The suggested version adds proper error handling using Result<()>...      │
├──────────────────────────────────────────────────────────────────────────────┤
│ [a]ccept  [r]eject  [e]dit  [n]ext  [p]rev  [q]uit                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Current Architecture Status

| Component | Status | Notes |
|-----------|--------|-------|
| Layout system | ✅ Ready | `LayoutMode::Split` exists, `secondary: Option<Rect>` works |
| State management | ⚠️ Partial | No diff state, no code review state |
| Panel abstraction | ✅ Ready | Can add new panel types easily |
| Event handling | ⚠️ Partial | Need diff-specific actions (Accept, Reject, Next, Prev) |
| pi integration | ❌ Missing | Need to capture code diffs from assistant messages |

### Required New Components

```rust
// src/core/state.rs - New state structures

/// Code review state.
#[derive(Debug, Default)]
pub struct CodeReviewState {
    /// Current diff being reviewed.
    pub current_diff: Option<PendingDiff>,
    
    /// Queue of pending diffs.
    pub pending_diffs: VecDeque<PendingDiff>,
    
    /// Accepted diffs (applied to files).
    pub accepted: Vec<AcceptedDiff>,
    
    /// Rejected diffs.
    pub rejected: Vec<RejectedDiff>,
    
    /// Current view position.
    pub scroll_offset: u16,
}

/// A pending diff awaiting review.
#[derive(Debug, Clone)]
pub struct PendingDiff {
    pub id: Uuid,
    pub file_path: String,
    pub original: String,
    pub suggested: String,
    pub hunks: Vec<DiffHunk>,
    pub explanation: String,
    pub created_at: DateTime<Utc>,
}

/// A diff hunk (contiguous change region).
#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

/// A single diff line.
#[derive(Debug, Clone)]
pub enum DiffLine {
    Context(String),
    Removed(String),
    Added(String),
}

/// An accepted diff (applied to file).
#[derive(Debug, Clone)]
pub struct AcceptedDiff {
    pub diff_id: Uuid,
    pub file_path: String,
    pub applied_at: DateTime<Utc>,
    pub backup_path: Option<String>, // For undo
}
```

```rust
// src/ui/panels/code_review.rs - New panel

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub struct CodeReviewPanel<'a> {
    review: &'a CodeReviewState,
    theme: &'a Theme,
    focused: bool,
}

impl<'a> CodeReviewPanel<'a> {
    /// Render side-by-side diff.
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Left: original, Right: suggested
        // Highlight removed lines in red, added lines in green
        // Show line numbers
        // ... implementation
    }
    
    /// Handle keyboard input.
    pub fn handle_key(&self, key: KeyEvent) -> Option<ReviewAction> {
        match key.code {
            KeyCode::Char('a') => Some(ReviewAction::Accept),
            KeyCode::Char('r') => Some(ReviewAction::Reject),
            KeyCode::Char('e') => Some(ReviewAction::Edit),
            KeyCode::Char('n') => Some(ReviewAction::Next),
            KeyCode::Char('p') => Some(ReviewAction::Prev),
            KeyCode::Char('q') => Some(ReviewAction::Quit),
            _ => None,
        }
    }
}
```

### Implementation Complexity

| Sub-feature | LoC Estimate | Effort | Risk |
|-------------|--------------|--------|------|
| Diff parsing (unified diff format) | 200-300 | Medium | Low |
| Diff state management | 100-150 | Low | Low |
| Side-by-side rendering | 300-400 | High | Medium |
| Syntax-aware diff highlighting | 200-300 | Medium | Medium |
| Accept/Reject workflow | 150-200 | Medium | Low |
| Backup and undo system | 100-150 | Medium | Medium |
| Integration with assistant messages | 200-300 | High | High |

**Total**: **1,250-1,800 LoC**

### Refactoring Required

**Minimal refactoring needed**:

1. Add `Panel::CodeReview` to enum (1 line)
2. Add `LayoutMode::CodeReview` variant (trivial)
3. Extend `Action` enum with review actions (10 lines)
4. Wire up panel in `App::render()` (20-30 lines)

**Why minimal?** The architecture already planned for this:
- Layout system supports secondary panels
- State management is extensible
- Panel rendering is modular

### Dependencies

**New crates needed**:

```toml
[dependencies]
# Diff generation and parsing
similar = "2.4"      # Fast diff algorithm
patch = "0.7"        # Unified diff parser (optional)

# File backup
tempfile = "3.10"    # Already used elsewhere
```

**Alternative**: Implement unified diff parsing manually (~200 LoC) to avoid dependency.

### Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Complex merge conflicts | Medium | High | Show conflicts clearly, require manual resolution |
| Large files cause UI lag | Medium | Medium | Lazy rendering, only render visible lines |
| Syntax highlighting slow | Low | Medium | Cache highlighted lines, use async rendering |
| User accepts wrong diff | Low | High | Backup files, provide undo |
| pi doesn't return diffs | High | Critical | Parse code blocks from assistant messages |

**Highest risk**: Parsing diffs from assistant messages. The AI might not always return code in a structured format.

### Integration with pi_bridge

```rust
// src/providers/pi_bridge.rs - Add diff extraction

impl PiBridge {
    /// Extract pending diffs from assistant message.
    pub fn extract_diffs(&self, message: &AssistantMessage) -> Vec<PendingDiff> {
        let mut diffs = Vec::new();
        
        for block in &message.content_blocks {
            if let ContentBlock::Code { language, text } = block {
                if let Some(file_path) = self.extract_file_path(&block) {
                    // Read original file
                    let original = std::fs::read_to_string(&file_path).ok()?;
                    
                    // Create diff
                    diffs.push(PendingDiff {
                        file_path,
                        original,
                        suggested: text.clone(),
                        // ... generate hunks using similar crate
                    });
                }
            }
        }
        
        diffs
    }
}
```

---

## Feature 2: Diff Visualization

### Overview

This is part of code review mode, but can also be used independently:
- Show git diff for a file
- Compare current file with HEAD
- Visualize staged changes

### Implementation Complexity

This is **included in Feature 1** implementation. The diff rendering logic is the same.

Additional work for git integration:

| Sub-feature | LoC | Effort |
|-------------|-----|--------|
| Git diff parsing | 150-200 | Medium |
| Git status integration | 100-150 | Low |
| Staged/unstaged toggle | 50-100 | Low |

**Total**: **300-450 LoC** (in addition to Feature 1)

---

## Feature 3: File Browser Integration

### Vision

A new panel showing:
- Project file tree
- Git status (modified, staged, untracked)
- File icons (based on extension)
- Search/filter functionality

### Current Architecture Status

| Component | Status | Notes |
|-----------|--------|-------|
| Panel abstraction | ✅ Ready | Can add `Panel::FileBrowser` |
| State management | ⚠️ Partial | Need file tree state |
| pi integration | ❌ Missing | pi doesn't expose file operations directly |

### Required New Components

```rust
// src/core/state.rs

#[derive(Debug, Default)]
pub struct FileBrowserState {
    /// Root directory path.
    pub root: Option<PathBuf>,
    
    /// Current directory contents.
    pub entries: Vec<FileEntry>,
    
    /// Selected entry index.
    pub selected_index: Option<usize>,
    
    /// Scroll offset.
    pub scroll_offset: u16,
    
    /// Filter/query string.
    pub filter: String,
    
    /// Show hidden files.
    pub show_hidden: bool,
    
    /// Git status cache.
    pub git_status: HashMap<PathBuf, GitStatus>,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub git_status: Option<GitStatus>,
}

#[derive(Debug, Clone, Copy)]
pub enum GitStatus {
    Unmodified,
    Modified,
    Added,
    Deleted,
    Untracked,
    Ignored,
}
```

```rust
// src/ui/panels/file_browser.rs

pub struct FileBrowserPanel<'a> {
    browser: &'a FileBrowserState,
    theme: &'a Theme,
    focused: bool,
}

impl<'a> FileBrowserPanel<'a> {
    pub fn render_tree(&self, entries: &[FileEntry], area: Rect, buf: &mut Buffer) {
        // Render as tree with icons:
        // 📁 src/
        //   📄 main.rs      (modified)
        //   📄 lib.rs       (added)
        // 📁 tests/
        //   📄 test_app.rs
    }
}
```

### Implementation Complexity

| Sub-feature | LoC | Effort | Risk |
|-------------|-----|--------|------|
| File system traversal | 150-200 | Medium | Low |
| Tree rendering | 200-300 | Medium | Low |
| Git status integration | 150-200 | Medium | Medium |
| File icons | 50-100 | Low | Low |
| Search/filter | 100-150 | Low | Low |
| Keyboard navigation | 100-150 | Low | Low |
| File actions (open, delete, rename) | 200-300 | Medium | Medium |

**Total**: **950-1,400 LoC**

### Refactoring Required

**Minimal**:

1. Add `Panel::FileBrowser` enum variant
2. Add state to `AppState`
3. Wire up panel rendering

### Dependencies

```toml
[dependencies]
walkdir = "2.5"       # Directory traversal (already used?)
git2 = "0.18"         # Git status (or use shell)
ignore = "0.4"       # .gitignore parsing
```

Alternative: Use shell commands (`git status --porcelain`) instead of git2 crate.

### Integration with pi_bridge

File browser would be independent of pi_bridge for local operations, but could:
- Add files to context (pass to agent)
- Open files in editor
- Show diff for selected file

---

## Feature 4: Plugin System

### Vision

Allow external extensions:
- Custom panels
- Custom commands
- Custom themes
- Event hooks

### Current Architecture Status

| Component | Status | Notes |
|-----------|--------|-------|
| Plugin architecture | ❌ Not designed | Current architecture is monolithic |
| Dynamic loading | ❌ Not supported | Rust requires unsafe for runtime loading |
| Event system | ⚠️ Basic | Events are hardcoded enum variants |

### Architectural Requirements

This is a **major refactoring**. Current architecture:

```rust
// Current: Hardcoded everything
enum Action {
    Quit,
    SendMessage,
    // ... 40+ hardcoded actions
}

enum Panel {
    Conversation,
    Sessions,
    Tools,
    Context,
    Input,
    // Hardcoded panel types
}
```

Need to move to:

```rust
// Plugin-friendly architecture
trait PanelPlugin {
    fn name(&self) -> &str;
    fn render(&self, area: Rect, buf: &mut Buffer, state: &dyn Any);
    fn handle_key(&self, key: KeyEvent) -> Option<Box<dyn Action>>;
}

trait ActionPlugin {
    fn name(&self) -> &str;
    fn execute(&self, app: &mut App) -> Result<()>;
}

trait CommandPlugin {
    fn name(&self) -> &str;
    fn parse(&self, args: &[String]) -> Result<Box<dyn ActionPlugin>>;
}
```

### Implementation Complexity

| Sub-feature | LoC | Effort | Risk |
|-------------|-----|--------|------|
| Plugin trait definitions | 100-150 | Medium | Medium |
| Plugin manager | 200-300 | High | High |
| Dynamic loading (libloading) | 300-400 | High | High |
| Plugin API stability | Ongoing | High | High |
| Security sandboxing | 200-300 | High | Critical |
| Plugin communication | 150-200 | Medium | Medium |

**Total**: **950-1,350 LoC** (plus significant architectural changes)

### Refactoring Required

**MAJOR REFACTORING**:

1. Convert `Action` enum to trait objects
2. Convert `Panel` enum to trait objects
3. Abstract state access for plugins
4. Add plugin lifecycle management
5. Create plugin API documentation
6. Build example plugins

**Estimated refactoring of existing code**: **500-800 LoC** changes

### Dependencies

```toml
[dependencies]
libloading = "0.8"    # Dynamic library loading (unsafe)
abi_stable = "0.11" # Stable ABI for plugins (alternative)
```

### Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Plugin crashes app | High | Critical | Run plugins in separate process |
| Security vulnerabilities | High | Critical | Sandbox plugins, restrict file access |
| API breaks plugins | High | High | Version plugins, semantic versioning |
| Plugin developers confused | Medium | Medium | Excellent documentation, examples |

**Recommendation**: Defer to v0.5.0+ and consider WASM-based plugins instead of native.

**Alternative**: Configuration-based "plugins" (TOML-defined commands, themes, keybindings) without code execution.

---

## Feature 5: Remote Session Support

### Vision

Connect to remote pi instances:
- SSH connection to remote server
- Work with remote files
- Sync sessions across machines

### Current Architecture Status

| Component | Status | Notes |
|-----------|--------|-------|
| Network layer | ❌ Missing | No networking code |
| pi_bridge | ⚠️ Local only | Assumes pi library is local |
| State sync | ❌ Missing | Sessions are local |

### Architectural Requirements

```rust
// src/providers/remote_bridge.rs

pub struct RemoteBridge {
    /// Connection to remote pi instance.
    connection: RemoteConnection,
    
    /// Session cache.
    sessions: Arc<Mutex<HashMap<Uuid, RemoteSession>>>,
}

pub enum RemoteConnection {
    Ssh { host: String, user: String, port: u16 },
    Tcp { addr: SocketAddr },
    Unix { path: PathBuf },
}

pub struct RemoteSession {
    pub id: Uuid,
    pub name: String,
    pub connection: RemoteConnection,
    // Events streamed over network
    pub event_rx: mpsc::Receiver<PiAgentEvent>,
}
```

### Implementation Complexity

| Sub-feature | LoC | Effort | Risk |
|-------------|-----|--------|------|
| SSH connection handling | 400-600 | High | High |
| Remote protocol design | 200-300 | High | Medium |
| Session serialization | 150-200 | Medium | Low |
| Event streaming | 300-400 | High | High |
| Authentication | 200-300 | High | Critical |
| Error handling (disconnect) | 150-200 | Medium | Medium |
| Local caching | 200-300 | Medium | Medium |

**Total**: **1,600-2,300 LoC**

### Refactoring Required

**Moderate refactoring**:

1. Abstract `PiBridge` to trait
2. Create `LocalBridge` and `RemoteBridge` implementations
3. Add connection state to `AppState`
4. Handle network errors gracefully

**Estimated refactoring**: **200-400 LoC** changes

### Dependencies

```toml
[dependencies]
ssh2 = "0.9"         # SSH client
tokio = { version = "1", features = ["net", "io-util"] }
serde_json = "1.0"   # Already used
```

### Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Network latency makes UI lag | High | High | Async operations, optimistic UI |
| Connection drops | High | High | Reconnection logic, local cache |
| SSH key management | Medium | Medium | Use system SSH agent |
| Security (MITM) | Low | Critical | Verify host keys, use known_hosts |

---

## Summary: Implementation Roadmap

### Total LoC Estimates

| Feature | LoC | Effort | Dependencies |
|---------|-----|--------|--------------|
| Code Review Mode | 1,250-1,800 | Medium-High | similar, patch? |
| Diff Visualization | 300-450 | Medium | (included in above) |
| File Browser | 950-1,400 | Medium | git2 or shell |
| Plugin System | 950-1,350 | High | libloading or WASM |
| Remote Sessions | 1,600-2,300 | High | ssh2, tokio |

**Grand Total**: **5,050-7,300 LoC**

### Refactoring Requirements

| Feature | Refactoring Needed | Impact on Existing Code |
|---------|-------------------|----------------------|
| Code Review Mode | Minimal | 50-100 LoC changes |
| Diff Visualization | None | (included in above) |
| File Browser | Minimal | 50-100 LoC changes |
| Plugin System | **Major** | 500-800 LoC changes, architectural shift |
| Remote Sessions | Moderate | 200-400 LoC changes, abstraction layer |

### Architecture Readiness Score

| Feature | Score | Notes |
|---------|-------|-------|
| Code Review Mode | 8/10 | Layout system ready, need diff logic |
| Diff Visualization | 8/10 | Same as above |
| File Browser | 7/10 | Panel system ready, need file logic |
| Plugin System | 3/10 | **Major architectural work needed** |
| Remote Sessions | 5/10 | Need networking layer and bridge abstraction |

---

## Recommendations

### Phase 4a: Quick Wins (v0.3.0)

Implement features with **minimal refactoring**:

1. **Code Review Mode** (1,500 LoC)
   - Highest user value
   - Architecture is ready
   - Low risk

2. **File Browser** (1,200 LoC)
   - High user value
   - Architecture is ready
   - Low risk

**Total**: ~2,700 LoC, 2-3 weeks effort

### Phase 4b: Networking (v0.4.0)

Implement **Remote Sessions**:

1. Abstract `PiBridge` to trait
2. Implement `LocalBridge` (current code)
3. Implement `RemoteBridge` (new)
4. Add connection management UI

**Total**: ~2,000 LoC, 3-4 weeks effort

### Phase 4c: Extensibility (v0.5.0+)

Implement **Plugin System**:

**Recommendation**: Use WASM-based plugins instead of native libraries:

- **Safer**: Sandboxed execution
- **Cross-platform**: Works everywhere
- **Stable ABI**: WASM ABI is stable
- **Hot-reloadable**: Can reload without restart

```toml
[dependencies]
wasmtime = "18.0"    # WASM runtime
```

**Total**: ~1,500 LoC for WASM-based system

---

## Conclusion

### Did We Plan for Phase 4?

**Yes and no**:

✅ **What we planned for**:
- Layout system supports split panels (for code review)
- Panel abstraction is extensible (for file browser)
- State management is modular (for new state types)

❌ **What we didn't plan for**:
- Plugin system requires major architectural changes
- Remote sessions need networking layer we don't have
- Dynamic plugin loading needs runtime abstraction

### Can It Be Done Without Refactoring?

| Feature | Without Refactoring | With Minimal Refactoring |
|---------|---------------------|------------------------|
| Code Review Mode | ❌ No (need diff state) | ✅ Yes (50-100 LoC) |
| File Browser | ❌ No (need file state) | ✅ Yes (50-100 LoC) |
| Plugin System | ❌ **No way** | ⚠️ Needs major work |
| Remote Sessions | ❌ No (need bridge trait) | ✅ Yes (200-400 LoC) |

### Bottom Line

**We can implement 60% of Phase 4 with minimal refactoring.**

The architecture we built is solid for:
- Code review mode
- File browser
- New panel types

The architecture needs work for:
- Plugin system (major)
- Remote sessions (moderate)

---

*"Plan for what's next, but build for now."* 🦅
