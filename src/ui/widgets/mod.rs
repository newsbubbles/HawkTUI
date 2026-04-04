//! Reusable widgets for HawkTUI.

pub mod spinner;
pub mod streaming;

pub use spinner::{DOTS_FRAMES, HAWK_FRAMES, MOON_FRAMES, SPINNER_FRAMES, Spinner, next_frame};
pub use streaming::{StreamingIndicator, ThinkingIndicator};
