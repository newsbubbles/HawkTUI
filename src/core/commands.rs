//! Slash commands for HawkTUI.
//!
//! Commands are invoked with `/command` syntax.

use std::collections::HashMap;

/// A slash command definition.
#[derive(Debug, Clone)]
pub struct SlashCommand {
    /// Command name (without the slash).
    pub name: &'static str,

    /// Short aliases.
    pub aliases: &'static [&'static str],

    /// Description.
    pub description: &'static str,

    /// Usage example.
    pub usage: &'static str,

    /// Whether the command takes arguments.
    pub takes_args: bool,
}

/// Built-in slash commands.
pub static COMMANDS: &[SlashCommand] = &[
    SlashCommand {
        name: "help",
        aliases: &["h", "?"],
        description: "Show help information",
        usage: "/help [command]",
        takes_args: true,
    },
    SlashCommand {
        name: "clear",
        aliases: &["cls"],
        description: "Clear the conversation",
        usage: "/clear",
        takes_args: false,
    },
    SlashCommand {
        name: "exit",
        aliases: &["quit", "q"],
        description: "Exit HawkTUI",
        usage: "/exit",
        takes_args: false,
    },
    SlashCommand {
        name: "model",
        aliases: &["m"],
        description: "Switch model",
        usage: "/model <model-name>",
        takes_args: true,
    },
    SlashCommand {
        name: "provider",
        aliases: &["p"],
        description: "Switch provider",
        usage: "/provider <provider-name>",
        takes_args: true,
    },
    SlashCommand {
        name: "session",
        aliases: &["s"],
        description: "Session management",
        usage: "/session [new|list|switch|delete] [name]",
        takes_args: true,
    },
    SlashCommand {
        name: "theme",
        aliases: &["t"],
        description: "Switch theme",
        usage: "/theme <theme-name>",
        takes_args: true,
    },
    SlashCommand {
        name: "layout",
        aliases: &["l"],
        description: "Switch layout mode",
        usage: "/layout [command-center|focus|split]",
        takes_args: true,
    },
    SlashCommand {
        name: "export",
        aliases: &["e"],
        description: "Export conversation",
        usage: "/export [format] [path]",
        takes_args: true,
    },
    SlashCommand {
        name: "context",
        aliases: &["ctx"],
        description: "Manage context/attachments",
        usage: "/context [add|remove|clear] [path]",
        takes_args: true,
    },
    SlashCommand {
        name: "tools",
        aliases: &[],
        description: "Manage tools",
        usage: "/tools [enable|disable|list] [tool-name]",
        takes_args: true,
    },
    SlashCommand {
        name: "system",
        aliases: &["sys"],
        description: "Set system prompt",
        usage: "/system <prompt>",
        takes_args: true,
    },
    SlashCommand {
        name: "branch",
        aliases: &["b"],
        description: "Manage conversation branches",
        usage: "/branch [create|switch|list|delete] [name]",
        takes_args: true,
    },
    SlashCommand {
        name: "compact",
        aliases: &[],
        description: "Compact conversation history",
        usage: "/compact",
        takes_args: false,
    },
    SlashCommand {
        name: "tokens",
        aliases: &[],
        description: "Show token usage",
        usage: "/tokens",
        takes_args: false,
    },
    SlashCommand {
        name: "cost",
        aliases: &[],
        description: "Show cost estimate",
        usage: "/cost",
        takes_args: false,
    },
    SlashCommand {
        name: "vim",
        aliases: &[],
        description: "Toggle vim mode",
        usage: "/vim",
        takes_args: false,
    },
    SlashCommand {
        name: "shortcuts",
        aliases: &["keys", "keybindings"],
        description: "Show keyboard shortcuts",
        usage: "/shortcuts",
        takes_args: false,
    },
];

/// Parsed command.
#[derive(Debug, Clone)]
pub struct ParsedCommand {
    /// Command name.
    pub name: String,

    /// Arguments.
    pub args: Vec<String>,

