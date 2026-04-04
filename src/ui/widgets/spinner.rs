//! Animated spinner widget.

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

/// Spinner animation frames.
pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
pub const DOTS_FRAMES: &[&str] = &["⠀", "⠁", "⠃", "⠇", "⠏", "⠟", "⠿", "⡿", "⣿"];
pub const MOON_FRAMES: &[&str] = &["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"];
pub const HAWK_FRAMES: &[&str] = &["🦅", "✨🦅", "🦅✨", "✨🦅✨"];

/// A spinner widget.
pub struct Spinner<'a> {
    frames: &'a [&'a str],
    frame_index: usize,
    style: Style,
    label: Option<&'a str>,
}

impl<'a> Spinner<'a> {
    /// Create a new spinner with default frames.
    pub fn new(frame_index: usize) -> Self {
        Self {
            frames: SPINNER_FRAMES,
            frame_index,
            style: Style::default(),
            label: None,
        }
    }

    /// Set the animation frames.
    #[must_use]
    pub const fn frames(mut self, frames: &'a [&'a str]) -> Self {
        self.frames = frames;
        self
    }

    /// Set the style.
    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set a label to display after the spinner.
    #[must_use]
    pub const fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Get the current frame.
    pub fn current_frame(&self) -> &str {
        self.frames
            .get(self.frame_index % self.frames.len())
            .unwrap_or(&"")
    }
}

impl Widget for Spinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let frame = self.current_frame();
        let text = if let Some(label) = self.label {
            format!("{frame} {label}")
        } else {
            frame.to_string()
        };

        buf.set_string(area.x, area.y, &text, self.style);
    }
}

/// Calculate the next frame index.
#[must_use]
pub const fn next_frame(current: usize, total_frames: usize) -> usize {
    (current + 1) % total_frames
}
