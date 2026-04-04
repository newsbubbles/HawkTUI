//! Syntax highlighting module using syntect.
//!
//! Provides syntax highlighting for code blocks in the conversation panel.

use std::sync::LazyLock;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;

/// Lazily loaded syntax set with all default syntaxes.
static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

/// Lazily loaded theme set with default themes.
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

/// Get a reference to the global syntax set.
pub fn syntax_set() -> &'static SyntaxSet {
    &SYNTAX_SET
}

/// Get a reference to the global theme set.
pub fn theme_set() -> &'static ThemeSet {
    &THEME_SET
}

/// Map a language identifier to a syntect syntax name.
///
/// Handles common aliases and variations.
fn normalize_language(lang: &str) -> String {
    let lower = lang.to_lowercase();
    match lower.as_str() {
        // Rust
        "rust" | "rs" => "Rust".to_string(),
        // Python
        "python" | "py" | "python3" => "Python".to_string(),
        // JavaScript
        "javascript" | "js" => "JavaScript".to_string(),
        "typescript" | "ts" => "TypeScript".to_string(),
        "jsx" => "JavaScript (JSX)".to_string(),
        "tsx" => "TypeScript (TSX)".to_string(),
        // Web
        "html" | "htm" => "HTML".to_string(),
        "css" => "CSS".to_string(),
        "scss" => "SCSS".to_string(),
        "sass" => "Sass".to_string(),
        "json" => "JSON".to_string(),
        "xml" => "XML".to_string(),
        "yaml" | "yml" => "YAML".to_string(),
        "toml" => "TOML".to_string(),
        // Shell
        "bash" | "sh" | "shell" | "zsh" => "Bourne Again Shell (bash)".to_string(),
        "fish" => "fish".to_string(),
        "powershell" | "ps1" => "PowerShell".to_string(),
        // Systems
        "c" => "C".to_string(),
        "cpp" | "c++" | "cxx" => "C++".to_string(),
        "go" | "golang" => "Go".to_string(),
        "zig" => "Zig".to_string(),
        // JVM
        "java" => "Java".to_string(),
        "kotlin" | "kt" => "Kotlin".to_string(),
        "scala" => "Scala".to_string(),
        "groovy" => "Groovy".to_string(),
        // .NET
        "csharp" | "c#" | "cs" => "C#".to_string(),
        "fsharp" | "f#" | "fs" => "F#".to_string(),
        // Functional
        "haskell" | "hs" => "Haskell".to_string(),
        "ocaml" | "ml" => "OCaml".to_string(),
        "elixir" | "ex" => "Elixir".to_string(),
        "erlang" | "erl" => "Erlang".to_string(),
        "clojure" | "clj" => "Clojure".to_string(),
        "lisp" | "cl" => "Lisp".to_string(),
        "scheme" | "scm" => "Scheme".to_string(),
        // Scripting
        "ruby" | "rb" => "Ruby".to_string(),
        "php" => "PHP".to_string(),
        "perl" | "pl" => "Perl".to_string(),
        "lua" => "Lua".to_string(),
        "r" => "R".to_string(),
        // Data
        "sql" => "SQL".to_string(),
        "graphql" | "gql" => "GraphQL".to_string(),
        // Config
        "dockerfile" | "docker" => "Dockerfile".to_string(),
        "makefile" | "make" => "Makefile".to_string(),
        "cmake" => "CMake".to_string(),
        // Docs
        "markdown" | "md" => "Markdown".to_string(),
        "latex" | "tex" => "LaTeX".to_string(),
        "rst" | "restructuredtext" => "reStructuredText".to_string(),
        // Other
        "diff" | "patch" => "Diff".to_string(),
        "git-commit" => "Git Commit".to_string(),
        "git-rebase" => "Git Rebase".to_string(),
        "ini" => "INI".to_string(),
        "nginx" => "nginx".to_string(),
        "apache" => "Apache Conf".to_string(),
        // Default - return as-is for syntect to try
        _ => lang.to_string(),
    }
}

/// Select the appropriate syntect theme based on whether we're using a dark or light theme.
const fn select_syntect_theme(is_dark_theme: bool) -> &'static str {
    if is_dark_theme {
        "base16-ocean.dark"
    } else {
        "base16-ocean.light"
    }
}

