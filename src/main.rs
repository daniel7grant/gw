use args::parse_args;
use gw_bin::{
    actions::{script::ScriptAction, Action},
    checks::{git::GitCheck, Check, CheckError},
    start::{start, StartError},
    triggers::{http::HttpTrigger, once::OnceTrigger, schedule::ScheduleTrigger, Trigger},
};
use log::{debug, error, warn, LevelFilter, SetLoggerError};
use simple_logger::SimpleLogger;
use std::{fs, process, time::Duration};
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

    if args.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    SimpleLogger::new()
        .with_level(match (args.quiet, args.verbose) {
            (true, _) => LevelFilter::Error,
            (false, 0) => LevelFilter::Info,
            (false, 1) => LevelFilter::Debug,
            (false, _) => LevelFilter::Trace,
        })
        .env()
        .init()?;

    // Check if directory exists and convert to full path
    let directory_relative = args.directory.ok_or(MainError::MissingDirectory)?;
    let directory_path =
        fs::canonicalize(directory_relative).map_err(|_| MainError::MissingDirectory)?;
    let directory = directory_path
        .to_str()
        .ok_or(MainError::MissingDirectory)?
        .to_string();

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
