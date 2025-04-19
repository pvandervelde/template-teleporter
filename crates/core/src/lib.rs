// Declare modules and re-export their public items.
mod types;
pub use types::*;

mod utils;
pub use utils::*;

mod state_manager;
pub use state_manager::*;

mod updater;
pub use updater::*;

mod filesystem_backend;
pub use filesystem_backend::*;

// Test module declaration
#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
