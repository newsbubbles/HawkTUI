//! Integration tests for HawkTUI.
//!
//! These tests verify that the core components work together correctly.

use hawktui::core::{
    commands::{find_command, get_completions, parse_command},
    events::{map_key_to_action, Action, AgentEvent, Event},
    keybindings::{parse_action_string, parse_key_string, KeyBindings},
    state::{
        AppMode, AppState, ConnectionStatus, InputState, LayoutMode, Message,
        MessageRole, Overlay, Panel, StatusInfo,
    },
};
use crossterm::event::{KeyCode, KeyModifiers};

// ============================================================================
// STATE TESTS
// ============================================================================

#[test]
fn test_app_state_default() {
    let state = AppState::new();
    
    assert_eq!(state.mode, AppMode::Normal);
    assert_eq!(state.layout, LayoutMode::CommandCenter);
    assert_eq!(state.active_panel, Panel::Input);
    assert!(!state.should_quit);
    assert!(state.overlay.is_none());
    assert!(!state.is_streaming());
}

#[test]
fn test_layout_mode_from_str() {
    assert_eq!(LayoutMode::from_str("focus"), LayoutMode::Focus);
    assert_eq!(LayoutMode::from_str("FOCUS"), LayoutMode::Focus);
    assert_eq!(LayoutMode::from_str("split"), LayoutMode::Split);
    assert_eq!(LayoutMode::from_str("command-center"), LayoutMode::CommandCenter);
    assert_eq!(LayoutMode::from_str("unknown"), LayoutMode::CommandCenter);
}

#[test]
fn test_message_creation() {
    let user_msg = Message::user("Hello, world!");
    assert_eq!(user_msg.role, MessageRole::User);
    assert_eq!(user_msg.content, "Hello, world!");
    assert!(!user_msg.is_streaming);
    
    let assistant_msg = Message::assistant("Hi there!");
    assert_eq!(assistant_msg.role, MessageRole::Assistant);
    assert!(!assistant_msg.is_streaming);
    
    let streaming_msg = Message::assistant_streaming();
    assert_eq!(streaming_msg.role, MessageRole::Assistant);
    assert!(streaming_msg.is_streaming);
    assert!(streaming_msg.content.is_empty());
}

#[test]
fn test_status_info_default() {
    let status = StatusInfo::default();
    
    assert_eq!(status.model, "claude-sonnet-4-20250514");
    assert_eq!(status.provider, "anthropic");
    assert_eq!(status.total_tokens, 0);
    assert_eq!(status.cost, 0.0);
    assert_eq!(status.connection, ConnectionStatus::Disconnected);
}

#[test]
fn test_input_state_default() {
    let input = InputState::default();
    
    assert!(input.text.is_empty());
    assert_eq!(input.cursor, 0);
    assert!(input.history.is_empty());
    assert!(!input.vim_mode);
}

// ============================================================================
// COMMANDS TESTS
// ============================================================================

#[test]
fn test_commands_defined() {
    // Verify all expected commands exist
    let expected = [
        "help", "clear", "exit", "model", "provider", "session",
        "theme", "layout", "export", "context", "tools", "system",
        "branch", "compact", "tokens", "cost", "vim",
    ];
    
    for name in expected {
        assert!(
            find_command(name).is_some(),
            "Command '{}' should be defined",
            name
        );
    }
}

#[test]
fn test_command_aliases() {
    // Test that aliases work
    assert!(find_command("h").is_some()); // help
    assert!(find_command("?").is_some()); // help
    assert!(find_command("q").is_some()); // quit
    assert!(find_command("cls").is_some()); // clear
    assert!(find_command("m").is_some()); // model
}

#[test]
fn test_parse_command_with_args() {
    let parsed = parse_command("/model gpt-4o").unwrap();
    assert_eq!(parsed.name, "model");
    assert_eq!(parsed.args, vec!["gpt-4o"]);
    
    let parsed = parse_command("/session new my-session").unwrap();
    assert_eq!(parsed.name, "session");
    assert_eq!(parsed.args, vec!["new", "my-session"]);
}

#[test]
fn test_parse_command_quoted_args() {
    let parsed = parse_command("/session new \"my session with spaces\"").unwrap();
    assert_eq!(parsed.name, "session");
    assert_eq!(parsed.args, vec!["new", "my session with spaces"]);
}

#[test]
fn test_command_completions() {
    let completions = get_completions("mo");
    assert_eq!(completions.len(), 1);
    assert_eq!(completions[0].name, "model");
    
    let completions = get_completions("c");
    assert!(completions.len() >= 3); // clear, compact, context, cost
}

// ============================================================================
// EVENTS TESTS
// ============================================================================

#[test]
fn test_event_creation() {
    let key_event = crossterm::event::KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
    let event = Event::key(key_event);
    
    match event {
        Event::Terminal(_) => {},
        _ => panic!("Expected Terminal event"),
    }
    
    let resize_event = Event::resize(80, 24);
    match resize_event {
        Event::Terminal(_) => {},
        _ => panic!("Expected Terminal event"),
    }
    
    let text_event = Event::text_delta("Hello");
    match text_event {
        Event::Agent(AgentEvent::TextDelta { text }) => {
            assert_eq!(text, "Hello");
        },
        _ => panic!("Expected Agent TextDelta event"),
    }
}