    /// Raw argument string.
    pub raw_args: String,
}

/// Parse a command string.
///
/// Returns `None` if the string doesn't start with `/`.
pub fn parse_command(input: &str) -> Option<ParsedCommand> {
    let input = input.trim();
    if !input.starts_with('/') {
        return None;
    }

    let input = &input[1..]; // Remove leading slash
    let mut parts = input.splitn(2, char::is_whitespace);

    let name = parts.next()?.to_lowercase();
    let raw_args = parts.next().unwrap_or("").trim().to_string();
    let args: Vec<String> = if raw_args.is_empty() {
        Vec::new()
    } else {
        shell_words::split(&raw_args).unwrap_or_else(|_| vec![raw_args.clone()])
    };

    Some(ParsedCommand {
        name,
        args,
        raw_args,
    })
}

/// Find a command by name or alias.
pub fn find_command(name: &str) -> Option<&'static SlashCommand> {
    let name = name.to_lowercase();
    COMMANDS
        .iter()
        .find(|cmd| cmd.name == name || cmd.aliases.iter().any(|&alias| alias == name))
}

/// Get command completions for a prefix.
pub fn get_completions(prefix: &str) -> Vec<&'static SlashCommand> {
    let prefix = prefix.to_lowercase();
    COMMANDS
        .iter()
        .filter(|cmd| {
            cmd.name.starts_with(&prefix)
                || cmd.aliases.iter().any(|&alias| alias.starts_with(&prefix))
        })
        .collect()
}

/// Build a command lookup map.
pub fn build_command_map() -> HashMap<&'static str, &'static SlashCommand> {
    let mut map = HashMap::new();
    for cmd in COMMANDS {
        map.insert(cmd.name, cmd);
        for &alias in cmd.aliases {
            map.insert(alias, cmd);
        }
    }
    map
}

/// Command execution result.
#[derive(Debug)]
pub enum CommandResult {
    /// Command executed successfully.
    Success(Option<String>),

    /// Command needs more input.
    NeedsInput(String),

    /// Command failed.
    Error(String),

    /// Quit the application.
    Quit,

    /// No-op (command was informational).
    None,
}

// Simple shell word splitting (basic implementation)
mod shell_words {
    pub fn split(s: &str) -> Result<Vec<String>, ()> {
        let mut words = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';
        let mut escape_next = false;

        for c in s.chars() {
            if escape_next {
                current.push(c);
                escape_next = false;
                continue;
            }

            match c {
                '\\' => escape_next = true,
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    quote_char = c;
                }
                c if in_quotes && c == quote_char => {
                    in_quotes = false;
                }
                ' ' | '\t' if !in_quotes => {
                    if !current.is_empty() {
                        words.push(std::mem::take(&mut current));
                    }
                }
                _ => current.push(c),
            }
        }

        if !current.is_empty() {
            words.push(current);
        }

        if in_quotes {
            return Err(());
        }

        Ok(words)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let cmd = parse_command("/help").unwrap();
        assert_eq!(cmd.name, "help");
        assert!(cmd.args.is_empty());

        let cmd = parse_command("/model claude-sonnet-4-20250514").unwrap();
        assert_eq!(cmd.name, "model");
        assert_eq!(cmd.args, vec!["claude-sonnet-4-20250514"]);

        let cmd = parse_command("/session new my-session").unwrap();
        assert_eq!(cmd.name, "session");
        assert_eq!(cmd.args, vec!["new", "my-session"]);

        assert!(parse_command("not a command").is_none());
    }

    #[test]
    fn test_find_command() {
        assert!(find_command("help").is_some());
        assert!(find_command("h").is_some());
        assert!(find_command("?").is_some());
        assert!(find_command("nonexistent").is_none());
    }

    #[test]
    fn test_completions() {
        let completions = get_completions("he");
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].name, "help");

        let completions = get_completions("s");
        assert!(completions.len() >= 2); // session, system
    }
}
