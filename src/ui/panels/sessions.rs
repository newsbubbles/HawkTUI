//! Sessions panel for listing and managing sessions.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Widget, StatefulWidget},
};

use crate::core::state::SessionInfo;
use crate::ui::themes::Theme;

/// Sessions panel widget.
pub struct SessionsPanel<'a> {
    sessions: &'a [SessionInfo],
    selected: Option<usize>,
    theme: &'a Theme,
    focused: bool,
}

impl<'a> SessionsPanel<'a> {
    pub fn new(
        sessions: &'a [SessionInfo],
        selected: Option<usize>,
        theme: &'a Theme,
        focused: bool,
    ) -> Self {
        Self {
            sessions,
            selected,
            theme,
            focused,
        }
    }
}

impl Widget for SessionsPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_color = if self.focused {
            self.theme.border_focused()
        } else {
            self.theme.border()
        };

        let block = Block::default()
            .title(Span::styled(
                " 📁 Sessions ",
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

        if self.sessions.is_empty() {
            let empty_msg = Line::from(vec![Span::styled(
                "No sessions",
                Style::default().fg(self.theme.muted()),
            )]);
            let para = ratatui::widgets::Paragraph::new(empty_msg);
            para.render(inner, buf);
            return;
        }

        // Build list items
        let items: Vec<ListItem> = self
            .sessions
            .iter()
            .map(|session| {
                let icon = if session.is_active { "●" } else { "○" };
                let icon_color = if session.is_active {
                    self.theme.success()
                } else {
                    self.theme.muted()
                };

                let line = Line::from(vec![
                    Span::styled(format!("{icon} "), Style::default().fg(icon_color)),
                    Span::styled(&session.name, Style::default().fg(self.theme.fg())),
                ]);

                ListItem::new(line)
            })
            .collect();

        let highlight_style = Style::default()
            .bg(Theme::parse_color(&self.theme.colors.highlight))
            .fg(self.theme.fg())
            .add_modifier(Modifier::BOLD);

        let list = List::new(items)
            .highlight_style(highlight_style)
            .highlight_symbol("▶ ");

        let mut state = ListState::default().with_selected(self.selected);
        StatefulWidget::render(list, inner, buf, &mut state);
    }
}
