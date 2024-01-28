use crate::Result;

/// An action to run a custom shell script.
pub mod script;
/// Test implementation of an action, internal use only.
pub mod test;

/// An action is a process that runs if any changes occured.
///
/// Actions may include:
///   - running scripts
///   - etc.
pub trait Action {
    /// Initiate the action
    fn run(&self) -> Result<()>;
}
