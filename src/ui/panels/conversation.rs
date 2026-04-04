//! Conversation panel for displaying chat messages.

use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Widget, Wrap,
    },
};
use unicode_width::UnicodeWidthStr;

use crate::core::state::{Conversation, Message, MessageRole, StreamingState};
use crate::ui::syntax::highlight_line;
use crate::ui::themes::Theme;
use crate::ui::widgets::{StreamingIndicator, ThinkingIndicator};

/// Conversation panel widget.
pub struct ConversationPanel<'a> {
    conversation: &'a Conversation,
    streaming: &'a StreamingState,
    theme: &'a Theme,
    focused: bool,
    frame: usize,
}

impl<'a> ConversationPanel<'a> {
    pub fn new(
        conversation: &'a Conversation,
        streaming: &'a StreamingState,
        theme: &'a Theme,
        focused: bool,
        frame: usize,
    ) -> Self {
        Self {
            conversation,
            streaming,
            theme,
            focused,
            frame,
        }
    }

    /// Render markdown content to styled lines.
    fn render_markdown(&self, content: &str, content_width: usize) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();
        let parser = Parser::new(content);
        
        // State tracking
        let mut current_spans: Vec<Span<'static>> = Vec::new();
        let mut in_code_block = false;
        let mut code_block_lang: Option<String> = None;
        let mut is_bold = false;
        let mut is_italic = false;
        let _is_code = false; // Reserved for future inline code state tracking
        let mut heading_level: Option<u8> = None;

        // Code block background color
        let code_bg = Color::Rgb(40, 42, 54); // Dark background for code
        let code_fg = Theme::parse_color(&self.theme.syntax.string);
        let inline_code_bg = Color::Rgb(50, 52, 64);

