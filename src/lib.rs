//! Watch local git repositories, keep in sync with remote and run commands.
//!
//! ## Keywords
//!
//!

use std::error::Error;

pub mod actions;
pub mod checks;
pub mod triggers;

/// Shorthand result type
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
