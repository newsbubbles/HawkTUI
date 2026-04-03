//! UI components for HawkTUI.
//!
//! Built on ratatui for high-performance terminal rendering.

pub mod layout;
pub mod panels;
pub mod themes;
pub mod widgets;

pub use layout::{Layout, LayoutManager};
pub use themes::{Theme, ThemeColors};
