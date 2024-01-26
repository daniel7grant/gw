use crate::Result;

/// A trigger that runs on an HTTP request
pub mod http;
/// A trigger that runs the checks periodically
pub mod schedule;

/// A trigger is a long running background process, which initiates the checks.
///
/// Triggers may include:
///   - schedules
///   - HTTP servers
///   - etc.
pub trait Trigger {
    /// Start the trigger process.
    fn listen(self: Self) -> Result<()>;
}
