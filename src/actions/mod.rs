use crate::Result;
use mockall::automock;

/// An action to run a custom shell script.
pub mod script;

/// An action is a process that runs if any changes occured.
///
/// Actions may include:
///   - running scripts ([script::ScriptAction])
///   - etc.
#[automock]
pub trait Action {
    /// Initiate the action
    fn run(&self) -> Result<()>;
}