#[test]
fn test_key_to_action_normal_mode() {
    let key = crossterm::event::KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
    let action = map_key_to_action(key, AppMode::Normal, None);
    assert_eq!(action, Action::ToggleHelp);
    
    let key = crossterm::event::KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE);
    let action = map_key_to_action(key, AppMode::Normal, None);
    assert_eq!(action, Action::ToggleLayout);
    
    let key = crossterm::event::KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    let action = map_key_to_action(key, AppMode::Normal, None);
    assert_eq!(action, Action::NextPanel);
}

#[test]
fn test_key_to_action_ctrl_shortcuts() {
    let key = crossterm::event::KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    let action = map_key_to_action(key, AppMode::Normal, None);
    assert_eq!(action, Action::Quit);
    
    let key = crossterm::event::KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL);
    let action = map_key_to_action(key, AppMode::Normal, None);
    assert_eq!(action, Action::ClearScreen);
    
    let key = crossterm::event::KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
    let action = map_key_to_action(key, AppMode::Normal, None);
    assert_eq!(action, Action::OpenCommandPalette);
}

#[test]
fn test_key_to_action_streaming_mode() {
    // Ctrl+C should cancel in streaming mode, not quit
    let key = crossterm::event::KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    let action = map_key_to_action(key, AppMode::Streaming, None);
    assert_eq!(action, Action::Cancel);
}

#[test]
fn test_key_to_action_with_overlay() {
    let overlay = Overlay::Help;
    
    // Esc should close overlay
    let key = crossterm::event::KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    let action = map_key_to_action(key, AppMode::Normal, Some(&overlay));
    assert_eq!(action, Action::CloseOverlay);
}

// ============================================================================
// KEYBINDINGS TESTS
// ============================================================================

#[test]
fn test_keybindings_default() {
    let bindings = KeyBindings::default();
    
    assert!(bindings.global.contains_key("ctrl+c"));
    assert!(bindings.global.contains_key("f1"));
    assert!(bindings.normal.contains_key("j"));
    assert!(bindings.insert.contains_key("esc"));
}

#[test]
fn test_parse_key_string() {
    let (code, mods) = parse_key_string("ctrl+c").unwrap();
    assert_eq!(code, KeyCode::Char('c'));
    assert!(mods.contains(KeyModifiers::CONTROL));
    
    let (code, mods) = parse_key_string("f1").unwrap();
    assert_eq!(code, KeyCode::F(1));
    assert!(mods.is_empty());
    
    let (code, mods) = parse_key_string("shift+tab").unwrap();
    assert_eq!(code, KeyCode::Tab);
    assert!(mods.contains(KeyModifiers::SHIFT));
    
    let (code, _) = parse_key_string("enter").unwrap();
    assert_eq!(code, KeyCode::Enter);
    
    let (code, _) = parse_key_string("esc").unwrap();
    assert_eq!(code, KeyCode::Esc);
}

#[test]
fn test_parse_action_string() {
    assert_eq!(parse_action_string("quit"), Some(Action::Quit));
    assert_eq!(parse_action_string("send"), Some(Action::SendMessage));
    assert_eq!(parse_action_string("help"), Some(Action::ToggleHelp));
    assert_eq!(parse_action_string("toggle_layout"), Some(Action::ToggleLayout));
    assert_eq!(parse_action_string("unknown_action"), None);
}

// ============================================================================
// THEME TESTS
// ============================================================================

use hawktui::ui::themes::{BorderStyle, Theme};
use ratatui::style::Color;

#[test]
fn test_theme_by_name() {
    let dark = Theme::by_name("hawk-dark");
    assert_eq!(dark.meta.name, "Hawk Dark");
    
    let light = Theme::by_name("hawk-light");
    assert_eq!(light.meta.name, "Hawk Light");
    
    let cyber = Theme::by_name("cyberpunk");
    assert_eq!(cyber.meta.name, "Cyberpunk");
    
    // Unknown defaults to dark
    let unknown = Theme::by_name("unknown");
    assert_eq!(unknown.meta.name, "Hawk Dark");
}

#[test]
fn test_theme_parse_color() {
    let color = Theme::parse_color("#ff0000");
    assert_eq!(color, Color::Rgb(255, 0, 0));
    
    let color = Theme::parse_color("#00ff00");
    assert_eq!(color, Color::Rgb(0, 255, 0));
    
    let color = Theme::parse_color("#0000ff");
    assert_eq!(color, Color::Rgb(0, 0, 255));
    
    // Invalid returns Reset
    let color = Theme::parse_color("invalid");
    assert_eq!(color, Color::Reset);
}

#[test]
fn test_border_style_to_ratatui() {
    use ratatui::widgets::BorderType;
    
    assert_eq!(BorderStyle::Rounded.to_ratatui(), BorderType::Rounded);
    assert_eq!(BorderStyle::Sharp.to_ratatui(), BorderType::Plain);
    assert_eq!(BorderStyle::Double.to_ratatui(), BorderType::Double);
    assert_eq!(BorderStyle::Thick.to_ratatui(), BorderType::Thick);
    assert_eq!(BorderStyle::None.to_ratatui(), BorderType::Plain);
}

