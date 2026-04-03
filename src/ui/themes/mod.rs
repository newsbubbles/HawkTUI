//! Theme system for HawkTUI.
//!
//! Themes are defined in TOML and control colors, borders, and styling.

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// A complete theme definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme metadata.
    pub meta: ThemeMeta,

    /// Color palette.
    pub colors: ThemeColors,

    /// Panel-specific colors.
    #[serde(default)]
    pub panels: PanelColors,

    /// Border configuration.
    #[serde(default)]
    pub borders: BorderConfig,

    /// Syntax highlighting colors.
    #[serde(default)]
    pub syntax: SyntaxColors,
}

impl Default for Theme {
    fn default() -> Self {
        Self::hawk_dark()
    }
}

impl Theme {
    /// The default Hawk Dark theme.
    pub fn hawk_dark() -> Self {
        Self {
            meta: ThemeMeta {
                name: "Hawk Dark".to_string(),
                author: "HawkTUI Team".to_string(),
                version: "1.0.0".to_string(),
            },
            colors: ThemeColors {
                background: "#0d1117".to_string(),
                foreground: "#c9d1d9".to_string(),
                accent: "#58a6ff".to_string(),
                accent_secondary: "#a371f7".to_string(),
                success: "#3fb950".to_string(),
                warning: "#d29922".to_string(),
                error: "#f85149".to_string(),
                info: "#58a6ff".to_string(),
                muted: "#8b949e".to_string(),
                highlight: "#388bfd".to_string(),
            },
            panels: PanelColors {
                conversation_bg: "#0d1117".to_string(),
                sidebar_bg: "#161b22".to_string(),
                input_bg: "#21262d".to_string(),
                status_bg: "#30363d".to_string(),
                user_message_bg: "#21262d".to_string(),
                assistant_message_bg: "#161b22".to_string(),
            },
            borders: BorderConfig {
                style: BorderStyle::Rounded,
                color: "#30363d".to_string(),
                focused_color: "#58a6ff".to_string(),
            },
            syntax: SyntaxColors::default(),
        }
    }

    /// Hawk Light theme.
    pub fn hawk_light() -> Self {
        Self {
            meta: ThemeMeta {
                name: "Hawk Light".to_string(),
                author: "HawkTUI Team".to_string(),
                version: "1.0.0".to_string(),
            },
            colors: ThemeColors {
                background: "#ffffff".to_string(),
                foreground: "#24292f".to_string(),
                accent: "#0969da".to_string(),
                accent_secondary: "#8250df".to_string(),
                success: "#1a7f37".to_string(),
                warning: "#9a6700".to_string(),
                error: "#cf222e".to_string(),
                info: "#0969da".to_string(),
                muted: "#57606a".to_string(),
                highlight: "#0969da".to_string(),
            },
            panels: PanelColors {
                conversation_bg: "#ffffff".to_string(),
                sidebar_bg: "#f6f8fa".to_string(),
                input_bg: "#f6f8fa".to_string(),
                status_bg: "#eaeef2".to_string(),
                user_message_bg: "#ddf4ff".to_string(),
                assistant_message_bg: "#f6f8fa".to_string(),
            },
            borders: BorderConfig {
                style: BorderStyle::Rounded,
                color: "#d0d7de".to_string(),
                focused_color: "#0969da".to_string(),
            },
            syntax: SyntaxColors::light(),
        }
    }

    /// Cyberpunk theme.
    pub fn cyberpunk() -> Self {
        Self {
            meta: ThemeMeta {
                name: "Cyberpunk".to_string(),
                author: "HawkTUI Team".to_string(),
                version: "1.0.0".to_string(),
            },
            colors: ThemeColors {
                background: "#0a0a0f".to_string(),
                foreground: "#00ff9f".to_string(),
                accent: "#ff00ff".to_string(),
                accent_secondary: "#00ffff".to_string(),
                success: "#00ff00".to_string(),
                warning: "#ffff00".to_string(),
                error: "#ff0055".to_string(),
                info: "#00ffff".to_string(),
                muted: "#666699".to_string(),
                highlight: "#ff00ff".to_string(),
            },
            panels: PanelColors {
                conversation_bg: "#0a0a0f".to_string(),
                sidebar_bg: "#0f0f1a".to_string(),
                input_bg: "#1a1a2e".to_string(),
                status_bg: "#16213e".to_string(),
                user_message_bg: "#1a1a2e".to_string(),
                assistant_message_bg: "#0f0f1a".to_string(),
            },
            borders: BorderConfig {
                style: BorderStyle::Double,
                color: "#ff00ff".to_string(),
                focused_color: "#00ffff".to_string(),
            },
            syntax: SyntaxColors::cyberpunk(),
        }
    }

