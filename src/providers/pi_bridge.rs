//! Bridge to pi_agent_rust library.
//!
//! This module provides the integration layer between HawkTUI and
//! pi_agent_rust's agent, session, and provider systems.
//!
//! ## Integration Strategy
//!
//! HawkTUI integrates with pi_agent_rust at the library level:
//!
//! 1. **Agent**: Use `pi::agent::Agent` directly for AI interactions
//! 2. **Session**: Share session storage with pi CLI
//! 3. **Providers**: Reuse provider implementations
//! 4. **Tools**: Access the same tool registry
//!
//! This allows HawkTUI to:
//! - Share sessions with the `pi` CLI
//! - Use the same configuration
//! - Benefit from pi's streaming and tool infrastructure

use crate::core::events::AgentEvent;
use crate::core::error::{Error, Result};

// Pi agent integration stubs.
// When pi_agent_rust is added as a dependency, uncomment and use:
//   use pi::agent::{Agent, AgentConfig, AgentEvent as PiAgentEvent};
//   use pi::session::Session;
//   use pi::providers;

/// Bridge to pi_agent_rust.
///
/// This struct manages the connection to pi's agent system and
/// translates between pi's events and HawkTUI's event system.
pub struct PiBridge {
    /// Model to use.
    model: String,

    /// Provider to use.
    provider: String,

    /// Whether the bridge is connected.
    connected: bool,

    // Pi agent integration fields (uncomment when dependency is available):
    // agent: Option<Agent>,
    // session: Option<Session>,
}

impl PiBridge {
    /// Create a new bridge.
    pub fn new(model: Option<String>, provider: Option<String>) -> Self {
        Self {
            model: model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
            provider: provider.unwrap_or_else(|| "anthropic".to_string()),
            connected: false,
        }
    }

    /// Connect to the agent.
    ///
    /// When pi_agent_rust is available, this will initialize the agent:
    /// ```ignore
    /// let config = AgentConfig {
    ///     model: self.model.clone(),
    ///     provider: self.provider.clone(),
    ///     ..Default::default()
    /// };
    /// self.agent = Some(Agent::new(config).await?);
    /// ```
    pub async fn connect(&mut self) -> Result<()> {
        self.connected = true;
        Ok(())
    }

    /// Check if connected.
    pub const fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get the current model.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Set the model.
    pub fn set_model(&mut self, model: impl Into<String>) {
        self.model = model.into();
    }

    /// Get the current provider.
    pub fn provider(&self) -> &str {
        &self.provider
    }

    /// Set the provider.
    pub fn set_provider(&mut self, provider: impl Into<String>) {
        self.provider = provider.into();
    }

    /// Send a message and stream the response.
    ///
    /// Returns a channel receiver for agent events.
    ///
    /// When pi_agent_rust is available, this will send through the agent:
    /// ```ignore
    /// let user_message = UserMessage::new(message);
    /// self.agent.as_mut().unwrap().send(user_message, |event| {
    ///     let hawk_event = translate_event(event);
    ///     on_event(hawk_event);
    /// }).await?;
    /// ```
    pub async fn send_message(
        &mut self,
        message: &str,
        _on_event: impl Fn(AgentEvent) + Send + 'static,
    ) -> Result<()> {
        if !self.connected {
            return Err(Error::agent("Not connected to agent"));
        }

        // Placeholder: log the message (actual streaming handled in app.rs)
        tracing::info!("Sending message: {message}");
        
        Ok(())
    }

    /// Cancel the current operation.
    ///
    /// When pi_agent_rust is available:
    /// ```ignore
    /// if let Some(agent) = &mut self.agent {
    ///     agent.cancel();
    /// }
    /// ```
    pub fn cancel(&mut self) {
        tracing::info!("Cancelling operation");
    }

    /// Load a session.
    ///
    /// When pi_agent_rust is available:
    /// ```ignore
    /// self.session = Some(Session::load(session_id).await?);
    /// ```
    pub async fn load_session(&mut self, _session_id: &str) -> Result<()> {
        Ok(())
    }

    /// Create a new session.
    ///
    /// When pi_agent_rust is available:
    /// ```ignore
    /// let session = Session::new(name).await?;
    /// let id = session.id().to_string();
    /// self.session = Some(session);
    /// Ok(id)
    /// ```
    pub async fn create_session(&mut self, _name: &str) -> Result<String> {
        Ok(uuid::Uuid::new_v4().to_string())
    }

    /// List available sessions.
    ///
    /// When pi_agent_rust is available, this will query pi's session storage.
    /// For now, returns an empty list.
    pub async fn list_sessions(&self) -> Result<Vec<SessionSummary>> {
        Ok(Vec::new())
    }

    /// Get available tools.
    ///
    /// When pi_agent_rust is available, this will query pi's tool registry.
    /// For now, returns a representative stub list.
    pub fn available_tools(&self) -> Vec<ToolSummary> {
        vec![
            ToolSummary {
                name: "read_file".to_string(),
                description: "Read contents of a file".to_string(),
                enabled: true,
            },
            ToolSummary {
                name: "write_file".to_string(),
                description: "Write contents to a file".to_string(),
                enabled: true,
            },
            ToolSummary {
                name: "bash".to_string(),
                description: "Execute bash commands".to_string(),
                enabled: true,
            },
            ToolSummary {
                name: "glob".to_string(),
                description: "Find files matching a pattern".to_string(),
                enabled: true,
            },
        ]
    }
}

/// Session summary for listing.
#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: String,
    pub name: String,
    pub message_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Tool summary.
#[derive(Debug, Clone)]
pub struct ToolSummary {
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

// Event translation for pi_agent_rust integration.
//
// When pi_agent_rust is available, implement this function:
// ```ignore
// fn translate_event(event: PiAgentEvent) -> AgentEvent {
//     match event {
//         PiAgentEvent::TextDelta(text) => AgentEvent::TextDelta { text },
//         PiAgentEvent::ThinkingDelta(text) => AgentEvent::ThinkingDelta { text },
//         PiAgentEvent::ToolStart { name, input, .. } => {
//             AgentEvent::ToolStart { id: "id".into(), name, input }
//         }
//         // ... additional event mappings
//     }
// }
// ```