// ============================================================================
// LAYOUT TESTS
// ============================================================================

use hawktui::ui::layout::LayoutManager;
use ratatui::layout::Rect;

#[test]
fn test_layout_manager_modes() {
    let mut manager = LayoutManager::new(LayoutMode::CommandCenter);
    assert_eq!(manager.mode(), LayoutMode::CommandCenter);
    
    manager.toggle_mode();
    assert_eq!(manager.mode(), LayoutMode::Focus);
    
    manager.toggle_mode();
    assert_eq!(manager.mode(), LayoutMode::Split);
    
    manager.toggle_mode();
    assert_eq!(manager.mode(), LayoutMode::CommandCenter);
}

#[test]
fn test_layout_calculation() {
    let manager = LayoutManager::new(LayoutMode::CommandCenter);
    let area = Rect::new(0, 0, 100, 50);
    let layout = manager.calculate(area);
    
    // Header should be at top
    assert_eq!(layout.header.y, 0);
    assert_eq!(layout.header.height, 1);
    
    // Footer should be at bottom
    assert_eq!(layout.footer.y, 49);
    assert_eq!(layout.footer.height, 1);
    
    // Sidebar should exist in command center mode
    assert!(layout.sidebar.is_some());
}

#[test]
fn test_focus_layout_no_sidebar() {
    let manager = LayoutManager::new(LayoutMode::Focus);
    let area = Rect::new(0, 0, 100, 50);
    let layout = manager.calculate(area);
    
    // Focus mode should not have sidebar
    assert!(layout.sidebar.is_none());
}

#[test]
fn test_split_layout_secondary() {
    let manager = LayoutManager::new(LayoutMode::Split);
    let area = Rect::new(0, 0, 100, 50);
    let layout = manager.calculate(area);
    
    // Split mode should have secondary panel
    assert!(layout.secondary.is_some());
    assert!(layout.sidebar.is_none());
}

// ============================================================================
// WIDGET TESTS
// ============================================================================

use hawktui::ui::widgets::{next_frame, Spinner, SPINNER_FRAMES};

#[test]
fn test_spinner_frames() {
    let spinner = Spinner::new(0);
    assert_eq!(spinner.current_frame(), SPINNER_FRAMES[0]);
    
    let spinner = Spinner::new(5);
    assert_eq!(spinner.current_frame(), SPINNER_FRAMES[5]);
    
    // Test wrapping
    let spinner = Spinner::new(SPINNER_FRAMES.len());
    assert_eq!(spinner.current_frame(), SPINNER_FRAMES[0]);
}

#[test]
fn test_next_frame() {
    assert_eq!(next_frame(0, 10), 1);
    assert_eq!(next_frame(9, 10), 0); // Wraps
    assert_eq!(next_frame(5, 10), 6);
}

// ============================================================================
// ERROR TESTS
// ============================================================================

use hawktui::core::error::Error;

#[test]
fn test_error_creation() {
    let err = Error::terminal("test error");
    assert!(err.to_string().contains("Terminal error"));
    
    let err = Error::config("bad config");
    assert!(err.to_string().contains("Configuration error"));
    
    let err = Error::agent("agent failed");
    assert!(err.to_string().contains("Agent error"));
}

// ============================================================================
// PROVIDER TESTS
// ============================================================================

use hawktui::providers::PiBridge;

#[test]
fn test_pi_bridge_creation() {
    let bridge = PiBridge::new(None, None);
    assert_eq!(bridge.model(), "claude-sonnet-4-20250514");
    assert_eq!(bridge.provider(), "anthropic");
    assert!(!bridge.is_connected());
}

#[test]
fn test_pi_bridge_custom_model() {
    let bridge = PiBridge::new(Some("gpt-4o".to_string()), Some("openai".to_string()));
    assert_eq!(bridge.model(), "gpt-4o");
    assert_eq!(bridge.provider(), "openai");
}

#[test]
fn test_pi_bridge_set_model() {
    let mut bridge = PiBridge::new(None, None);
    bridge.set_model("new-model");
    assert_eq!(bridge.model(), "new-model");
    
    bridge.set_provider("new-provider");
    assert_eq!(bridge.provider(), "new-provider");
}

#[test]
fn test_pi_bridge_available_tools() {
    let bridge = PiBridge::new(None, None);
    let tools = bridge.available_tools();
    
    assert!(!tools.is_empty());
    assert!(tools.iter().any(|t| t.name == "read_file"));
    assert!(tools.iter().any(|t| t.name == "write_file"));
    assert!(tools.iter().any(|t| t.name == "bash"));
}

#[tokio::test]
async fn test_pi_bridge_connect() {
    let mut bridge = PiBridge::new(None, None);
    assert!(!bridge.is_connected());
    
    bridge.connect().await.unwrap();
    assert!(bridge.is_connected());
}
