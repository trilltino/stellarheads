pub mod audio;
pub mod gameplay;
pub mod scoring;
pub mod ui;

// Re-export commonly used UI types for better IDE support
pub use ui::{AppState, StateUIPlugin, UIPlugin};