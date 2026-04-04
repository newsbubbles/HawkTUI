//! Bridge to pi_agent_rust library.
//!
//! This module provides the integration layer between HawkTUI and
//! pi_agent_rust's agent, session, and provider systems.
//!
//! ## Integration Strategy
//!
//! HawkTUI integrates with pi_agent_rust at the library level using the
//! stable `pi::sdk` API surface:
//!
//! 1. **Agent**: Use `AgentSessionHandle` for AI interactions
//! 2. **Session**: Share session storage with pi CLI
//! 3. **Providers**: Reuse provider implementations
//! 4. **Tools**: Access the same tool registry
//!
//! This allows HawkTUI to:
//! - Share sessions with the `pi` CLI
//! - Use the same configuration
//! - Benefit from pi's streaming and tool infrastructure

use crate::core::error::{Error, Result};
use crate::core::events::{AgentEvent, StopReason};

use pi::model::{AssistantMessageEvent, StopReason as PiStopReason};
use pi::sdk::{
    AbortHandle, AgentEvent as PiAgentEvent, AgentSessionHandle, AssistantMessage,
    Config as PiConfig, ContentBlock, SessionOptions, ToolDefinition, ToolOutput,
    all_tool_definitions, create_agent_session,
};

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Bridge to pi_agent_rust.
///
/// This struct manages the connection to pi's agent system and
/// translates between pi's events and HawkTUI's event system.
pub struct PiBridge {
    /// Session handle (None until connected).
    handle: Option<Arc<Mutex<AgentSessionHandle>>>,

    /// Abort handle for current operation.
    abort_handle: Option<AbortHandle>,

    /// Model to use (None = use default from config).
    model: Option<String>,

    /// Provider to use (None = use default from config).
    provider: Option<String>,

    /// Working directory for tools.
    working_directory: Option<PathBuf>,

    /// Whether the bridge is connected.
    connected: bool,
}

impl PiBridge {
    /// Create a new bridge.
    pub fn new(model: Option<String>, provider: Option<String>) -> Self {
        Self {
            handle: None,
            abort_handle: None,
            model,
            provider,
            working_directory: None,
            connected: false,
        }
    }

    /// Set the working directory for tools.
    pub fn set_working_directory(&mut self, path: impl Into<PathBuf>) {
        self.working_directory = Some(path.into());
    }

    /// Connect to the agent.
    ///
    /// Initializes the agent session using pi's SDK.
    pub async fn connect(&mut self) -> Result<()> {
        let options = SessionOptions {
            provider: self.provider.clone(),
            model: self.model.clone(),
            working_directory: self.working_directory.clone(),
            no_session: false, // Enable session persistence
            include_cwd_in_prompt: true,
            ..SessionOptions::default()
        };

        let handle = create_agent_session(options)
            .await
            .map_err(|e| Error::agent(format!("Failed to create agent session: {e}")))?;

        self.handle = Some(Arc::new(Mutex::new(handle)));
        self.connected = true;

        tracing::info!("Connected to pi agent");
        Ok(())
    }

