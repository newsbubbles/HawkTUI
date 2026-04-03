//! Reusable widgets for HawkTUI.

pub mod spinner;
pub mod streaming;

pub use spinner::{next_frame, Spinner, DOTS_FRAMES, HAWK_FRAMES, MOON_FRAMES, SPINNER_FRAMES};
pub use streaming::{StreamingIndicator, ThinkingIndicator};
