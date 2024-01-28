use crate::Result;
use std::sync::mpsc::Sender;

/// A trigger that runs on an HTTP request
pub mod http;
/// A trigger that runs the checks once and then exits
pub mod once;
/// A trigger that runs the checks periodically
pub mod schedule;
/// Test implementation for trigger, internal use only
pub mod test;

/// A trigger is a long running background process, which initiates the checks.
///
/// Triggers may include:
///   - schedules
///   - HTTP servers
///   - etc.
pub trait Trigger {
    /// Start the trigger process.
    fn listen(&self, tx: Sender<Option<()>>) -> Result<()>;
}
