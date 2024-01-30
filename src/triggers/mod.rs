use crate::Result;
use mockall::automock;
use std::sync::mpsc::Sender;

/// A trigger that runs on an HTTP request.
pub mod http;
/// A trigger that runs the checks once and then exits.
pub mod once;
/// A trigger that runs the checks periodically.
pub mod schedule;

/// A trigger is a long running background process, which initiates the checks.
///
/// Triggers may include:
///   - schedules
///   - HTTP servers
///   - etc.
#[automock]
pub trait Trigger: Sync + Send {
    /// Start the trigger process.
    fn listen(&self, tx: Sender<Option<()>>) -> Result<()>;
}
