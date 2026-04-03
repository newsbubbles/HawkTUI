//! # HawkTUI
//!
//! A premium terminal user interface for pi_agent_rust.
//!
//! > "See everything. Control everything. Code like a hawk." 🦅
//!
//! HawkTUI transforms the powerful pi_agent_rust AI coding agent into an
//! immersive, visually stunning terminal experience with:
//!
//! - **Multi-panel layouts** for conversations, sessions, tools, and context
//! - **Real-time streaming** with smooth animations
//! - **Session management** with branching and history
//! - **Beautiful themes** customizable via TOML
//! - **Keyboard-first** workflow with zero friction
//!
//! ## Quick Start
//!
//! ```no_run
//! use hawktui::App;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let app = App::new()?;
//!     app.run().await
//! }
//! ```

#![forbid(unsafe_code)]
#![allow(dead_code, clippy::unused_async)]
#![allow(
    clippy::must_use_candidate,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::similar_names
)]

pub mod app;
pub mod core;
pub mod providers;
pub mod ui;

pub use app::App;
pub use core::error::{Error, Result};
