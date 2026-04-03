//! Tools panel for showing available and executing tools.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

use crate::core::state::ToolsState;
use crate::ui::themes::Theme;

/// Tools panel widget.
pub struct ToolsPanel<'a> {
    tools_state: &'a ToolsState,
    theme: &'a Theme,
    focused: bool,
}

impl<'a> ToolsPanel<'a> {
    pub fn new(tools_state: &'a ToolsState, theme: &'a Theme, focused: bool) -> Self {
        Self {
            tools_state,
            theme,
            focused,
        }
    }
}

impl Widget for ToolsPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_color = if self.focused {
            self.theme.border_focused()
        } else {
            self.theme.border()
        };

        let block = Block::default()
            .title(Span::styled(
                " 🔧 Tools ",
                Style::default()
                    .fg(self.theme.accent())
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(self.theme.borders.style.to_ratatui())
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(Theme::parse_color(&self.theme.panels.sidebar_bg)));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut items: Vec<ListItem> = Vec::new();

        // Executing tools first
        if !self.tools_state.executing.is_empty() {
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "Running:",
                Style::default()
                    .fg(self.theme.warning())
                    .add_modifier(Modifier::BOLD),
            )])));

            for tool in &self.tools_state.executing {
                let progress = tool
                    .progress
                    .map(|p| format!(" {:.0}%", p * 100.0))
                    .unwrap_or_default();

                items.push(ListItem::new(Line::from(vec![
                    Span::styled("⚡ ", Style::default().fg(self.theme.warning())),
                    Span::styled(&tool.name, Style::default().fg(self.theme.fg())),
                    Span::styled(progress, Style::default().fg(self.theme.muted())),
                ])));
            }

            items.push(ListItem::new(Line::raw("")));
        }

        // Available tools
        if !self.tools_state.available.is_empty() {
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "Available:",
                Style::default()
                    .fg(self.theme.muted())
                    .add_modifier(Modifier::BOLD),
            )])));

            for tool in &self.tools_state.available {
                let (icon, color) = if tool.enabled {
                    ("✓", self.theme.success())
                } else {
                    ("○", self.theme.muted())
                };

                items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("{icon} "), Style::default().fg(color)),
                    Span::styled(&tool.name, Style::default().fg(self.theme.fg())),
                ])));
            }
        }

        if items.is_empty() {
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "No tools loaded",
                Style::default().fg(self.theme.muted()),
            )])));
        }

        let list = List::new(items);
        list.render(inner, buf);
    }
}
