use args::parse_args;
use gw_bin::{
    actions::{script::ScriptAction, Action},
    checks::{git::GitCheck, Check},
    start::start,
    triggers::{http::HttpTrigger, once::OnceTrigger, schedule::ScheduleTrigger, Trigger},
    Result,
};
use std::{error::Error, process, time::Duration};

mod args;

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
