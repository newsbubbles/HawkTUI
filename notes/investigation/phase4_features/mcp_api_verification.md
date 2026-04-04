# MCP API Verification - pi_agent_rust Investigation

**Date**: 2026-04-03  
**Status**: Completed  
**Goal**: Verify pi_agent_rust's MCP API for HawkTUI integration

---

## Summary

**Key Finding**: MCP support in pi_agent_rust is handled through **JavaScript extensions**, not a separate native Rust MCP API.

---

## Investigation Results

### 1. MCP SDK Implementation

**File**: `../pi_agent_rust_source/src/extensions_js.rs:8164`

```javascript
// pi provides STUB implementations of the MCP SDK for JS extensions
let mcp_client = r"
export class Client {
  constructor(_opts = {}) {}
  async connect(_transport) {}
  async listTools() { return { tools: [] }; }
  async listResources() { return { resources: [] }; }
  async callTool(_name, _args) { return { content: [] }; }
  async close() {}
}
";

let mcp_transport = r"
export class StdioClientTransport {
  constructor(_opts = {}) {}
  async start() {}
  async close() {}
}
";
```

**Interpretation**: 
- pi provides stub implementations of `@modelcontextprotocol/sdk` for JavaScript extensions
- The actual MCP server communication happens **inside extensions**
- Extensions can use MCP servers to provide tools/resources to the agent

---

### 2. Extension Manager API

**File**: `../pi_agent_rust_source/src/sdk.rs:44`

```rust
// SDK exports ExtensionManager
pub use crate::extensions::{ExtensionManager, ExtensionPolicy, ExtensionRegion};
```

**File**: `../pi_agent_rust_source/src/sdk.rs:1249`

```rust
impl AgentSessionHandle {
    /// Return a reference to the extension manager (if extensions are loaded).
    pub fn extension_manager(&self) -> Option<&ExtensionManager> {
        self.session
            .extensions
            .as_ref()
            .map(ExtensionRegion::manager)
    }
}
```

**Available Methods** (from `extensions.rs`):

```rust
impl ExtensionManager {
    // List loaded extensions
    pub fn extension_count(&self) -> usize { ... }
    
    // List tools from all extensions
    pub fn list_tools(&self) -> Vec<Value> { ... }
    
    // List commands from extensions
    pub fn list_commands(&self) -> Vec<Value> { ... }
    
    // List shortcuts from extensions
    pub fn list_shortcuts(&self) -> Vec<Value> { ... }
    
    // List event hooks
    pub fn list_event_hooks(&self) -> Vec<String> { ... }
    
    // Load new extensions (async)
    pub async fn load_js_extensions(&self, specs: Vec<JsExtensionLoadSpec>) -> Result<()>;
    pub async fn load_native_extensions(&self, specs: Vec<NativeRustExtensionLoadSpec>) -> Result<()>;
    
    // Check capabilities
    pub fn has_tool(&self, name: &str) -> bool { ... }
    pub fn has_command(&self, name: &str) -> bool { ... }
}
```

---

### 3. Extension Loading Configuration

**File**: `../pi_agent_rust_source/src/sdk.rs:281`

```rust
pub struct SessionOptions {
    pub provider: Option<String>,
    pub model: Option<String>,
    // ...
    
    /// Paths to extension files/directories to load
    pub extension_paths: Vec<PathBuf>,
    
    /// Extension policy ("safe", "balanced", "permissive")
    pub extension_policy: Option<String>,
    
    // ...
}
```

**Key Insight**: Extensions are configured at **session creation time** via `extension_paths`.

---

### 4. Extension Types

pi_agent_rust supports three extension types:

1. **JavaScript Extensions** (`.js`/`.ts`)
   - Can use MCP SDK stubs
   - Most flexible for MCP integration
   - Loaded via `load_js_extensions()`

2. **Native Rust Extensions** (`.native.json`)
   - Pre-compiled Rust code
   - Fast, but no dynamic MCP
   - Loaded via `load_native_extensions()`

3. **WASM Extensions** (`.wasm`)
   - Sandboxed WebAssembly modules
   - Feature-gated: `#[cfg(feature = "wasm-host")]`
   - Loaded via `load_wasm_extensions()`

---

## What This Means for HawkTUI

### ❌ Original Plan Assumptions (INCORRECT)

1. ~~pi has a native MCP server management API~~
2. ~~We can call `list_mcp_servers()` directly~~
3. ~~We can add/remove MCP servers at runtime~~

### ✅ Correct Understanding

1. **MCP servers are managed by extensions, not pi core**
   - Extensions implement MCP client logic
   - pi provides stub MCP SDK for extensions to use

2. **We CAN list extensions and their tools**
   - `extension_manager().list_tools()`
   - `extension_manager().list_commands()`
   - `extension_manager().extension_count()`

3. **We CAN load new extensions at runtime**
   - `load_js_extensions(specs)`
   - Requires knowing the extension path

4. **We CANNOT directly manage MCP servers**
   - No `add_mcp_server()` API
   - No `restart_mcp_server()` API
   - MCP servers are internal to extensions

---

## Revised Implementation Options

### Option A: Extension Manager UI (Recommended)

