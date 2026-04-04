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
use crate::ui::widgets::Spinner;

/// Tools panel widget.
pub struct ToolsPanel<'a> {
    tools_state: &'a ToolsState,
    selected_index: Option<usize>,
    theme: &'a Theme,
    focused: bool,
    frame: usize,
}

impl<'a> ToolsPanel<'a> {
    pub fn new(
        tools_state: &'a ToolsState,
        selected_index: Option<usize>,
        theme: &'a Theme,
        focused: bool,
        frame: usize,
    ) -> Self {
        Self {
            tools_state,
            selected_index,
            theme,
            focused,
            frame,
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

                // Use animated spinner instead of static icon
                let spinner = Spinner::new(self.frame);
                let spinner_char = spinner.current_frame();

                items.push(ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{spinner_char} "),
                        Style::default().fg(self.theme.warning()),
                    ),
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

            // Note: header_offset would be used for scroll calculations if needed
            // let _header_offset = if self.tools_state.executing.is_empty() { 1 } else {
            //     self.tools_state.executing.len() + 3
            // };

            for (idx, tool) in self.tools_state.available.iter().enumerate() {
                let is_selected = self.focused && self.selected_index == Some(idx);

                let (icon, color) = if tool.enabled {
                    ("✓", self.theme.success())
                } else {
                    ("○", self.theme.muted())
                };

                let selector = if is_selected { "▶ " } else { "  " };
                let name_style = if is_selected {
                    Style::default()
                        .fg(self.theme.accent())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.fg())
                };

                items.push(ListItem::new(Line::from(vec![
                    Span::styled(selector, Style::default().fg(self.theme.accent())),
                    Span::styled(format!("{icon} "), Style::default().fg(color)),
                    Span::styled(&tool.name, name_style),
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
