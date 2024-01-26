use crate::Result;

/// A trigger is a long running background process, which initiates the checks.
///
/// Triggers may include:
///   - schedules
///   - HTTP servers
///   - etc.
pub trait Trigger {
	/// Start the trigger process.
    fn listen() -> Result<()>;
}