/// Highlight a single line of code.
/// 
/// Returns a vector of styled spans for the line.
pub fn highlight_line(
    code: &str,
    language: Option<&str>,
    is_dark_theme: bool,
    fallback_fg: Color,
    code_bg: Color,
) -> Vec<Span<'static>> {
    let ss = syntax_set();
    let ts = theme_set();
    
    // Find syntax for the language
    let syntax = language
        .and_then(|lang| {
            let normalized = normalize_language(lang);
            ss.find_syntax_by_name(&normalized)
                .or_else(|| ss.find_syntax_by_extension(lang))
                .or_else(|| ss.find_syntax_by_token(lang))
        })
        .unwrap_or_else(|| ss.find_syntax_plain_text());
    
    // Get the theme
    let theme_name = select_syntect_theme(is_dark_theme);
    let theme = ts.themes.get(theme_name).unwrap_or_else(|| {
        ts.themes.values().next().expect("No themes available")
    });
    
    // Create highlighter
    let mut highlighter = HighlightLines::new(syntax, theme);
    
    // Highlight the line
    match highlighter.highlight_line(code, ss) {
        Ok(ranges) => {
            ranges
                .into_iter()
                .map(|(style, text)| {
                    let fg = Color::Rgb(
                        style.foreground.r,
                        style.foreground.g,
                        style.foreground.b,
                    );
                    
                    let mut ratatui_style = Style::default().fg(fg).bg(code_bg);
                    
                    // Apply font style modifiers
                    if style.font_style.contains(FontStyle::BOLD) {
                        ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
                    }
                    if style.font_style.contains(FontStyle::ITALIC) {
                        ratatui_style = ratatui_style.add_modifier(Modifier::ITALIC);
                    }
                    if style.font_style.contains(FontStyle::UNDERLINE) {
                        ratatui_style = ratatui_style.add_modifier(Modifier::UNDERLINED);
                    }
                    
                    Span::styled(text.to_string(), ratatui_style)
                })
                .collect()
        }
        Err(_) => {
            // Fallback: return unstyled text
            vec![Span::styled(
                code.to_string(),
                Style::default().fg(fallback_fg).bg(code_bg),
            )]
        }
    }
}

/// Check if a language is supported for syntax highlighting.
pub fn is_language_supported(language: &str) -> bool {
    let ss = syntax_set();
    let normalized = normalize_language(language);

    ss.find_syntax_by_name(&normalized).is_some()
        || ss.find_syntax_by_extension(language).is_some()
        || ss.find_syntax_by_token(language).is_some()
}

/// Get a list of supported language names.
#[allow(dead_code)]
pub fn supported_languages() -> Vec<&'static str> {
    syntax_set()
        .syntaxes()
        .iter()
        .map(|s| s.name.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_language() {
        assert_eq!(normalize_language("rust"), "Rust");
        assert_eq!(normalize_language("rs"), "Rust");
        assert_eq!(normalize_language("RUST"), "Rust");
        assert_eq!(normalize_language("python"), "Python");
        assert_eq!(normalize_language("py"), "Python");
        assert_eq!(normalize_language("javascript"), "JavaScript");
        assert_eq!(normalize_language("js"), "JavaScript");
    }

    #[test]
    fn test_highlight_rust_code() {
        let code = "fn main() { println!(\"Hello\"); }";
        let spans = highlight_line(
            code,
            Some("rust"),
            true,
            Color::White,
            Color::Rgb(40, 42, 54),
        );
        
        // Should produce multiple spans (keywords, strings, etc.)
        assert!(!spans.is_empty());
        
        // Verify the content is preserved
        let reconstructed: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(reconstructed, code);
    }

    #[test]
    fn test_highlight_unknown_language() {
        let code = "some random code";
        let spans = highlight_line(
            code,
            Some("nonexistent_language_xyz"),
            true,
            Color::White,
            Color::Rgb(40, 42, 54),
        );
        
        // Should still produce spans (plain text fallback)
        assert!(!spans.is_empty());
    }

    #[test]
    fn test_highlight_no_language() {
        let code = "plain text content";
        let spans = highlight_line(
            code,
            None,
            true,
            Color::White,
            Color::Rgb(40, 42, 54),
        );
        
        assert!(!spans.is_empty());
    }

    #[test]
    fn test_is_language_supported() {
        assert!(is_language_supported("rust"));
        assert!(is_language_supported("python"));
        assert!(is_language_supported("javascript"));
        // Unknown languages should still return false gracefully
    }
}
