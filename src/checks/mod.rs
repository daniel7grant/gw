use crate::context::Context;
use mockall::automock;
use thiserror::Error;

/// A check to fetch and pull a local git repository.
pub mod git;
/// A check to watch a directory for changes.
pub mod watch;

/// A custom error for describing the error cases for checks
#[derive(Debug, Error)]
pub enum CheckError {
    /// Cannot initialize check, because it has a misconfiguration.
    #[error("not configured correctly: {0}")]
    Misconfigured(String),
    /// Cannot run check, because there isn't enough permission.
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    /// Cannot update the check, because there is a conflict.
    /// This can be a merge conflict, a filesystem issue
    #[error("there is a conflict: {0}")]
    Conflict(String),
    /// Running the trigger failed.
    #[error("failed while running: {0}")]
    FailedUpdate(String),
}

/// A check is a process that tests if there are any changes and updates it.
///
/// Checks may include:
///   - git fetch and git pull ([git::GitCheck])
///   - watch a directory for updates ([watch::WatchCheck])
///   - etc.
#[automock]
pub trait Check {
    /// Check if there are changes and update if necessary.
    fn check(&mut self, context: &mut Context) -> Result<bool, CheckError>;
}
