//! Core library for Template Teleporter.
//! Defines shared types, utilities, state management traits, and update logic.

// Declare modules and re-export their public items.
mod types;
pub use types::*;

mod utils;
pub use utils::*;

mod state_manager;
pub use state_manager::*;

mod updater;
pub use updater::*;

mod filesystem_backend; // Added for testing
pub use filesystem_backend::*; // Added for testing
