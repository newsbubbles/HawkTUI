//! Error types for HawkTUI.

use thiserror::Error;

/// Result type alias using HawkTUI's error type.
pub type Result<T> = std::result::Result<T, Error>;

/// HawkTUI error types.
#[derive(Error, Debug)]
pub enum Error {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Terminal error.
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Theme error.
    #[error("Theme error: {0}")]
    Theme(String),

    /// Session error.
    #[error("Session error: {0}")]
    Session(String),

    /// Agent error.
    #[error("Agent error: {0}")]
    Agent(String),

    /// Provider error.
    #[error("Provider error: {0}")]
    Provider(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// TOML parsing error.
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Generic error with context.
    #[error("{context}: {source}")]
    WithContext {
        context: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl Error {
    /// Create a terminal error.
    pub fn terminal(msg: impl Into<String>) -> Self {
        Self::Terminal(msg.into())
    }

    /// Create a configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a theme error.
    pub fn theme(msg: impl Into<String>) -> Self {
        Self::Theme(msg.into())
    }

    /// Create a session error.
    pub fn session(msg: impl Into<String>) -> Self {
        Self::Session(msg.into())
    }

    /// Create an agent error.
    pub fn agent(msg: impl Into<String>) -> Self {
        Self::Agent(msg.into())
    }

    /// Create a provider error.
    pub fn provider(msg: impl Into<String>) -> Self {
        Self::Provider(msg.into())
    }
}
