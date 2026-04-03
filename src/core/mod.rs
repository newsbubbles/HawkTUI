//! Core application logic for HawkTUI.
//!
//! This module contains:
//! - State management
//! - Event handling
//! - Keybindings
//! - Slash commands

pub mod commands;
pub mod error;
pub mod events;
pub mod keybindings;
pub mod state;

pub use error::{Error, Result};
pub use events::Event;
pub use keybindings::KeyBindings;
pub use state::AppState;