Build a UI for **managing extensions**, not MCP servers directly.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ 🦅 HawkTUI │ Extensions                                             [E] ? │
├──────────────────────────────────────────────────────────────────────────────┤
│  Extensions (3 loaded)                                                        │
│                                                                              │
│  ● filesystem-tools.js     5 tools     File system operations               │
│  ● github-extension.js     8 tools     GitHub API integration                │
│  ● postgres-client.js      3 tools     Database queries                      │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ [l]oad  [r]eload  [t]ools  [q]uit                                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Features**:
- List loaded extensions
- Show tools/commands from each extension
- Load new extensions (via path)
- Reload extensions

**Pros**:
- Works with pi's actual API
- Still provides extensibility visibility
- Simpler implementation

**Cons**:
- No direct MCP server management
- Requires users to know extension paths

---

### Option B: MCP Extension Convention

Create a **convention** for MCP-focused extensions.

**Idea**: Define a standard extension format that:
1. Declares MCP servers in `extension.json`
2. Exposes MCP server management commands
3. Provides tools for starting/stopping MCP servers

**Example extension.json**:
```json
{
  "name": "mcp-manager",
  "version": "1.0.0",
  "mcpServers": {
    "filesystem": {
      "command": "mcp-server-filesystem",
      "args": ["--root", "./src"]
    },
    "github": {
      "command": "mcp-server-github",
      "env": {
        "GITHUB_TOKEN": "${GITHUB_TOKEN}"
      }
    }
  }
}
```

**Pros**:
- Standardized way to manage MCP servers
- Works within pi's extension system
- Could be adopted by pi itself

**Cons**:
- Requires building a custom MCP manager extension
- More complex initial implementation

---

### Option C: Configuration-Based Approach

Manage MCP servers through **session configuration**.

**Idea**: 
1. HawkTUI reads a config file with MCP server definitions
2. Generates extension files for each MCP server
3. Loads the generated extensions at startup

**Pros**:
- Declarative configuration
- Hot-reload by regenerating extensions

**Cons**:
- Requires custom extension generator
- Not directly integrated with pi

---

## Recommendation

### Phase 4 Revised: Extension Manager UI

**Scope**: Build a TUI panel for **extension management**, not MCP server management.

| Feature | Priority | Effort |
|---------|----------|--------|
| List loaded extensions | HIGH | Low |
| Show tools per extension | HIGH | Low |
| Show commands per extension | MEDIUM | Low |
| Load extension by path | MEDIUM | Medium |
| Reload all extensions | LOW | Medium |

**Estimated LoC**: 300-500 (reduced from 450-750!)

### Future Enhancement: MCP Manager Extension

Later, create a JavaScript extension that:
1. Manages MCP server lifecycle
2. Exposes management commands
3. Integrates with HawkTUI's extension panel

This would give us full MCP management capabilities **within** pi's extension system.

---

## Code Examples

### Using ExtensionManager in HawkTUI

```rust
// src/providers/pi_bridge.rs

impl PiBridge {
    /// List all loaded extensions.
    pub fn list_extensions(&self) -> Vec<ExtensionInfo> {
        let handle = self.handle.as_ref()?;
        let manager = handle.extension_manager()?;
        
        // Get tool count
        let tools = manager.list_tools();
        let commands = manager.list_commands();
        
        // Build extension info
        // Note: pi doesn't expose extension names directly, 
        // we'll need to infer from tools/commands
        
        vec![]
    }
    
    /// Get all tools from extensions.
    pub fn list_extension_tools(&self) -> Vec<ToolInfo> {
        let handle = self.handle.as_ref()?;
        let manager = handle.extension_manager()?;
        
        manager.list_tools()
            .into_iter()
            .map(|t| ToolInfo {
                name: t["name"].as_str().unwrap_or("unknown").to_string(),
                description: t["description"].as_str().unwrap_or("").to_string(),
            })
            .collect()
    }
}
```

### Extension State for HawkTUI

```rust
// src/core/state.rs

#[derive(Debug, Default)]
pub struct ExtensionState {
    /// Number of loaded extensions.
    pub extension_count: usize,
    
    /// All tools from extensions.
    pub tools: Vec<ToolInfo>,
    
    /// All commands from extensions.
    pub commands: Vec<CommandInfo>,
    
    /// Selected tool/command index.
    pub selected_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub source_extension: Option<String>,
}
```

---

## Action Items

### Immediate (v0.3.0)

1. ✅ **Update implementation plan** - Focus on Extension Manager, not MCP Manager
2. ⬜ **Add ExtensionState** to `AppState`
3. ⬜ **Add extension methods** to `PiBridge`
4. ⬜ **Create ExtensionPanel** UI

### Future (v0.4.0+)

1. ⬜ **Create MCP Manager Extension** (JavaScript)
2. ⬜ **Add hot-reload support**
3. ⬜ **Add extension configuration UI**

---

## Conclusion

The investigation revealed that **MCP support in pi_agent_rust is extension-based**, not a native API. This is actually a **better architecture** because:

1. **Consistency**: MCP servers are just extensions, not a special case
2. **Security**: Extensions run in sandboxed JS runtime
3. **Flexibility**: Any extension can use MCP, not just "MCP servers"

HawkTUI should build an **Extension Manager UI** first, then consider a **MCP Manager Extension** for advanced use cases.

---

*"Investigation before implementation saves rewriting."* 🦅