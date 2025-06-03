//! The `settings` module is a simple utility that requires manual verification.
//! See `bin/settings_demo.rs` for a test binary demonstrating its usage.

mod cli;
pub use cli::*;
pub use clap::Parser;

mod settings;
pub use settings::*;