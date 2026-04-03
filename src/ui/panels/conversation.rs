//! Conversation panel for displaying chat messages.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget, Wrap},
};
use unicode_width::UnicodeWidthStr;

use crate::core::state::{Conversation, Message, MessageRole};
use crate::ui::themes::Theme;

/// Conversation panel widget.
pub struct ConversationPanel<'a> {
    conversation: &'a Conversation,
    theme: &'a Theme,
    focused: bool,
}

impl<'a> ConversationPanel<'a> {
    pub fn new(conversation: &'a Conversation, theme: &'a Theme, focused: bool) -> Self {
        Self {
            conversation,
            theme,
            focused,
        }
    }

    /// Render a single message.
    fn render_message(&self, message: &Message, width: u16) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let content_width = width.saturating_sub(4) as usize;

        // Message header
        let (role_icon, role_name, role_color) = match message.role {
            MessageRole::User => ("👤", "You", self.theme.accent()),
            MessageRole::Assistant => ("🤖", "Assistant", Theme::parse_color("#a371f7")),
            MessageRole::System => ("⚙️", "System", self.theme.muted()),
            MessageRole::Tool => ("🔧", "Tool", self.theme.warning()),
        };

        // Time ago
        let time_ago = format_time_ago(message.timestamp);

        // Header line
        let header = Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{role_icon} {role_name}"),
                Style::default()
                    .fg(role_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" ─ {time_ago}"),
                Style::default().fg(self.theme.muted()),
            ),
            if message.is_streaming {
                Span::styled(" ●", Style::default().fg(self.theme.success()))
            } else {
                Span::raw("")
            },
        ]);
        lines.push(header);

        // Thinking content (if present)
        if let Some(ref thinking) = message.thinking {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "💭 Thinking...",
                    Style::default()
                        .fg(self.theme.muted())
                        .add_modifier(Modifier::ITALIC),
                ),
            ]));
            for line in thinking.lines().take(3) {
                let truncated = truncate_str(line, content_width);
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(
                        truncated.to_string(),
                        Style::default()
                            .fg(self.theme.muted())
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }
            lines.push(Line::raw(""));
        }

        // Message content
        for line in message.content.lines() {
            // Check if this is a code block
            if line.starts_with("```") {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        line.to_string(),
                        Style::default().fg(Theme::parse_color(&self.theme.syntax.keyword)),
                    ),
                ]));
            } else if line.starts_with("  ") || line.starts_with("\t") {
                // Code content
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        line.to_string(),
                        Style::default().fg(Theme::parse_color(&self.theme.syntax.string)),
                    ),
                ]));
            } else {
                // Regular text - wrap long lines
                let wrapped = textwrap::wrap(line, content_width);
                for wrapped_line in wrapped {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(wrapped_line.to_string(), Style::default().fg(self.theme.fg())),
                    ]));
                }
            }
        }

        // Tool calls
        for tool_call in &message.tool_calls {
            let status_icon = match tool_call.status {
                crate::core::state::ToolCallStatus::Pending => "⏳",
                crate::core::state::ToolCallStatus::Running => "⚡",
                crate::core::state::ToolCallStatus::Success => "✓",
                crate::core::state::ToolCallStatus::Error => "✗",
            };
            let status_color = match tool_call.status {
                crate::core::state::ToolCallStatus::Pending => self.theme.muted(),
                crate::core::state::ToolCallStatus::Running => self.theme.warning(),
                crate::core::state::ToolCallStatus::Success => self.theme.success(),
                crate::core::state::ToolCallStatus::Error => self.theme.error(),
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{status_icon} {}", tool_call.name),
                    Style::default().fg(status_color),
                ),
            ]));
        }

        // Separator
        lines.push(Line::raw(""));

        lines
    }
}

impl Widget for ConversationPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_color = if self.focused {
            self.theme.border_focused()
        } else {
            self.theme.border()
        };

        let block = Block::default()
            .title(Span::styled(
                " 💬 Conversation ",
                Style::default()
                    .fg(self.theme.accent())
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(self.theme.borders.style.to_ratatui())
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(Theme::parse_color(&self.theme.panels.conversation_bg)));

        let inner = block.inner(area);
        block.render(area, buf);

        // Render messages
        let mut all_lines: Vec<Line> = Vec::new();
        for message in &self.conversation.messages {
            all_lines.extend(self.render_message(message, inner.width));
        }

        // Empty state
        if all_lines.is_empty() {
            all_lines.push(Line::from(vec![
                Span::styled(
                    "  🦅 Welcome to HawkTUI!",
                    Style::default()
                        .fg(self.theme.accent())
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            all_lines.push(Line::raw(""));
            all_lines.push(Line::from(vec![
                Span::styled(
                    "  Type a message below or use /help for commands.",
                    Style::default().fg(self.theme.muted()),
                ),
            ]));
        }

        let text = Text::from(all_lines.clone());
        let total_lines = all_lines.len();
        let visible_lines = inner.height as usize;

        // Calculate scroll
        let scroll = if self.conversation.auto_scroll && total_lines > visible_lines {
            (total_lines - visible_lines) as u16
        } else {
            self.conversation.scroll_offset
        };

        let para = Paragraph::new(text)
            .scroll((scroll, 0))
            .wrap(Wrap { trim: false });

        para.render(inner, buf);

        // Scrollbar
        if total_lines > visible_lines {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            let mut scrollbar_state = ScrollbarState::new(total_lines)
                .position(scroll as usize)
                .viewport_content_length(visible_lines);

            scrollbar.render(
                area.inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                buf,
                &mut scrollbar_state,
            );
        }
    }
}

/// Format a timestamp as "X ago".
fn format_time_ago(timestamp: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(timestamp);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        let mins = duration.num_minutes();
        format!("{mins} min{} ago", if mins == 1 { "" } else { "s" })
    } else if duration.num_hours() < 24 {
        let hours = duration.num_hours();
        format!("{hours} hour{} ago", if hours == 1 { "" } else { "s" })
    } else {
        let days = duration.num_days();
        format!("{days} day{} ago", if days == 1 { "" } else { "s" })
    }
}

/// Truncate a string to fit within a width.
fn truncate_str(s: &str, max_width: usize) -> &str {
    if s.width() <= max_width {
        return s;
    }

    let mut width = 0;
    let mut end = 0;
    for (i, c) in s.char_indices() {
        let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if width + char_width > max_width.saturating_sub(3) {
            break;
        }
        width += char_width;
        end = i + c.len_utf8();
    }

    &s[..end]
}
