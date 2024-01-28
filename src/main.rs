use args::parse_args;
use gw_bin::{
    actions::{script::ScriptAction, Action},
    checks::{git::GitCheck, Check},
    triggers::{http::HttpTrigger, once::OnceTrigger, schedule::ScheduleTrigger, Trigger},
    Result,
};
use std::{error::Error, process, sync::mpsc, time::Duration};

mod args;

fn start(
    triggers: &Vec<Box<dyn Trigger>>,
    check: &mut Box<dyn Check>,
    actions: &Vec<Box<dyn Action>>,
) -> Result<()> {
    let (tx, rx) = mpsc::channel::<Option<()>>();

    if triggers.len() > 0 {
        for trigger in triggers {
            let tx = tx.clone();
            trigger.listen(tx)?;
        }
    } else {
        return Err(Box::<dyn Error>::from(String::from(
            "You have to define at least one trigger.",
        )));
    }

    while let Ok(Some(())) = rx.recv() {
        if check.check()? {
            for action in actions.iter() {
                action.run()?;
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
    match start(&triggers, &mut check, &mut actions) {
        Ok(()) => Ok(()),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use gw_bin::{
        actions::{test::TestAction, Action},
        checks::{test::TestCheck, Check},
        triggers::{test::TestTrigger, Trigger},
    };

    use crate::start;

    #[test]
    fn it_should_call_once() {
        let triggers: Vec<Box<dyn Trigger>> = vec![Box::new(TestTrigger::new())];
        let mut check: Box<dyn Check> = Box::new(TestCheck::new());
        let actions: Vec<Box<dyn Action>> = vec![Box::new(TestAction::new())];

        let result = start(&triggers, &mut check, &actions);
        assert!(result.is_ok());
    }

    #[test]
    fn it_should_call_fail_without_triggers() {
        let triggers: Vec<Box<dyn Trigger>> = vec![];
        let mut check: Box<dyn Check> = Box::new(TestCheck::new());
        let actions: Vec<Box<dyn Action>> = vec![Box::new(TestAction::new())];

        let result = start(&triggers, &mut check, &actions);
        assert!(result.is_err());
    }
}
