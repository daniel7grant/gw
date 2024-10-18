use crate::{
    actions::Action,
    checks::{Check, CheckError},
    context::Context,
    triggers::{Trigger, TriggerError},
};
use log::{debug, error, info};
use std::{sync::mpsc, thread};
use thiserror::Error;

/// A custom error implementation for the start function
#[derive(Debug, Error)]
pub enum StartError {
    #[error("You have to define at least one trigger.")]
    NoTriggers,
    #[error("Trigger failed: {0}.")]
    MisconfiguredTrigger(#[from] TriggerError),
    #[error("Check failed: {0}.")]
    FailedCheck(#[from] CheckError),
}

/// The main program loop, that runs the triggers, checks and actions infinitely.
pub fn start(
    triggers: Vec<Box<dyn Trigger>>,
    check: &mut Box<dyn Check>,
    actions: &mut [Box<dyn Action>],
) -> Result<(), StartError> {
    let (tx, rx) = mpsc::channel::<Option<Context>>();

    if triggers.is_empty() {
        return Err(StartError::NoTriggers);
    }

    for trigger in triggers {
        let tx = tx.clone();
        thread::spawn(move || {
            let result = trigger.listen(tx);
            if let Err(err) = result {
                error!("Trigger failed: {err}.");
            }
        });
    }

    debug!("Waiting on triggers.");
    while let Ok(Some(mut context)) = rx.recv() {
        match check.check(&mut context) {
            Ok(true) => {
                info!(
                    "There are updates, {}.",
                    if actions.is_empty() {
                        "pulling"
                    } else {
                        "running actions"
                    }
                );
                for action in actions.iter_mut() {
                    let result = action.run(&context);
                    if let Err(err) = result {
                        error!("Action failed, we will not continue: {err}.");
                        break;
                    }
                }
            }
            Ok(false) => {
                debug!("There are no updates.");
            }
            Err(err) => {
                error!("Check failed: {err}.");
            }
        }
    }

    debug!("Finished running.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        actions::{Action, MockAction},
        checks::{Check, MockCheck},
        triggers::{MockTrigger, Trigger},
    };
    use std::collections::HashMap;

    #[test]
    fn it_should_call_once() {
        // Setup mock triggers.
        let mut mock_trigger = MockTrigger::new();
        mock_trigger.expect_listen().returning(|tx| {
            tx.send(Some(HashMap::new()))?;
            tx.send(None)?;
            Ok(())
        });
        let triggers: Vec<Box<dyn Trigger>> = vec![Box::new(mock_trigger)];

        // Setup mock check.
        let mut mock_check = MockCheck::new();
        mock_check.expect_check().times(1).returning(|_| Ok(true));
        let mut check: Box<dyn Check> = Box::new(mock_check);

        // Setup mock action.
        let mut mock_action = MockAction::new();
        mock_action.expect_run().times(1).returning(|_| Ok(()));
        let actions: &mut [Box<dyn Action>] = &mut [Box::new(mock_action)];

        let result = start(triggers, &mut check, actions);
        assert!(result.is_ok());
    }

    #[test]
    fn it_should_not_run_on_a_false_check() {
        // Setup mock triggers.
        let mut mock_trigger = MockTrigger::new();
        mock_trigger.expect_listen().returning(|tx| {
            tx.send(Some(HashMap::new()))?;
            tx.send(None)?;
            Ok(())
        });
        let triggers: Vec<Box<dyn Trigger>> = vec![Box::new(mock_trigger)];

        // Setup mock check.
        let mut mock_check = MockCheck::new();
        mock_check.expect_check().times(1).returning(|_| Ok(false));
        let mut check: Box<dyn Check> = Box::new(mock_check);

        // Setup mock action.
        let mut mock_action = MockAction::new();
        mock_action.expect_run().times(0);
        let actions: &mut [Box<dyn Action>] = &mut [Box::new(mock_action)];

        let result = start(triggers, &mut check, actions);
        assert!(result.is_ok());
    }

    #[test]
    fn it_should_not_run_on_a_failed_check() {
        // Setup mock triggers.
        let mut mock_trigger = MockTrigger::new();
        mock_trigger.expect_listen().returning(|tx| {
            tx.send(Some(HashMap::new()))?;
            tx.send(None)?;
            Ok(())
        });
        let triggers: Vec<Box<dyn Trigger>> = vec![Box::new(mock_trigger)];

        // Setup mock check.
        let mut mock_check = MockCheck::new();
        mock_check
            .expect_check()
            .times(1)
            .returning(|_| Err(CheckError::Misconfigured(String::from("Testing purposes."))));
        let mut check: Box<dyn Check> = Box::new(mock_check);

        // Setup mock action.
        let mut mock_action = MockAction::new();
        mock_action.expect_run().times(0);
        let actions: &mut [Box<dyn Action>] = &mut [Box::new(mock_action)];

        let result = start(triggers, &mut check, actions);
        assert!(result.is_ok());
    }

    #[test]
    fn it_should_fail_without_triggers() {
        // Setup empty triggers.
        let triggers: Vec<Box<dyn Trigger>> = vec![];

        // Setup mock check.
        let mut mock_check = MockCheck::new();
        mock_check.expect_check().times(0);
        let mut check: Box<dyn Check> = Box::new(mock_check);

        // Setup mock action.
        let mut mock_action = MockAction::new();
        mock_action.expect_run().times(0);
        let actions: &mut [Box<dyn Action>] = &mut [Box::new(mock_action)];

        let result = start(triggers, &mut check, actions);
        assert!(result.is_err());
    }
}
