//! Keybinding configuration for HawkTUI.

use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::events::Action;

/// Keybinding configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    /// Global keybindings (work in any mode).
    #[serde(default)]
    pub global: HashMap<String, String>,

    /// Normal mode keybindings.
    #[serde(default)]
    pub normal: HashMap<String, String>,

    /// Insert mode keybindings.
    #[serde(default)]
    pub insert: HashMap<String, String>,

    /// Command mode keybindings.
    #[serde(default)]
    pub command: HashMap<String, String>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            global: default_global_bindings(),
            normal: default_normal_bindings(),
            insert: default_insert_bindings(),
            command: default_command_bindings(),
        }
    }
}

fn default_global_bindings() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("ctrl+c".to_string(), "quit".to_string());
    map.insert("ctrl+q".to_string(), "quit".to_string());
    map.insert("ctrl+l".to_string(), "clear".to_string());
    map.insert("ctrl+p".to_string(), "command_palette".to_string());
    map.insert("ctrl+s".to_string(), "session_picker".to_string());
    map.insert("ctrl+h".to_string(), "help".to_string());
    map.insert("ctrl+enter".to_string(), "send".to_string());
    map.insert("f1".to_string(), "help".to_string());
    map.insert("f2".to_string(), "toggle_layout".to_string());
    map
}

fn default_normal_bindings() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("i".to_string(), "insert_mode".to_string());
    map.insert(":".to_string(), "command_mode".to_string());
    map.insert("/".to_string(), "search".to_string());
    map.insert("j".to_string(), "scroll_down".to_string());
    map.insert("k".to_string(), "scroll_up".to_string());
    map.insert("g".to_string(), "scroll_top".to_string());
    map.insert("G".to_string(), "scroll_bottom".to_string());
    map.insert("tab".to_string(), "next_panel".to_string());
    map.insert("shift+tab".to_string(), "prev_panel".to_string());
    map
}

fn default_insert_bindings() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("esc".to_string(), "normal_mode".to_string());
    map
}

fn default_command_bindings() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("esc".to_string(), "cancel".to_string());
    map.insert("enter".to_string(), "execute".to_string());
    map.insert("tab".to_string(), "autocomplete".to_string());
    map
}

/// Parse a key string into `KeyCode` and `KeyModifiers`.
pub fn parse_key_string(s: &str) -> Option<(KeyCode, KeyModifiers)> {
    let parts: Vec<&str> = s.split('+').collect();
    let mut modifiers = KeyModifiers::empty();
    let mut key_part = "";

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
            "alt" => modifiers |= KeyModifiers::ALT,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            "super" | "meta" | "cmd" => modifiers |= KeyModifiers::SUPER,
            _ => key_part = part,
        }
    }

    let code = match key_part.to_lowercase().as_str() {
        "enter" | "return" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "backspace" | "bs" => KeyCode::Backspace,
        "delete" | "del" => KeyCode::Delete,
        "insert" | "ins" => KeyCode::Insert,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" | "pgup" => KeyCode::PageUp,
        "pagedown" | "pgdn" => KeyCode::PageDown,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "space" => KeyCode::Char(' '),
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        s if s.len() == 1 => KeyCode::Char(s.chars().next().unwrap()),
        _ => return None,
    };

    Some((code, modifiers))
}

/// Convert an action string to an `Action`.
pub fn parse_action_string(s: &str) -> Option<Action> {
    Some(match s.to_lowercase().as_str() {
        "quit" | "exit" => Action::Quit,
        "send" | "submit" => Action::SendMessage,
        "cancel" | "abort" => Action::Cancel,
        "clear" => Action::ClearScreen,
        "help" => Action::ToggleHelp,
        "command_palette" => Action::OpenCommandPalette,
        "session_picker" => Action::OpenSessionPicker,
        "model_picker" => Action::OpenModelPicker,
        "toggle_layout" => Action::ToggleLayout,
        "next_panel" => Action::NextPanel,
        "prev_panel" => Action::PrevPanel,
        "scroll_up" => Action::ScrollUp(1),
        "scroll_down" => Action::ScrollDown(1),
        "scroll_top" => Action::ScrollToTop,
        "scroll_bottom" => Action::ScrollToBottom,
        "new_session" => Action::NewSession,
        "continue_session" => Action::ContinueSession,
        "copy" => Action::Copy,
        "paste" => Action::Paste,
        "undo" => Action::Undo,
        "redo" => Action::Redo,
        "toggle_vim" => Action::ToggleVimMode,
        _ => return None,
    })
}
