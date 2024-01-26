use crate::Result;

/// A check to fetch and pull a local git repository
pub mod git;
/// Test implementation of check, internal use only
pub mod test;
/// A check to watch a directory for changes
pub mod watch;

/// A check is a process that tests if there are any changes and updates it.
///
/// Checks may include:
///   - git fetch and git pull
///   - watch a directory for updates
///   - etc.
pub trait Check {
	/// Check if there are changes and update if necessary.
    fn check(&mut self) -> Result<bool>;
}