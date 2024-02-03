use args::parse_args;
use gw_bin::{
    actions::{script::ScriptAction, Action},
    checks::{git::GitCheck, Check, CheckError},
    start::{start, StartError},
    triggers::{http::HttpTrigger, once::OnceTrigger, schedule::ScheduleTrigger, Trigger},
};
use std::{process, time::Duration};
use thiserror::Error;

mod args;

#[derive(Debug, Error)]
pub enum MainError {
    #[error("You have to pass a directory to watch.")]
    MissingDirectory,
    #[error("Check failed: {0}.")]
    FailedCheck(#[from] CheckError),
    #[error(transparent)]
    FailedStart(#[from] StartError),
}

fn main_inner() -> Result<(), MainError> {
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
    let directory = args.directory.ok_or(MainError::MissingDirectory)?;
    let mut check: Box<dyn Check> = Box::new(GitCheck::open(&directory)?);

    // Setup actions.
    let mut actions: Vec<Box<dyn Action>> = vec![];
    for script in args.scripts {
        actions.push(Box::new(ScriptAction::new(directory.clone(), script)));
    }

    // Start the main script.
    start(triggers, &mut check, &actions)?;
    Ok(())
}

fn main() {
    if let Err(err) = main_inner() {
        eprintln!("{err}");
        process::exit(1);
    }
}
