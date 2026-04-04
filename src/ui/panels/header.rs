//! Header/status bar panel.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::core::state::{ConnectionStatus, StatusInfo};
use crate::ui::themes::Theme;

/// Header panel showing status information.
pub struct HeaderPanel<'a> {
    status: &'a StatusInfo,
    theme: &'a Theme,
    version: &'a str,
}

impl<'a> HeaderPanel<'a> {
    pub fn new(status: &'a StatusInfo, theme: &'a Theme, version: &'a str) -> Self {
        Self {
            status,
            theme,
            version,
        }
    }
}

impl Widget for HeaderPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Build the header line
        let mut spans = vec![
            Span::styled(
                " 🦅 HawkTUI ",
                Style::default()
                    .fg(self.theme.accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("v{} ", self.version),
                Style::default().fg(self.theme.muted()),
            ),
            Span::styled("│ ", Style::default().fg(self.theme.border())),
        ];

        // Model
        spans.push(Span::styled(
            &self.status.model,
            Style::default()
                .fg(self.theme.fg())
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            " │ ",
            Style::default().fg(self.theme.border()),
        ));

        // Tokens
        spans.push(Span::styled(
            format!("tokens: {}", format_tokens(self.status.total_tokens)),
            Style::default().fg(self.theme.muted()),
        ));
        spans.push(Span::styled(
            " │ ",
            Style::default().fg(self.theme.border()),
        ));

        // Cost
        spans.push(Span::styled(
            format!("${:.3}", self.status.cost),
            Style::default().fg(Theme::parse_color("#3fb950")),
        ));
        spans.push(Span::styled(
            " │ ",
            Style::default().fg(self.theme.border()),
        ));

        // Connection status
        let (status_icon, status_color) = match self.status.connection {
            ConnectionStatus::Disconnected => ("○", self.theme.muted()),
            ConnectionStatus::Connecting => ("◐", self.theme.warning()),
            ConnectionStatus::Connected => ("●", self.theme.success()),
            ConnectionStatus::Streaming => ("⚡", self.theme.accent()),
            ConnectionStatus::Error => ("✗", self.theme.error()),
        };
        spans.push(Span::styled(
            format!("{status_icon} "),
            Style::default().fg(status_color),
        ));

        let status_text = match self.status.connection {
            ConnectionStatus::Disconnected => "disconnected",
            ConnectionStatus::Connecting => "connecting...",
            ConnectionStatus::Connected => "ready",
            ConnectionStatus::Streaming => "streaming",
            ConnectionStatus::Error => "error",
        };
        spans.push(Span::styled(status_text, Style::default().fg(status_color)));

        // Session name (right-aligned)
        if let Some(ref session_name) = self.status.session_name {
            // Calculate remaining space
            let used: usize = spans.iter().map(|s| s.content.chars().count()).sum();
            let remaining = area.width as usize - used - session_name.len() - 5;
            if remaining > 0 {
                spans.push(Span::raw(" ".repeat(remaining)));
                spans.push(Span::styled("│ ", Style::default().fg(self.theme.border())));
                spans.push(Span::styled(
                    format!("📁 {session_name}"),
                    Style::default().fg(self.theme.accent()),
                ));
            }
        }

        let line = Line::from(spans);
        let para = Paragraph::new(line)
            .style(Style::default().bg(Theme::parse_color(&self.theme.panels.status_bg)));

        para.render(area, buf);
    }
}

/// Format token count with K/M suffixes.
fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}k", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}
