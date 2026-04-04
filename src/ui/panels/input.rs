//! Input panel for message composition.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use unicode_width::UnicodeWidthStr;

use crate::core::state::{AppMode, InputState, VimState};
use crate::ui::themes::Theme;

/// Input panel widget.
pub struct InputPanel<'a> {
    input: &'a InputState,
    mode: AppMode,
    theme: &'a Theme,
    focused: bool,
}

impl<'a> InputPanel<'a> {
    pub fn new(input: &'a InputState, mode: AppMode, theme: &'a Theme, focused: bool) -> Self {
        Self {
            input,
            mode,
            theme,
            focused,
        }
    }
}

impl Widget for InputPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_color = if self.focused {
            self.theme.border_focused()
        } else {
            self.theme.border()
        };

        // Mode indicator
        let mode_indicator = match self.mode {
            AppMode::Normal => ("NORMAL", self.theme.muted()),
            AppMode::Insert => ("INSERT", self.theme.success()),
            AppMode::Command => ("COMMAND", self.theme.warning()),
            AppMode::Streaming => ("STREAMING", self.theme.accent()),
            AppMode::Waiting => ("WAITING", self.theme.warning()),
        };

        // Vim state indicator
        let vim_indicator = if self.input.vim_mode {
            match self.input.vim_state {
                VimState::Normal => Some(("[vim:n]", self.theme.muted())),
                VimState::Insert => Some(("[vim:i]", self.theme.success())),
                VimState::Visual => Some(("[vim:v]", self.theme.warning())),
            }
        } else {
            None
        };

        // Build title
        let mut title_spans = vec![
            Span::raw(" "),
            Span::styled(
                mode_indicator.0,
                Style::default()
                    .fg(mode_indicator.1)
                    .add_modifier(Modifier::BOLD),
            ),
        ];

        if let Some((vim_text, vim_color)) = vim_indicator {
            title_spans.push(Span::raw(" "));
            title_spans.push(Span::styled(vim_text, Style::default().fg(vim_color)));
        }

        title_spans.push(Span::raw(" "));

        let block = Block::default()
            .title(Line::from(title_spans))
            .borders(Borders::ALL)
            .border_type(self.theme.borders.style.to_ratatui())
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(Theme::parse_color(&self.theme.panels.input_bg)));

        let inner = block.inner(area);
        block.render(area, buf);

        // Render input text with cursor
        let text = &self.input.text;
        let cursor_pos = self.input.cursor; // This is now a character index

        // Build the text with cursor - using character indexing
        let char_count = text.chars().count();
        let (before_cursor, at_cursor, after_cursor) = if cursor_pos < char_count {
            // Convert char index to byte index for splitting
            let byte_pos = text
                .char_indices()
                .nth(cursor_pos)
                .map(|(idx, _)| idx)
                .unwrap_or(text.len());
            let (before, rest) = text.split_at(byte_pos);
            let mut chars = rest.chars();
            let cursor_char = chars.next().unwrap_or(' ');
            (before, cursor_char, chars.as_str())
        } else {
            (text.as_str(), ' ', "")
        };

        let prompt = "> ";
        let hint = if text.is_empty() && matches!(self.mode, AppMode::Normal | AppMode::Insert) {
            "Type your message... (Ctrl+Enter to send, /help for commands)"
        } else {
            ""
        };

        let line = if text.is_empty() && !hint.is_empty() {
            Line::from(vec![
                Span::styled(prompt, Style::default().fg(self.theme.accent())),
                Span::styled(
                    hint,
                    Style::default()
                        .fg(self.theme.muted())
                        .add_modifier(Modifier::ITALIC),
                ),
            ])
        } else if self.focused {
            Line::from(vec![
                Span::styled(prompt, Style::default().fg(self.theme.accent())),
                Span::raw(before_cursor),
                Span::styled(
                    at_cursor.to_string(),
                    Style::default()
                        .bg(self.theme.accent())
                        .fg(Theme::parse_color(&self.theme.panels.input_bg)),
                ),
                Span::raw(after_cursor),
            ])
        } else {
            Line::from(vec![
                Span::styled(prompt, Style::default().fg(self.theme.accent())),
                Span::raw(text.as_str()),
            ])
        };

        let para = Paragraph::new(line);
        para.render(inner, buf);

        // Right-side indicators
        let indicators = format_indicators(&self.input.text, self.theme);
        if !indicators.is_empty() && inner.width > 20 {
            let indicator_width: usize =
                indicators.iter().map(|s| s.content.width()).sum::<usize>() + 2;
            let x = inner.x + inner.width - indicator_width as u16;
            let indicator_line = Line::from(indicators);
            buf.set_line(x, inner.y, &indicator_line, indicator_width as u16);
        }
    }
}

/// Format right-side indicators (character count, attachment icon, etc.).
fn format_indicators(text: &str, theme: &Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    // Character count
    let char_count = text.chars().count();
    if char_count > 0 {
        spans.push(Span::styled(
            format!("{char_count}"),
            Style::default().fg(theme.muted()),
        ));
        spans.push(Span::raw(" "));
    }

    // Attachment indicator
    spans.push(Span::styled("📎", Style::default().fg(theme.muted())));

    spans
}