    /// Check if connected.
    pub const fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get the current model.
    pub async fn model(&self) -> String {
        if let Some(handle) = &self.handle {
            let guard = handle.lock().await;
            let (_, model_id) = guard.model();
            model_id
        } else {
            self.model
                .clone()
                .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string())
        }
    }

    /// Get the current provider.
    pub async fn provider(&self) -> String {
        if let Some(handle) = &self.handle {
            let guard = handle.lock().await;
            let (provider, _) = guard.model();
            provider
        } else {
            self.provider
                .clone()
                .unwrap_or_else(|| "anthropic".to_string())
        }
    }

    /// Set the model and provider.
    pub async fn set_model(
        &mut self,
        provider: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<()> {
        let provider = provider.into();
        let model = model.into();

        if let Some(handle) = &self.handle {
            let mut guard = handle.lock().await;
            guard
                .set_model(&provider, &model)
                .await
                .map_err(|e| Error::agent(format!("Failed to set model: {e}")))?;
        }

        self.provider = Some(provider);
        self.model = Some(model);
        Ok(())
    }

    /// Send a message and stream the response.
    ///
    /// The callback receives translated HawkTUI events.
    pub async fn send_message<F>(&mut self, message: &str, on_event: F) -> Result<AssistantMessage>
    where
        F: Fn(AgentEvent) + Send + Sync + 'static,
    {
        let handle = self
            .handle
            .as_ref()
            .ok_or_else(|| Error::agent("Not connected to agent"))?;

        // Create abort handle for this operation
        let (abort_handle, abort_signal) = AbortHandle::new();
        self.abort_handle = Some(abort_handle);

        let on_event = Arc::new(on_event);
        let on_event_clone = Arc::clone(&on_event);

        // Translate pi events to HawkTUI events
        let translator = move |event: PiAgentEvent| {
            if let Some(hawk_event) = translate_pi_event(&event) {
                on_event_clone(hawk_event);
            }
        };

        let mut guard = handle.lock().await;
        let result = guard
            .prompt_with_abort(message, abort_signal, translator)
            .await
            .map_err(|e| Error::agent(format!("Agent error: {e}")))?;

        // Clear abort handle after completion
        self.abort_handle = None;

        // Send usage event
        // Send usage event
        let usage = &result.usage;
        on_event(AgentEvent::Usage {
            input_tokens: usage.input,
            output_tokens: usage.output,
            cache_read_tokens: Some(usage.cache_read),
            cache_write_tokens: Some(usage.cache_write),
        });

        Ok(result)
    }

    /// Cancel the current operation.
    pub fn cancel(&mut self) {
        if let Some(abort_handle) = self.abort_handle.take() {
            abort_handle.abort();
            tracing::info!("Cancelled current operation");
        }
    }

    /// Load a session by path.
    pub async fn load_session(&mut self, session_path: &str) -> Result<()> {
        // Disconnect current session
        self.handle = None;
        self.connected = false;

        // Reconnect with specific session
        let options = SessionOptions {
            provider: self.provider.clone(),
            model: self.model.clone(),
            working_directory: self.working_directory.clone(),
            session_path: Some(PathBuf::from(session_path)),
            no_session: false,
            include_cwd_in_prompt: true,
            ..SessionOptions::default()
        };

        let handle = create_agent_session(options)
            .await
            .map_err(|e| Error::agent(format!("Failed to load session: {e}")))?;

        self.handle = Some(Arc::new(Mutex::new(handle)));
        self.connected = true;

        tracing::info!("Loaded session: {session_path}");
        Ok(())
    }

    /// Create a new session.
    pub async fn create_session(&mut self, _name: &str) -> Result<String> {
        // Disconnect current session
        self.handle = None;
        self.connected = false;

        // Create new session
        let options = SessionOptions {
            provider: self.provider.clone(),
            model: self.model.clone(),
            working_directory: self.working_directory.clone(),
            no_session: false,
            include_cwd_in_prompt: true,
            ..SessionOptions::default()
        };

        let handle = create_agent_session(options)
            .await
            .map_err(|e| Error::agent(format!("Failed to create session: {e}")))?;

        // Get the session ID from state
        let session_id = {
            let state = handle.state().await
                .map_err(|e| Error::agent(format!("Failed to get session state: {e}")))?;
            state.session_id.unwrap_or_else(|| "new_session".to_string())
        };

        self.handle = Some(Arc::new(Mutex::new(handle)));
        self.connected = true;

        tracing::info!("Created new session: {session_id}");
        Ok(session_id)
    }

    /// List available sessions.
    ///
    /// Queries pi's session storage directory.
    pub async fn list_sessions(&self) -> Result<Vec<SessionSummary>> {
        // Get session directory from pi's config
        let session_dir = PiConfig::sessions_dir();

        let mut sessions = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&session_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        // Try to read session metadata
                        let metadata = entry.metadata().ok();
                        let created = metadata
                            .as_ref()
                            .and_then(|m| m.created().ok())
                            .map(chrono::DateTime::<chrono::Utc>::from)
                            .unwrap_or_else(chrono::Utc::now);
                        let modified = metadata
                            .as_ref()
                            .and_then(|m| m.modified().ok())
                            .map(chrono::DateTime::<chrono::Utc>::from)
                            .unwrap_or_else(chrono::Utc::now);

                        sessions.push(SessionSummary {
                            id: path.to_string_lossy().to_string(),
                            name: name.to_string(),
                            message_count: 0, // Would need to parse file to get this
                            created_at: created,
                            updated_at: modified,
                        });
                    }
                }
            }
        }

        // Sort by updated_at descending
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(sessions)
    }

    /// Get available tools.
    ///
    /// Returns tool definitions from pi's tool registry.
    pub fn available_tools(&self) -> Vec<ToolSummary> {
        let cwd = self
            .working_directory
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        all_tool_definitions(&cwd)
            .into_iter()
            .map(|def| ToolSummary {
                name: def.name,
                description: def.description,
                enabled: true,
            })
            .collect()
    }

    /// Get a specific tool definition.
    pub fn get_tool(&self, name: &str) -> Option<ToolDefinition> {
        let cwd = self
            .working_directory
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        all_tool_definitions(&cwd)
            .into_iter()
            .find(|def| def.name == name)
    }

    /// Get session state.
    pub async fn state(&self) -> Result<Option<SessionState>> {
        let Some(handle) = &self.handle else {
            return Ok(None);
        };

        let guard = handle.lock().await;
        let state = guard
            .state()
            .await
            .map_err(|e| Error::agent(format!("Failed to get state: {e}")))?;

        let (provider, model_id) = guard.model();

        Ok(Some(SessionState {
            session_id: state.session_id,
            provider,
            model_id,
            message_count: state.message_count,
            save_enabled: state.save_enabled,
        }))
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

/// Session state snapshot.
#[derive(Debug, Clone)]
pub struct SessionState {
    pub session_id: Option<String>,
    pub provider: String,
    pub model_id: String,
    pub message_count: usize,
    pub save_enabled: bool,
}

/// Translate pi agent events to HawkTUI events.
fn translate_pi_event(event: &PiAgentEvent) -> Option<AgentEvent> {
    match event {
        PiAgentEvent::TurnStart { .. } => Some(AgentEvent::StreamStart {
            message_id: uuid::Uuid::new_v4(),
        }),

        PiAgentEvent::MessageUpdate {
            assistant_message_event,
            ..
        } => translate_message_event(assistant_message_event),

        PiAgentEvent::ToolExecutionStart {
            tool_call_id,
            tool_name,
            args,
        } => Some(AgentEvent::ToolStart {
            id: tool_call_id.clone(),
            name: tool_name.clone(),
            input: serde_json::to_string_pretty(args).unwrap_or_default(),
        }),

        PiAgentEvent::ToolExecutionUpdate { tool_call_id, .. } => {
            // Progress updates - we don't have exact progress, so estimate
            Some(AgentEvent::ToolProgress {
                id: tool_call_id.clone(),
                progress: 0.5, // Indeterminate progress
            })
        }

        PiAgentEvent::ToolExecutionEnd {
            tool_call_id,
            tool_name: _,
            result,
            is_error,
        } => Some(AgentEvent::ToolEnd {
            id: tool_call_id.clone(),
            output: format_tool_output(result),
            is_error: *is_error,
        }),

        PiAgentEvent::TurnEnd { .. } => Some(AgentEvent::StreamEnd {
            stop_reason: StopReason::EndTurn,
        }),

        PiAgentEvent::AgentEnd { error, .. } => {
            if let Some(err) = error {
                Some(AgentEvent::Error {
                    message: err.to_string(),
                })
            } else {
                Some(AgentEvent::StreamEnd {
                    stop_reason: StopReason::EndTurn,
                })
            }
        }

        PiAgentEvent::AutoRetryStart {
            attempt,
            max_attempts,
            error_message,
            ..
        } => Some(AgentEvent::Error {
            message: format!("Retrying ({attempt}/{max_attempts}): {error_message}"),
        }),

        PiAgentEvent::AutoRetryEnd {
            success,
            final_error,
            ..
        } => {
            if !success {
                if let Some(err) = final_error {
                    return Some(AgentEvent::Error {
                        message: format!("Retry failed: {err}"),
                    });
                }
            }
            None
        }

        PiAgentEvent::ExtensionError {
            extension_id,
            error,
            ..
        } => {
            let ext_id = extension_id.as_deref().unwrap_or("unknown");
            Some(AgentEvent::Error {
                message: format!("Extension {ext_id} error: {error}"),
            })
        }

        // Events we don't translate
        PiAgentEvent::AgentStart { .. } => None,
        PiAgentEvent::MessageStart { .. } => None,
        PiAgentEvent::MessageEnd { .. } => None,
        PiAgentEvent::AutoCompactionStart { .. } => None,
        PiAgentEvent::AutoCompactionEnd { .. } => None,
    }
}

/// Translate assistant message events to HawkTUI events.
fn translate_message_event(event: &AssistantMessageEvent) -> Option<AgentEvent> {
    match event {
        AssistantMessageEvent::TextDelta { delta, .. } => Some(AgentEvent::TextDelta {
            text: delta.clone(),
        }),

        AssistantMessageEvent::ThinkingStart { .. } => Some(AgentEvent::ThinkingStart),

        AssistantMessageEvent::ThinkingDelta { delta, .. } => Some(AgentEvent::ThinkingDelta {
            text: delta.clone(),
        }),

        AssistantMessageEvent::ThinkingEnd { .. } => Some(AgentEvent::ThinkingEnd),

        AssistantMessageEvent::Done { reason, .. } => Some(AgentEvent::StreamEnd {
            stop_reason: translate_stop_reason(*reason),
        }),

        AssistantMessageEvent::Error { error, .. } => {
            // error is Arc<AssistantMessage>, extract text from content blocks
            let message = error
                .content
                .iter()
                .filter_map(|block| {
                    if let ContentBlock::Text(text) = block {
                        Some(text.text.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            Some(AgentEvent::Error {
                message: if message.is_empty() {
                    "Unknown error".to_string()
                } else {
                    message
                },
            })
        }

        // Events we don't translate individually
        AssistantMessageEvent::Start { .. } => None,
        AssistantMessageEvent::TextStart { .. } => None,
        AssistantMessageEvent::TextEnd { .. } => None,
        AssistantMessageEvent::ToolCallStart { .. } => None,
        AssistantMessageEvent::ToolCallDelta { .. } => None,
        AssistantMessageEvent::ToolCallEnd { .. } => None,
    }
}

/// Translate pi stop reason to HawkTUI stop reason.
fn translate_stop_reason(reason: PiStopReason) -> StopReason {
    match reason {
        PiStopReason::Stop => StopReason::EndTurn,
        PiStopReason::Length => StopReason::MaxTokens,
        PiStopReason::ToolUse => StopReason::ToolUse,
        PiStopReason::Error => StopReason::Error,
        PiStopReason::Aborted => StopReason::Cancelled,
    }
}

/// Format tool output for display.
fn format_tool_output(output: &ToolOutput) -> String {
    // ToolOutput has content: Vec<ContentBlock> and is_error: bool
    let text_parts: Vec<String> = output
        .content
        .iter()
        .filter_map(|block| match block {
            ContentBlock::Text(text) => Some(text.text.clone()),
            _ => None,
        })
        .collect();

    if text_parts.is_empty() {
        if let Some(details) = &output.details {
            serde_json::to_string_pretty(details).unwrap_or_default()
        } else {
            String::new()
        }
    } else {
        text_parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pi_bridge_creation() {
        let bridge = PiBridge::new(None, None);
        assert!(!bridge.is_connected());
    }

    #[test]
    fn test_pi_bridge_custom_model() {
        let bridge = PiBridge::new(
            Some("claude-opus-4-20250514".to_string()),
            Some("anthropic".to_string()),
        );
        assert!(!bridge.is_connected());
    }

    #[test]
    fn test_available_tools() {
        let bridge = PiBridge::new(None, None);
        let tools = bridge.available_tools();
        // Should have pi's built-in tools
        assert!(!tools.is_empty());
        // Check for known tools
        let tool_names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"read"));
        assert!(tool_names.contains(&"bash"));
        assert!(tool_names.contains(&"edit"));
        assert!(tool_names.contains(&"write"));
    }

    #[test]
    fn test_translate_text_delta() {
        let partial_msg = Arc::new(AssistantMessage::default());
        let event = AssistantMessageEvent::TextDelta {
            content_index: 0,
            delta: "Hello".to_string(),
            partial: partial_msg,
        };
        let result = translate_message_event(&event);
        assert!(matches!(result, Some(AgentEvent::TextDelta { text }) if text == "Hello"));
    }

    #[test]
    fn test_translate_thinking_events() {
        let partial_msg = Arc::new(AssistantMessage::default());

        let start = AssistantMessageEvent::ThinkingStart {
            content_index: 0,
            partial: partial_msg.clone(),
        };
        assert!(matches!(
            translate_message_event(&start),
            Some(AgentEvent::ThinkingStart)
        ));

        let delta = AssistantMessageEvent::ThinkingDelta {
            content_index: 0,
            delta: "thinking...".to_string(),
            partial: partial_msg.clone(),
        };
        assert!(matches!(
            translate_message_event(&delta),
            Some(AgentEvent::ThinkingDelta { text }) if text == "thinking..."
        ));

        let end = AssistantMessageEvent::ThinkingEnd {
            content_index: 0,
            content: "done".to_string(),
            partial: partial_msg,
        };
        assert!(matches!(
            translate_message_event(&end),
            Some(AgentEvent::ThinkingEnd)
        ));
    }
}
