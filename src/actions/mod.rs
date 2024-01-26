use crate::Result;

/// An action is a process that runs if any changes occured.
/// 
/// Actions may include:
///   - running scripts
///   - etc.
pub trait Action {
    /// Initiate the action
    fn run() -> Result<()>;
}
