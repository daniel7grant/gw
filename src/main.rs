use args::parse_args;
use gw_bin::{
    actions::{script::ScriptAction, Action},
    checks::{git::GitCheck, Check, CheckError},
    start::{start, StartError},
    triggers::{http::HttpTrigger, once::OnceTrigger, schedule::ScheduleTrigger, Trigger},
};
use log::{debug, error, warn, LevelFilter, SetLoggerError};
use simple_logger::SimpleLogger;
use std::{process, time::Duration};
use thiserror::Error;

mod args;

#[derive(Debug, Error)]
pub enum MainError {
    #[error("You have to pass a directory to watch.")]
    MissingDirectory,
    #[error("Check failed: {0}.")]
    FailedCheck(#[from] CheckError),
    #[error("Failed setting up logger.")]
    FailedLogger(#[from] SetLoggerError),
    #[error(transparent)]
    FailedStart(#[from] StartError),
}

fn main_inner() -> Result<(), MainError> {
    let args = parse_args();

    SimpleLogger::new()
        .with_level(match args.verbose {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        })
        .env()
        .init()?;

    // Setup triggers.
    let mut triggers: Vec<Box<dyn Trigger>> = vec![];
    if args.once {
        debug!("Setting up OnceTrigger (this will disable all other triggers).");
        triggers.push(Box::new(OnceTrigger));
    } else {
        let duration: Duration = args.delay.into();
        if !duration.is_zero() {
            debug!("Setting up ScheduleTrigger on every {}.", args.delay);
            triggers.push(Box::new(ScheduleTrigger::new(duration)));
        }
        if let Some(http) = args.http {
            debug!("Setting up HttpTrigger on {http}.");
            triggers.push(Box::new(HttpTrigger::new(http)));
        }
    }

    // Setup check.
    let directory = args.directory.ok_or(MainError::MissingDirectory)?;
    debug!("Setting up directory {directory} for GitCheck.");
    let mut check: Box<dyn Check> = Box::new(GitCheck::open(&directory)?);

    // Setup actions.
    let mut actions: Vec<Box<dyn Action>> = vec![];
    for script in args.scripts {
        debug!("Setting up ScriptAction '{script}' on change.");
        actions.push(Box::new(ScriptAction::new(directory.clone(), script)));
    }

    if actions.is_empty() {
        warn!("There are no actions defined: we will only pull!");
    }

    // Start the main script.
    start(triggers, &mut check, &actions)?;
    Ok(())
}

fn main() {
    if let Err(err) = main_inner() {
        error!("{err}");
        process::exit(1);
    }
}