    /// Get a theme by name.
    pub fn by_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "hawk-light" | "light" => Self::hawk_light(),
            "cyberpunk" | "cyber" => Self::cyberpunk(),
            _ => Self::hawk_dark(),
        }
    }

    /// Parse a hex color to ratatui Color.
    pub fn parse_color(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Color::Reset;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

        Color::Rgb(r, g, b)
    }

    /// Get background color.
    pub fn bg(&self) -> Color {
        Self::parse_color(&self.colors.background)
    }

    /// Get foreground color.
    pub fn fg(&self) -> Color {
        Self::parse_color(&self.colors.foreground)
    }

    /// Get accent color.
    pub fn accent(&self) -> Color {
        Self::parse_color(&self.colors.accent)
    }

    /// Get success color.
    pub fn success(&self) -> Color {
        Self::parse_color(&self.colors.success)
    }

    /// Get warning color.
    pub fn warning(&self) -> Color {
        Self::parse_color(&self.colors.warning)
    }

    /// Get error color.
    pub fn error(&self) -> Color {
        Self::parse_color(&self.colors.error)
    }

    /// Get muted color.
    pub fn muted(&self) -> Color {
        Self::parse_color(&self.colors.muted)
    }

    /// Get border color.
    pub fn border(&self) -> Color {
        Self::parse_color(&self.borders.color)
    }

    /// Get focused border color.
    pub fn border_focused(&self) -> Color {
        Self::parse_color(&self.borders.focused_color)
    }

    /// Get default style.
    pub fn default_style(&self) -> Style {
        Style::default().fg(self.fg()).bg(self.bg())
    }

    /// Get accent style.
    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent())
    }

    /// Get muted style.
    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted())
    }

    /// Get bold style.
    pub fn bold_style(&self) -> Style {
        Style::default().fg(self.fg()).add_modifier(Modifier::BOLD)
    }

    /// Get title style.
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.accent())
            .add_modifier(Modifier::BOLD)
    }
}

/// Theme metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMeta {
    pub name: String,
    pub author: String,
    pub version: String,
}

/// Main color palette.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub accent_secondary: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
    pub muted: String,
    pub highlight: String,
}

/// Panel-specific colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelColors {
    pub conversation_bg: String,
    pub sidebar_bg: String,
    pub input_bg: String,
    pub status_bg: String,
    pub user_message_bg: String,
    pub assistant_message_bg: String,
}

impl Default for PanelColors {
    fn default() -> Self {
        Self {
            conversation_bg: "#0d1117".to_string(),
            sidebar_bg: "#161b22".to_string(),
            input_bg: "#21262d".to_string(),
            status_bg: "#30363d".to_string(),
            user_message_bg: "#21262d".to_string(),
            assistant_message_bg: "#161b22".to_string(),
        }
    }
}

/// Border configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderConfig {
    pub style: BorderStyle,
    pub color: String,
    pub focused_color: String,
}

impl Default for BorderConfig {
    fn default() -> Self {
        Self {
            style: BorderStyle::Rounded,
            color: "#30363d".to_string(),
            focused_color: "#58a6ff".to_string(),
        }
    }
}

/// Border styles.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BorderStyle {
    #[default]
    Rounded,
    Sharp,
    Double,
    Thick,
    None,
}

impl BorderStyle {
    /// Convert to ratatui border type.
    pub const fn to_ratatui(self) -> ratatui::widgets::BorderType {
        match self {
            Self::Rounded => ratatui::widgets::BorderType::Rounded,
            Self::Sharp | Self::None => ratatui::widgets::BorderType::Plain,
            Self::Double => ratatui::widgets::BorderType::Double,
            Self::Thick => ratatui::widgets::BorderType::Thick,
        }
    }
}

/// Syntax highlighting colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxColors {
    pub keyword: String,
    pub string: String,
    pub comment: String,
    pub function: String,
    pub r#type: String,
    pub number: String,
    pub operator: String,
    pub variable: String,
    pub constant: String,
}

impl Default for SyntaxColors {
    fn default() -> Self {
        Self {
            keyword: "#ff7b72".to_string(),
            string: "#a5d6ff".to_string(),
            comment: "#8b949e".to_string(),
            function: "#d2a8ff".to_string(),
            r#type: "#79c0ff".to_string(),
            number: "#79c0ff".to_string(),
            operator: "#ff7b72".to_string(),
            variable: "#ffa657".to_string(),
            constant: "#79c0ff".to_string(),
        }
    }
}

impl SyntaxColors {
    /// Light theme syntax colors.
    pub fn light() -> Self {
        Self {
            keyword: "#cf222e".to_string(),
            string: "#0a3069".to_string(),
            comment: "#6e7781".to_string(),
            function: "#8250df".to_string(),
            r#type: "#0550ae".to_string(),
            number: "#0550ae".to_string(),
            operator: "#cf222e".to_string(),
            variable: "#953800".to_string(),
            constant: "#0550ae".to_string(),
        }
    }

    /// Cyberpunk theme syntax colors.
    pub fn cyberpunk() -> Self {
        Self {
            keyword: "#ff00ff".to_string(),
            string: "#00ff9f".to_string(),
            comment: "#666699".to_string(),
            function: "#00ffff".to_string(),
            r#type: "#ff0055".to_string(),
            number: "#ffff00".to_string(),
            operator: "#ff00ff".to_string(),
            variable: "#00ff9f".to_string(),
            constant: "#ffff00".to_string(),
        }
    }
}
