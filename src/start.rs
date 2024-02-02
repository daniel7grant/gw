use crate::{actions::Action, checks::Check, triggers::Trigger, Result};
use std::{error::Error, sync::mpsc, thread};

/// The main program loop. It takes a list of triggers, one check and list of actions.
pub fn start(
    triggers: Vec<Box<dyn Trigger>>,
    check: &mut Box<dyn Check>,
    actions: &[Box<dyn Action>],
) -> Result<()> {
    let (tx, rx) = mpsc::channel::<Option<()>>();

    if triggers.is_empty() {
        return Err(Box::<dyn Error>::from(String::from(
            "You have to define at least one trigger.",
        )));
    }

    for trigger in triggers {
        let tx = tx.clone();
        thread::spawn(move || {
            let result = trigger.listen(tx);
            if let Err(err) = result {
                println!("Trigger failed: {err}.");
            }
        });
    }

    while let Ok(Some(())) = rx.recv() {
        if check.check()? {
            for action in actions.iter() {
                let _ = action.run();
            }
        }
    }

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

    #[test]
    fn it_should_call_once() {
        // Setup mock triggers.
        let mut mock_trigger = MockTrigger::new();
        mock_trigger.expect_listen().returning(|tx| {
            tx.send(Some(()))?;
            tx.send(None)?;
            Ok(())
        });
        let triggers: Vec<Box<dyn Trigger>> = vec![Box::new(mock_trigger)];

        // Setup mock check.
        let mut mock_check = MockCheck::new();
        mock_check.expect_check().times(1).returning(|| Ok(true));
        let mut check: Box<dyn Check> = Box::new(mock_check);

        // Setup mock action.
        let mut mock_action = MockAction::new();
        mock_action.expect_run().times(1).returning(|| Ok(()));
        let actions: &[Box<dyn Action>] = &[Box::new(mock_action)];

        let result = start(triggers, &mut check, actions);
        assert!(result.is_ok());
    }

    #[test]
    fn it_should_not_run_on_a_false_check() {
        // Setup mock triggers.
        let mut mock_trigger = MockTrigger::new();
        mock_trigger.expect_listen().returning(|tx| {
            tx.send(Some(()))?;
            tx.send(None)?;
            Ok(())
        });
        let triggers: Vec<Box<dyn Trigger>> = vec![Box::new(mock_trigger)];

        // Setup mock check.
        let mut mock_check = MockCheck::new();
        mock_check.expect_check().times(1).returning(|| Ok(false));
        let mut check: Box<dyn Check> = Box::new(mock_check);

        // Setup mock action.
        let mut mock_action = MockAction::new();
        mock_action.expect_run().times(0);
        let actions: &[Box<dyn Action>] = &[Box::new(mock_action)];

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
        let actions: &[Box<dyn Action>] = &[Box::new(mock_action)];

        let result = start(triggers, &mut check, actions);
        assert!(result.is_err());
    }
}
