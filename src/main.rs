use args::parse_args;
use gw_bin::{
    actions::{script::ScriptAction, Action},
    checks::{git::GitCheck, Check},
    triggers::{http::HttpTrigger, once::OnceTrigger, schedule::ScheduleTrigger, Trigger},
    Result,
};
use std::{error::Error, process, sync::mpsc, thread, time::Duration};

mod args;

fn start(
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

fn main() -> Result<()> {
    let args = parse_args();

    // Setup triggers.
    let mut triggers: Vec<Box<dyn Trigger>> = vec![];
    if args.once {
        triggers.push(Box::new(OnceTrigger));
    } else {
        let duration: Duration = args.delay.into();
        if !duration.is_zero() {
            triggers.push(Box::new(ScheduleTrigger::new(duration)));
        }
        if let Some(http) = args.http {
            triggers.push(Box::new(HttpTrigger::new(http)));
        }
    }

    // Setup check.
    let directory = args.directory.ok_or(Box::<dyn Error>::from(String::from(
        "You have to pass a directory to watch.",
    )))?;
    let mut check: Box<dyn Check> = Box::new(GitCheck::open(&directory)?);

    // Setup actions.
    let mut actions: Vec<Box<dyn Action>> = vec![];
    for script in args.scripts {
        actions.push(Box::new(ScriptAction::new(directory.clone(), script)));
    }

    // Start the main script.
    match start(triggers, &mut check, &actions) {
        Ok(()) => Ok(()),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::start;
    use gw_bin::{
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
