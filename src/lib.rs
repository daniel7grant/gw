//! Watch local git repositories, keep in sync with remote and run commands.
//!
//! ## How it works
//!
//! `gw` is built up from **triggers**, **checks** and **actions**.
//! Triggers are long running background processes that initiates checks
//! (for example periodic triggers, or HTTP triggers). Checks tests
//! if there are any changes in the directory (be it git or filesystem changes)
//! and runs the actions if there was. Actions are arbitrary code that runs
//! (e.g. user-defined shell scripts).
//!
//! ```ignore
//! +---------+       +--------+       +--------+
//! | trigger | ----> | checks | ----> | action |
//! +---------+       +--------+       +--------+
//! ```
//!

use std::error::Error;

/// An action is a process that runs if any changes occured (e.g. [running scripts](actions::script::ScriptAction)).
pub mod actions;
/// A check is a process that tests if there are any changes and updates it.
pub mod checks;
/// A trigger is a long running background process, which initiates the checks
/// (e.g. [on a schedule](triggers::schedule::ScheduleTrigger), [on HTTP request](triggers::http::HttpTrigger)
/// or [once](triggers::once::OnceTrigger)).
pub mod triggers;

/// Shorthand result type
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
