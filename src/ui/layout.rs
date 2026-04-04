//! Layout management for HawkTUI.
//!
//! Supports multiple layout modes:
//! - Command Center: Full panels with sidebar
//! - Focus: Conversation only
//! - Split: Side-by-side code review

use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};

use crate::core::state::LayoutMode;

/// Layout regions for the UI.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    /// Header/status bar area.
    pub header: Rect,

    /// Main content area.
    pub main: Rect,

    /// Left sidebar (sessions, tools).
    pub sidebar: Option<Rect>,

    /// Conversation panel.
    pub conversation: Rect,

    /// Secondary panel (for split mode).
    pub secondary: Option<Rect>,

    /// Input area.
    pub input: Rect,

    /// Footer/status area.
    pub footer: Rect,
}

/// Layout manager.
#[derive(Debug)]
pub struct LayoutManager {
    /// Current layout mode.
    mode: LayoutMode,

    /// Sidebar width (percentage).
    sidebar_width: u16,

    /// Input height (lines).
    input_height: u16,
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self {
            mode: LayoutMode::CommandCenter,
            sidebar_width: 25,
            input_height: 3,
        }
    }
}

impl LayoutManager {
    /// Create a new layout manager.
    pub fn new(mode: LayoutMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }

    /// Set the layout mode.
    pub fn set_mode(&mut self, mode: LayoutMode) {
        self.mode = mode;
    }

    /// Get the current mode.
    pub const fn mode(&self) -> LayoutMode {
        self.mode
    }

    /// Toggle to the next layout mode.
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            LayoutMode::CommandCenter => LayoutMode::Focus,
            LayoutMode::Focus => LayoutMode::Split,
            LayoutMode::Split => LayoutMode::CommandCenter,
        };
    }

    /// Calculate layout for the given area.
    pub fn calculate(&self, area: Rect) -> Layout {
        match self.mode {
            LayoutMode::CommandCenter => self.command_center_layout(area),
            LayoutMode::Focus => self.focus_layout(area),
            LayoutMode::Split => self.split_layout(area),
        }
    }

    /// Command Center layout: sidebar + conversation + input.
    fn command_center_layout(&self, area: Rect) -> Layout {
        // Vertical: header, main, footer
        let vertical = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),                 // Header
                Constraint::Min(10),                   // Main
                Constraint::Length(self.input_height), // Input
                Constraint::Length(1),                 // Footer
            ])
            .split(area);

        let header = vertical[0];
        let main_area = vertical[1];
        let input = vertical[2];
        let footer = vertical[3];

        // Horizontal: sidebar + conversation
        let horizontal = RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(self.sidebar_width),
                Constraint::Percentage(100 - self.sidebar_width),
            ])
            .split(main_area);

        let sidebar = horizontal[0];
        let conversation = horizontal[1];

        Layout {
            header,
            main: main_area,
            sidebar: Some(sidebar),
            conversation,
            secondary: None,
            input,
            footer,
        }
    }

    /// Focus layout: conversation only.
    fn focus_layout(&self, area: Rect) -> Layout {
        let vertical = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),                 // Header
                Constraint::Min(10),                   // Conversation
                Constraint::Length(self.input_height), // Input
                Constraint::Length(1),                 // Footer
            ])
            .split(area);

        Layout {
            header: vertical[0],
            main: vertical[1],
            sidebar: None,
            conversation: vertical[1],
            secondary: None,
            input: vertical[2],
            footer: vertical[3],
        }
    }

    /// Split layout: two panels side by side.
    fn split_layout(&self, area: Rect) -> Layout {
        let vertical = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),                 // Header
                Constraint::Min(10),                   // Main
                Constraint::Length(self.input_height), // Input
                Constraint::Length(1),                 // Footer
            ])
            .split(area);

        let header = vertical[0];
        let main_area = vertical[1];
        let input = vertical[2];
        let footer = vertical[3];

        // Split main area horizontally
        let horizontal = RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_area);

        Layout {
            header,
            main: main_area,
            sidebar: None,
            conversation: horizontal[0],
            secondary: Some(horizontal[1]),
            input,
            footer,
        }
    }

    /// Adjust sidebar width.
    pub fn set_sidebar_width(&mut self, width: u16) {
        self.sidebar_width = width.clamp(15, 40);
    }

    /// Adjust input height.
    pub fn set_input_height(&mut self, height: u16) {
        self.input_height = height.clamp(1, 10);
    }
}

/// Calculate inner area with padding.
pub fn inner_area(area: Rect, padding: u16) -> Rect {
    Rect {
        x: area.x + padding,
        y: area.y + padding,
        width: area.width.saturating_sub(padding * 2),
        height: area.height.saturating_sub(padding * 2),
    }
}

/// Split an area into rows.
pub fn split_rows(area: Rect, heights: &[u16]) -> Vec<Rect> {
    let constraints: Vec<Constraint> = heights.iter().map(|&h| Constraint::Length(h)).collect();

    RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area)
        .to_vec()
}

/// Split an area into columns.
pub fn split_cols(area: Rect, widths: &[u16]) -> Vec<Rect> {
    let constraints: Vec<Constraint> = widths.iter().map(|&w| Constraint::Length(w)).collect();

    RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area)
        .to_vec()
}
