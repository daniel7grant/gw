use crate::context::Context;
use mockall::automock;
use thiserror::Error;

/// An action to run in the background and restart a subprocess.
pub mod process;
/// An action to run a custom shell script.
pub mod script;

/// A custom error for describing the error cases for actions
#[derive(Debug, Error)]
pub enum ActionError {
    /// Cannot initialize action, because it has a misconfiguration.
    #[error("not configured correctly: {0}")]
    Misconfigured(String),
    /// Cannot run action, because there isn't enough permission.
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    /// Running action failed. It is usually a runtime issue.
    #[error("{0}")]
    FailedAction(String),
}

/// An action is a process that runs if any changes occured.
///
/// Actions may include:
///   - running scripts ([script::ScriptAction])
///   - etc.
#[automock]
pub trait Action {
    /// Initiate the action
    fn run(&mut self, context: &Context) -> Result<(), ActionError>;
}
