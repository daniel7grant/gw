use crate::Result;

/// A check is a process that tests if there are any changes and updates it.
///
/// Checks may include:
///   - git fetch and git pull
///   - watch a directory for updates
///   - etc.
pub trait Check {
	/// Check if there are changes and update if necessary.
    fn check(self: &mut Self) -> Result<bool>;
}