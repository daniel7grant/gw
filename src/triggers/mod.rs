use mockall::automock;
use std::sync::mpsc::Sender;
use thiserror::Error;

/// A trigger that runs on an HTTP request.
pub mod http;
/// A trigger that runs the checks once and then exits.
pub mod once;
/// A trigger that runs the checks periodically.
pub mod schedule;

/// A custom error for describing the error cases for triggers
#[derive(Debug, Error)]
pub enum TriggerError {
    /// Cannot initialize trigger, because it has a misconfiguration.
    #[error("not configured correctly: {0}")]
    Misconfigured(String),
    /// Cannot send trigger with Sender. This usually because the receiver is dropped.
    #[error("cannot trigger changes, receiver hang up")]
    ReceiverHangup(#[from] std::sync::mpsc::SendError<Option<()>>),
    /// Running the trigger failed.
    #[error("{0}")]
    FailedTrigger(String),
}

/// A trigger is a long running background process, which initiates the checks.
///
/// Triggers may include:
///   - schedules ([schedule::ScheduleTrigger])
///   - HTTP servers ([http::HttpTrigger])
///   - etc.
#[automock]
pub trait Trigger: Sync + Send {
    /// Start the trigger process.
    fn listen(&self, tx: Sender<Option<()>>) -> Result<(), TriggerError>;
}