        // Determine if we're using a dark theme for syntect
        let is_dark_theme = self.theme.bg().eq(&Color::Rgb(13, 17, 23))
            || self.theme.bg().eq(&Color::Rgb(10, 10, 18));
        
        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    heading_level = Some(level as u8);
                }
                Event::End(TagEnd::Heading(_)) => {
                    // Flush heading line with special styling
                    if !current_spans.is_empty() {
                        let mut heading_spans = vec![Span::raw("  ")];
                        let level = heading_level.unwrap_or(1);
                        let prefix = "#".repeat(level as usize);
                        heading_spans.push(Span::styled(
                            format!("{prefix} "),
                            Style::default().fg(self.theme.accent()),
                        ));
                        for span in current_spans.drain(..) {
                            heading_spans.push(Span::styled(
                                span.content.to_string(),
                                Style::default()
                                    .fg(self.theme.accent())
                                    .add_modifier(Modifier::BOLD),
                            ));
                        }
                        lines.push(Line::from(heading_spans));
                    }
                    heading_level = None;
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    code_block_lang = match kind {
                        CodeBlockKind::Fenced(lang) => {
                            let lang_str = lang.to_string();
                            if lang_str.is_empty() { None } else { Some(lang_str) }
                        }
                        CodeBlockKind::Indented => None,
                    };
                    // Flush any pending content
                    if !current_spans.is_empty() {
                        let mut line_spans = vec![Span::raw("  ")];
                        line_spans.extend(current_spans.drain(..));
                        lines.push(Line::from(line_spans));
                    }
                    // Code block header
                    let lang_display = code_block_lang.as_deref().unwrap_or("code");
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            format!("┌─ {lang_display} "),
                            Style::default().fg(self.theme.muted()),
                        ),
                        Span::styled(
                            "─".repeat(content_width.saturating_sub(lang_display.len() + 6)),
                            Style::default().fg(self.theme.muted()),
                        ),
                    ]));
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                    // Code block footer
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            format!("└{}", "─".repeat(content_width.saturating_sub(3))),
                            Style::default().fg(self.theme.muted()),
                        ),
                    ]));
                    code_block_lang = None; // Reset for next code block
                }
                Event::Start(Tag::Strong) => {
                    is_bold = true;
                }
                Event::End(TagEnd::Strong) => {
                    is_bold = false;
                }
                Event::Start(Tag::Emphasis) => {
                    is_italic = true;
                }
                Event::End(TagEnd::Emphasis) => {
                    is_italic = false;
                }
                Event::Code(code) => {
                    // Inline code
                    current_spans.push(Span::styled(
                        format!(" {code} "),
                        Style::default()
                            .fg(code_fg)
                            .bg(inline_code_bg),
                    ));
                }
                Event::Text(text) => {
                    if in_code_block {
                        // Render code block lines with syntax highlighting
                        for code_line in text.lines() {
                            let mut line_spans = vec![
                                Span::raw("  "),
                                Span::styled(
                                    "│ ",
                                    Style::default().fg(self.theme.muted()),
                                ),
                            ];

                            // Apply syntax highlighting using syntect
                            let highlighted_spans = highlight_line(
                                code_line,
                                code_block_lang.as_deref(),
                                is_dark_theme,
                                code_fg,
                                code_bg,
                            );
                            line_spans.extend(highlighted_spans);

                            lines.push(Line::from(line_spans));
                        }
                    } else {
                        // Regular text with styling
                        let mut style = Style::default().fg(self.theme.fg());
                        if is_bold {
                            style = style.add_modifier(Modifier::BOLD);
                        }
                        if is_italic {
                            style = style.add_modifier(Modifier::ITALIC);
                        }
                        if heading_level.is_some() {
                            style = style.fg(self.theme.accent()).add_modifier(Modifier::BOLD);
                        }
                        current_spans.push(Span::styled(text.to_string(), style));
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if !in_code_block && !current_spans.is_empty() {
                        // Wrap and flush current line
                        let full_text: String = current_spans.iter().map(|s| s.content.as_ref()).collect();
                        let wrapped = textwrap::wrap(&full_text, content_width);
                        for wrapped_line in wrapped {
                            lines.push(Line::from(vec![
                                Span::raw("  "),
                                Span::styled(wrapped_line.to_string(), Style::default().fg(self.theme.fg())),
                            ]));
                        }
                        current_spans.clear();
                    }
                }
                Event::Start(Tag::Paragraph) => {
                    // Start fresh paragraph
                }
                Event::End(TagEnd::Paragraph) => {
                    // Flush paragraph
                    if !current_spans.is_empty() {
                        let full_text: String = current_spans.iter().map(|s| s.content.as_ref()).collect();
                        let wrapped = textwrap::wrap(&full_text, content_width);
                        
                        // Preserve styling for simple cases
                        if current_spans.len() == 1 {
                            for wrapped_line in wrapped {
                                let mut line_spans = vec![Span::raw("  ")];
                                line_spans.push(Span::styled(
                                    wrapped_line.to_string(),
                                    current_spans[0].style,
                                ));
                                lines.push(Line::from(line_spans));
                            }
                        } else {
                            // Complex styling - just use the collected spans
                            let mut line_spans = vec![Span::raw("  ")];
                            line_spans.extend(current_spans.drain(..));
                            lines.push(Line::from(line_spans));
                        }
                        current_spans.clear();
                    }
                    lines.push(Line::raw(""));
                }
                Event::Start(Tag::List(_)) | Event::End(TagEnd::List(_)) => {
                    // Lists handled via items
                }
                Event::Start(Tag::Item) => {
                    current_spans.push(Span::styled(
                        "• ",
                        Style::default().fg(self.theme.accent()),
                    ));
                }
                Event::End(TagEnd::Item) => {
                    if !current_spans.is_empty() {
                        let mut line_spans = vec![Span::raw("  ")];
                        line_spans.extend(current_spans.drain(..));
                        lines.push(Line::from(line_spans));
                    }
                }
                _ => {}
            }
        }
        
        // Flush any remaining content
        if !current_spans.is_empty() {
            let mut line_spans = vec![Span::raw("  ")];
            line_spans.extend(current_spans);
            lines.push(Line::from(line_spans));
        }
        
        lines
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

        // Message content - parse markdown
        lines.extend(self.render_markdown(&message.content, content_width));

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

        // Render streaming indicators at the bottom if streaming is active
        if self.streaming.is_active {
            // Create a rect for the indicator (same for both types)
            let indicator_area = Rect {
                x: inner.x + 2,
                y: inner.y + inner.height.saturating_sub(1),
                width: inner.width.saturating_sub(4),
                height: 1,
            };

            // Show thinking indicator if thinking is visible and no tokens yet
            if self.streaming.thinking_visible && self.streaming.tokens_streamed == 0 {
                ThinkingIndicator::new(self.frame, self.theme).render(indicator_area, buf);
            } else {
                // Show streaming indicator with token count
                StreamingIndicator::new(self.streaming.tokens_streamed, self.frame, self.theme)
                    .render(indicator_area, buf);
            }
        }

        // Empty state
        if all_lines.is_empty() && !self.streaming.is_active {
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
