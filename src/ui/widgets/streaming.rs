//! Streaming indicator widget.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::ui::themes::Theme;

/// Streaming indicator showing live response status.
pub struct StreamingIndicator<'a> {
    tokens: u64,
    frame: usize,
    theme: &'a Theme,
}

impl<'a> StreamingIndicator<'a> {
    pub const fn new(tokens: u64, frame: usize, theme: &'a Theme) -> Self {
        Self {
            tokens,
            frame,
            theme,
        }
    }
}

impl Widget for StreamingIndicator<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Animated dots
        let dots = match self.frame % 4 {
            0 => "   ",
            1 => ".  ",
            2 => ".. ",
            _ => "...",
        };

        // Pulse effect for the icon
        let icon_style = if self.frame % 2 == 0 {
            Style::default()
                .fg(self.theme.accent())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.success())
        };

        let line = Line::from(vec![
            Span::styled("⚡", icon_style),
            Span::styled(" Streaming", Style::default().fg(self.theme.fg())),
            Span::styled(dots, Style::default().fg(self.theme.muted())),
            Span::styled(
                format!(" ({} tokens)", self.tokens),
                Style::default().fg(self.theme.muted()),
            ),
        ]);

        buf.set_line(area.x, area.y, &line, area.width);
    }
}

/// Thinking indicator for models with thinking capability.
pub struct ThinkingIndicator<'a> {
    frame: usize,
    theme: &'a Theme,
}

impl<'a> ThinkingIndicator<'a> {
    pub const fn new(frame: usize, theme: &'a Theme) -> Self {
        Self { frame, theme }
    }
}

impl Widget for ThinkingIndicator<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let brain_frames = ["🧠", "💡", "🧠", "✨"];
        let brain = brain_frames[self.frame % brain_frames.len()];

        let line = Line::from(vec![
            Span::raw(brain),
            Span::styled(
                " Thinking...",
                Style::default()
                    .fg(self.theme.muted())
                    .add_modifier(Modifier::ITALIC),
            ),
        ]);

        buf.set_line(area.x, area.y, &line, area.width);
    }
}
