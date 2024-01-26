use gw_bin::{
    actions::Action,
    checks::{self, Check},
    triggers::{http::HttpTrigger, schedule::ScheduleTrigger, Trigger},
    Result,
};
use std::{process, sync::mpsc};

fn start(
    triggers: &Vec<Box<dyn Trigger>>,
    check: &mut Box<dyn Check>,
    actions: &Vec<Box<dyn Action>>,
) -> Result<()> {
    let (tx, rx) = mpsc::channel::<Option<()>>();

    for trigger in triggers {
        let tx = tx.clone();
        trigger.listen(&tx)?;
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
    let triggers: Vec<Box<dyn Trigger>> = vec![Box::new(HttpTrigger), Box::new(ScheduleTrigger)];
    let mut check: Box<dyn Check> = Box::new(checks::git::GitCheck);
    let mut actions: Vec<Box<dyn Action>> = vec![];

    match start(&triggers, &mut check, &mut actions) {
        Ok(()) => Ok(()),
        Err(err) => {
            eprintln!("{}", err.to_string());
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
}
