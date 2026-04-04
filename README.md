<p align="center">
  <img src="https://raw.githubusercontent.com/hawktui/hawktui/main/assets/banner.png" alt="HawkTUI Banner" width="800"/>
</p>

<h1 align="center">🦅 HawkTUI</h1>

<p align="center">
  <strong>"Spit on that thang!"</strong> 🦅
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#usage">Usage</a> •
  <a href="#keyboard-shortcuts">Shortcuts</a> •
  <a href="#themes">Themes</a> •
  <a href="#configuration">Configuration</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/rust-2024%20edition-orange?logo=rust" alt="Rust 2024">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT">
  <img src="https://img.shields.io/badge/unsafe-forbidden-brightgreen" alt="No Unsafe Code">
</p>

---

**HawkTUI** is a premium terminal user interface for [pi_agent_rust](https://github.com/Dicklesworthstone/pi_agent_rust), the high-performance AI coding agent. It transforms your terminal into a powerful command center for AI-assisted development.

```
┌────────────────────────────────────────────────────────────────────────┐
│ 🦅 HawkTUI v0.1.0 │ claude-sonnet-4-20250514 │ 12.4k tokens │ $0.042 │ ⚡ │
├────────────────────┬───────────────────────────────────────────────────┤
│ 📁 Sessions        │ 💬 Conversation                                   │
│ ● refactor-auth    │                                                    │
│ ○ debug-parser     │ 👤 You: Can you help me optimize this function?   │
│ ○ feature-xyz      │                                                    │
│                    │ 🤖 Assistant: I'd be happy to help! Let me        │
│ 🔧 Tools           │ analyze the function and suggest optimizations... │
│ ✓ read_file        │                                                    │
│ ✓ write_file       │ ```rust                                           │
│ ⚠ bash             │ fn optimized() {                                   │
│ ✓ glob             │     items.iter().filter(|x| x.valid()).collect()  │
│                    │ }                                                  │
│                    │ ```                                                │
├────────────────────┴───────────────────────────────────────────────────┤
│ > Type your message... (Ctrl+Enter to send)                     📎 │
└────────────────────────────────────────────────────────────────────────┘
```

## Features

🎯 **Multi-Panel Layout** - See your conversations, sessions, tools, and context all at once

⚡ **Real-Time Streaming** - Watch AI responses appear token by token with smooth animations

📁 **Session Management** - Switch between sessions, branch conversations, export history

🔧 **Tool Dashboard** - Monitor tool executions with progress indicators

🎨 **Beautiful Themes** - Hawk Dark, Hawk Light, Cyberpunk, and custom themes via TOML

⌨️ **Keyboard-First** - Every action accessible without touching the mouse

🦅 **Zero Unsafe Code** - Built with Rust's safety guarantees

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/hawktui/hawktui.git
cd hawktui

# Build and install
cargo install --path .
```

### Prerequisites

- Rust 1.85+ (2024 edition)
- A terminal with 256-color or true-color support
- API key for your preferred AI provider (Anthropic, OpenAI, etc.)

## Usage

```bash
# Start HawkTUI
hawk

# Start with a message
hawk "Help me refactor this function"

# Continue last session
hawk --continue

# Use a specific model
hawk --model gpt-4o

# Use a specific theme
hawk --theme cyberpunk

# Focus mode (conversation only)
hawk --layout focus
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+Enter` | Send message |
| `Ctrl+C` | Cancel/Quit |
| `Ctrl+L` | Clear screen |
| `Ctrl+P` | Command palette |
| `Ctrl+S` | Session picker |
| `F1` | Toggle help |
| `F2` | Toggle layout |
| `Tab` | Next panel |
| `Shift+Tab` | Previous panel |
| `PageUp/Down` | Scroll conversation |
| `Esc` | Close overlay |

## Slash Commands

| Command | Description |
|---------|-------------|
| `/help` | Show help |
| `/clear` | Clear conversation |
| `/model <name>` | Switch model |
| `/session new <name>` | Create session |
| `/session list` | List sessions |
| `/layout <mode>` | Switch layout (command-center, focus, split) |
| `/theme <name>` | Switch theme |
| `/vim` | Toggle vim mode |
| `/exit` | Exit HawkTUI |

## Themes

HawkTUI comes with three built-in themes:

- **Hawk Dark** (default) - Sleek, professional dark theme
- **Hawk Light** - Clean light theme for bright environments
- **Cyberpunk** - Neon-soaked terminal aesthetics

### Custom Themes

Create your own theme in `~/.config/hawktui/themes/`:

```toml
# my_theme.toml
[meta]
name = "My Theme"
author = "Your Name"
version = "1.0.0"

[colors]
background = "#1a1b26"
foreground = "#a9b1d6"
accent = "#7aa2f7"
# ... see themes/hawk_dark.toml for full example
```

## Layout Modes

### Command Center (default)
Full dashboard with sidebar panels for sessions and tools.

### Focus
Minimal layout with just the conversation - maximum screen real estate for code.

### Split
Side-by-side panels for code review and comparison.

## Configuration

HawkTUI looks for configuration in:
- `~/.config/hawktui/config.toml`
- `$HAWKTUI_CONFIG` environment variable

```toml
# config.toml
[general]
theme = "hawk-dark"
layout = "command-center"
vim_mode = false

[agent]
default_model = "claude-sonnet-4-20250514"
default_provider = "anthropic"

[keybindings]
# Custom keybindings (see docs for format)
```

## Architecture

HawkTUI integrates with `pi_agent_rust` at the library level:

```
HawkTUI
├── UI Layer (ratatui)
│   ├── Panels (conversation, sessions, tools)
│   ├── Widgets (streaming, spinners)
│   └── Themes
├── Core Layer
│   ├── State Management
│   ├── Event Handling
│   └── Commands
└── Integration Layer
    └── pi_agent_rust (library)
        ├── Agent
        ├── Sessions
        ├── Providers
        └── Tools
```

## Development

```bash
# Run in development
cargo run

# Run with logging
RUST_LOG=hawktui=debug cargo run

# Run tests
cargo test

# Run clippy
cargo clippy

# Format code
cargo fmt
```

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [pi_agent_rust](https://github.com/Dicklesworthstone/pi_agent_rust) - The powerful AI agent we wrap
- [ratatui](https://github.com/ratatui-org/ratatui) - The TUI framework that makes this possible
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal library

---

<p align="center">
  <em>"Spit on that thang!"</em> 🦅
</p>
